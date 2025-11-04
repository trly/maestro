# Git Authentication for Private Repositories

Maestro uses a dual-authentication approach for Git operations, preferring SSH when available but seamlessly falling back to HTTPS with Personal Access Tokens (PATs).

## Authentication Preference

1. **SSH (Preferred)**: When SSH keys are in ssh-agent, Maestro uses SSH for faster, key-based authentication
2. **HTTPS (Fallback)**: When SSH is unavailable or fails, Maestro automatically uses HTTPS with your configured PAT

This allows seamless access to both public and private repositories regardless of your authentication setup.

## Option 1: HTTPS with Personal Access Token (Recommended)

### Why HTTPS + PAT?

- **Universal**: Works in all environments without SSH agent setup
- **Simple**: Just configure token in Settings
- **Secure**: Tokens stored in system keyring
- **Automatic**: No manual key management

### Setup Steps

1. **Generate a PAT**:
   - GitHub: https://github.com/settings/tokens (requires `repo` and `workflow` scopes)
   - GitLab: https://gitlab.com/-/profile/personal_access_tokens (requires `api` and `write_repository` scopes)

2. **Configure in Maestro**:
   - Open Settings → Integrations
   - Enter your GitHub/GitLab token
   - Token is securely stored in your system keyring

3. **Done!** Maestro will automatically use HTTPS authentication for all git operations.

## Option 2: SSH Keys (Optional)

### Why SSH?

- **Faster**: Slightly faster for large repositories
- **Familiar**: Standard developer workflow
- **Preferred**: When both are configured, Maestro prefers SSH

### 1. SSH Key Setup

If you don't already have an SSH key:

```bash
# Generate ED25519 key (recommended)
ssh-keygen -t ed25519 -C "your_email@example.com"

# Or generate RSA key (alternative)
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"
```

Default locations:

- ED25519: `~/.ssh/id_ed25519` (private), `~/.ssh/id_ed25519.pub` (public)
- RSA: `~/.ssh/id_rsa` (private), `~/.ssh/id_rsa.pub` (public)

### 2. Add Public Key to GitHub

1. Copy your public key:

   ```bash
   # ED25519
   cat ~/.ssh/id_ed25519.pub | pbcopy

   # Or RSA
   cat ~/.ssh/id_rsa.pub | pbcopy
   ```

2. Go to GitHub Settings → SSH and GPG keys → New SSH key
3. Paste and save

### 3. Start SSH Agent & Add Key

```bash
# Start ssh-agent (usually already running on macOS)
eval "$(ssh-agent -s)"

# Add your SSH key to the agent
ssh-add ~/.ssh/id_ed25519
# Or for RSA:
ssh-add ~/.ssh/id_rsa

# Verify key is loaded
ssh-add -l
```

### 4. Test GitHub Connection

```bash
ssh -T git@github.com
# Should see: "Hi username! You've successfully authenticated..."
```

## How Maestro Handles Authentication

### Clone Operation

1. **SSH First** (if ssh-agent has keys):
   - Attempts clone using SSH URL: `git@github.com:owner/repo.git`
   - Uses credentials from ssh-agent
2. **HTTPS Fallback** (if SSH unavailable or fails):
   - Falls back to HTTPS URL: `https://github.com/owner/repo.git`
   - Authenticates with your configured PAT

### Fetch Operation

Same dual-authentication approach: SSH preferred, HTTPS fallback.

### Push Operation

Same dual-authentication approach: SSH preferred, HTTPS fallback. You'll see progress messages indicating which method was used.

## Troubleshooting

### Error: "GitHub/GitLab token not configured"

**Cause**: No PAT configured in Settings

**Solution**:

1. Generate a PAT with required scopes (see Option 1 above)
2. Open Maestro → Settings → Integrations
3. Enter your token
4. Token will be used automatically for HTTPS authentication

### Error: "HTTPS clone/push failed"

**Cause**: Token invalid or lacks required scopes

**Solution**:

1. Verify token has required scopes:
   - GitHub: `repo`, `workflow`
   - GitLab: `api`, `write_repository`
2. Generate new token if needed
3. Update in Settings → Integrations

### SSH Falls Back to HTTPS

**Expected Behavior**: If SSH fails (no keys in agent, wrong key, network issues), Maestro automatically retries with HTTPS.

### Multiple SSH Keys

If you have multiple SSH keys for different GitHub accounts:

1. Configure `~/.ssh/config`:

   ```
   Host github.com
       HostName github.com
       User git
       IdentityFile ~/.ssh/id_ed25519_work
       IdentitiesOnly yes
   ```

2. Restart ssh-agent and add the correct key:
   ```bash
   ssh-add -D  # Remove all keys
   ssh-add ~/.ssh/id_ed25519_work
   ```

## Technical Implementation

### Authentication Flow

1. **Check SSH Availability**:
   - Runs `ssh-add -l` to detect loaded SSH keys in agent
2. **SSH First** (if available):
   - Uses `git2::Cred::ssh_key_from_agent()` for authentication
   - Connects to ssh-agent automatically
   - No manual key path or passphrase needed
3. **HTTPS Fallback** (if SSH unavailable or fails):
   - Uses `git2::Cred::userpass_plaintext()` with username `oauth2` and PAT
   - Retrieves PAT from system keyring
   - Constructs HTTPS URL (`https://github.com/owner/repo.git`)

4. **Transparent Retry**:
   - If SSH fails, automatically retries with HTTPS
   - User sees progress messages indicating auth method used

### Why Dual Authentication?

- **Flexibility**: Works in any environment (corporate, cloud, local)
- **Convenience**: No SSH setup required for basic use
- **Security**: Tokens stored in system keyring, SSH keys in agent
- **Performance**: SSH preferred for speed when available
- **Reliability**: HTTPS fallback ensures operations never fail due to SSH issues
