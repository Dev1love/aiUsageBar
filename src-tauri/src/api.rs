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

/// Combined usage from all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllUsage {
    pub claude: Option<UsageData>,
    pub codex: Option<CodexUsageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexUsageData {
    pub primary: PeriodUsage,
    pub secondary: Option<PeriodUsage>,
    pub credits: Option<CodexCredits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCredits {
    pub remaining: f64,
    pub has_credits: bool,
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

// ── Claude ──

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

// ── Codex ──

pub async fn fetch_codex_usage() -> Result<CodexUsageData, String> {
    // Read access token from ~/.codex/auth.json
    let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
    let auth_path = format!("{home}/.codex/auth.json");
    let auth_str = std::fs::read_to_string(&auth_path)
        .map_err(|_| "Codex not logged in (~/.codex/auth.json not found). Run 'codex login'.")?;
    let auth: serde_json::Value = serde_json::from_str(&auth_str)
        .map_err(|e| format!("Failed to parse codex auth.json: {e}"))?;
    let access_token = auth
        .get("tokens").and_then(|t| t.get("access_token")).and_then(|v| v.as_str())
        .ok_or("No access_token in codex auth.json")?;

    // Call ChatGPT web API for usage
    let client = reqwest::Client::new();
    let response = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(access_token)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("Codex API error: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Codex API HTTP {}", response.status()));
    }

    let data: serde_json::Value = response.json().await
        .map_err(|e| format!("Codex parse error: {e}"))?;

    parse_wham_usage(&data)
}

fn parse_wham_usage(data: &serde_json::Value) -> Result<CodexUsageData, String> {
    let rate_limit = data.get("rate_limit")
        .ok_or("No rate_limit in Codex response")?;

    let primary_window = rate_limit.get("primary_window")
        .ok_or("No primary_window")?;

    let primary = PeriodUsage {
        utilization: primary_window.get("used_percent").and_then(|v| v.as_f64()).unwrap_or(0.0),
        resets_at: timestamp_to_iso(primary_window.get("reset_at").and_then(|v| v.as_i64()).unwrap_or(0)),
    };

    let secondary = rate_limit.get("secondary_window")
        .and_then(|w| if w.is_null() { None } else { Some(w) })
        .map(|w| PeriodUsage {
            utilization: w.get("used_percent").and_then(|v| v.as_f64()).unwrap_or(0.0),
            resets_at: timestamp_to_iso(w.get("reset_at").and_then(|v| v.as_i64()).unwrap_or(0)),
        });

    let credits = data.get("credits").map(|c| {
        let balance_str = c.get("balance").and_then(|v| v.as_str()).unwrap_or("0");
        CodexCredits {
            remaining: balance_str.parse::<f64>().unwrap_or(0.0),
            has_credits: c.get("has_credits").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    });

    Ok(CodexUsageData {
        primary,
        secondary,
        credits,
    })
}

fn timestamp_to_iso(ts: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_secs(ts as u64);
    // Format as ISO 8601
    let secs = dt.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    // Simple epoch to date calculation
    let (year, month, day) = epoch_days_to_ymd(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn epoch_days_to_ymd(days: i64) -> (i32, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}
