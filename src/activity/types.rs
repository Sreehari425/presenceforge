use serde::{Deserialize, Serialize};

/// Rich Presence Activity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<ActivityTimestamps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<ActivityAssets>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub party: Option<ActivityParty>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<ActivitySecrets>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<ActivityButton>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<bool>,
}

impl Activity {
    /// Validate the activity according to Discord's requirements
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, or Err(String) with the reason if invalid
    pub fn validate(&self) -> Result<(), String> {
        // Check text field lengths
        if let Some(state) = &self.state {
            if state.len() > 128 {
                return Err("State must be 128 characters or less".to_string());
            }
        }

        if let Some(details) = &self.details {
            if details.len() > 128 {
                return Err("Details must be 128 characters or less".to_string());
            }
        }

        // Validate buttons
        if let Some(buttons) = &self.buttons {
            // Discord allows a maximum of 2 buttons
            if buttons.len() > 2 {
                return Err("Discord allows a maximum of 2 buttons".to_string());
            }

            for button in buttons {
                if button.label.len() > 32 {
                    return Err("Button label must be 32 characters or less".to_string());
                }

                if button.url.len() > 512 {
                    return Err("Button URL must be 512 characters or less".to_string());
                }

                // Validate URL format (simple check)
                if !button.url.starts_with("http://") && !button.url.starts_with("https://") {
                    return Err("Button URL must start with http:// or https://".to_string());
                }
            }
        }

        // Validate asset keys
        if let Some(assets) = &self.assets {
            if let Some(large_image) = &assets.large_image {
                if large_image.len() > 256 {
                    return Err("Large image key must be 256 characters or less".to_string());
                }
            }

            if let Some(small_image) = &assets.small_image {
                if small_image.len() > 256 {
                    return Err("Small image key must be 256 characters or less".to_string());
                }
            }

            if let Some(large_text) = &assets.large_text {
                if large_text.len() > 128 {
                    return Err("Large text must be 128 characters or less".to_string());
                }
            }

            if let Some(small_text) = &assets.small_text {
                if small_text.len() > 128 {
                    return Err("Small text must be 128 characters or less".to_string());
                }
            }
        }

        // Validate party size
        if let Some(party) = &self.party {
            if let Some(size) = &party.size {
                if size[0] > size[1] {
                    return Err(
                        "Current party size cannot be greater than max party size".to_string()
                    );
                }
            }
        }

        Ok(())
    }
}

/// Activity timestamps
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

/// Activity assets (images)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityAssets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

/// Activity party information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityParty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<[u32; 2]>, // [current, max]
}

/// Activity secrets for join/spectate
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivitySecrets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectate: Option<String>,

    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_secret: Option<String>,
}

/// Activity button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityButton {
    pub label: String,
    pub url: String,
}
