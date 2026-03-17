use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodUsage {
    pub utilization: f64,
    pub resets_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub five_hour: PeriodUsage,
    pub seven_day: PeriodUsage,
    pub extra_usage: ExtraUsage,
}

#[derive(Debug)]
pub enum ApiError {
    TokenExpired,
    Network(String),
    Parse(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::TokenExpired => write!(f, "Token expired (HTTP 401). Run 'claude login' to re-authenticate."),
            ApiError::Network(msg) => write!(f, "Network error: {msg}"),
            ApiError::Parse(msg) => write!(f, "Failed to parse usage response: {msg}"),
        }
    }
}

/// Attempt to refresh an expired OAuth access token using the refresh token.
/// Returns the new access token on success.
pub async fn refresh_access_token(refresh_token: &str) -> Result<String, ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://console.anthropic.com/v1/oauth/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .header("User-Agent", "claude-code/2.0.32")
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("Token refresh request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(ApiError::TokenExpired);
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let token_resp: TokenResponse = response
        .json()
        .await
        .map_err(|e| ApiError::Parse(format!("Failed to parse token refresh response: {e}")))?;

    Ok(token_resp.access_token)
}

pub async fn fetch_usage(access_token: &str) -> Result<UsageData, ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(access_token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code/2.0.32")
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ApiError::TokenExpired);
    }

    if !response.status().is_success() {
        return Err(ApiError::Network(format!(
            "HTTP {} from usage API",
            response.status()
        )));
    }

    let usage: UsageData = response
        .json()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))?;

    Ok(usage)
}
