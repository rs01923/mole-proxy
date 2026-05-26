use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, watch};

#[derive(Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub target_domain: String,
    pub custom_location: Option<String>,
    pub mullvad_account_id: Option<String>,
}

pub struct AppState {
    pub config: RwLock<ProxyConfig>,
    pub restart_tx: watch::Sender<String>,
    pub running_tx: watch::Sender<bool>,
}
