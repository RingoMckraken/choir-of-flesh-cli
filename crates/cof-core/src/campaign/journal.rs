use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One entry in a campaign's journal: a roll, an oracle answer, an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub at: DateTime<Utc>,
    /// What produced it, e.g. "Yes/No Oracle".
    pub source: String,
    /// The result, possibly several lines.
    pub text: String,
}
