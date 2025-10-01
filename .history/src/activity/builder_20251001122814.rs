use std::time::{SystemTime, UNIX_EPOCH};
use crate::activity::types::{Activity, ActivityAssets, ActivityTimestamps, ActivityParty, ActivitySecrets, ActivityButton};

/// Builder for creating Discord Rich Presence activities
#[derive(Debug, Default)]
pub struct ActivityBuilder {
    activity: Activity,
}

impl ActivityBuilder {
    /// Create a new activity builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the activity state (what the player is currently doing)
    pub fn state<S: Into<String>>(mut self, state: S) -> Self {
        self.activity.state = Some(state.into());
        self
    }

    /// Set the activity details (what the player is currently doing)
    pub fn details<S: Into<String>>(mut self, details: S) -> Self {
        self.activity.details = Some(details.into());
        self
    }

    /// Set the start timestamp to now
    pub fn start_timestamp_now(mut self) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let timestamps = self.activity.timestamps.get_or_insert_with(ActivityTimestamps::default);
        timestamps.start = Some(now);
        self
    }

    /// Set the start timestamp
    pub fn start_timestamp(mut self, timestamp: i64) -> Self {
        let timestamps = self.activity.timestamps.get_or_insert_with(ActivityTimestamps::default);
        timestamps.start = Some(timestamp);
        self
    }

    /// Set the end timestamp
    pub fn end_timestamp(mut self, timestamp: i64) -> Self {
        let timestamps = self.activity.timestamps.get_or_insert_with(ActivityTimestamps::default);
        timestamps.end = Some(timestamp);
        self
    }

    /// Set the large image asset
    pub fn large_image<S: Into<String>>(mut self, key: S) -> Self {
        let assets = self.activity.assets.get_or_insert_with(ActivityAssets::default);
        assets.large_image = Some(key.into());
        self
    }

    /// Set the large image text
    pub fn large_text<S: Into<String>>(mut self, text: S) -> Self {
        let assets = self.activity.assets.get_or_insert_with(ActivityAssets::default);
        assets.large_text = Some(text.into());
        self
    }

    /// Set the small image asset
    pub fn small_image<S: Into<String>>(mut self, key: S) -> Self {
        let assets = self.activity.assets.get_or_insert_with(ActivityAssets::default);
        assets.small_image = Some(key.into());
        self
    }

    /// Set the small image text
    pub fn small_text<S: Into<String>>(mut self, text: S) -> Self {
        let assets = self.activity.assets.get_or_insert_with(ActivityAssets::default);
        assets.small_text = Some(text.into());
        self
    }

    /// Set party information
    pub fn party<S: Into<String>>(mut self, id: S, current_size: u32, max_size: u32) -> Self {
        self.activity.party = Some(ActivityParty {
            id: Some(id.into()),
            size: Some([current_size, max_size]),
        });
        self
    }

    /// Add a button to the activity
    pub fn button<L: Into<String>, U: Into<String>>(mut self, label: L, url: U) -> Self {
        let buttons = self.activity.buttons.get_or_insert_with(Vec::new);
        buttons.push(ActivityButton {
            label: label.into(),
            url: url.into(),
        });
        self
    }

    /// Set join secret
    pub fn join_secret<S: Into<String>>(mut self, secret: S) -> Self {
        let secrets = self.activity.secrets.get_or_insert_with(ActivitySecrets::default);
        secrets.join = Some(secret.into());
        self
    }

    /// Set spectate secret
    pub fn spectate_secret<S: Into<String>>(mut self, secret: S) -> Self {
        let secrets = self.activity.secrets.get_or_insert_with(ActivitySecrets::default);
        secrets.spectate = Some(secret.into());
        self
    }

    /// Set match secret
    pub fn match_secret<S: Into<String>>(mut self, secret: S) -> Self {
        let secrets = self.activity.secrets.get_or_insert_with(ActivitySecrets::default);
        secrets.match_secret = Some(secret.into());
        self
    }

    /// Set instance flag
    pub fn instance(mut self, instance: bool) -> Self {
        self.activity.instance = Some(instance);
        self
    }

    /// Build the activity
    pub fn build(self) -> Activity {
        self.activity
    }
}

impl Default for ActivityTimestamps {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl Default for ActivityAssets {
    fn default() -> Self {
        Self {
            large_image: None,
            large_text: None,
            small_image: None,
            small_text: None,
        }
    }
}

impl Default for ActivitySecrets {
    fn default() -> Self {
        Self {
            join: None,
            spectate: None,
            match_secret: None,
        }
    }
}