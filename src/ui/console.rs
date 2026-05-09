use colored::Colorize;
use std::sync::RwLock;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Silent = 0,
    Normal = 1,
    Operator = 2,
    Developer = 3,
    Debug = 4,
}

lazy_static! {
    static ref GLOBAL_VERBOSITY: RwLock<Verbosity> = RwLock::new(Verbosity::Normal);
}

pub struct SreConsole;

impl SreConsole {
    pub fn set_verbosity(v: Verbosity) {
        if let Ok(mut lock) = GLOBAL_VERBOSITY.write() {
            *lock = v;
        }
    }

    pub fn get_verbosity() -> Verbosity {
        match GLOBAL_VERBOSITY.read() {
            Ok(v) => *v,
            Err(_) => Verbosity::Normal,
        }
    }

    /// 🛰️ Discovery & Infra Narrative
    pub fn info(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("🛰️  {}", msg);
        }
    }

    /// 🧠 Intelligence & Decision Narrative
    pub fn logic(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Operator {
            println!("🧠 {}", msg.cyan());
        }
    }

    /// ⚡ Action & Execution Narrative
    pub fn action(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("⚡ {}", msg.yellow().bold());
        }
    }

    /// 🔧 Remediation & Recovery Narrative
    pub fn remediate(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("🔧 {}", msg.magenta().bold());
        }
    }

    /// ✅ Success & Convergence Narrative
    pub fn success(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("✅ {}", msg.green().bold());
        }
    }

    /// 🚨 Error & Alert Narrative
    pub fn error(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("🚨 {}", msg.red().bold());
        }
    }

    /// 💻 Developer Telemetry (Internal Stages, Confidence, etc.)
    pub fn telemetry(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Developer {
            println!("💻 [Telemetry] {}", msg.dimmed());
        }
    }

    /// 🐛 Debug Logs (Raw SSH, JSON Sentinels)
    pub fn debug(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Debug {
            println!("🐛 [Debug] {}", msg.black().on_white());
        }
    }

    /// 📝 Detailed Explanation (Always Normal)
    pub fn note(msg: &str) {
        if Self::get_verbosity() >= Verbosity::Normal {
            println!("📝 {}", msg);
        }
    }
}
