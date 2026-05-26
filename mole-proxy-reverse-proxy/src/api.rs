use crate::config::{AppState, ProxyConfig};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::process::Command;
use tracing::{error, info};

pub async fn get_config(State(state): State<Arc<AppState>>) -> Json<ProxyConfig> {
    let config = state.config.read().await;
    Json(config.clone())
}

pub async fn start_proxy(State(state): State<Arc<AppState>>) -> Result<&'static str, StatusCode> {
    {
        let config = state.config.read().await;
        if let Some(ref location) = config.custom_location {
            let mut processed_location = location.clone();

            if location.contains('-') && !location.contains("-wg-") {
                processed_location = processed_location.replace('-', " ");
            }

            info!(
                "Setting Mullvad relay location to {}...",
                processed_location
            );

            let mut cmd = Command::new("mullvad");
            cmd.args(["relay", "set", "location"]);
            for part in processed_location.split_whitespace() {
                cmd.arg(part);
            }

            let relay_status = cmd.status().await.map_err(|e| {
                error!("Failed to execute mullvad relay set: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            if !relay_status.success() {
                error!("Mullvad relay set command returned non-zero exit code");
            }
        }
    }

    info!("Connecting to Mullvad...");
    let connect_status = Command::new("mullvad")
        .arg("connect")
        .status()
        .await
        .map_err(|e| {
            error!("Failed to execute mullvad connect: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !connect_status.success() {
        error!("Mullvad connect command returned non-zero exit code");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Fetching new public IP...");
    let ip_output = Command::new("curl")
        .args(["-s", "https://checkip.amazonaws.com"])
        .output()
        .await;

    match ip_output {
        Ok(output) => {
            println!(
                "New Public IP: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            );
        }
        Err(e) => {
            error!("Failed to fetch public IP: {}", e);
        }
    }

    info!("Mullvad status:");
    let status_output = Command::new("mullvad").arg("status").output().await;

    match status_output {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => {
            error!("Failed to get mullvad status: {}", e);
        }
    }

    let _ = state.running_tx.send(true);
    info!("Proxy start signal sent");
    Ok("Proxy starting...")
}

pub async fn stop_proxy(State(state): State<Arc<AppState>>) -> &'static str {
    let _ = state.running_tx.send(false);
    info!("Proxy stop signal sent");
    "Proxy stopping..."
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<ProxyConfig>,
) -> Result<Json<ProxyConfig>, StatusCode> {
    if let Some(ref account_id) = new_config.mullvad_account_id {
        info!("Setting Mullvad account ID...");
        let _ = Command::new("mullvad")
            .args(["account", "login", account_id])
            .status()
            .await;

        sleep(Duration::from_secs(1));
        info!("Allowing Mullvad LAN access...");
        let lan_status = Command::new("mullvad")
            .args(["lan", "set", "allow"])
            .status()
            .await
            .map_err(|e| {
                error!("Failed to execute mullvad lan set: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !lan_status.success() {
            error!("Mullvad lan set command returned non-zero exit code");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let mut config = state.config.write().await;
    let old_listen_addr = config.listen_addr.clone();
    *config = new_config.clone();

    if old_listen_addr != new_config.listen_addr {
        info!(
            listen_addr = %new_config.listen_addr,
            "Listen address changed. Restarting proxy..."
        );
        let _ = state.restart_tx.send(new_config.listen_addr.clone());
    } else {
        info!(
            target_domain = %new_config.target_domain,
            "Target domain updated"
        );
    }

    Ok(Json(new_config))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/config", get(get_config).post(update_config))
        .route("/start", get(start_proxy))
        .route("/stop", get(stop_proxy))
        .with_state(state)
}
