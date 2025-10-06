use presenceforge::{Activity, ActivityAssets, ActivityBuilder, ActivityParty};
use serde_json::Value;

fn activity_to_value(activity: &Activity) -> Value {
    serde_json::to_value(activity).expect("activity should serialize")
}

#[test]
fn builder_produces_serializable_activity() {
    let activity = ActivityBuilder::new()
        .state("Playing campaign")
        .details("Mission 5")
        .start_timestamp(1234)
        .end_timestamp(5678)
        .large_image("cover-art")
        .large_text("Cover Art")
        .small_image("icon")
        .small_text("Icon Text")
        .party("party-id", 2, 4)
        .button("Join", "https://example.com/join")
        .build();

    let value = activity_to_value(&activity);

    assert_eq!(value["state"], "Playing campaign");
    assert_eq!(value["details"], "Mission 5");

    let timestamps = value
        .get("timestamps")
        .and_then(Value::as_object)
        .expect("timestamps serialized");
    assert_eq!(timestamps["start"], 1234);
    assert_eq!(timestamps["end"], 5678);

    let assets = value
        .get("assets")
        .and_then(Value::as_object)
        .expect("assets serialized");
    assert_eq!(assets["large_image"], "cover-art");
    assert_eq!(assets["small_text"], "Icon Text");

    let party = value
        .get("party")
        .and_then(Value::as_object)
        .expect("party serialized");
    let size = party["size"].as_array().expect("party size array");
    assert_eq!(size[0], 2);
    assert_eq!(size[1], 4);

    let buttons = value["buttons"].as_array().expect("buttons serialize");
    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0]["label"], "Join");
}

#[test]
fn manual_activity_validation_matches_serialization() {
    let mut activity = Activity::default();
    activity.state = Some("Multiplayer".to_string());
    activity.party = Some(ActivityParty {
        id: Some("group".into()),
        size: Some([1, 4]),
    });
    activity.assets = Some(ActivityAssets {
        large_image: Some("hero".into()),
        large_text: Some("Hero".into()),
        ..ActivityAssets::default()
    });

    activity.validate().expect("activity should be valid");

    let value = activity_to_value(&activity);
    assert_eq!(value["state"], "Multiplayer");
    assert!(value.get("assets").is_some());
}
