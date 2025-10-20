use keyring::Entry;
use serde::{Serialize, Deserialize};

const SERVICE_NAME: &str = "dev.trly.maestro";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllTokens {
    pub amp_token: Option<String>,
    pub github_token: Option<String>,
    pub sourcegraph_endpoint: Option<String>,
    pub sourcegraph_token: Option<String>,
}

fn get_entry(key: &str) -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, key).map_err(|e| format!("Failed to access keyring: {}", e))
}

/// Internal helper to get token value (not a Tauri command)
pub(crate) fn get_token_value(key: &str) -> Result<Option<String>, String> {
    let entry = get_entry(key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve token: {}", e))
    }
}

#[tauri::command]
pub fn set_token(key: String, value: String) -> Result<(), String> {
    let entry = get_entry(&key)?;
    entry.set_password(&value)
        .map_err(|e| format!("Failed to save token: {}", e))
}

#[tauri::command]
pub fn delete_token(key: String) -> Result<(), String> {
    let entry = get_entry(&key)?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Failed to delete token: {}", e))
    }
}

#[tauri::command]
pub fn get_all_tokens() -> Result<AllTokens, String> {
    Ok(AllTokens {
        amp_token: get_token_value("amp_token")?,
        github_token: get_token_value("github_token")?,
        sourcegraph_endpoint: get_token_value("sourcegraph_endpoint")?,
        sourcegraph_token: get_token_value("sourcegraph_token")?,
    })
}

#[tauri::command]
pub fn get_all_tokens_masked() -> Result<AllTokens, String> {
    let mask = |token: Option<String>| -> Option<String> {
        token.map(|password| {
            if password.len() > 8 {
                format!("{}...{}", &password[..4], &password[password.len()-4..])
            } else if password.len() > 0 {
                "••••••••".to_string()
            } else {
                "".to_string()
            }
        })
    };
    
    Ok(AllTokens {
        amp_token: mask(get_token_value("amp_token")?),
        github_token: mask(get_token_value("github_token")?),
        sourcegraph_endpoint: mask(get_token_value("sourcegraph_endpoint")?),
        sourcegraph_token: mask(get_token_value("sourcegraph_token")?),
    })
}
