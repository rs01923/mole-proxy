use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub target_domain: String,
    pub custom_location: Option<String>,
    pub mullvad_account_id: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MoleProxyApp {
    pub server_addr: String,
    pub listen_addr: String,
    pub mullvad_account_id: String,
    pub use_custom_location: bool,
    pub location: String,
    pub randomize_enabled: bool,
    pub current_country: String,
    #[serde(skip)]
    pub countries: Vec<(String, String)>, // (Code, Name)
    #[serde(skip)]
    pub active_relay_ip: Option<String>,
    #[serde(skip)]
    pub is_running: bool,
}

impl MoleProxyApp {
    pub fn new() -> Self {
        let mut app = Self::load().unwrap_or_else(|e| {
            eprintln!("Warning: Could not load config, using defaults: {}", e);
            Self {
                server_addr: "anticheat-test.com:25565".to_owned(),
                listen_addr: "0.0.0.0:25565".to_owned(),
                mullvad_account_id: "".to_owned(),
                use_custom_location: false,
                location: "al-tia".to_owned(),
                randomize_enabled: false,
                current_country: "Select Country".to_owned(),
                ..Default::default()
            }
        });

        if let Ok(countries) = crate::modules::network::get_countries() {
            app.countries = countries;
        }

        app
    }

    pub fn config_path() -> PathBuf {
        PathBuf::from("config.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let content = fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn normalize_addresses(&mut self) {
        if !self.server_addr.is_empty() && !self.server_addr.contains(':') {
            self.server_addr = format!("{}:25565", self.server_addr);
        }
        if !self.listen_addr.is_empty() && !self.listen_addr.contains(':') {
            self.listen_addr = format!("{}:25565", self.listen_addr);
        }
    }
}
