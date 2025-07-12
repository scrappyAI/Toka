# Toka Dev Container Setup Guide

## 🚀 Quick Start for New Contributors

This dev container provides a fully configured Rust development environment with GitHub authentication for secure workspace access.

### Prerequisites

1. **Docker Desktop** - Install and ensure it's running
2. **Dev Container Compatible IDE** - Either Cursor or VS Code with Dev Containers extension
3. **GitHub Account** - You'll need access to authenticate

### 🧪 Testing Your Setup

We provide two test scripts to validate different parts of your setup:

#### 🖥️ Test Your Host Machine (Run This First)
```bash
bash .devcontainer/test-host.sh
```
This validates that your local machine has the prerequisites for dev containers (Docker, IDE, etc.).

#### 🐳 Test the Dev Container (Run Inside Container)
```bash
bash .devcontainer/test-setup.sh
```
This validates the dev container environment itself. **Only run this inside the dev container.**

### 🎯 IDE Support

This dev container works with both major dev container IDEs:

#### **Cursor (Recommended for AI Development)**
- ✅ **Native dev container support**
- ✅ **Built-in AI assistance**
- ✅ **Modern interface**
- 📦 **Download:** https://cursor.sh/

#### **VS Code (Traditional Choice)**
- ✅ **Mature dev container ecosystem**
- ✅ **Rich extension marketplace**
- ✅ **Wide community support**
- 📦 **Download:** https://code.visualstudio.com/
- 🔧 **Required Extension:** Dev Containers

### 🔐 Authentication Setup

The dev container supports multiple authentication methods for security and convenience:

#### Method 1: Environment Variables (Recommended for Local Development)

Create a `.env.local` file in your home directory or project root:

```bash
# GitHub Authentication
GITHUB_TOKEN=ghp_your_github_token_here

# LLM API Keys (optional)
ANTHROPIC_API_KEY=your_anthropic_key_here
# OR
OPENAI_API_KEY=your_openai_key_here
LLM_PROVIDER=anthropic  # or openai
```

**Getting a GitHub Token:**
1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select scopes: `repo`, `read:org`, `read:user`, `gist`
4. Copy the token to your `.env.local` file

#### Method 2: Interactive Authentication (GitHub Device Flow)

If no `GITHUB_TOKEN` is provided, the container will prompt you to authenticate via GitHub's device flow:

1. The container will display a device code
2. Open the provided GitHub URL in your browser
3. Enter the device code and authorize the application
4. Authentication will complete automatically

### 🏗️ Starting the Dev Container

1. **Test your host environment:**
   ```bash
   bash .devcontainer/test-host.sh
   ```
   Fix any issues before proceeding.

2. **Clone the repository:**
   ```bash
   git clone https://github.com/ScrappyAI/toka.git
   cd toka
   ```

3. **Set up authentication (optional but recommended):**
   ```bash
   cp .devcontainer/env.local.template .env.local
   # Edit .env.local with your tokens
   ```

4. **Open in your IDE:**

   **Using Cursor:**
   ```bash
   cursor .
   ```

   **Using VS Code:**
   ```bash
   code .
   ```

5. **Start the Dev Container:**
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
   - Type "Dev Containers: Reopen in Container"
   - Select it and wait for the container to build

6. **Test the container setup:**
   ```bash
   bash .devcontainer/test-setup.sh
   ```

### 🔧 What's Included

- **Rust Development Environment:**
  - Latest stable Rust toolchain
  - Essential tools: `clippy`, `rustfmt`, `rust-analyzer`
  - Cargo extensions: `cargo-watch`, `cargo-expand`, `cargo-audit`

- **GitHub Integration:**
  - GitHub CLI with authentication
  - Git configuration with your GitHub identity
  - Helpful aliases for common operations

- **Development Tools:**
  - Python 3 with pip (for project scripts)
  - Modern CLI tools: `exa`, `bat`, `ripgrep`, `fd`
  - Development utilities: `htop`, `jq`, `tree`

- **IDE Extensions:**
  - Rust analyzer for intelligent code completion
  - GitHub Pull Requests and Issues (if supported)
  - Debugging tools and test runners

### 🎯 Quick Commands

Once in the container, you can use these helpful commands:

```bash
# Development shortcuts
build-all          # Build entire workspace
test-all           # Run all tests
check-all          # Run cargo check on all crates
toka-quick-test    # Quick workspace validation

# GitHub operations
gh repo view       # View repository information
gh pr list         # List pull requests
gh issue list      # List issues
pv                 # Alias for 'gh pr view'
pc                 # Alias for 'gh pr create'

# Rust development
cb                 # cargo build
ct                 # cargo test
cc                 # cargo check
cf                 # cargo fmt
ccl                # cargo clippy
```

### 🔒 Security Features

- **Secure Authentication:** GitHub tokens are never stored in the repository
- **Environment Isolation:** Your credentials are isolated to your container instance
- **Minimal Permissions:** GitHub tokens use minimal required scopes
- **Auto-rotation Support:** Easy token renewal process

### 🌐 Multi-User Safety

This dev container is designed to be safely shared across team members:

- **No Hardcoded Secrets:** All authentication is user-specific
- **Isolated Environments:** Each user gets their own container instance
- **Version Controlled Setup:** Container configuration is tracked in git
- **Consistent Environment:** Everyone gets the same development tools

