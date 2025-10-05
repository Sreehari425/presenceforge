use crate::activity::types::{
    Activity, ActivityAssets, ActivityButton, ActivityParty, ActivitySecrets, ActivityTimestamps,
};
use crate::error::{DiscordIpcError, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Builder for creating Discord Rich Presence activities
#[derive(Debug, Default)]
pub struct ActivityBuilder {
    activity: Activity,
}

impl ActivityBuilder {
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// Returns an error if the system time is before the UNIX epoch (Jan 1, 1970).
    /// This should never happen on properly configured systems.
    pub fn start_timestamp_now(mut self) -> Result<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| {
            DiscordIpcError::SystemTimeError(format!("System time is before UNIX epoch: {}", e))
        })?;

        self.get_timestamps().start = Some(now.as_secs());
        Ok(self)
    }

    /// Set the start timestamp
    pub fn start_timestamp(mut self, timestamp: u64) -> Self {
        self.get_timestamps().start = Some(timestamp);
        self
    }

    /// Set the end timestamp
    pub fn end_timestamp(mut self, timestamp: i64) -> Self {
        self.get_timestamps().end = Some(timestamp);
        self
    }

    /// Set the large image asset
    pub fn large_image<S: Into<String>>(mut self, key: S) -> Self {
        self.get_assets().large_image = Some(key.into());
        self
    }

    /// Set the large image text
    pub fn large_text<S: Into<String>>(mut self, text: S) -> Self {
        self.get_assets().large_text = Some(text.into());
        self
    }

    /// Set the small image asset
    pub fn small_image<S: Into<String>>(mut self, key: S) -> Self {
        self.get_assets().small_image = Some(key.into());
        self
    }

    /// Set the small image text
    pub fn small_text<S: Into<String>>(mut self, text: S) -> Self {
        self.get_assets().small_text = Some(text.into());
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

    pub fn button<L: Into<String>, U: Into<String>>(mut self, label: L, url: U) -> Self {
        let buttons = self.activity.buttons.get_or_insert_with(Vec::new);
        buttons.push(ActivityButton {
            label: label.into(),
            url: url.into(),
        });
        self
    }

    pub fn join_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.get_secrets().join = Some(secret.into());
        self
    }

    pub fn spectate_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.get_secrets().spectate = Some(secret.into());
        self
    }

    pub fn match_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.get_secrets().match_secret = Some(secret.into());
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

    fn get_secrets(&mut self) -> &mut ActivitySecrets {
        self.activity
            .secrets
            .get_or_insert_with(ActivitySecrets::default)
    }

    fn get_timestamps(&mut self) -> &mut ActivityTimestamps {
        self.activity
            .timestamps
            .get_or_insert_with(ActivityTimestamps::default)
    }

    fn get_assets(&mut self) -> &mut ActivityAssets {
        self.activity
            .assets
            .get_or_insert_with(ActivityAssets::default)
    }
}
