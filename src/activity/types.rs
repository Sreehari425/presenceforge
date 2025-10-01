use serde::{Deserialize, Serialize};

/// Rich Presence Activity
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Activity timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

/// Activity assets (images)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Activity {
    fn default() -> Self {
        Self {
            state: None,
            details: None,
            timestamps: None,
            assets: None,
            party: None,
            secrets: None,
            buttons: None,
            instance: None,
        }
    }
}