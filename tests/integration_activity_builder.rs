use presenceforge::ActivityBuilder;

#[test]
fn default_activity_valid() {
    let activity = ActivityBuilder::new().build();
    assert!(activity.validate().is_ok());
}

#[test]
fn set_state_and_details() {
    let activity = ActivityBuilder::new()
        .state("Playing")
        .details("In game")
        .build();
    assert_eq!(activity.state.as_deref(), Some("Playing"));
    assert_eq!(activity.details.as_deref(), Some("In game"));
    assert!(activity.validate().is_ok());
}

#[test]
fn state_length_exceeds_limit() {
    let long_state = "a".repeat(129);
    let activity = ActivityBuilder::new()
        .state(long_state)
        .build();
    assert!(activity.validate().is_err());
}

#[test]
fn button_limit_exceeded() {
    let activity = ActivityBuilder::new()
         .button("label1", "http://example.com/1")
         .button("label2", "http://example.com/2")
         .button("label3", "http://example.com/3")
         .build();
     assert!(activity.validate().is_err());
}

#[test]
fn invalid_button_url_scheme() {
    let activity = ActivityBuilder::new()
        .button("Play", "ftp://example.com")
        .build();
    assert!(activity.validate().is_err());
}

#[test]
fn large_image_key_too_long() {
    let activity = ActivityBuilder::new()
        .large_image("x".repeat(257))
        .build();
    assert!(activity.validate().is_err());
}

#[test]
fn valid_party_and_buttons_pass_validation() {
    let activity = ActivityBuilder::new()
        .state("Raiding")
        .party("raid-123", 3, 6)
        .button("Join", "https://example.com/join")
        .button("Watch", "https://example.com/watch")
        .build();

    assert!(activity.validate().is_ok());
}

#[test]
fn party_size_invalid() {
    let activity = ActivityBuilder::new()
        .party("id", 5, 4)
        .build();
    assert!(activity.validate().is_err());
}
