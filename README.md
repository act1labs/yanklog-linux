# yanklog Linux

Open-source Linux app and shared Rust core for yanklog.

YankLog 2.0 includes complete-history paging, a redesigned keyboard-first Quick Pick,
password-encrypted backups, Secret Service key protection, sensitive-value filters,
timed pauses, safer clearing with keep-pinned support, deletion undo, onboarding,
automatic settings saving, and native tray integration.

This repository contains:

- `crates/yanklog-core`: shared Rust backend
- `apps/linux`: native Linux app built with Rust, GTK4, and libadwaita
- Linux installer, AppImage and Flatpak build workflows, reproducible build notes, and the Flathub packaging

## License

The Linux app and shared Rust core are licensed under the MIT License. See `LICENSES/MIT.txt`.

## Install

YankLog is on [Flathub](https://flathub.org/apps/com.yanklog.YankLog) for x86_64 and
aarch64. This is the recommended install, and Flathub delivers updates. Install it from
your software center, or run:

```sh
flatpak install flathub com.yanklog.YankLog
```

On systems without Flatpak, use the AppImage installer. It needs glibc 2.35 or newer,
as on Ubuntu 22.04 and later:

```sh
curl -fsSLO https://downloads.yanklog.com/install.sh
less install.sh
sh install.sh --no-path-update
```

A self-hosted Flatpak repository is also available; see
[PACKAGING_LINUX.md](PACKAGING_LINUX.md#self-hosted-repository).

## Development

On Ubuntu or another supported Linux desktop:

```sh
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev libdbus-1-dev
cargo run -p yanklog-linux-native
```
