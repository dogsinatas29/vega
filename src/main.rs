mod system;
mod storage;
mod safety;
mod security;
mod ai;
mod remote;
mod reporting;

use system::{global, env_scanner::EnvScanner};
use storage::db::Database;
use safety::{sanitizer, checker, ui};
use ai::router::SmartRouter;
use remote::RemoteManager;
use reporting::ReportGenerator;
use system::executor::Executor;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use serde::Deserialize;
use colored::*;
use std::io::{self, Write, BufRead};
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct AiResponse {
    command: String,
    explanation: String,
    risk_level: String,
    #[serde(default)]
    needs_clarification: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Project VEGA: The Sovereign SRE Agent", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Natural language query (optional if using subcommands)
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,

    /// Force specific AI engine (gemini, claude, openai)
    #[arg(short, long)]
    engine: Option<String>,

    /// Generate a session report
    #[arg(long)]
    report: bool,

    /// Execute command on remote host (format: user@host)
    #[arg(long)]
    remote: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check and validate API keys
    CheckKey,
}

// Helper for masking
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &key[0..4], &key[key.len()-4..])
}

// Helper for validation
async fn validate_gemini_key(key: &str) -> bool {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}", 
        key
    );
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

// Helper for loading spinner
async fn spin_loader(active: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    let frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut i = 0;
    while active.load(std::sync::atomic::Ordering::Relaxed) {
        print!("\r{} 🧠 VEGA가 생각 중입니다...", frames[i % frames.len()].cyan());
        std::io::stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        i += 1;
    }
    print!("\r\x1b[2K"); // Clear line
    std::io::stdout().flush().unwrap();
}

