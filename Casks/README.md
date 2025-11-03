# Maestro Homebrew Tap

Homebrew tap for installing Maestro via `brew`.

## Installation

```bash
brew tap trly/maestro https://github.com/trly/maestro
brew install maestro
```

## Updating the Cask (for Maintainers)

When releasing a new version:

1. **Download the DMG** from the GitHub release
2. **Calculate SHA256**:
   ```bash
   shasum -a 256 Maestro_X.Y.Z_aarch64.dmg
   ```
   Or calculate remotely:
   ```bash
   curl -sL https://github.com/trly/maestro/releases/download/vX.Y.Z/Maestro_X.Y.Z_aarch64.dmg | shasum -a 256
   ```

3. **Update the cask file** (`Casks/maestro.rb`):
   - Change `version` to the new version (e.g., `"0.4.5"`)
   - Update `sha256` with the calculated hash
   
4. **Commit and push**:
   ```bash
   git add homebrew/Casks/maestro.rb
   git commit -m "chore: bump homebrew cask to vX.Y.Z"
   git push
   ```

5. **Verify the update**:
   ```bash
   brew update
   brew upgrade maestro
   ```

## First Launch on macOS

Due to the app being unsigned (no Apple Developer certificate), users must bypass Gatekeeper on first launch:

1. Right-click `Maestro.app` in Applications
2. Select "Open"
3. Click "Open" in the security dialog

Or via terminal:
```bash
xattr -cr /Applications/Maestro.app
open /Applications/Maestro.app
```

Subsequent launches will work normally.
