# [![Slice: 4D Shooter](https://github.com/maffi44/slice-4d-shooter/blob/main/media/slice_4d_shooter_poster.png)](https://slice4d.info)
# Slice: 4D Shooter - multiplayer shooter set in 4D space

### To build the project, you need to install the rust programming language and the cargo utility. 

For unix-like operating systems:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Windows, you need to download `rustup-init.exe` according to [this link](https://rust-lang.github.io/rustup/installation/other.html).


## Build game client on multiple operating systems

### Debian-based Linux distros

Install dependencies:

```bash
apt install build-essential pkg-config libasound2-dev
```

Build the game client:

```bash
cargo build --release --bin game-client
```

### RHEL-based Linux distros

Install dependencies:

```bash
dnf install gcc make glibc-devel pkgconf-pkg-config alsa-lib-devel
```

Build the game client:

```bash
cargo build --release --bin game-client
```

### macOS (Apple Silicon)

Build the game client:

```bash
cargo build --release --bin game-client --target aarch64-apple-darwin
```

### Windows 10/11

Install dependencies:

```powershell
rustup target add x86_64-pc-windows-msvc
```

Build the game client:

```powershell
cargo build --release --bin game-client-without-autoupdate --target x86_64-pc-windows-msvc
```

---

## Launching the game

After compiling and launching the game client (created in the `target` directory), shader compilation is underway, which may take some time, depending on the performance of your computer.

If the FPS of the game client is **too low**, you can press the `NumPad 7` key to degrade the graphics rendering parameters.

Currently, the client can make several connection attempts until success is achieved.

If you want to play online on your own servers, you need to create a game-client `settings.json` executable file in the same directory, and enter the ipv4 address and port of the matchmaking server in the `matchmaking_server_url` field. Playing on the official Slice 4D Shooter servers does **not require** a settings.json file.

```bash
tee settings.json > /dev/null <<EOF
{
    "mouse_sensivity" : 0.21,

    "matchmaking_server_url" : "ws://8.8.8.8:45123/",

    "bash_and_turn_servers" : [],

    "turn_server_username" : "homeo",
    "turn_server_credential" : "homeo",
    
    "screen_resolution_scale" : 1.0
}
EOF
```

The address `8.8.8.8` needs to be replaced with your host's address.

---

## Hosting your own servers

#### While on the game node, follow these steps:

#### 1) Clone the repository:

```bash
git clone https://github.com/maffi44/slice-4d-shooter.git
```

#### 2) Install dependencies:

For RHEL-based Linux distros

```bash
dnf install gcc make glibc-devel pkgconf-pkg-config alsa-lib-devel
```

For Debian-based Linux distros

```bash
apt install build-essential pkg-config libasound2-dev
```

#### 3) Compile matchmaking server

```bash
cd slice-4d-shooter
```
```bash
cargo build --release -p matchmaking_server
```

#### 4) Compile game server

```bash
cargo build --release -p game_server
```

Matchmaking server and game server **must** be located in the same directory.

#### 5) Create the matchmaking-server-config configuration file.json for the matchmaking server

Go to the directory where matchmaking_server is located.

```bash
cd target/x86_64-unknown-linux-gnu/release
```

Сreate a matchmaking-server-config.json file.

```bash
tee matchmaking-server-config.json > /dev/null <<EOF
{
  "current_game_version": "0.5.2",
  "matchmaking_server_ip": "8.8.8.8",
  "matchmaking_server_port_for_clients": 45123,
  "matchmaking_server_port_for_servers": 45124,
  "clients_connecting_via_proxy_server": false,
  "proxy_server_ip": "8.8.8.8",
  "proxy_server_port": 45123,
  "game_severs_public_ip": "8.8.8.8",
  "game_severs_min_port_for_signaling_servers": 45125,
  "game_severs_max_port_for_signaling_servers": 46125,
  "game_severs_min_port_for_tcp_listener": 45125,
  "game_severs_max_port_for_tcp_listener": 46125,
  "game_severs_ice_config": {
    "urls": [
      "stun:stun.l.google.com:19302",
      "stun:stun1.l.google.com:19302"
    ],
    "username": "homeo",
    "credential": "homeo"
  },
  "max_game_sessions": 10,
  "max_players_per_game_session": 6
}
EOF
```

The address `8.8.8.8` **needs to be replaced** with your host's address.

---

### Configuration fields explained

The table below describes the purpose of each field in the `matchmaking-server-config.json` file:

| Field                                        | Description                                                                                                                                                         |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `current_game_version`                       | The current game version that clients must match to connect.                                                                                                         |
| `matchmaking_server_ip`                      | The **external IPv4 address** of the matchmaking server. Clients and game servers will connect to this IP.                                                          |
| `matchmaking_server_port_for_clients`        | Port used by **clients** to connect to the matchmaking server.                                                                                                      |
| `matchmaking_server_port_for_servers`        | Port used by **game servers** to connect to the matchmaking server.                                                                                                 |
| `clients_connecting_via_proxy_server`        | Whether a **proxy server** is used between clients and game servers. Set to `true` if clients connect via a proxy. The default value is `false`.                                                 |
| `proxy_server_ip`                            | IP address of the proxy server (used only if `clients_connecting_via_proxy_server` is `true`).                                                                      |
| `proxy_server_port`                          | Port on which the proxy server listens (used only if proxy is enabled).                                                                                             |
| `game_severs_public_ip`                      | Public IP address of the game servers. Currently, this should be the **same as** `matchmaking_server_ip` since multi-node game server hosting is not yet supported. |
| `game_severs_min_port_for_signaling_servers` | **Start of the port range** used by game servers for signaling (WebRTC communication setup).                                                                        |
| `game_severs_max_port_for_signaling_servers` | **End of the port range** used by game servers for signaling.                                                                                                       |
| `game_severs_min_port_for_tcp_listener`      | **Start of the port range** used by game servers for TCP listener (actual data exchange).                                                                           |
| `game_severs_max_port_for_tcp_listener`      | **End of the port range** used by game servers for TCP listener.                                                                                                    |
| `game_severs_ice_config`                     | Configuration for WebRTC ICE servers. You can specify your own **STUN** and/or **TURN** servers here (e.g., via [coturn](https://github.com/coturn/coturn)).        |
| `max_game_sessions`                          | The maximum number of **concurrent game sessions** (i.e., game server instances) that the matchmaking server can handle.                                                                          |
| `max_players_per_game_session`               | The maximum number of **players per game session** (i.e., per game server instance).                                                                                |

---

### Using a Proxy Server (e.g., NGINX)

If you **do not want to use a proxy server**, such as NGINX, which sits between the players and the game servers, then set the field `clients_connecting_via_proxy_server` to `false`.

If you **do want to use a proxy**, set `clients_connecting_via_proxy_server` to `true` and make sure to fill in the `proxy_server_ip` and `proxy_server_port` fields accordingly.

Below is an example NGINX configuration that acts as a WebSocket proxy between players and game servers. This example uses Lua to restrict access only to a specific port range.

The allowed port range in the Lua block **must match** the ranges defined in your `matchmaking-server-config.json` file:

```json
"game_severs_min_port_for_signaling_servers": 45125,
"game_severs_max_port_for_signaling_servers": 46125,
"game_severs_min_port_for_tcp_listener": 45125,
"game_severs_max_port_for_tcp_listener": 46125
```

#### Example NGINX Config (with Lua port filtering)

```nginx
# path: /etc/nginx/sites-enabled/your-config.conf

server {
    listen 45123;

    location ~ ^/ws/(\d+)$ {
        set $target_port $1;

        access_by_lua_block {
            local port = tonumber(ngx.var.target_port)
            if not port or port < 45125 or port > 46125 then
                ngx.status = 403
                ngx.say("Port out of allowed range")
                return ngx.exit(ngx.HTTP_FORBIDDEN)
            end
        }

        rewrite ^/ws/\d+$ / break;

        proxy_pass http://127.0.0.1:$target_port;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 3600;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

In this example, only ports `45125` to `46125` are allowed.  
Make sure to update this range to match your actual configured ranges in `matchmaking-server-config.json`.

#### Additional Configuration Fields

| Field | Description |
|-------|-------------|
| `clients_connecting_via_proxy_server` | Whether a **proxy server** (e.g., NGINX) will be used between clients and game servers. Set to `true` to enable proxying, `false` to allow direct connections. |
| `proxy_server_ip` | IP address of the proxy server that clients will connect to (if proxying is enabled). |
| `proxy_server_port` | Port on which the proxy server listens for client connections. Must match the `listen` directive in the NGINX config (e.g., `45123`). |
