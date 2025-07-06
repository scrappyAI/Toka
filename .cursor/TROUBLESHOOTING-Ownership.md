# Cursor Agent Ownership Troubleshooting Guide

This guide helps you resolve the **"Currently, we can only support a single owner (for private/write access)"** error and other ownership-related issues with cursor background agents.

## 🚨 Quick Fix for Ownership Conflicts

If you're seeing the ownership error right now, here's the immediate fix:

### Option 1: Use the Cleanup Script (Recommended)

**If running locally (outside container):**
```bash
# Run the cleanup script to resolve ownership conflicts
.cursor/scripts/cursor-cleanup.sh --force

# Or if you want to reset everything
.cursor/scripts/cursor-cleanup.sh --force --reset-db
```

**If running in container:**
```bash
# Run the cleanup script to resolve ownership conflicts
/app/scripts/cursor-cleanup.sh --force

# Or if you want to reset everything
/app/scripts/cursor-cleanup.sh --force --reset-db
```

### Option 2: Manual Cleanup

**Local environment:**
```bash
# Stop all cursor agent processes
pkill -f "toka-cli.*cursor-mode"

# Remove lock files (adjust paths based on your setup)
rm -rf ./data/locks/*.lock

# Clean up database locks
rm -f ./data/cursor-agents.db-wal
rm -f ./data/cursor-agents.db-shm

# Restart the agents
.cursor/scripts/cursor-agent-init.sh
```

**Container environment:**
```bash
# Stop all cursor agent processes
pkill -f "toka-cli.*cursor-mode"

# Remove lock files
rm -rf /app/data/locks/*.lock

# Clean up database locks
rm -f /app/data/cursor-agents.db-wal
rm -f /app/data/cursor-agents.db-shm

# Restart the agents
/app/scripts/cursor-agent-init.sh
```

## 🔍 Understanding the Ownership Error

### What Causes This Error?

The ownership error occurs when:

1. **Multiple Instances**: Another cursor agent process is already running
2. **Stale Locks**: Previous agent crashed and left lock files
3. **Database Conflicts**: SQLite database is locked by another process
4. **Permission Issues**: File ownership or permissions are incorrect
5. **Environment Conflicts**: Multiple environments claiming the same resources

### Error Patterns

```
Currently, we can only support a single owner (for private/write access).
Request ID: [some-uuid]
```

This error comes from cursor's backend when it detects ownership conflicts.

## 🛠️ Diagnostic Steps

### Step 1: Check Running Processes

```bash
# Check for running cursor agents
ps aux | grep "toka-cli.*cursor-mode"

# Check process IDs and ownership
pgrep -fl "cursor"
```

### Step 2: Examine Lock Files

**Local environment:**
```bash
# List all lock files (adjust path based on your setup)
find ./data/locks -name "*.lock" -ls 2>/dev/null || echo "No local lock files found"

# Check lock file contents
cat ./data/locks/cursor-agent.lock 2>/dev/null || echo "No lock file found"
```

**Container environment:**
```bash
# List all lock files
find /app/data/locks -name "*.lock" -ls

# Check lock file contents
cat /app/data/locks/cursor-agent.lock
```

### Step 3: Database Status

**Local environment:**
```bash
# Check database locks (adjust path based on your setup)
ls -la ./data/cursor-agents.db* 2>/dev/null || echo "No local database found"

# Test database access
sqlite3 ./data/cursor-agents.db "SELECT 1;" 2>/dev/null || echo "Cannot access database"
```

**Container environment:**
```bash
# Check database locks
ls -la /app/data/cursor-agents.db*

# Test database access
sqlite3 /app/data/cursor-agents.db "SELECT 1;"
```

### Step 4: File Permissions

**Local environment:**
```bash
# Check ownership and permissions
ls -la ./data/ 2>/dev/null || echo "No local data directory"
ls -la ./logs/ 2>/dev/null || echo "No local logs directory"

# Check current user
whoami
id
```

**Container environment:**
```bash
# Check ownership and permissions
ls -la /app/data/
ls -la /app/logs/

# Check current user
whoami
id
```

## 🔧 Resolution Methods

### Method 1: Automatic Cleanup (Recommended)

**Local environment:**
```bash
# Basic cleanup
.cursor/scripts/cursor-cleanup.sh

# Force cleanup without confirmation
.cursor/scripts/cursor-cleanup.sh --force

# Complete reset (removes all data)
.cursor/scripts/cursor-cleanup.sh --force --reset-db
```

**Container environment:**
```bash
# Basic cleanup
/app/scripts/cursor-cleanup.sh

# Force cleanup without confirmation
/app/scripts/cursor-cleanup.sh --force

# Complete reset (removes all data)
/app/scripts/cursor-cleanup.sh --force --reset-db
```

### Method 2: Manual Process Cleanup

