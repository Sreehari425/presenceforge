#[cfg(feature = "secrets")]
use crate::activity::types::ActivitySecrets;
use crate::activity::types::{
    Activity, ActivityAssets, ActivityButton, ActivityParty, ActivityTimestamps,
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

    #[cfg(feature = "secrets")]
    pub fn join_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.get_secrets().join = Some(secret.into());
        self
    }

    #[cfg(feature = "secrets")]
    pub fn spectate_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.get_secrets().spectate = Some(secret.into());
        self
    }

    #[cfg(feature = "secrets")]
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

    #[cfg(feature = "secrets")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_basic_fields() {
        let activity = ActivityBuilder::new()
            .state("Playing")
            .details("Level 1")
            .large_image("cover")
            .large_text("Cover Art")
            .small_image("icon")
            .small_text("Icon Art")
            .instance(true)
            .button("Join", "https://example.com/join")
            .build();

        let state = activity.state.as_deref();
        let details = activity.details.as_deref();
        let assets = activity.assets.unwrap();
        let buttons = activity.buttons.unwrap();

        assert_eq!(state, Some("Playing"));
        assert_eq!(details, Some("Level 1"));
        assert_eq!(assets.large_image.as_deref(), Some("cover"));
        assert_eq!(assets.small_text.as_deref(), Some("Icon Art"));
        assert!(activity.instance.unwrap());
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].label, "Join");
    }

    #[test]
    fn builder_sets_party_information() {
        let activity = ActivityBuilder::new().party("group", 2, 5).build();
        let party = activity.party.unwrap();
        assert_eq!(party.id.as_deref(), Some("group"));
        assert_eq!(party.size, Some([2, 5]));
    }

    #[test]
    fn start_timestamp_now_sets_current_time() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let activity = ActivityBuilder::new()
            .start_timestamp_now()
            .expect("timestamp should succeed")
            .build();

        let timestamp = activity
            .timestamps
            .and_then(|t| t.start)
            .expect("start timestamp set");

        assert!(timestamp >= before);
        assert!(timestamp - before <= 2);
    }

    #[test]
    fn start_and_end_timestamps_are_applied() {
        let activity = ActivityBuilder::new()
            .start_timestamp(100)
            .end_timestamp(200)
            .build();

        let timestamps = activity.timestamps.unwrap();
        assert_eq!(timestamps.start, Some(100));
        assert_eq!(timestamps.end, Some(200));
    }

    #[cfg(feature = "secrets")]
    #[test]
    fn secrets_are_applied_when_feature_enabled() {
        let activity = ActivityBuilder::new()
            .join_secret("join")
            .match_secret("match")
            .spectate_secret("spectate")
            .build();

        let secrets = activity.secrets.expect("secrets should exist");
        assert_eq!(secrets.join.as_deref(), Some("join"));
        assert_eq!(secrets.match_secret.as_deref(), Some("match"));
        assert_eq!(secrets.spectate.as_deref(), Some("spectate"));
    }
}
