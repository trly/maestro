# Distribution Guide

## Prerequisites

### Apple Developer Account

You need a **paid Apple Developer account** ($99/year) to:
- Sign apps with "Developer ID Application" certificate
- Notarize apps for distribution outside the App Store
- Prevent "unverified developer" warnings on user machines

### Required Certificates

Create a **Developer ID Application** certificate:
1. Generate a Certificate Signing Request (CSR) on your Mac:
   - Open Keychain Access → Certificate Assistant → Request a Certificate from a Certificate Authority
   - Save the CSR file
2. Go to [Apple Developer Certificates](https://developer.apple.com/account/resources/certificates/list)
3. Click "Create a certificate"
4. Choose **Developer ID Application**
5. Upload your CSR and download the certificate

**Note:** Only the Account Holder can create Developer ID Application certificates.

## GitHub Actions Setup

### Step 1: Export Certificate for CI/CD

Export your certificate as a `.p12` file:

```bash
# 1. Open Keychain Access
# 2. Go to "My Certificates" in the "login" keychain
# 3. Find your "Developer ID Application" certificate
# 4. Right-click on the key item → Export
# 5. Save as certificate.p12 with a strong password
```

Convert to base64:

```bash
openssl base64 -in certificate.p12 -out certificate-base64.txt
```

### Step 2: Create App-Specific Password

For notarization, you need an app-specific password:

1. Go to [Apple ID account page](https://appleid.apple.com/account/manage)
2. Sign in
3. Security → App-Specific Passwords → Generate password
4. Label it "GitHub Actions - Maestro" and copy the password

### Step 3: Configure GitHub Secrets

Add these secrets to your GitHub repository (Settings → Secrets and variables → Actions):

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `APPLE_CERTIFICATE` | Content of `certificate-base64.txt` | Base64-encoded .p12 file |
| `APPLE_CERTIFICATE_PASSWORD` | Your .p12 password | Password set when exporting |
| `KEYCHAIN_PASSWORD` | Any strong password | Temporary keychain password for CI |
| `APPLE_ID` | your.email@example.com | Your Apple ID email |
| `APPLE_PASSWORD` | xxxx-xxxx-xxxx-xxxx | App-specific password |
| `APPLE_TEAM_ID` | XXXXXXXXXX | Find at [Membership Details](https://developer.apple.com/account#MembershipDetailsCard) |

### Step 4: Trigger a Release

Create a new release by pushing a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This will:
1. Build for macOS (Apple Silicon + Intel) and Linux
2. Code sign the macOS builds
3. Notarize the macOS builds with Apple
4. Upload all artifacts to GitHub Releases

## Manual Distribution

### macOS

For local builds that you want to distribute:

1. **Set signing identity** in [tauri.conf.json](../src-tauri/tauri.conf.json):
   ```json
   {
     "bundle": {
       "macOS": {
         "signingIdentity": "Developer ID Application: Your Name (TEAMID)"
       }
     }
   }
   ```

2. **Set notarization credentials** (choose one method):

   **Option A: Apple ID** (easier, less secure)
   ```bash
   export APPLE_ID="your.email@example.com"
   export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"  # App-specific password
   export APPLE_TEAM_ID="XXXXXXXXXX"
   ```

   **Option B: App Store Connect API** (more secure)
   ```bash
   export APPLE_API_ISSUER="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
   export APPLE_API_KEY="XXXXXXXXXX"
   export APPLE_API_KEY_PATH="/path/to/AuthKey_XXXXXXXXXX.p8"
   ```

3. **Build**:
   ```bash
   bun run build
   ```

The signed and notarized `.dmg` will be in `src-tauri/target/release/bundle/dmg/`.

### Linux

No code signing required:

```bash
bun run build
```

The `.deb` package will be in `src-tauri/target/release/bundle/deb/`.

## Troubleshooting

### "No signing identity found"

**Solution:** Run `security find-identity -v -p codesigning` to verify your certificate is installed.

### "Notarization failed"

**Possible causes:**
- Invalid Apple ID or app-specific password
- Wrong Team ID
- Certificate doesn't match Team ID

**Solution:** Verify all credentials in GitHub Secrets match your Apple Developer account.

### "Keychain prompts on user machines"

**Cause:** App wasn't properly signed or notarized.

**Solution:** Ensure the workflow completes notarization successfully (check GitHub Actions logs).

## Security Best Practices

1. **Never commit certificates or passwords** to the repository
2. **Use app-specific passwords** instead of your main Apple ID password
3. **Rotate credentials** if they're ever exposed
4. **Limit repository access** to trusted collaborators
5. **Review GitHub Actions logs** carefully (secrets are masked but be cautious)

## Distribution Channels

### GitHub Releases (Current)

Users download `.dmg` (macOS) or `.deb` (Linux) from GitHub Releases page.

**Pros:**
- Simple, no approval process
- Direct control over releases
- Fast iteration

**Cons:**
- Users must manually check for updates
- No built-in update mechanism

### Future: Auto-Updates

Tauri supports auto-updates via the updater plugin. To enable:

1. Add `tauri-plugin-updater` to dependencies
2. Configure update server URL in `tauri.conf.json`
3. App will check for updates on launch

See [Tauri Updater docs](https://v2.tauri.app/plugin/updater/) for details.

### Mac App Store (Optional)

For broader distribution, you can submit to the Mac App Store:

1. Use **Apple Distribution** certificate instead of Developer ID
2. Create an App Store listing
3. Submit for review (1-2 weeks approval time)

**Trade-offs:** More reach, but Apple's review process and 30% commission on paid apps.