```bash
# Find and kill cursor processes
pkill -f "toka-cli.*cursor-mode"
pkill -f "cursor-agent"

# Force kill if needed
pkill -9 -f "toka-cli.*cursor-mode"

# Verify no processes remain
pgrep -f "cursor" || echo "No cursor processes running"
```

### Method 3: Lock File Cleanup

**Local environment:**
```bash
# Remove all lock files
rm -rf ./data/locks/*.lock 2>/dev/null || echo "No lock files to remove"

# Recreate lock directory with proper permissions
mkdir -p ./data/locks
chmod 755 ./data/locks
```

**Container environment:**
```bash
# Remove all lock files
rm -rf /app/data/locks/*.lock

# Recreate lock directory with proper permissions
mkdir -p /app/data/locks
chown toka:toka /app/data/locks
chmod 755 /app/data/locks
```

### Method 4: Database Reset

**Local environment:**
```bash
# Stop all processes first
pkill -f "toka-cli.*cursor-mode"

# Remove database and locks
rm -f ./data/cursor-agents.db* 2>/dev/null

# Reinitialize database (if you have the CLI)
# ./bin/toka-config-cli init-db --cursor-mode --single-owner --db-path ./data/cursor-agents.db
```

**Container environment:**
```bash
# Stop all processes first
pkill -f "toka-cli.*cursor-mode"

# Remove database and locks
rm -f /app/data/cursor-agents.db*

# Reinitialize database
/app/bin/toka-config-cli init-db \
    --cursor-mode \
    --single-owner \
    --db-path /app/data/cursor-agents.db
```

### Method 5: Permission Reset

**Local environment:**
```bash
# Reset all permissions (adjust based on your user)
chmod -R 755 ./data ./logs 2>/dev/null || echo "Directories may not exist locally"
```

**Container environment:**
```bash
# Reset all permissions
chown -R toka:toka /app/data /app/logs
chmod -R 755 /app/data /app/logs
chmod 644 /app/data/cursor-agents.db
```

## 🔄 Prevention Strategies

### 1. Use Single-Owner Mode

Ensure your configuration uses single-owner mode:

```toml
# In config/cursor-agents.toml
[orchestration]
single_owner_mode = true
ownership_validation = "strict"
conflict_resolution = "terminate_conflicting"
```

### 2. Proper Shutdown

Always shut down agents properly:

```bash
# Graceful shutdown
pkill -TERM -f "toka-cli.*cursor-mode"

# Wait for cleanup
sleep 5

# Verify shutdown
pgrep -f "cursor" || echo "Clean shutdown"
```

### 3. Environment Isolation

Use unique environment names in your cursor environment.json:

```json
{
  "name": "Toka Cursor Agent Environment - ${USER}",
  "repositoryDependencies": [
    "github.com/ScrappyAI/toka"
  ]
}
```

### 4. Health Monitoring

Regularly check agent health:

**Local environment:**
```bash
# Run health check (if available)
.cursor/scripts/cursor-health-check.sh

# Check ownership status (if applicable)
curl http://localhost:9000/agents/ownership 2>/dev/null || echo "Service not available"
```

**Container environment:**
```bash
# Run health check
/app/scripts/cursor-health-check.sh

# Check ownership status
curl http://localhost:9000/agents/ownership
```

## 🚨 Emergency Recovery

If nothing else works, use this emergency recovery process:

### Complete Reset Procedure

**Local environment:**
```bash
# 1. Force stop everything
pkill -9 -f "toka"
pkill -9 -f "cursor"

# 2. Remove all data (WARNING: This deletes everything)
rm -rf ./data/* ./logs/* 2>/dev/null

# 3. Recreate directories
mkdir -p ./data/{locks,cache,sessions,context}
mkdir -p ./logs

# 4. Restart from scratch (if available)
.cursor/scripts/cursor-agent-init.sh
```

**Container environment:**
```bash
# 1. Force stop everything
pkill -9 -f "toka"
pkill -9 -f "cursor"

# 2. Remove all data (WARNING: This deletes everything)
rm -rf /app/data/*
rm -rf /app/logs/*

# 3. Recreate directories
mkdir -p /app/data/{locks,cache,sessions,context}
mkdir -p /app/logs
chown -R toka:toka /app/data /app/logs

# 4. Restart from scratch
/app/scripts/cursor-agent-init.sh
```

### Container Restart

If running in Docker:

```bash
# Stop and remove container
docker stop toka-cursor-agents
docker rm toka-cursor-agents

# Rebuild and restart
docker build -f .cursor/Dockerfile -t toka-cursor-agents .
docker run -d --name toka-cursor-agents \
  -p 8080:8080 -p 9000:9000 -p 9001:9001 -p 3000:3000 \
  -e CURSOR_AGENT_MODE=true \
  toka-cursor-agents
```

## 📊 Monitoring and Logging

### Enable Debug Logging

```bash
# Set debug environment
export CURSOR_DEBUG=true
export RUST_LOG=debug

# Run with verbose output
.cursor/scripts/cursor-agent-init.sh  # Local
# OR
/app/scripts/cursor-agent-init.sh     # Container
```