#[tokio::main]
async fn main() {
    // Initialize Logger
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", "error"); }
    }
    env_logger::init();
    
    dotenv().ok();
    
    // 0. Environment Discovery
    let discovered_keys = EnvScanner::scan_shell_configs();
    for (key, env_key) in &discovered_keys {
        if env::var(key).is_err() {
            unsafe { env::set_var(key, &env_key.value); }
        }
    }

    let args = Args::parse();
    
    // 1. Handle Subcommands
    if let Some(Commands::CheckKey) = args.command {
        // ... (CheckKey Implementation) ...
        println!("🔑 Checking API Keys...");
        let target_keys = vec!["GEMINI_API_KEY"];
        for key_name in target_keys {
            if let Some(env_key) = discovered_keys.get(key_name) {
                println!("\n📌 Key: {}", key_name.green().bold());
                println!("   📍 Source: {:?}:{}", env_key.source_file, env_key.line_num);
                println!("   🔒 Value:  {}", mask_key(&env_key.value));
                print!("   📡 Validating... ");
                std::io::stdout().flush().unwrap();
                let is_valid = if key_name == "GEMINI_API_KEY" { validate_gemini_key(&env_key.value).await } else { false };
                if is_valid { println!("{}", "✅ Active".green()); } else { println!("{}", "❌ Invalid or Expired".red()); }
            } else {
                 println!("\n📌 Key: {}", key_name.red().bold());
                 println!("   ❌ Not found in scanned shell configs.");
            }
        }
        return;
    }

    // 2. Handle Reporting
    if args.report {
        // ... (Reporting Implementation) ...
        println!("📊 Generating VEGA Report...");
        let mock_cmds = vec!["df -h".to_string(), "apt update".to_string(), "docker ps".to_string()];
        let md = ReportGenerator::generate_markdown("SESSION_LATEST", &mock_cmds);
        println!("{}", md);
        if let Err(e) = ReportGenerator::generate_pdf("vega_report.pdf", &md) { eprintln!("⚠️ PDF Generation failed: {}", e); } else { println!("✅ PDF Saved: vega_report.pdf"); }
        return;
    }

    // 3. Handle Remote Execution
    if let Some(target) = args.remote {
        // ... (Remote Implementation) ...
        let parts: Vec<&str> = target.split('@').collect();
        if parts.len() != 2 { eprintln!("❌ Invalid remote format. Use: user@host"); return; }
        let (user, host) = (parts[0], parts[1]);
        if args.query.is_empty() { eprintln!("❌ Please provide a command to run remotely."); return; }
        let command = args.query.join(" ");
        println!("🔌 Connecting to {}...", target);
        let mut remote = RemoteManager::new();
        match remote.connect(host, user, None) {
             Ok(_) => { match remote.exec_command(&command) { Ok(output) => println!("OUTPUT:\n{}", output), Err(e) => eprintln!("❌ Execution failed: {}", e), } },
             Err(e) => eprintln!("❌ Connection failed: {}", e),
        }
        return;
    }

    // 4. Interactive Event Loop
    global::initialize();
    
    let db = match Database::new() {
        Ok(db) => Some(db),
        Err(e) => {
            eprintln!("⚠️  Warning: Failed to initialize database: {}", e);
            None
        }
    };
    
    let context = global::get_context(); 
    
    // Load History
    let mut session_history: Vec<(String, String)> = if let Some(database) = &db {
        database.get_recent_history(5).unwrap_or(Vec::new())
    } else {
        Vec::new()
    };

    // Determine initial input
    let mut next_input = if !args.query.is_empty() {
        Some(args.query.join(" "))
    } else {
        println!("🌌 Vega Interactive Shell (Type 'exit' to quit)");
        None
    };

    let mut pending_error: Option<String> = None;
    let mut retry_count: u32 = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        // Get Input
        let raw_query = if let Some(err_msg) = pending_error.take() {
            println!("{}", "🔄 Attempting self-correction based on error...".yellow());
            // Add error to context for AI
            format!("The previous command failed with this error: {}. Please fix it and provide the corrected command.", err_msg)
        } else {
            // Normal User Input
            retry_count = 0; // Reset retries on new user input
            
            match next_input.take() {
                Some(s) => s,
                None => {
                    print!("❯ ");
                    io::stdout().flush().unwrap();
                    let mut buffer = String::new();
                    let stdin = io::stdin();
                    if stdin.lock().read_line(&mut buffer).is_err() {
                        break;
                    }
                    let trimmed = buffer.trim().to_string();
                    if trimmed.eq_ignore_ascii_case("exit") {
                        break;
                    }
                    if trimmed.is_empty() {
                        continue;
                    }
                    trimmed
                }
            }
        };

        // Log User Input (only if not an internal retry query, or log retries distinctively?)
        // For now, log everything to keep history linear
        if let Some(database) = &db {
            let _ = database.save_chat_message("user", &raw_query);
        }
        session_history.push(("user".to_string(), raw_query.clone()));

        let sanitized_query = sanitizer::sanitize_input(&raw_query);
        
        // Build Context-Aware Prompt
        let mut full_prompt = String::new();
        if !session_history.is_empty() {
            full_prompt.push_str("Recent Conversation History:\n");
            for (role, msg) in &session_history {
                full_prompt.push_str(&format!("{}: {}\n", role.to_uppercase(), msg));
            }
            full_prompt.push_str("\nCurrent Request: ");
        }
        full_prompt.push_str(&sanitized_query);

        let selected_engine = SmartRouter::determine_engine(&raw_query, args.engine.clone());
        
        // AI Call
        let response_result = match SmartRouter::get_provider(selected_engine) {
            Ok(ai_client) => {
                println!("🧠 Analyzing with {}...", ai_client.name());
                // Spinner
                let loading = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let loading_clone = loading.clone();
                let spinner_handle = tokio::spawn(async move { spin_loader(loading_clone).await; });

                let res = ai_client.generate_response(context, &full_prompt).await;
                
                loading.store(false, std::sync::atomic::Ordering::Relaxed);
                let _ = spinner_handle.await;
                
                res
            },
            Err(e) => Err(format!("Provider Error: {}", e).into())
        };

        match response_result {
            Ok(response_text) => {
                 match serde_json::from_str::<AiResponse>(&response_text) {
                    Ok(parsed) => {
                        // Log AI Response
                        if let Some(database) = &db {
                            let _ = database.save_chat_message("ai", &parsed.explanation);
                        }
                        session_history.push(("ai".to_string(), parsed.explanation.clone()));

                        println!("\n{}", "🤖 AI Reasoning:".blue().bold());
                        println!("{}", parsed.explanation);

                        if parsed.needs_clarification {
                            // Loop continues to get user input for clarification
                            println!("{}", "\n💬 Please provide more details below:".cyan());
                            continue;
                        }

                        println!("\n{}", "🛡  AI Risk Assessment:".yellow().bold());
                        println!("{}", parsed.risk_level);
                        
                        println!("\n{}", "🛠  Proposed Command:".green().bold());
                        println!("{}", parsed.command);

                        if !parsed.command.is_empty() {
                             let risk = checker::check_risk_level(&parsed.command);
                             if ui::confirm_action(risk, &parsed.command) {
                                println!("🚀 Executing: {}", parsed.command);
                                
                                // Capture Output/Error for Feedback Loop
                                match Executor::execute_command(&parsed.command) {
                                    Ok(output) => {
                                        println!("{}", output);
                                        if let Some(database) = &db {
                                            let _ = database.log_command(&parsed.command, &format!("User query: {}", sanitized_query), true);
                                        }
                                        println!("✅ Done.");
                                        
                                        // If we were retrying and succeeded, clear error/count
                                        pending_error = None;
                                        retry_count = 0;
                                        
                                        // If this was a one-shot CLI arg (not REPL), break
                                        // Logic: if args.query was used, next_input was Some, then taken. next_input is now None.
                                        // AND we are not in REPL loop (args.query was not empty initially)
                                        // Wait, the logic for REPL is implicit in the loop. 
                                        // If we want to behave like CLI for args, we should break if args.query existed.
                                        if !args.query.is_empty() && retry_count == 0 {
                                            break;
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("❌ Execution Failed: {}", e);
                                        if let Some(database) = &db {
                                            let _ = database.log_command(&parsed.command, &format!("User query: {}; Error: {}", sanitized_query, e), false);
                                        }
                                        
                                        // Trigger Self-Correction
                                        if retry_count < MAX_RETRIES {
                                            retry_count += 1;
                                            pending_error = Some(e.to_string());
                                            continue; // Loop again with error as input
                                        } else {
                                            eprintln!("{}", "❌ Maximum retries reached. Please contact the expert (User).".red().bold());
                                            pending_error = None;
                                            retry_count = 0;
                                            // Fall through to normal loop (user prompt) or break if one-shot
                                            if !args.query.is_empty() {
                                                 break;
                                            }
                                        }
                                    }
                                }
                             } else {
                                 println!("❌ Action cancelled by user.");
                                 // If cancelled, stop retrying
                                 pending_error = None;
                                 retry_count = 0;
                                 if !args.query.is_empty() { break; }
                             }
                        }
                    },
                    Err(e) => {
                        eprintln!("⚠️  JSON Parsing Failed: {}", e);
                        println!("✨ Suggestion: {}", response_text.trim());
                        // Parsing error -> maybe retry asking for JSON? For now, just stop.
                        if !args.query.is_empty() { break; }
                    }
                 }
            },
            Err(e) => {
                eprintln!("❌ AI Error: {}", e);
                if !args.query.is_empty() { break; }
            }
        }
    }
}
