# SSH Authentication for Private Repositories

Maestro uses SSH authentication for all GitHub repository operations (clone, fetch, push). This allows seamless access to both public and private repositories.

## Prerequisites

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

## How Maestro Uses SSH

### Clone Operation

When cloning a repository, Maestro:

1. Converts repo to SSH URL format: `git@github.com:owner/repo.git`
2. Authenticates via ssh-agent
3. Retrieves credentials automatically (no manual key path needed)

### Fetch Operation

All fetch operations use the same SSH authentication mechanism through ssh-agent integration.

## Troubleshooting

### Error: "Failed to clone repository"

**Cause**: SSH key not added to ssh-agent or not on GitHub

**Solution**:

```bash
# 1. Check if key is in agent
ssh-add -l

# 2. If not listed, add it
ssh-add ~/.ssh/id_ed25519

# 3. Verify GitHub connection
ssh -T git@github.com
```

### Error: "Permission denied (publickey)"

**Cause**: Public key not added to GitHub account or wrong key in agent

**Solution**:

1. Verify public key is on GitHub (Settings → SSH and GPG keys)
2. Ensure correct key is in ssh-agent (`ssh-add -l`)
3. Test connection: `ssh -T git@github.com`

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

1. **Clone/Fetch Request**:
   - Ensures admin repository exists and is up-to-date
   - Initiates git operation via SSH
2. **SSH Authentication**:
   - Connects to ssh-agent for credentials
   - Uses standard git SSH username ("git")
3. **Key Retrieval**:
   - Queries ssh-agent for loaded keys
   - Tries keys sequentially until one succeeds
   - No manual file path or passphrase needed

### Why SSH Over HTTPS?

- **Private repos**: No need to manage Personal Access Tokens
- **Security**: Keys stored in ssh-agent, not in code/config
- **Consistency**: Same auth for clone, fetch, and push
- **Standard practice**: Matches typical git workflow

## Future Enhancements

Potential improvements:

- HTTPS fallback with PAT for environments without SSH
- Config option to choose auth method per repository
- Better error messages with specific setup instructions
