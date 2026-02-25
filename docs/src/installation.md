# Installation

The easiest way to install `wai` is using a package manager.

## Homebrew (macOS & Linux)

```bash
brew tap charly-vibes/charly
brew install wai
```

## Scoop (Windows)

```powershell
scoop bucket add charly https://github.com/charly-vibes/scoop-charly.git
scoop install wai
```

## From Binary (All Platforms)

Download the latest pre-compiled binary for your architecture from the [GitHub Releases](https://github.com/charly-vibes/wai/releases) page.

1. Unpack the archive.
2. Move the `wai` binary to a directory in your PATH (e.g., `/usr/local/bin` or `C:\Windows\System32`).

## From Source (Rust Developers)

Requires [Rust](https://www.rust-lang.org/tools/install) (stable toolchain).

```bash
cargo install --path .
```

This installs the `wai` binary to `~/.cargo/bin/`.

## Verify Installation

```bash
wai --version
wai --help
```
