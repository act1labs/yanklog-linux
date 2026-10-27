# Flatpak Packaging

This directory contains the upstream Flatpak packaging for YankLog. The Flatpak ID is
`com.yanklog.YankLog`; the older starter ID `com.yanklog.app` must not be submitted.

## Build

From this directory:

```sh
flatpak-builder --force-clean build-dir com.yanklog.YankLog.yml
flatpak-builder --user --install --force-clean build-dir com.yanklog.YankLog.yml
flatpak run com.yanklog.YankLog
```

Install the matching runtime and SDK first. The manifest downloads its own Rust
toolchain, so no SDK extension is needed:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Platform//51 org.gnome.Sdk//51
```

The upstream manifest builds from this checkout and uses `generated-sources.json` for
offline Cargo dependencies. The Flathub manifest, kept in
[`flathub/com.yanklog.YankLog`](https://github.com/flathub/com.yanklog.YankLog) with a
reference copy in [`../flathub/`](../flathub/), replaces the `dir` source with the public
release tag and its exact commit.

## Sandbox Permissions

- `--socket=wayland` is the clipboard path on Wayland: the app reads copies through GTK,
  without X11. `--socket=fallback-x11` grants X11 only in X11 sessions, where GTK and the
  clipboard use X11 anyway. `--share=ipc` is required by GTK.
- `--device=dri` enables accelerated GTK rendering.
- The encrypted database key is kept through the Secret portal, which needs no D-Bus
  permission and works with any password manager.
- `org.kde.StatusNotifierWatcher` exposes the tray/status-notifier menu. The tray registers
  under the connection's unique name (`ksni`'s `disable_dbus_name`), because Flatpak's dbus
  proxy cannot grant ownership of the dynamic `org.kde.StatusNotifierItem-<pid>-<id>` name
  and Flatpak rejects `--own-name=org.kde.StatusNotifierItem-*` as an invalid D-Bus name.
- The XDG Background portal handles the user's optional start-at-login approval.

The Flatpak has no network or host-filesystem permission. Backup import and export use
the GTK file chooser portal. Self-updates and host autostart-file writes are disabled;
updates are delivered by Flatpak.

## Regenerate Cargo Sources

After `Cargo.lock` changes, regenerate `generated-sources.json`:

```sh
curl -fsSLO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
python3 -m pip install --user tomlkit aiohttp
python3 flatpak-cargo-generator.py ../../Cargo.lock -o generated-sources.json
rm flatpak-cargo-generator.py
```

## Flathub

YankLog is published on Flathub. Releases reach it through pull requests in
[`flathub/com.yanklog.YankLog`](https://github.com/flathub/com.yanklog.YankLog), which
Flathub's external data checker opens when it finds a new `v*` tag. See
[Releasing To Flathub](../../PACKAGING_LINUX.md#releasing-to-flathub) for the steps,
including when to regenerate `generated-sources.json`.
