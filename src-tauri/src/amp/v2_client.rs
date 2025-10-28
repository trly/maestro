use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

const AUTH_URL: &str = "https://auth.ampcode.com/oauth2/token";
const API_BASE: &str = "https://ampcode.com/api/v2";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub first_synced_at: String,
    pub creator_user_id: String,
    pub main_thread_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMessagesResponse {
    pub messages: Vec<Message>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,                    // "user", "assistant", "info"
    pub content: Vec<serde_json::Value>, // Flexible content blocks
}

/// Amp V2 API Client with OAuth2 authentication
pub struct AmpV2Client {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
}

impl AmpV2Client {
    /// Create a new Amp V2 API client
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
            client_secret,
            access_token: None,
        }
    }

    /// Get OAuth2 access token (cached)
    async fn get_access_token(&mut self) -> Result<String> {
        // Return cached token if available
        if let Some(token) = &self.access_token {
            return Ok(token.clone());
        }

        // Request new token
        let params = [
            ("grant_type", "client_credentials"),
            (
                "scope",
                "amp.api:workspace.threads.meta:view amp.api:workspace.threads.contents:view",
            ),
        ];

        let response = self
            .client
            .post(AUTH_URL)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await
            .context("Failed to request OAuth2 token")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OAuth2 token request failed: {}", error_text);
        }

        let token_response: TokenResponse = response.json().await?;
        self.access_token = Some(token_response.access_token.clone());

        Ok(token_response.access_token)
    }

    /// Get thread metadata
    pub async fn get_thread(&mut self, thread_id: &str) -> Result<Thread> {
        let token = self.get_access_token().await?;
        let url = format!("{}/threads/{}", API_BASE, thread_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch thread")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to fetch thread {}: {}", thread_id, error_text);
        }

        Ok(response.json().await?)
    }

    /// Get all messages for a thread (handles pagination)
    pub async fn get_thread_messages(&mut self, thread_id: &str) -> Result<Vec<Message>> {
        let token = self.get_access_token().await?;
        let mut all_messages = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let url = format!("{}/threads/{}/messages", API_BASE, thread_id);
            let mut request = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/json")
                .query(&[("limit", "100")]);

            if let Some(c) = &cursor {
                request = request.query(&[("cursor", c)]);
            }

            let response = request
                .send()
                .await
                .context("Failed to fetch thread messages")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                anyhow::bail!(
                    "Failed to fetch messages for thread {}: {}",
                    thread_id,
                    error_text
                );
            }

            let page: ThreadMessagesResponse = response.json().await?;
            all_messages.extend(page.messages);

            if page.next_cursor.is_none() {
                break;
            }
            cursor = page.next_cursor;
        }

        Ok(all_messages)
    }

    /// Extract thread ID from Amp thread URL
    pub fn extract_thread_id(url: &str) -> Option<String> {
        url.split("/threads/")
            .nth(1)
            .and_then(|s| s.split('?').next())
            .map(|s| s.to_string())
    }

    /// Format messages as text for analysis prompt
    pub fn format_messages_for_analysis(messages: &[Message]) -> String {
        let mut output = String::new();

        for (i, msg) in messages.iter().enumerate() {
            output.push_str(&format!("\n=== Message {} ({}) ===\n", i + 1, msg.role));

            for content in &msg.content {
                if let Some(text) = content.get("text").and_then(|v| v.as_str()) {
                    output.push_str(text);
                    output.push('\n');
                } else if let Some(tool_use) = content.get("type").and_then(|t| {
                    if t.as_str() == Some("tool_use") {
                        Some(content)
                    } else {
                        None
                    }
                }) {
                    if let Some(name) = tool_use.get("name").and_then(|v| v.as_str()) {
                        output.push_str(&format!("[Tool use: {}]\n", name));
                        if let Some(input) = tool_use.get("input") {
                            output.push_str(&format!(
                                "Input: {}\n",
                                serde_json::to_string_pretty(input).unwrap_or_default()
                            ));
                        }
                    }
                } else if let Some(tool_result) = content.get("type").and_then(|t| {
                    if t.as_str() == Some("tool_result") {
                        Some(content)
                    } else {
                        None
                    }
                }) {
                    output.push_str("[Tool result]\n");
                    if let Some(run) = tool_result.get("run") {
                        if let Some(status) = run.get("status").and_then(|v| v.as_str()) {
                            output.push_str(&format!("Status: {}\n", status));
                        }
                        if let Some(output_val) = run.get("output").and_then(|v| v.as_str()) {
                            output.push_str(&format!("Output: {}\n", output_val));
                        }
                    }
                }
            }
        }

        output
    }
}
