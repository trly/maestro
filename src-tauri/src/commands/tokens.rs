use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock, RwLock};
use zeroize::Zeroize;

const SERVICE_NAME: &str = "dev.trly.maestro";
const CREDENTIALS: &str = "credentials";

static CREDENTIAL_CACHE: OnceLock<Arc<RwLock<AllTokens>>> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AllTokens {
    pub amp_token: Option<String>,
    pub github_token: Option<String>,
    pub gitlab_token: Option<String>,
    pub gitlab_instance_url: Option<String>,
    pub sourcegraph_endpoint: Option<String>,
    pub sourcegraph_token: Option<String>,
    pub amp_client_id: Option<String>,
    pub amp_client_secret: Option<String>,
}

fn get_tokens_entry() -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, CREDENTIALS).map_err(|e| format!("Failed to access keyring: {}", e))
}

/// Internal helper to load all tokens from keyring (called once at startup)
fn load_all_tokens_from_keyring() -> Result<AllTokens, String> {
    let entry = get_tokens_entry()?;
    match entry.get_password() {
        Ok(mut json_str) => {
            let tokens = serde_json::from_str(&json_str)
                .map_err(|e| format!("Failed to parse tokens JSON: {}", e))?;
            json_str.zeroize(); // Wipe plaintext JSON after use
            Ok(tokens)
        }
        Err(keyring::Error::NoEntry) => Ok(AllTokens::default()),
        Err(e) => Err(format!("Failed to retrieve tokens: {}", e)),
    }
}

/// Internal helper to save all tokens to keyring
fn save_all_tokens_to_keyring(tokens: &AllTokens) -> Result<(), String> {
    let entry = get_tokens_entry()?;

    // If all tokens are empty, delete the entry instead of storing empty JSON
    if tokens == &AllTokens::default() {
        return match entry.delete_credential() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("Failed to delete tokens from keyring: {}", e)),
        };
    }

    let mut json_str =
        serde_json::to_string(tokens).map_err(|e| format!("Failed to serialize tokens: {}", e))?;
    let res = entry
        .set_password(&json_str)
        .map_err(|e| format!("Failed to save tokens to keyring: {}", e));
    json_str.zeroize(); // Wipe plaintext JSON after storage
    res
}

/// Initialize token cache at startup (call this once from setup)
pub fn init_token_cache() -> Result<(), String> {
    let tokens = load_all_tokens_from_keyring()?;
    CREDENTIAL_CACHE.get_or_init(|| Arc::new(RwLock::new(tokens)));
    Ok(())
}

/// Internal helper to get token value from cache
pub(crate) fn get_token_value(key: &str) -> Result<Option<String>, String> {
    let cache = CREDENTIAL_CACHE
        .get()
        .ok_or_else(|| "Token cache not initialized".to_string())?;

    let tokens = cache
        .read()
        .map_err(|e| format!("Failed to read token cache: {}", e))?;

    match key {
        "amp_token" => Ok(tokens.amp_token.clone()),
        "github_token" => Ok(tokens.github_token.clone()),
        "gitlab_token" => Ok(tokens.gitlab_token.clone()),
        "gitlab_instance_url" => Ok(tokens.gitlab_instance_url.clone()),
        "sourcegraph_endpoint" => Ok(tokens.sourcegraph_endpoint.clone()),
        "sourcegraph_token" => Ok(tokens.sourcegraph_token.clone()),
        "amp_client_id" => Ok(tokens.amp_client_id.clone()),
        "amp_client_secret" => Ok(tokens.amp_client_secret.clone()),
        _ => Err(format!("Unknown token key: {}", key)),
    }
}

