// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::client::{AtlasClient, ClientConfig, ErrorClass};

    #[test]
    fn client_rejects_non_http_base_url() {
        let config = ClientConfig {
            base_url: "ftp://invalid".to_string(),
            ..ClientConfig::default()
        };
        let err = match AtlasClient::new(config) {
            Ok(_) => panic!("invalid config should fail"),
            Err(err) => err,
        };
        assert_eq!(err.class, ErrorClass::InvalidConfig);
    }
}
