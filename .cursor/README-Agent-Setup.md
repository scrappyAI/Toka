# Toka Cursor Agent Setup

This directory contains a comprehensive, agentic setup for Cursor background agents that automates environment configuration, secret management, and development workflows.

## 🚀 **Key Features**

### **Fully Agentic Operation**
- **Zero manual configuration** - Environment auto-configures on startup
- **Automatic secret detection** - Finds and configures API keys from environment
- **Auto-building binaries** - Builds Toka CLI tools as needed
- **Smart project detection** - Adapts to project state automatically

### **Comprehensive Development Environment**
- **Multi-terminal setup** - Interactive CLI, development, testing, monitoring
- **Rich development aliases** - `tk`, `tki`, `tktest`, etc. for quick workflows
- **Automatic dependency caching** - Persistent cargo and target volumes
- **Performance monitoring** - Built-in health checks and metrics

### **Security & Secrets Management**
- **Automatic secret injection** - Supports GitHub, OpenAI, Anthropic API keys
- **Secure file permissions** - Proper secret file protection
- **Key rotation ready** - Integration with Toka's security framework

## 📁 **File Structure**

```
.cursor/
├── Dockerfile              # Optimized container for Cursor agents
├── environment.json         # Complete runtime configuration  
├── scripts/
│   ├── auto-setup.sh       # Automatic environment setup
│   ├── dev-setup.sh        # Development environment configuration
│   ├── cursor-agent-init.sh # Agent initialization and startup
│   └── cursor-health-check.sh # Health monitoring
└── README-Agent-Setup.md   # This documentation
```

## 🔧 **Configuration**

### **Environment Variables (Auto-Detected)**
```bash
GITHUB_TOKEN=github_pat_*     # GitHub API access
OPENAI_API_KEY=sk-*          # OpenAI LLM access  
ANTHROPIC_API_KEY=sk-*       # Anthropic LLM access
TOKA_SECRET_KEY=*            # Toka internal encryption
```

### **Runtime Configuration**
- **Workspace**: `/home/vscode` (user home directory)
- **Config**: `/home/vscode/config/cursor-agents.toml`
- **Data**: `/home/vscode/data/` (with persistent volumes)
- **Logs**: `/home/vscode/logs/` (with rotation)

## 🛠 **Development Workflow**

### **Available Terminals**
1. **Toka CLI** - Interactive project management with `tki`
2. **Development** - Main terminal with auto-setup and aliases
3. **Testing** - Automated test runner with `tktest`
4. **Monitoring** - Performance monitoring with `htop`

### **Development Aliases**
```bash
# Quick commands
tk              # Run toka-cli with arguments
tki             # Interactive Toka CLI
tktest          # Run test suite with nextest
tkcheck         # Quick cargo check
tkfmt           # Format all code
tklint          # Run clippy linting

# Navigation shortcuts  
tkconfig        # Go to config directory
tklogs          # Go to logs and tail output
tkdata          # Go to data directory

# Development functions
tk_quick_test   # Fast development tests
tk_full_test    # Complete test suite
tk_dev_status   # Show current development status
tk_clean_build  # Clean rebuild of workspace
```

## 🚦 **Getting Started**

### **For Cursor Users**
1. Open project in Cursor
2. Start background agent (uses this configuration automatically)
3. Wait for auto-setup to complete (~30 seconds)
4. Use any terminal - everything is pre-configured!

### **Manual Setup (if needed)**
```bash
# Run auto-setup manually
/usr/local/bin/cursor-scripts/auto-setup.sh

# Start development environment
/usr/local/bin/cursor-scripts/dev-setup.sh

# Check status
tk_dev_status
```

## 🔍 **Monitoring & Health**

### **Health Checks**
- **Container health**: Automated every 30 seconds
- **Build status**: Validates project can compile
- **Agent activity**: Monitors log file updates
- **Resource usage**: CPU, memory, disk monitoring

### **Logs & Debugging**
```bash
# View agent logs
tklogs

# Check agent status  
tkstatus

# Health check
/usr/local/bin/cursor-scripts/cursor-health-check.sh
```

## 🔐 **Security Features**

### **Secret Management**
- Auto-detection from environment variables
- Secure file permissions (600)
- No secrets in logs or git
- Integration with Toka security framework

### **Container Security**
- Non-root user (`vscode`)
- Minimal attack surface
- Security scanning tools included
- Capability-based access control

## 🎯 **Agentic Benefits**

This setup embodies the **stateful, interactive Toka CLI** preference by:
- Providing persistent, configured terminals ready for interaction
- Auto-configuring all necessary credentials and environment
- Offering rich command shortcuts that understand Toka's structure
- Maintaining state across agent sessions with persistent volumes

The result is a **zero-configuration, fully capable development environment** that agents can use immediately without any manual setup steps.

## 🐛 **Troubleshooting**

### **Common Issues**
```bash
# Project not found
# Solution: Ensure git clone completed successfully

# Missing binaries  
# Solution: Run auto-setup to build required tools

# Permission issues
# Solution: Check ownership and run setup scripts

# Secret access issues
# Solution: Verify environment variables are set
```

### **Reset Environment**
```bash
# Clean restart
rm -rf /home/vscode/{data,logs,config}
/usr/local/bin/cursor-scripts/auto-setup.sh
```

---

This setup transforms Cursor background agents from basic containers into **fully-capable, agentic development environments** that require zero manual configuration and provide rich, interactive workflows out of the box. 