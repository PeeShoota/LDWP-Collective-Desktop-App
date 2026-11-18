# LDWP Collective Desktop App

The official desktop application for the **LDWP Collective** project.

A simple desktop wrapper around [ldwp.org](https://ldwp.org), available for:

- **Windows**: a **WebView2**-based app created in Visual Studio LTSC 18.11 (source in [`LDWP Collective Desktop App/`](LDWP%20Collective%20Desktop%20App/))
- **Linux**: a **Rust + GTK 4 + WebKitGTK** app distributed as a Flatpak (source in [`linux/`](linux/))

## Windows

### Installation

#### Version 1.0.2 and newer
1. Download the latest `LDWP-Collective-Desktop-App-Setup.exe`
2. Run the installer and follow the steps

#### Older versions (Pre 1.0.2)
1. Download the release zip
2. Extract the zip anywhere and put the folder somewhere safe
3. Run `LDWP Collective Desktop App.exe`

### Prerequisites

- **Windows 10** or **Windows 11**
- **.NET 10** (`net10.0-windows`)
- **Microsoft.Web.WebView2** (version 1.0.3650.58 or compatible)

> **Note for Windows users:** This app has only been built and tested on my daily driver which runs **Windows 10 IoT Enterprise LTSC 21H2**. It should still work on most other modern Windows versions if you have the dependencies needed, but that hasn't been verified.

## Linux

The Linux version is a native reimplementation in Rust using GTK 4 and WebKitGTK. It was built and tested on **Fedora 45 (KDE Plasma)**, and should work on any distribution with Flatpak.

### Installation

1. Download `LDWP-Collective-Desktop-App-x86_64.flatpak` and `SHA256SUMS` from the latest release
2. (Optional) Verify the download:
   ```sh
   sha256sum -c SHA256SUMS
   ```
3. Make sure Flathub is set up (the app uses the GNOME 51 runtime from Flathub):
   ```sh
   flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
   ```
4. Install the bundle:
   ```sh
   flatpak install --user LDWP-Collective-Desktop-App-x86_64.flatpak
   ```

To uninstall: `flatpak uninstall --user org.ldwp.CollectiveDesktop`

### Usage

Launch **LDWP Collective** from your application menu, or run:

```sh
flatpak run org.ldwp.CollectiveDesktop
```

- Your sign-in, cookies and site data are kept between sessions (stored in `~/.var/app/org.ldwp.CollectiveDesktop/`).
- Downloads are saved to your `~/Downloads` folder. A bar at the bottom of the window shows progress, with **Open** and **Show in Folder** buttons when a download finishes.
- Links to LDWP pages that would open a new window stay in the app. Links to other websites open in your default browser.
- Keyboard shortcuts:

  | Shortcut | Action |
  | --- | --- |
  | `F5` / `Ctrl+R` | Reload |
  | `Ctrl+F5` / `Ctrl+Shift+R` | Reload, bypassing the cache |
  | `Alt+Left` / `Alt+Right` | Back / Forward |
  | `Alt+Home` | Go to the LDWP home page |
  | `Ctrl++` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset |
  | `F11` | Toggle fullscreen |
  | `F12` / `Ctrl+Shift+I` | Web inspector |

### Building from source

Requirements: `flatpak` and `python3`. Everything else is installed as user Flatpaks, so you don't need root.

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.flatpak.Builder
cd linux
./build-flatpak.sh
```

The script installs the GNOME 51 SDK and the Rust extension if they're missing, builds the app, and writes these files to `linux/dist/`:

- `LDWP-Collective-Desktop-App-x86_64.flatpak`: the installable bundle
- `SHA256SUMS`: its checksum

If you change dependencies, the script regenerates `cargo-sources.json` from `Cargo.lock` with `build-aux/flatpak-cargo-generator.py`, because Flatpak builds run offline.

To build without Flatpak, you need the GTK 4 and WebKitGTK 6.0 development packages (on Fedora: `sudo dnf install gtk4-devel webkitgtk6.0-devel`). Then run `cargo run --release` inside `linux/`.

### Differences from the Windows version

- **WebView2 runtime installer:** not needed. WebKitGTK comes with the Flatpak runtime, so the Linux version has no auto-install step.
- **Download UI:** WebView2 has a built-in download flyout and WebKitGTK does not, so the Linux app has its own download bar.
- **New-window links to other websites** open in your default browser instead of a popup window.

## Screenshots (Windows)

The Desktop download page... In the desktop app:
<img width="1202" height="832" alt="image" src="https://github.com/user-attachments/assets/58618d1b-e073-4aa3-a4c0-c4e14c20f6d1" />

What downloading a package looks like:
<img width="2207" height="1024" alt="image" src="https://github.com/user-attachments/assets/41131e58-390d-4d28-b4a6-a3181cfc12d8" />

## Contributors

- **XiaoLing6028**: created the Linux version (Rust / GTK 4 / WebKitGTK Flatpak)
