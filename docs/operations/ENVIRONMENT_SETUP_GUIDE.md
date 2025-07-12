# Environment Setup Guide

**Quick Start Guide for Toka OS Environment Configuration**

## 🚀 Quick Setup (2 minutes)

### Step 1: Choose Your Environment

**For Development:**
```bash
# Copy the main template
cp config/environments/toka.env.example .env.local

# Edit with your API keys
nano .env.local
```

**For Cursor IDE:**
```bash
# Copy the cursor template
cp config/environments/cursor.env.example .env.cursor

# Edit with your API keys
nano .env.cursor
```

### Step 2: Add Your API Keys

Add your LLM provider API key:
```bash
# For Anthropic Claude (recommended)
ANTHROPIC_API_KEY=your_actual_api_key_here

# For OpenAI GPT-4 (alternative)
OPENAI_API_KEY=your_actual_api_key_here
```

### Step 3: Validate Your Setup

```bash
# Run validation script
./scripts/validate-env.sh

# Test your configuration
cargo test --lib
```

## 📁 File Organization

### **What Goes Where?**

| File Location | Purpose | When to Use |
|---------------|---------|-------------|
| `.env.local` | Your personal development environment | Daily development |
| `.env.cursor` | Cursor IDE background agents | When using Cursor |
| `.env.dev` | Team development environment | Shared dev setup |
| `.env.prod` | Production environment | Production deployment |

### **Templates (Safe to Commit)**

| Template | Purpose |
|----------|---------|
| `config/environments/toka.env.example` | Main Toka environment |
| `config/environments/cursor.env.example` | Cursor-specific config |
| `config/environments/dev.env.example` | Development environment |
| `config/environments/prod.env.example` | Production environment |

## 🔧 Common Scenarios

### **Scenario 1: Local Development**
```bash
# 1. Copy template
cp config/environments/toka.env.example .env.local

# 2. Add your API key
echo "ANTHROPIC_API_KEY=your_key_here" >> .env.local

# 3. Load environment
source .env.local
```

### **Scenario 2: Cursor IDE Integration**
```bash
# 1. Copy cursor template
cp config/environments/cursor.env.example .env.cursor

# 2. Add your API key
echo "ANTHROPIC_API_KEY=your_key_here" >> .env.cursor

# 3. Cursor will automatically load .env.cursor
```

### **Scenario 3: Docker Development**
```bash
# 1. Copy development template
cp config/environments/dev.env.example .env.dev

# 2. Edit with your settings
nano .env.dev

# 3. Use with Docker
docker-compose --env-file .env.dev up -d
```

### **Scenario 4: Production Deployment**
```bash
# 1. Copy production template
cp config/environments/prod.env.example .env.prod

# 2. Set production values (secure passwords, etc.)
nano .env.prod

# 3. Deploy
docker-compose --env-file .env.prod up -d
```

## 🔒 Security Best Practices

### **✅ Do This**
- Copy templates to working files (`.env.local`, `.env.cursor`, etc.)
- Add real API keys to working files
- Keep working files in `.gitignore`
- Use strong secrets for production

### **❌ Don't Do This**
- Never commit files with real API keys
- Don't edit template files directly
- Don't use weak secrets in production
- Don't store secrets in code

## 🔍 Troubleshooting

### **Problem: Missing API Key**
```bash
# Check if your environment file exists
ls -la .env.local

# Check if API key is set
grep ANTHROPIC_API_KEY .env.local

# Validate configuration
./scripts/validate-env.sh
```

### **Problem: Cursor Not Working**
```bash
# Check cursor environment
ls -la .env.cursor

# Validate cursor configuration
./scripts/validate-env.sh

# Check cursor logs
tail -f ~/.cursor/logs/cursor.log
```

### **Problem: Environment Variables Not Loading**
```bash
# Load environment manually
source .env.local

# Check if variables are set
echo $ANTHROPIC_API_KEY

# Export variables
export $(cat .env.local | grep -v '^#' | xargs)
```

## 📋 Validation Checklist

Before running Toka OS, ensure:

- [ ] Environment file exists (`.env.local` or `.env.cursor`)
- [ ] API keys are set and valid
- [ ] No placeholder values remain
- [ ] File permissions are secure (`chmod 600`)
- [ ] Validation script passes (`./scripts/validate-env.sh`)

## 🎯 Next Steps

1. **Choose your environment** (development, cursor, production)
2. **Copy the appropriate template**
3. **Add your API keys**
4. **Run validation**
5. **Test your setup**

## 📚 Related Documentation

- [Environment Organization Guide](ENVIRONMENT_ORGANIZATION.md) - Complete technical details
- [Docker Setup Guide](README-Docker.md) - Docker-specific configuration
- [Security Guide](../security/README.md) - Security best practices
- [Troubleshooting](../troubleshooting/README.md) - Common issues and solutions

## 🆘 Getting Help

If you encounter issues:

1. **Run validation**: `./scripts/validate-env.sh`
2. **Check logs**: Look for error messages in terminal
3. **Review templates**: Compare your config with templates
4. **Ask for help**: Create an issue with your error message

---

**Remember**: Environment files with real secrets should never be committed to version control! 