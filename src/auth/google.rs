use oauth2::{
    basic::{BasicClient, BasicTokenResponse},
    AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
// use oauth2::reqwest::async_http_client; // Removed in v5.0
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
// use url::Url;
// use keyring::Entry; // Removed, using crate::security::keyring helper
// use serde::{Deserialize, Serialize}; // Removed unused

// Standard Google OAuth2 constants
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v4/token";

pub async fn login() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Initiating Google OAuth Login...");

    // 1. Get Client ID & Secret
    let client_id_input = prompt_input("Enter Google Client ID: ")?;
    if !client_id_input.ends_with(".apps.googleusercontent.com") {
        return Err(
            "Invalid Client ID format. It should end with '.apps.googleusercontent.com'".into(),
        );
    }

    let client_secret_input = prompt_input("Enter Google Client Secret: ")?;
    if client_secret_input.is_empty() {
        return Err("Client Secret cannot be empty.".into());
    }

    // Clone before moving into ClientId/ClientSecret structs
    let client_id_str = client_id_input.clone();
    let client_secret_str = client_secret_input.clone();

    let client_id = ClientId::new(client_id_input);
    let client_secret = ClientSecret::new(client_secret_input);

    // 2. Set up the OAuth client
    let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string())?;
    let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?;

    // Bind to a local port for callback
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();

    let redirect_url = RedirectUrl::new(format!("http://127.0.0.1:{}", port))?;

    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url);

    // 3. Generate Authorization URL
    // CRITICAL: We need access_type=offline and prompt=consent to get a refresh_token
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/generative-language.retriever".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/cloud-platform".to_string(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    println!("\n🌍 Opening browser for authentication...");
    println!("👉 Authorization URL (Copy this if browser doesn't open):");
    println!("{}\n", authorize_url);

    // Try to open browser
    if let Err(_) = std::process::Command::new("xdg-open")
        .arg(authorize_url.as_str())
        .spawn()
    {
        println!("❌ Failed to open browser automatically. Please copy the URL above.");
    }

    // 4. Listen for Callback
    println!("⏳ Waiting for callback on port {}...", port);

    let (mut stream, _) = listener.accept()?;
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse the code from "GET /?code=... HTTP/1.1"
    let redirect_url_path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let url = url::Url::parse(&format!("http://localhost{}", redirect_url_path))?;

    let code_pair = url.query_pairs().find(|(key, _)| key == "code");

    let message = if let Some((_, code)) = code_pair {
        // Exchange code for token
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or(reqwest::Client::new());

        let token_result: Result<BasicTokenResponse, _> = client
            .exchange_code(oauth2::AuthorizationCode::new(code.into_owned()))
            .request_async(&http_client)
            .await;

        match token_result {
            Ok(token) => {
                let access_secret = token.access_token().secret();

                // Save Access Token
                let _ = crate::security::keyring::set_token("google_oauth_token", access_secret);

                // Save Refresh Token (if provided)
                if let Some(refresh_token) = token.refresh_token() {
                    let refresh_secret = refresh_token.secret();
                    match crate::security::keyring::set_token(
                        "google_refresh_token",
                        refresh_secret,
                    ) {
                        Ok(_) => println!("✅ Refresh token saved for persistent sessions."),
                        Err(e) => eprintln!("⚠️  Failed to save refresh token: {}", e),
                    }
                } else {
                    println!("⚠️  Note: No refresh token received. Persistence might be limited.");
                }

                // Save Client ID/Secret for future auto-refresh
                let _ = crate::security::keyring::set_token("google_client_id", &client_id_str);
                let _ =
                    crate::security::keyring::set_token("google_client_secret", &client_secret_str);

                println!("✅ Login successful! Tokens saved.");
                "Authentication successful! You can close this window."
            }
            Err(e) => {
                eprintln!("❌ Token exchange failed: {}", e);
                "Authentication failed during token exchange."
            }
        }
    } else {
        "Invalid request. Missing authorization code."
    };

    // Send response to browser
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        message.len(),
        message
    );
    stream.write_all(response.as_bytes())?;

    Ok(())
}

fn prompt_input(prompt: &str) -> Result<String, std::io::Error> {
    print!("{}", prompt);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
