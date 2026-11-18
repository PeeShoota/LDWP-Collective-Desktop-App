#!/usr/bin/env bash
# Build the Flatpak bundle and its SHA256SUMS into linux/dist/.
#
# Requirements (user installs, no root needed):
#   flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
#   flatpak install --user flathub org.flatpak.Builder
# The GNOME 51 SDK and Rust extension are installed by this script if missing.
set -euo pipefail

cd "$(dirname "$0")"

APP_ID=org.ldwp.CollectiveDesktop
MANIFEST="$APP_ID.yml"
BUNDLE="LDWP-Collective-Desktop-App-x86_64.flatpak"

if command -v flatpak-builder >/dev/null 2>&1; then
    BUILDER=(flatpak-builder)
else
    # The sandboxed builder forwards its own XDG_DATA_HOME (~/.var/app/...)
    # to host-side `flatpak` calls, which hides --user runtimes. `flatpak run
    # --env` cannot override it, so reset it inside the sandbox instead.
    BUILDER=(flatpak run --command=sh org.flatpak.Builder -c \
        'd=$1; shift; XDG_DATA_HOME=$d exec flatpak-builder "$@"' sh \
        "${XDG_DATA_HOME:-$HOME/.local/share}")
fi

# Keep vendored crate list in sync with Cargo.lock.
python3 build-aux/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json

# Install the SDK/runtime/Rust extension (no-op if already present).
flatpak install --user --noninteractive flathub \
    org.gnome.Platform//51 org.gnome.Sdk//51 \
    org.freedesktop.Sdk.Extension.rust-stable//26.08

"${BUILDER[@]}" --user --force-clean \
    --repo=repo build-dir "$MANIFEST"

mkdir -p dist
flatpak build-bundle repo "dist/$BUNDLE" "$APP_ID" \
    --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo

(cd dist && sha256sum "$BUNDLE" > SHA256SUMS)

echo
echo "Bundle:   $(pwd)/dist/$BUNDLE"
echo "Checksum: $(cat dist/SHA256SUMS)"
