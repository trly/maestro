use keyring::Entry;

const SERVICE_NAME: &str = "dev.trly.maestro";

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
pub fn get_token(key: String) -> Result<Option<String>, String> {
    let entry = get_entry(&key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve token: {}", e))
    }
}

#[tauri::command]
pub fn get_token_masked(key: String) -> Result<Option<String>, String> {
    let entry = get_entry(&key)?;
    match entry.get_password() {
        Ok(password) => {
            let masked = if password.len() > 8 {
                format!("{}...{}", &password[..4], &password[password.len()-4..])
            } else if password.len() > 0 {
                "••••••••".to_string()
            } else {
                return Ok(None);
            };
            Ok(Some(masked))
        },
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve token: {}", e))
    }
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
pub fn has_token(key: String) -> Result<bool, String> {
    let entry = get_entry(&key)?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Failed to check token: {}", e))
    }
}
