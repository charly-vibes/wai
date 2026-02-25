#!/usr/bin/env python3
"""Update a goreleaser-generated Homebrew formula with new version, URLs, and SHAs."""
import re
import sys

formula_path = sys.argv[1]
version = sys.argv[2]
tag = sys.argv[3]
checksums_path = sys.argv[4]

shas = {}
with open(checksums_path) as f:
    for line in f:
        parts = line.split()
        if len(parts) == 2:
            sha, name = parts
            for platform in ["darwin_arm64", "darwin_amd64", "linux_arm64", "linux_amd64"]:
                if platform in name:
                    shas[platform] = sha

base_url = f"https://github.com/charly-vibes/wai/releases/download/{tag}"

with open(formula_path) as f:
    lines = f.readlines()

result = []
prev_platform = None
for line in lines:
    # Track platform from URL lines
    url_match = re.search(r'url "https://[^"]*?((?:darwin|linux)_(?:arm64|amd64))', line)
    if url_match:
        platform = url_match.group(1)
        filename = f"wai_{version}_{platform}.tar.gz"
        line = re.sub(r'url ".*?"', f'url "{base_url}/{filename}"', line)
        prev_platform = platform
    elif prev_platform and re.search(r'sha256 "', line):
        if prev_platform in shas:
            line = re.sub(r'sha256 ".*?"', f'sha256 "{shas[prev_platform]}"', line)
        prev_platform = None
    # Update version line
    if re.search(r'^\s*version "', line):
        line = re.sub(r'version ".*?"', f'version "{version}"', line)
    result.append(line)

with open(formula_path, "w") as f:
    f.writelines(result)

print(f"Updated {formula_path} to version {version}")
for p, s in shas.items():
    print(f"  {p}: {s[:16]}...")
