#!/usr/bin/env python3
"""Generate flatpak-builder sources for every crate in Cargo.lock.

Flatpak builds run without network access, so all crates are downloaded by
flatpak-builder up front and vendored into $CARGO_HOME. Standard-library only
(Python 3.11+ for tomllib); supports crates.io dependencies, which is all this
project uses.

Usage: build-aux/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
"""
import argparse
import json
import sys
import tomllib

CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
VENDOR_DIR = "cargo/vendor"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("lockfile")
    parser.add_argument("-o", "--output", default="cargo-sources.json")
    args = parser.parse_args()

    with open(args.lockfile, "rb") as f:
        lock = tomllib.load(f)

    sources = []
    for pkg in lock.get("package", []):
        source = pkg.get("source")
        if source is None:
            continue  # the workspace's own crate(s)
        if source != CRATES_IO:
            print(f"error: unsupported source for {pkg['name']}: {source}", file=sys.stderr)
            return 1
        name, version, checksum = pkg["name"], pkg["version"], pkg["checksum"]
        dest = f"{VENDOR_DIR}/{name}-{version}"
        sources.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        })
        sources.append({
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        })

    sources.append({
        "type": "inline",
        "contents": (
            "[source.vendored-sources]\n"
            f'directory = "{VENDOR_DIR}"\n\n'
            "[source.crates-io]\n"
            'replace-with = "vendored-sources"\n'
        ),
        "dest": "cargo",
        "dest-filename": "config.toml",
    })

    with open(args.output, "w") as f:
        json.dump(sources, f, indent=4)
        f.write("\n")
    print(f"Wrote {len(sources)} sources to {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
