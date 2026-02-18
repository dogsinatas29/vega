use keyring::Entry;
use log::{debug, warn};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "com.dogsinatas.vega";

fn get_fallback_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vega");
    path.push(".secrets.json");
    path
}

fn read_fallback() -> Value {
    let path = get_fallback_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    }
}

fn write_fallback(data: Value) {
    let path = get_fallback_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(path, content);
    }
}

pub fn get_api_key(key_name: &str) -> Option<String> {
    // 1. Try Keyring
    if let Ok(entry) = Entry::new(SERVICE_NAME, key_name) {
        if let Ok(pw) = entry.get_password() {
            return Some(pw);
        }
    }

    // 2. Try Fallback File
    let fallback = read_fallback();
    if let Some(val) = fallback[key_name].as_str() {
        return Some(val.to_string());
    }

    // 3. Fallback to Environment Variable
    if let Ok(env_val) = env::var(key_name) {
        return Some(env_val);
    }

    None
}

pub fn set_api_key(key_name: &str, value: &str) -> Result<(), String> {
    // 1. Try Keyring
    let mut keyring_ok = false;
    if let Ok(entry) = Entry::new(SERVICE_NAME, key_name) {
        if entry.set_password(value).is_ok() {
            keyring_ok = true;
        }
    }

    // 2. Always Save to Fallback (for robustness)
    let mut fallback = read_fallback();
    fallback[key_name] = json!(value);
    write_fallback(fallback);

    if keyring_ok {
        debug!("   ✅ API Key saved to OS Keyring and Fallback.");
    } else {
        warn!("   ⚠️  OS Keyring failed for API Key, saved only to Fallback.");
    }
    Ok(())
}

pub fn get_token(user: &str) -> Option<String> {
    // 1. Try Keyring
    if let Ok(entry) = Entry::new(SERVICE_NAME, user) {
        if let Ok(pw) = entry.get_password() {
            return Some(pw);
        }
    }

    // 2. Try Fallback File
    let fallback = read_fallback();
    if let Some(val) = fallback[user].as_str() {
        return Some(val.to_string());
    }

    None
}

pub fn set_token(user: &str, value: &str) -> Result<(), String> {
    // 1. Try Keyring
    let mut keyring_ok = false;
    if let Ok(entry) = Entry::new(SERVICE_NAME, user) {
        if entry.set_password(value).is_ok() {
            keyring_ok = true;
        }
    }

    // 2. Always Save to Fallback
    let mut fallback = read_fallback();
    fallback[user] = json!(value);
    write_fallback(fallback);

    if keyring_ok {
        debug!("   ✅ Saved to OS Keyring and Fallback.");
    } else {
        warn!("   ⚠️  OS Keyring failed, saved only to Fallback.");
    }

    Ok(())
}

pub fn delete_token(user: &str) -> Result<(), String> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, user) {
        let _ = entry.delete_credential();
    }

    let mut fallback = read_fallback();
    if let Some(obj) = fallback.as_object_mut() {
        obj.remove(user);
        write_fallback(fallback);
    }

    Ok(())
}
pub fn debug_persistence() {
    let test_key = "vega_persistence_test";
    let test_val = "PERSISTED";

    println!(
        "1. Environment: DBUS_SESSION_BUS_ADDRESS={}",
        env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_else(|_| "NOT SET".to_string())
    );

    println!("2. Saving test token...");
    match set_token(test_key, test_val) {
        Ok(_) => println!("   ✅ Successfully saved."),
        Err(e) => println!("   ❌ Save failed: {}", e),
    }

    println!("3. Reading test token...");
    println!("   A) Via OS Keyring directly:");
    match Entry::new(SERVICE_NAME, test_key) {
        Ok(entry) => match entry.get_password() {
            Ok(v) if v == test_val => println!("      ✅ Successfully read: {}", v),
            Ok(v) => println!("      ❌ Read wrong value: {}", v),
            Err(e) => println!("      ❌ Read failed: {}", e),
        },
        Err(e) => println!("      ❌ Could not create entry: {}", e),
    }

    println!("   B) Via VEGA Auth Logic (with fallback):");
    match get_token(test_key) {
        Some(v) if v == test_val => println!("      ✅ Successfully read: {}", v),
        Some(v) => println!("      ❌ Read wrong value: {}", v),
        None => println!("      ❌ Read failed (returned None)."),
    }
}
