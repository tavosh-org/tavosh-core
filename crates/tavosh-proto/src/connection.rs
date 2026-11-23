//! Shared, credential-free SSH session schemas.

pub mod v1 {
    use serde::{Deserialize, Serialize};

    fn enabled() -> bool {
        true
    }
    fn is_enabled(value: &bool) -> bool {
        *value
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SshSessionConfig {
        pub host: String,
        pub port: u16,
        pub username: String,
        pub auth: SshAuth,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub tmux_session: Option<String>,
        #[serde(default = "enabled", skip_serializing_if = "is_enabled")]
        pub shell_integration: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "method", rename_all = "snake_case")]
    pub enum SshAuth {
        PublicKey { credential_id: String },
        Password { credential_id: String },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct HostKeyFingerprint {
        pub algorithm: String,
        /// OpenSSH SHA256 fingerprint, including the SHA256: prefix.
        pub sha256: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum HostKeyDecision {
        TrustAndSave,
        Reject,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "state", rename_all = "snake_case")]
    pub enum ConnectionState {
        Disconnected {
            reason: Option<DisconnectReason>,
        },
        Connecting,
        VerifyingHostKey {
            request_id: Option<u64>,
            fingerprint: HostKeyFingerprint,
        },
        Authenticating,
        Connected,
        Reconnecting {
            attempt: u32,
            next_retry_ms: u64,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "reason", rename_all = "snake_case")]
    pub enum DisconnectReason {
        UserRequested,
        EndOfStream,
        NetworkError,
        TimedOut,
        HostKeyRejected,
        HostKeyMismatch {
            expected: HostKeyFingerprint,
            received: HostKeyFingerprint,
        },
        AuthenticationFailed,
        CredentialUnavailable,
        TrustStoreUnavailable,
        SessionSetupFailed,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn reconnect_state_has_stable_tags_and_relative_delay() {
            let state = ConnectionState::Reconnecting {
                attempt: 3,
                next_retry_ms: 1000,
            };
            let value = json!({"state": "reconnecting", "attempt": 3, "next_retry_ms": 1000});
            assert_eq!(serde_json::to_value(&state).unwrap(), value);
            assert_eq!(
                serde_json::from_value::<ConnectionState>(value).unwrap(),
                state
            );
        }

        #[test]
        fn shell_profile_can_omit_tmux() {
            let value = json!({
                "host": "example.org",
                "port": 22,
                "username": "tavo",
                "auth": {"method": "public_key", "credential_id": "primary"}
            });
            let config: SshSessionConfig = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(config.tmux_session, None);
            assert_eq!(serde_json::to_value(config).unwrap(), value);
        }
    }
}
