use std::fs;
use std::io::Write;
use std::path::Path;

use super::StorageError;

/// Writes `contents` to `path` so that a crash can never leave a half-written file: write a
/// temp file in the same folder, flush it to disk, then rename it over the target (rename is
/// atomic on the same filesystem). Creates parent folders as needed.
pub(super) fn write_atomic(path: &Path, contents: &str) -> Result<(), StorageError> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir).map_err(StorageError::io(dir))?;

    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp = Path::new(&tmp_name);

    let mut file = fs::File::create(tmp).map_err(StorageError::io(tmp))?;
    file.write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(StorageError::io(tmp))?;
    drop(file);

    fs::rename(tmp, path).map_err(StorageError::io(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_creates_folders_and_replaces() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nested/deeper/file.json");
        write_atomic(&path, "first").unwrap();
        write_atomic(&path, "second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(leftovers.len(), 1, "temp file should be renamed away");
    }
}
