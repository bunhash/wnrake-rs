# wnrake-rs

## Docker Compose files for gluetun

```yaml
services:
  vpn1:
    image: qmcgaw/gluetun
    container_name: vpn1
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=provider
      - VPN_TYPE=openvpn
      - OPENVPN_USER=
      - OPENVPN_PASSWORD=
      - HTTPPROXY=on
    volumes:
      - "vpn.toml:/gluetun/auth/config.toml"
    ports:
      - '9000:8888/tcp'
      - '8000:8000/tcp'
  vpn2:
    image: qmcgaw/gluetun
    container_name: vpn2
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=provider
      - VPN_TYPE=openvpn
      - OPENVPN_USER=
      - OPENVPN_PASSWORD=
      - HTTPPROXY=on
    volumes:
      - "vpn.toml:/gluetun/auth/config.toml"
    ports:
      - '9001:8888/tcp'
      - '8001:8000/tcp'
  vpn3:
    image: qmcgaw/gluetun
    container_name: vpn3
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=provider
      - VPN_TYPE=openvpn
      - OPENVPN_USER=
      - OPENVPN_PASSWORD=
      - HTTPPROXY=on
    volumes:
      - "vpn.toml:/gluetun/auth/config.toml"
    ports:
      - '9002:8888/tcp'
      - '8002:8000/tcp'
  vpn4:
    image: qmcgaw/gluetun
    container_name: vpn4
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=provider
      - VPN_TYPE=openvpn
      - OPENVPN_USER=
      - OPENVPN_PASSWORD=
      - HTTPPROXY=on
    volumes:
      - "vpn.toml:/gluetun/auth/config.toml"
    ports:
      - '9003:8888/tcp'
      - '8003:8000/tcp'
```

## wnrake.toml config file

```toml
solver = "http://localhost:8191/v1"
cache = "/path/to/wnrake-cache"

# default proxy
proxy = "vpn1"

[proxies]
vpn1 = { url = "http://localhost:9000", api = "http://localhost:8000", api_key = "<key>" }
vpn2 = { url = "http://localhost:9001", api = "http://localhost:8001", api_key = "<key>" }
vpn3 = { url = "http://localhost:9002", api = "http://localhost:8002", api_key = "<key>" }
vpn4 = { url = "http://localhost:9003", api = "http://localhost:8003", api_key = "<key>" }
```