#[tauri::command]
pub fn set_token(key: String, value: String) -> Result<(), String> {
    let cache = CREDENTIAL_CACHE
        .get()
        .ok_or_else(|| "Token cache not initialized".to_string())?;

    let mut tokens = cache
        .write()
        .map_err(|e| format!("Failed to lock token cache: {}", e))?;

    // Clone original state for rollback on save failure
    let original = tokens.clone();

    // Update the specific token, zeroizing old values
    match key.as_str() {
        "amp_token" => {
            if let Some(mut old) = tokens.amp_token.replace(value) {
                old.zeroize();
            }
        }
        "github_token" => {
            if let Some(mut old) = tokens.github_token.replace(value) {
                old.zeroize();
            }
        }
        "gitlab_token" => {
            if let Some(mut old) = tokens.gitlab_token.replace(value) {
                old.zeroize();
            }
        }
        "gitlab_instance_url" => {
            let _ = tokens.gitlab_instance_url.replace(value);
        }
        "sourcegraph_endpoint" => {
            let _ = tokens.sourcegraph_endpoint.replace(value);
        }
        "sourcegraph_token" => {
            if let Some(mut old) = tokens.sourcegraph_token.replace(value) {
                old.zeroize();
            }
        }
        "amp_client_id" => {
            let _ = tokens.amp_client_id.replace(value);
        }
        "amp_client_secret" => {
            if let Some(mut old) = tokens.amp_client_secret.replace(value) {
                old.zeroize();
            }
        }
        _ => return Err(format!("Unknown token key: {}", key)),
    }

    // Save entire bundle to keyring; revert on failure to keep cache consistent
    if let Err(e) = save_all_tokens_to_keyring(&tokens) {
        *tokens = original;
        return Err(e);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_token(key: String) -> Result<(), String> {
    let cache = CREDENTIAL_CACHE
        .get()
        .ok_or_else(|| "Token cache not initialized".to_string())?;

    let mut tokens = cache
        .write()
        .map_err(|e| format!("Failed to lock token cache: {}", e))?;

    // Clone original state for rollback on save failure
    let original = tokens.clone();

    // Clear the specific token, zeroizing removed values
    match key.as_str() {
        "amp_token" => {
            if let Some(mut s) = tokens.amp_token.take() {
                s.zeroize();
            }
        }
        "github_token" => {
            if let Some(mut s) = tokens.github_token.take() {
                s.zeroize();
            }
        }
        "gitlab_token" => {
            if let Some(mut s) = tokens.gitlab_token.take() {
                s.zeroize();
            }
        }
        "gitlab_instance_url" => {
            let _ = tokens.gitlab_instance_url.take();
        }
        "sourcegraph_endpoint" => {
            let _ = tokens.sourcegraph_endpoint.take();
        }
        "sourcegraph_token" => {
            if let Some(mut s) = tokens.sourcegraph_token.take() {
                s.zeroize();
            }
        }
        "amp_client_id" => {
            let _ = tokens.amp_client_id.take();
        }
        "amp_client_secret" => {
            if let Some(mut s) = tokens.amp_client_secret.take() {
                s.zeroize();
            }
        }
        _ => return Err(format!("Unknown token key: {}", key)),
    }

    // Save entire bundle to keyring; revert on failure to keep cache consistent
    if let Err(e) = save_all_tokens_to_keyring(&tokens) {
        *tokens = original;
        return Err(e);
    }

    Ok(())
}

#[tauri::command]
pub fn get_all_tokens() -> Result<AllTokens, String> {
    let cache = CREDENTIAL_CACHE
        .get()
        .ok_or_else(|| "Token cache not initialized".to_string())?;

    let tokens = cache
        .read()
        .map_err(|e| format!("Failed to read token cache: {}", e))?;

    Ok(tokens.clone())
}

#[tauri::command]
pub fn get_all_tokens_masked() -> Result<AllTokens, String> {
    let cache = CREDENTIAL_CACHE
        .get()
        .ok_or_else(|| "Token cache not initialized".to_string())?;

    let tokens = cache
        .read()
        .map_err(|e| format!("Failed to read token cache: {}", e))?;

    let mask = |token: &Option<String>| -> Option<String> {
        token.as_ref().map(|password| {
            if password.len() > 8 {
                format!("{}...{}", &password[..4], &password[password.len() - 4..])
            } else if !password.is_empty() {
                "••••••••".to_string()
            } else {
                "".to_string()
            }
        })
    };

    Ok(AllTokens {
        amp_token: mask(&tokens.amp_token),
        github_token: mask(&tokens.github_token),
        gitlab_token: mask(&tokens.gitlab_token),
        gitlab_instance_url: mask(&tokens.gitlab_instance_url),
        sourcegraph_endpoint: mask(&tokens.sourcegraph_endpoint),
        sourcegraph_token: mask(&tokens.sourcegraph_token),
        amp_client_id: mask(&tokens.amp_client_id),
        amp_client_secret: mask(&tokens.amp_client_secret),
    })
}
