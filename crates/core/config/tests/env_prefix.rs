#![cfg(feature = "test")]

#[tokio::test]
async fn klyde_env_prefix_overrides_config() {
    unsafe {
        std::env::set_var("KLYDE_DATABASE__POSTGRESQL", "postgres://env-prefix-test");
        std::env::set_var("KLYDE_HOSTS__API", "https://api.example.com");
    }

    let config = revolt_config::read().await;
    assert_eq!(
        config.get_string("database.postgresql").unwrap(),
        "postgres://env-prefix-test"
    );
    assert_eq!(config.get_string("hosts.api").unwrap(), "https://api.example.com");
}