### 🛠️ Customization

#### Adding Your Own Environment Variables

Create a `.env.local` file (git-ignored) in the project root:

```bash
# Your custom environment variables
MY_API_KEY=your_secret_here
CUSTOM_SETTING=value
```

#### Extending the Container

To add additional tools or configurations:

### 🔧 Troubleshooting

#### Network Connectivity Issues

If you encounter network-related errors during container setup:

1. **Test network connectivity:**
   ```bash
   bash .devcontainer/test-network.sh
   ```

2. **Manual DNS fix:**
   ```bash
   echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
   echo "nameserver 8.8.4.4" | sudo tee -a /etc/resolv.conf
   ```

3. **Retry cargo tool installation:**
   ```bash
   bash .devcontainer/install-cargo-tools.sh
   ```

#### Cargo Tool Installation Failures

If specific cargo tools fail to install:

1. **Check network connectivity first:**
   ```bash
   bash .devcontainer/test-network.sh
   ```

2. **Install tools individually:**
   ```bash
   cargo install --locked cargo-outdated
   cargo install --locked cargo-tree
   ```

3. **Use the fallback installer:**
   ```bash
   bash .devcontainer/install-cargo-tools.sh
   ```

#### Common Error Messages

- **"Could not resolve hostname"**: DNS resolution issue - run network test script
- **"download of config.json failed"**: Network connectivity issue - check firewall/proxy settings
- **"Failed to install cargo-*"**: Individual tool installation failed - use fallback installer

#### Container Build Failures

If the container fails to build:

1. **Check Docker resources:**
   - Ensure Docker has enough memory (4GB+ recommended)
   - Check available disk space

2. **Clear Docker cache:**
   ```bash
   docker system prune -a
   ```

3. **Rebuild without cache:**
   - In your IDE, use "Dev Containers: Rebuild Container"

1. Modify `.devcontainer/Dockerfile` for system-level changes
2. Modify `.devcontainer/post-create.sh` for setup scripts
3. Modify `.devcontainer/devcontainer.json` for IDE configuration

### 🆘 Troubleshooting

#### Host Machine Issues

**Problem:** `bash .devcontainer/test-host.sh` reports failures
**Solution:** Follow the specific recommendations in the script output. Common issues:

- **No dev container IDE found:** 
  ```bash
  # Install Cursor (recommended)
  # Download from: https://cursor.sh/
  
  # OR install VS Code
  # Download from: https://code.visualstudio.com/
  # Don't forget the Dev Containers extension!
  ```

- **Docker not running:** Start Docker Desktop from Applications

- **GitHub CLI missing:** 
  ```bash
  # macOS
  brew install gh
  # Windows  
  winget install GitHub.cli
  ```

#### Container Issues

**Problem:** Container won't start
**Solution:**
1. Ensure Docker Desktop is running
2. Try rebuilding: `Ctrl+Shift+P` → "Dev Containers: Rebuild Container"
3. Check Docker Desktop logs for errors

**Problem:** GitHub authentication fails in container
**Solution:**
```bash
# Inside container, re-authenticate
gh auth logout
gh auth login --web --scopes "repo,read:org,read:user,gist"
```

**Problem:** Build failures in container
**Solution:**
```bash
# Inside container
cargo clean
cargo build --workspace
```

#### IDE-Specific Issues

**Problem:** Cursor not recognizing dev container
**Solution:**
- Ensure you have the latest Cursor version
- Dev container support is built-in, no extensions needed
- Try: `Ctrl+Shift+P` → "Dev Containers: Rebuild Container"

**Problem:** VS Code missing dev container commands
**Solution:**
- Install the "Dev Containers" extension
- Reload VS Code window: `Ctrl+Shift+P` → "Developer: Reload Window"

#### Testing Issues

**Problem:** Ran `.devcontainer/test-setup.sh` on host machine
**Solution:** 
- That script is for testing **inside** the container
- Use `.devcontainer/test-host.sh` to test your host machine
- Use `.devcontainer/test-setup.sh` only after opening the dev container

### 📚 Additional Resources

- [Toka OS Documentation](../docs/README.md)
- [Contributing Guide](../docs/development/CONTRIBUTING.md)
- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [Cursor Documentation](https://cursor.sh/docs)
- [VS Code Dev Containers](https://code.visualstudio.com/docs/devcontainers/containers)

### 💡 Tips for New Contributors

1. **Test Host First:** Always run `bash .devcontainer/test-host.sh` before starting
2. **Choose Your IDE:** Both Cursor and VS Code work great - pick what you prefer
3. **Use Environment File:** Copy the template to `.env.local` for easy setup
4. **Start Small:** Begin with `toka-quick-test` to verify everything works
5. **Use Aliases:** The container provides many helpful aliases to speed up development

### 🤝 Getting Help

If you encounter issues:

1. **Check the test scripts:** Run the appropriate test script for detailed diagnostics
2. **Review this README:** Look for your specific issue in the troubleshooting section
3. **Check IDE documentation:** Cursor and VS Code both have excellent dev container docs
4. **Review the logs:** Check Docker Desktop and IDE dev container logs
5. **Open an issue:** Create a GitHub issue with your test script output and environment details

Happy coding! 🦀 