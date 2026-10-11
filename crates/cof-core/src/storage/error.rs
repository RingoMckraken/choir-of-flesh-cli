use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("couldn't find a home directory to store saves in")]
    NoHomeDirectory,

    #[error("couldn't access {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} isn't a readable save file: {source}")]
    Corrupt {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error(
        "{path} was saved by a newer version of the toolbox (format {found}, this build reads up to {supported})"
    )]
    NewerFormat {
        path: PathBuf,
        found: u32,
        supported: u32,
    },

    #[error("no campaign with id \"{0}\"")]
    NotFound(String),

    #[error("a campaign needs a name")]
    EmptyName,
}

impl StorageError {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| StorageError::Io { path, source }
    }

    pub(crate) fn corrupt(path: impl Into<PathBuf>) -> impl FnOnce(serde_json::Error) -> Self {
        let path = path.into();
        move |source| StorageError::Corrupt { path, source }
    }
}
