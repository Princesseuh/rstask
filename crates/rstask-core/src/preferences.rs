use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncFrequency {
    Never,
    AfterEveryModification,
}

#[allow(clippy::derivable_impls)]
impl Default for SyncFrequency {
    fn default() -> Self {
        SyncFrequency::Never
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BulkCommitStrategy {
    Single,
    PerTask,
}

#[allow(clippy::derivable_impls)]
impl Default for BulkCommitStrategy {
    fn default() -> Self {
        BulkCommitStrategy::Single
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Preferences {
    #[serde(default)]
    pub sync_frequency: SyncFrequency,
    #[serde(default)]
    pub bulk_commit_strategy: BulkCommitStrategy,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            sync_frequency: SyncFrequency::Never,
            bulk_commit_strategy: BulkCommitStrategy::PerTask,
        }
    }
}

impl Preferences {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("rstask").join("config.styx"))
    }

    /// Load preferences from config file, or return default if file doesn't exist
    pub fn load() -> Self {
        let config_path = match Self::config_path() {
            Some(path) => path,
            None => return Self::default(),
        };

        let config_content = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(_) => return Self::default(),
        };

        serde_styx::from_str(&config_content).unwrap_or_default()
    }
}
