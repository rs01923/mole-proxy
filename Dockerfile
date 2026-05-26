FROM rust:slim as builder
WORKDIR /usr/src/app

COPY mole-proxy-reverse-proxy/Cargo.toml .
COPY mole-proxy-reverse-proxy/src ./src
RUN cargo build --release

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y curl iproute2 iptables dbus ca-certificates
RUN curl -fsSLo mullvad.deb "https://mullvad.net/download/app/deb/latest" && \
    apt-get install -y ./mullvad.deb && \
    rm mullvad.deb

COPY --from=builder /usr/src/app/target/release/mole-proxy-reverse-proxy /usr/local/bin/mole-proxy-reverse-proxy
RUN echo '#!/bin/bash\n\
mkdir -p /run/dbus\n\
rm -f /run/dbus/pid\n\
dbus-daemon --system\n\
\n\
echo "Starting Mullvad Daemon..."\n\
mullvad-daemon -v &\n\
\n\
echo "Waiting for Mullvad to initialize..."\n\
sleep 5\n\
\n\
echo "Starting Proxy..."\n\
exec mole-proxy-reverse-proxy\n\
' > /start.sh && chmod +x /start.sh

EXPOSE 25565 8555

CMD ["/start.sh"]
