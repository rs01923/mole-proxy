mod api;
mod config;
mod protocol;
mod resolver;

use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, watch};
use tracing::{debug, error, info};

use crate::config::{AppState, ProxyConfig};
use crate::protocol::{encode_varint, read_varint, read_varint_sync};
use crate::resolver::resolve_target;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let initial_config = ProxyConfig {
        listen_addr: "0.0.0.0:25565".to_string(),
        target_domain: "anticheat-test.com".to_string(),
        custom_location: None,
        mullvad_account_id: None,
    };

    let (restart_tx, mut restart_rx) = watch::channel(initial_config.listen_addr.clone());
    let (running_tx, mut running_rx) = watch::channel(false);
    let state = Arc::new(AppState {
        config: RwLock::new(initial_config),
        restart_tx,
        running_tx,
    });

    let api_state = Arc::clone(&state);
    tokio::spawn(async move {
        let app = api::create_router(api_state);
        let listener = TcpListener::bind("0.0.0.0:8555").await.unwrap();
        info!(addr = "0.0.0.0:8555", "HTTP API listening");
        axum::serve(listener, app).await.unwrap();
    });

    let (dns_config, dns_opts) = hickory_resolver::system_conf::read_system_conf()
        .unwrap_or_else(|_| (ResolverConfig::default(), ResolverOpts::default()));
    let resolver = Arc::new(TokioAsyncResolver::tokio(dns_config, dns_opts));

    loop {
        if !*running_rx.borrow() {
            info!("Waiting for start signal...");
            while !*running_rx.borrow() {
                if running_rx.changed().await.is_err() {
                    return Ok(());
                }
            }
            info!("Start signal received.");
        }

        let current_listen_addr = restart_rx.borrow().clone();
        let listener = TcpListener::bind(&current_listen_addr).await?;
        info!(addr = %current_listen_addr, "Mole Proxy listening");

        loop {
            tokio::select! {
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((mut client_stream, client_addr)) => {
                            let resolver_clone = Arc::clone(&resolver);
                            let state_clone = Arc::clone(&state);

                            let mut running_rx_clone = state_clone.running_tx.subscribe();
                            tokio::spawn(async move {
                                let _ = client_stream.set_nodelay(true);
                                let res = async {
                                    let packet_len = read_varint(&mut client_stream).await?;
                                    let mut packet_data = vec![0u8; packet_len as usize];
                                    client_stream.read_exact(&mut packet_data).await?;

                                    let mut cursor = &packet_data[..];
                                    let packet_id = read_varint_sync(&mut cursor)?;
                                    if packet_id != 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected handshake")); }

                                    let protocol_version = read_varint_sync(&mut cursor)?;
                                    let addr_len = read_varint_sync(&mut cursor)? as usize;
                                    let original_addr = String::from_utf8_lossy(&cursor[..addr_len]).to_string();
                                    let cursor = &cursor[addr_len..];
                                    let _port = u16::from_be_bytes([cursor[0], cursor[1]]);
                                    let mut cursor = &cursor[2..];
                                    let next_state = read_varint_sync(&mut cursor)?;

                                    let target = {
                                        let config = state_clone.config.read().await;
                                        if original_addr == "localhost" || original_addr == "127.0.0.1" || original_addr.is_empty() {
                                            config.target_domain.clone()
                                        } else {
                                            original_addr
                                        }
                                    };

                                    let (resolved_ip, resolved_port) = resolve_target(&resolver_clone, &target).await?;

                                    let mut login_packet_data = Vec::new();
                                    let mut username = String::from("unknown");
                                    if next_state == 2 {
                                        let packet_len = read_varint(&mut client_stream).await?;
                                        let mut data = vec![0u8; packet_len as usize];
                                        client_stream.read_exact(&mut data).await?;
                                        let mut cursor = &data[..];
                                        let packet_id = read_varint_sync(&mut cursor)?;
                                        if packet_id == 0 {
                                            let name_len = read_varint_sync(&mut cursor)? as usize;
                                            username = String::from_utf8_lossy(&cursor[..name_len]).to_string();
                                        }
                                        let mut full_packet = encode_varint(packet_len);
                                        full_packet.extend(data);
                                        login_packet_data = full_packet;
                                    }

                                    if next_state == 2 {
                                        info!(%client_addr, %username, %target, resolved = %format!("{}:{}", resolved_ip, resolved_port), "Player connecting");
                                    } else {
                                        debug!(%client_addr, %target, resolved = %format!("{}:{}", resolved_ip, resolved_port), "Ping");
                                    }

                                    let mut server_stream = TcpStream::connect(format!("{}:{}", resolved_ip, resolved_port)).await?;
                                    let _ = server_stream.set_nodelay(true);

                                    let mut new_packet = Vec::new();
                                    new_packet.extend(encode_varint(0));
                                    new_packet.extend(encode_varint(protocol_version));
                                    let target_no_port = target.split(':').next().unwrap_or(&target);
                                    new_packet.extend(encode_varint(target_no_port.len() as i32));
                                    new_packet.extend(target_no_port.as_bytes());
                                    new_packet.extend(&resolved_port.to_be_bytes());
                                    new_packet.extend(encode_varint(next_state));

                                    let mut final_handshake = encode_varint(new_packet.len() as i32);
                                    final_handshake.extend(new_packet);
                                    server_stream.write_all(&final_handshake).await?;

                                    if !login_packet_data.is_empty() {
                                        server_stream.write_all(&login_packet_data).await?;
                                    }

                                    tokio::select! {
                                        res = io::copy_bidirectional(&mut client_stream, &mut server_stream) => {
                                            res?;
                                        },
                                        _ = async {
                                            while *running_rx_clone.borrow() {
                                                if running_rx_clone.changed().await.is_err() {
                                                    break;
                                                }
                                            }
                                        } => {
                                            return Err(io::Error::new(io::ErrorKind::Other, "Proxy stopped"));
                                        }
                                    }
                                    Ok::<(), io::Error>(())
                                }.await;
                                if let Err(e) = res { error!(%client_addr, error = %e, "Proxy error"); }
                            });
                        }
                        Err(e) => error!(error = %e, "Accept error"),
                    }
                }
                _ = restart_rx.changed() => {
                    info!("Restarting listener loop...");
                    break;
                }
                _ = running_rx.changed() => {
                    if !*running_rx.borrow() {
                        info!("Stopping listener...");
                        break;
                    }
                }
            }
        }
    }
}
