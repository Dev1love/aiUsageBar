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

pub fn fetch_codex_usage() -> Result<CodexUsageData, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    // Find codex binary
    let codex_path = find_codex_binary()?;

    let mut child = Command::new(&codex_path)
        .args(["-s", "read-only", "-a", "untrusted", "app-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch codex: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open codex stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to open codex stdout")?;
    let mut reader = BufReader::new(stdout);

    // Send initialize
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientName":"aiUsageBar","clientVersion":"0.1.0"}}"#;
    writeln!(stdin, "{init_req}").map_err(|e| format!("Failed to write to codex: {e}"))?;

    // Read initialize response
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| format!("Failed to read codex response: {e}"))?;

    // Send rateLimits request
    let limits_req = r#"{"jsonrpc":"2.0","id":2,"method":"account/rateLimits/read","params":{}}"#;
    writeln!(stdin, "{limits_req}").map_err(|e| format!("Failed to write to codex: {e}"))?;

    // Read rateLimits response
    line.clear();
    reader.read_line(&mut line).map_err(|e| format!("Failed to read codex response: {e}"))?;

    // Kill the process
    let _ = child.kill();

    // Parse response
    let resp: serde_json::Value = serde_json::from_str(line.trim())
        .map_err(|e| format!("Failed to parse codex JSON-RPC response: {e}"))?;

    let result = resp.get("result")
        .ok_or("No result in codex response")?;

    parse_codex_rate_limits(result)
}

fn find_codex_binary() -> Result<String, String> {
    // Check common locations
    let candidates = [
        // NVM paths
        format!("{}/.nvm/versions/node/v22.22.1/bin/codex", std::env::var("HOME").unwrap_or_default()),
        // Common global install
        "/usr/local/bin/codex".to_string(),
        "/opt/homebrew/bin/codex".to_string(),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.clone());
        }
    }

    // Try which
    let output = std::process::Command::new("which")
        .arg("codex")
        .output()
        .map_err(|e| format!("Failed to find codex: {e}"))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err("Codex CLI not found. Install with: npm install -g @openai/codex".to_string())
}

fn parse_codex_rate_limits(result: &serde_json::Value) -> Result<CodexUsageData, String> {
    // Try to extract usage windows from the result
    // Format varies — handle both flat and nested structures

    let mut primary = PeriodUsage {
        utilization: 0.0,
        resets_at: String::new(),
    };
    let mut secondary: Option<PeriodUsage> = None;
    let mut credits: Option<CodexCredits> = None;

    // Try windows array format
    if let Some(windows) = result.get("windows").and_then(|v| v.as_array()) {
        for (i, window) in windows.iter().enumerate() {
            let used_pct = window.get("usedPercent")
                .or_else(|| window.get("used_percent"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let resets = window.get("resetsAt")
                .or_else(|| window.get("resets_at"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let period = PeriodUsage {
                utilization: used_pct,
                resets_at: resets,
            };

            if i == 0 {
                primary = period;
            } else if i == 1 {
                secondary = Some(period);
            }
        }
    }
    // Try primary/secondary format
    else if let Some(p) = result.get("primary") {
        primary.utilization = p.get("usedPercent").and_then(|v| v.as_f64()).unwrap_or(0.0);
        primary.resets_at = p.get("resetsAt").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if let Some(s) = result.get("secondary") {
            secondary = Some(PeriodUsage {
                utilization: s.get("usedPercent").and_then(|v| v.as_f64()).unwrap_or(0.0),
                resets_at: s.get("resetsAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
    }

    // Extract credits
    if let Some(c) = result.get("credits") {
        credits = Some(CodexCredits {
            remaining: c.get("remaining").or_else(|| c.get("balance")).and_then(|v| v.as_f64()).unwrap_or(0.0),
            has_credits: c.get("hasCredits").or_else(|| c.get("has_credits")).and_then(|v| v.as_bool()).unwrap_or(true),
        });
    }

    Ok(CodexUsageData {
        primary,
        secondary,
        credits,
    })
}