### Check Logs

**Local environment:**
```bash
# View initialization logs (adjust paths based on your setup)
tail -f ./logs/init.log 2>/dev/null || echo "No init log found"

# View agent logs
tail -f ./logs/cursor-agents.log 2>/dev/null || echo "No agent log found"

# View error logs
tail -f ./logs/cursor-errors.log 2>/dev/null || echo "No error log found"

# Search for ownership issues
grep -i "owner\|conflict\|lock" ./logs/*.log 2>/dev/null || echo "No logs found"
```

**Container environment:**
```bash
# View initialization logs
tail -f /app/logs/init.log

# View agent logs
tail -f /app/logs/cursor-agents.log

# View error logs
tail -f /app/logs/cursor-errors.log

# Search for ownership issues
grep -i "owner\|conflict\|lock" /app/logs/*.log
```

### Health Check Dashboard

Create a monitoring script:

```bash
#!/bin/bash
# ownership-monitor.sh

# Detect environment
if [[ -d "/app/data" ]]; then
    DATA_DIR="/app/data"
    SCRIPT_PATH="/app/scripts/cursor-health-check.sh"
else
    DATA_DIR="./data"
    SCRIPT_PATH=".cursor/scripts/cursor-health-check.sh"
fi

while true; do
    echo "=== $(date) ==="
    echo "Processes: $(pgrep -c -f "cursor" || echo 0)"
    echo "Locks: $(ls $DATA_DIR/locks/*.lock 2>/dev/null | wc -l)"
    echo "DB Status: $(sqlite3 $DATA_DIR/cursor-agents.db "SELECT 1;" 2>/dev/null || echo "LOCKED")"
    echo "Health: $($SCRIPT_PATH --quick >/dev/null 2>&1 && echo "OK" || echo "FAIL")"
    echo "---"
    sleep 30
done
```

## 🔍 Advanced Debugging

### Database Analysis

**Local environment:**
```bash
# Check database schema (if database exists)
sqlite3 ./data/cursor-agents.db ".schema" 2>/dev/null || echo "No database found"

# Check for locks in database
sqlite3 ./data/cursor-agents.db "PRAGMA locking_mode;" 2>/dev/null || echo "Cannot access database"

# Check database integrity
sqlite3 ./data/cursor-agents.db "PRAGMA integrity_check;" 2>/dev/null || echo "Cannot check integrity"
```

**Container environment:**
```bash
# Check database schema
sqlite3 /app/data/cursor-agents.db ".schema"

# Check for locks in database
sqlite3 /app/data/cursor-agents.db "PRAGMA locking_mode;"

# Check database integrity
sqlite3 /app/data/cursor-agents.db "PRAGMA integrity_check;"
```

### System Analysis

```bash
# Check system resources
df -h . # Local: current directory, Container: /app/data
free -h
lsof | grep cursor

# Check network usage
netstat -tlnp | grep ":9001\|:8080\|:9000"

# Check file descriptors
lsof -u $(whoami) | grep cursor
```

### Ownership Validation

**Container environment (if CLI available):**
```bash
# Validate current ownership
/app/bin/toka-config-cli validate-ownership \
    --cursor-mode \
    --db-path /app/data/cursor-agents.db

# Check ownership history
grep "owner" /app/logs/*.log | tail -20
```

## 📞 Getting Help

If you're still experiencing issues:

1. **Collect Logs**: Gather all relevant log files
2. **System Info**: Run `uname -a` and `docker version` (if using Docker)
3. **Configuration**: Share your cursor-agents.toml (remove sensitive data)
4. **Error Details**: Include the full error message and Request ID

### Support Information Template

```
Environment: Cursor + [Local/Docker]
OS: [Your OS]
Setup: [Local/Container]
Error Message: [Full error]
Request ID: [From error message]
Logs: [Attach relevant logs]
Configuration: [Share cursor-agents.toml]
```

## 🎯 Quick Reference

| Problem | Local Fix | Container Fix |
|---------|-----------|---------------|
| "Single owner" error | `.cursor/scripts/cursor-cleanup.sh --force` | `/app/scripts/cursor-cleanup.sh --force` |
| Process won't stop | `pkill -9 -f "cursor"` | `pkill -9 -f "cursor"` |
| Database locked | `rm -f ./data/cursor-agents.db-*` | `rm -f /app/data/cursor-agents.db-*` |
| Permission denied | `chmod -R 755 ./data ./logs` | `chown -R toka:toka /app/data /app/logs` |
| Stale locks | `rm -rf ./data/locks/*.lock` | `rm -rf /app/data/locks/*.lock` |
| Complete reset | `cursor-cleanup.sh --force --reset-db` | `cursor-cleanup.sh --force --reset-db` |

**Important**: Always use the correct paths based on whether you're running locally (`.cursor/scripts/`) or in a container (`/app/scripts/`).

Remember: When in doubt, use the cleanup script first before attempting manual fixes! 