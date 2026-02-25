#!/usr/bin/env python3
"""Update a goreleaser-generated Scoop manifest with new version, URL, and SHA."""
import json
import sys

manifest_path = sys.argv[1]
version = sys.argv[2]
tag = sys.argv[3]
checksums_path = sys.argv[4]

sha_win = None
with open(checksums_path) as f:
    for line in f:
        parts = line.split()
        if len(parts) == 2 and "windows_amd64" in parts[1]:
            sha_win = parts[0]
            break

if sha_win is None:
    print("ERROR: windows_amd64 checksum not found", file=sys.stderr)
    sys.exit(1)

url_win = f"https://github.com/charly-vibes/wai/releases/download/{tag}/wai_{version}_windows_amd64.zip"

with open(manifest_path) as f:
    manifest = json.load(f)

manifest["version"] = version
manifest["url"] = url_win
manifest["hash"] = f"sha256:{sha_win}"

with open(manifest_path, "w") as f:
    json.dump(manifest, f, indent=4)
    f.write("\n")

print(f"Updated {manifest_path} to version {version}")
print(f"  url: {url_win}")
print(f"  sha256: {sha_win[:16]}...")
