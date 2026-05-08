# Installation

`keygen-cli` ships a single binary named `keygen`. The Homebrew formula and
the release tarballs also create a short alias, `kg`. Pick whichever path
matches your platform.

## Homebrew (macOS / Linux)

```bash
brew install okooo5km/tap/keygen-cli
```

Upgrade an existing copy:

```bash
brew update
brew upgrade okooo5km/tap/keygen-cli
```

> The tap is updated by CI a few minutes after a new tag is pushed. Right
> after a release, `brew install` may still serve the previous version until
> you run `brew update`.

## Pre-built tarballs

Each release ships signed tarballs (with a `.sha256` next to each archive)
under [Releases](https://github.com/okooo5km/keygen-cli/releases/latest).
Pick the file matching your platform.

### macOS (universal — arm64 + x86_64)

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_darwin_universal.tar.gz
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_darwin_universal.tar.gz.sha256
shasum -a 256 -c keygen-cli_${VERSION}_darwin_universal.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_darwin_universal.tar.gz
sudo mv keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
# If macOS Gatekeeper quarantines the binary:
#   xattr -d com.apple.quarantine /usr/local/bin/keygen
```

### Linux x86_64

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_linux_x86_64.tar.gz
sha256sum -c keygen-cli_${VERSION}_linux_x86_64.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_linux_x86_64.tar.gz
sudo install -m 0755 keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
```

### Linux arm64

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_linux_arm64.tar.gz
sha256sum -c keygen-cli_${VERSION}_linux_arm64.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_linux_arm64.tar.gz
sudo install -m 0755 keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
```

### Windows x86_64 (PowerShell)

```powershell
$Version = "0.2.0"
Invoke-WebRequest -Uri "https://github.com/okooo5km/keygen-cli/releases/download/v$Version/keygen-cli_${Version}_windows_x86_64.zip" -OutFile keygen.zip
Expand-Archive keygen.zip -DestinationPath "$Env:USERPROFILE\bin"
# Add %USERPROFILE%\bin to PATH if it isn't already.
```

## From source (Cargo)

Requires a Rust toolchain `>= 1.81`.

```bash
cargo install --locked --git https://github.com/okooo5km/keygen-cli --tag v0.2.0
```

Or, after cloning:

```bash
git clone https://github.com/okooo5km/keygen-cli
cd keygen-cli
cargo install --locked --path .
```

## Shell completion

```bash
keygen completion zsh  > ~/.zfunc/_keygen
keygen completion bash > ~/.local/share/bash-completion/completions/keygen
keygen completion fish > ~/.config/fish/completions/keygen.fish
```

## Skill installation

The repository bundles a Claude Code skill at `skills/keygen/`. Install it
with the wrapper script:

```bash
./skills/keygen/install.sh
```

What it does:

1. Resolves `$CLAUDE_SKILLS_DIR` (defaults to `~/.claude/skills`).
2. Removes any prior `keygen/` symlink or directory under that path.
3. Symlinks the in-repo `skills/keygen/` directory in.
4. Warns if the `keygen` binary is missing from `$PATH`.

Restart Claude Code (or run `/reload`) afterwards so the new skill metadata
is picked up.

## Verify the install

```bash
keygen --version
keygen doctor                # probes connectivity + capabilities
keygen schema --format json  # full command tree, useful for agents
```
