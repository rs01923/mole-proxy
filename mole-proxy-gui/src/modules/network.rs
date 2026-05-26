use crate::modules::config::ProxyConfig;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct LocationInfo {
    country: String,
    city: String,
}

#[derive(Deserialize)]
struct MullvadData {
    locations: std::collections::HashMap<String, LocationInfo>,
}

pub fn get_countries() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let res = client
        .get("https://api.mullvad.net/public/relays/wireguard/v2")
        .send()?
        .json::<MullvadData>()?;

    let mut sorted: Vec<(String, String)> = res
        .locations
        .into_iter()
        .map(|(id, info)| (id, format!("{} - {}", info.country, info.city)))
        .collect();

    sorted.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(sorted)
}

#[derive(Deserialize)]
struct MullvadRelays {
    wireguard: WireguardData,
}

#[derive(Deserialize)]
struct WireguardData {
    relays: Vec<Relay>,
}

#[derive(Deserialize)]
struct Relay {
    hostname: String,
    location: String,
    active: bool,
    ipv4_addr_in: String,
}

pub fn get_random_relay(input: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let res = client
        .get("https://api.mullvad.net/public/relays/wireguard/v2")
        .send()?
        .json::<MullvadRelays>()?;

    let target_location = res
        .wireguard
        .relays
        .iter()
        .find(|r| r.hostname == input)
        .map(|r| r.location.clone())
        .unwrap_or_else(|| input.to_string());

    let filtered: Vec<&Relay> = res
        .wireguard
        .relays
        .iter()
        .filter(|r| {
            r.active && (r.location == target_location || r.hostname.starts_with(&target_location))
        })
        .collect();

    if filtered.is_empty() {
        return Err("No active relays found for this location".into());
    }

    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    let relay = filtered.choose(&mut rng).unwrap();
    Ok((relay.hostname.clone(), relay.ipv4_addr_in.clone()))
}

pub fn send_config(config: &ProxyConfig) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    client
        .post("http://localhost:8555/config")
        .json(config)
        .send()
}

pub fn start_proxy() -> Result<reqwest::blocking::Response, reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    client.get("http://localhost:8555/start").send()
}

pub fn stop_proxy() -> Result<reqwest::blocking::Response, reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    client.get("http://localhost:8555/stop").send()
}
