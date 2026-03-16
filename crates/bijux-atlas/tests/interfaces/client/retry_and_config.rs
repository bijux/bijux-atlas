// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::adapters::inbound::client::{
    run_with_retry, ClientConfig, ClientError, ErrorClass,
};
use reqwest as _;
use serde as _;
use serde_json as _;

#[test]
fn config_validation_rejects_zero_timeout() {
    let config = ClientConfig {
        timeout_millis: 0,
        ..ClientConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn error_classification_enum_is_stable() {
    let classes = [
        ErrorClass::Transport,
        ErrorClass::Timeout,
        ErrorClass::RateLimited,
        ErrorClass::Server,
        ErrorClass::Client,
        ErrorClass::Decode,
        ErrorClass::InvalidConfig,
    ];
    assert_eq!(classes.len(), 7);
}

#[test]
fn retry_helper_retries_until_success() {
    let mut attempts = 0;
    let result = run_with_retry(3, 0, || {
        attempts += 1;
        if attempts < 2 {
            Err(ClientError::new(ErrorClass::Transport, "temporary"))
        } else {
            Ok(42)
        }
    });
    let result = match result {
        Ok(value) => value,
        Err(error) => panic!("retry helper should succeed: {error}"),
    };
    assert_eq!(result, 42);
}
