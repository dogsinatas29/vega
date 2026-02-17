use keyring::Entry;
use std::env;

const SERVICE_NAME: &str = "project-vega";

pub fn get_api_key(key_name: &str) -> Option<String> {
    // 1. Try Keyring
    match Entry::new(SERVICE_NAME, key_name) {
        Ok(entry) => {
            if let Ok(pw) = entry.get_password() {
                return Some(pw);
            }
        }
        Err(_) => {}
    }

    // 2. Fallback to Environment Variable
    if let Ok(env_val) = env::var(key_name) {
        return Some(env_val);
    }

    None
}

#[allow(dead_code)]
pub fn set_api_key(key_name: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to save secret: {}", e))
}

pub fn get_token(user: &str) -> Option<String> {
    match Entry::new(SERVICE_NAME, user) {
        Ok(entry) => entry.get_password().ok(),
        Err(_) => None,
    }
}

pub fn set_token(user: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, user)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(value)
        .map_err(|e| format!("Failed to save secret: {}", e))
}

pub fn delete_token(user: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, user)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete secret: {}", e))
}
