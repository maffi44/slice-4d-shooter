# [![Slice: 4D Shooter](https://github.com/maffi44/slice-4d-shooter/blob/main/media/slice_4d_shooter_poster.png)](https://slice4d.info)
# Slice: 4D Shooter - multiplayer shooter set in 4D space

## Build game client on multiple operating systems

### To build the project, you need to install the rust programming language and the cargo utility. 

For unix-like operating systems:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Windows, you need to download `rustup-init.exe` according to [this link](https://rust-lang.github.io/rustup/installation/other.html).


## Debian-based Linux distros

Install dependencies:

```bash
apt install build-essential pkg-config libasound2-dev
```

Build the game client:

```bash
cargo build --release --bin game-client
```

## RHEL-based Linux distros

Install dependencies:

```bash
dnf install gcc make glibc-devel pkgconf-pkg-config alsa-lib-devel
```

Build the game client:

```bash
cargo build --release --bin game-client
```

## macOS (Apple Silicon)

Build the game client:

```bash
cargo build --release --bin game-client --target aarch64-apple-darwin
```

## Windows 10/11

Install dependencies:

```powershell
rustup target add x86_64-pc-windows-msvc
```

Build the game client:

```powershell
cargo build --release --bin game-client-without-autoupdate --target x86_64-pc-windows-msvc
```