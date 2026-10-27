# yanklog Linux

Open-source Linux app and shared Rust core for yanklog.

YankLog keeps a searchable history of the text you copy, in an encrypted database on
your device. Clipboard content is never sent anywhere. As of 2.5:

- **History**: search everything you have copied, grouped into Pinned, Today,
  Yesterday and Earlier. Pin clips, undo deletions, and clear history while keeping
  pinned items.
- **Quick Pick**: a keyboard-first picker you open from a desktop shortcut, with a
  full-text preview of the highlighted clip.
- **Privacy**: filters skip secret-like values, one-time codes and words you list, and
  history can be capped by size and age. Pause capture for 15 minutes, an hour, until
  tomorrow or indefinitely; a pause survives a restart.
- **Key storage**: the Flatpak keeps the database key through the Secret portal. The
  AppImage uses the Secret Service, with a protected local key file as a fallback.
- **Backups**: password-encrypted backups of the full history or pinned items only,
  with or without settings.
- **Desktop**: the Flatpak captures the clipboard on Wayland, GNOME included, without
  X11. A tray icon is optional; the main window's menu has Quit (Ctrl+Q). Settings
  save as you change them, with light, dark and system themes and optional start at
  login.
- **Updates**: Flathub updates the Flatpak; the AppImage checks for new releases
  itself.

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
