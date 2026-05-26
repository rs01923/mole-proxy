use hickory_resolver::TokioAsyncResolver;
use tokio::io;

pub async fn resolve_target(
    resolver: &TokioAsyncResolver,
    domain: &str,
) -> io::Result<(String, u16)> {
    if let Some((host, port_str)) = domain.split_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return Ok((host.to_string(), port));
        }
    }

    let srv_query = format!("_minecraft._tcp.{}", domain);
    if let Ok(srv_lookup) = resolver.srv_lookup(&srv_query).await {
        if let Some(record) = srv_lookup.iter().next() {
            let target_domain = record
                .target()
                .to_string()
                .trim_end_matches('.')
                .to_string();
            let port = record.port();
            if let Ok(ip_lookup) = resolver.lookup_ip(&target_domain).await {
                if let Some(ip) = ip_lookup.iter().next() {
                    return Ok((ip.to_string(), port));
                }
            }
        }
    }

    if let Ok(ip_lookup) = resolver.lookup_ip(domain).await {
        if let Some(ip) = ip_lookup.iter().next() {
            return Ok((ip.to_string(), 25565));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Could not resolve host: {}", domain),
    ))
}
