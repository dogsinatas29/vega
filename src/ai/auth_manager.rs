use crate::security::keyring;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RefreshToken, TokenResponse, TokenUrl};
use serde_json::Value;
use std::error::Error;
use std::fs;

pub struct AuthManager;

impl AuthManager {
    pub async fn get_bearer_token() -> Result<String, Box<dyn Error>> {
        // 1. Try saved Access Token
        if let Some(token) = keyring::get_token("google_oauth_token") {
            // Ideally check expiry, but for now we'll rely on refresh if hit 401
            return Ok(token);
        }

        // 2. Try Refresh Token
        if let Ok(token) = Self::refresh_token().await {
            return Ok(token);
        }

        // 3. Try ADC (Application Default Credentials)
        if let Ok(token) = Self::get_adc_token() {
            return Ok(token);
        }

        Err("No valid Google authentication found. Run 'vega login' or 'gcloud auth application-default login'.".into())
    }

    async fn refresh_token() -> Result<String, Box<dyn Error>> {
        println!("⏳ [auth: renewing...]");
        let refresh_token =
            keyring::get_token("google_refresh_token").ok_or("No refresh token found")?;
        let client_id =
            keyring::get_token("google_client_id").ok_or("Missing Client ID for refresh")?;
        let client_secret = keyring::get_token("google_client_secret")
            .ok_or("Missing Client Secret for refresh")?;

        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(AuthUrl::new(
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            )?)
            .set_token_uri(TokenUrl::new(
                "https://www.googleapis.com/oauth2/v4/token".to_string(),
            )?);

        let http_client = reqwest::Client::new();
        let token_res = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(&http_client)
            .await?;

        let new_access_token = token_res.access_token().secret();
        let _ = keyring::set_token("google_oauth_token", new_access_token);

        // If a new refresh token is returned, update it too
        if let Some(new_refresh) = token_res.refresh_token() {
            let _ = keyring::set_token("google_refresh_token", new_refresh.secret());
        }

        Ok(new_access_token.to_string())
    }

    fn get_adc_token() -> Result<String, Box<dyn Error>> {
        let adc_path = dirs::home_dir()
            .map(|h| h.join(".config/gcloud/application_default_credentials.json"))
            .ok_or("Could not determine home directory")?;

        if !adc_path.exists() {
            return Err("ADC file not found".into());
        }

        // Note: For real ADC, we'd need to parse the JSON and potentially refresh it.
        // For now, let's just log that we found it.
        // A better implementation would use the google-cloud-auth crate, but we'll stick to a simple check.
        let content = fs::read_to_string(adc_path)?;
        let json: Value = serde_json::from_str(&content)?;

        // If it's a service account or authorized_user, it has different fields.
        // This is a placeholder for real ADC logic.
        if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
            return Ok(token.to_string());
        }

        Err("ADC found but no access_token available. Try running 'gcloud auth application-default login'.".into())
    }
}
