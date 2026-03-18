use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct KeychainCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Deserialize)]
struct OAuthEntry {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: u64,
}

#[derive(Deserialize)]
struct KeychainData {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OAuthEntry,
}

pub fn read_credentials() -> Result<KeychainCredentials, String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| format!("Failed to run `security` command: {e}"))?;

    if !output.status.success() {
        return Err(
            "Could not read Claude Code credentials from macOS Keychain. \
             Make sure you have logged in with 'claude login'."
                .to_string(),
        );
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Keychain value is not valid UTF-8: {e}"))?;
    let json_str = json_str.trim();

    let data: KeychainData = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse keychain JSON: {e}"))?;

    Ok(KeychainCredentials {
        access_token: data.claude_ai_oauth.access_token,
        refresh_token: data.claude_ai_oauth.refresh_token,
        expires_at: data.claude_ai_oauth.expires_at,
    })
}
