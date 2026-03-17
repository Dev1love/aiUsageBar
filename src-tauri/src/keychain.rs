use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct KeychainCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
struct OAuthEntry {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

#[derive(Deserialize)]
struct KeychainData {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OAuthEntry,
}

pub fn read_credentials() -> Result<KeychainCredentials, String> {
    let password = security_framework::passwords::get_generic_password(
        "Claude Code-credentials",
        "Claude Code-credentials",
    )
    .map_err(|e| {
        format!(
            "Could not read Claude Code credentials from macOS Keychain: {e}. \
             Make sure you have logged in with 'claude login'."
        )
    })?;

    let json_str = String::from_utf8(password.to_vec())
        .map_err(|e| format!("Keychain value is not valid UTF-8: {e}"))?;

    let data: KeychainData = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse keychain JSON: {e}"))?;

    Ok(KeychainCredentials {
        access_token: data.claude_ai_oauth.access_token,
        refresh_token: data.claude_ai_oauth.refresh_token,
        expires_at: data.claude_ai_oauth.expires_at,
    })
}
