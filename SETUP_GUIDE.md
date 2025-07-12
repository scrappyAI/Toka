# 🚀 Dead Simple Setup Guide

**Test the Toka Collaborative Auth System in 5 minutes**

## Prerequisites

- **Rust installed** (get it from [rustup.rs](https://rustup.rs/))
- **GitHub account** 
- **5 minutes** ⏱️

## Step 1: Create a GitHub App (2 minutes)

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click **"New GitHub App"**
3. Fill out the form:
   - **App Name**: `Toka Auth Test` (or whatever you want)
   - **Homepage URL**: `http://localhost:3000`
   - **Authorization callback URL**: `http://localhost:3000/auth/callback`
   - **Webhook URL**: Leave blank
   - **Repository permissions**: 
     - Metadata: **Read**
     - Contents: **Read**
   - **Organization permissions**:
     - Members: **Read**
   - **User permissions**:
     - Email addresses: **Read**
4. Click **"Create GitHub App"**
5. **Copy the Client ID** (you'll need this)
6. Click **"Generate a new client secret"** and **copy it** (you'll need this too)

## Step 2: Set Environment Variables (30 seconds)

Create a `.env` file in the Toka root directory:

```bash
cat > .env << 'EOF'
GITHUB_CLIENT_ID=your_client_id_here
GITHUB_CLIENT_SECRET=your_client_secret_here
REDIRECT_URI=http://localhost:3000/auth/callback
JWT_SECRET=dev-secret-for-testing
GITHUB_ORGANIZATION=your-github-username
EOF
```

**Replace `your_client_id_here` and `your_client_secret_here` with your actual values!**

## Step 3: Run the Auth Service (30 seconds)

```bash
# Load environment variables
source .env

# Run the auth service
cargo run --bin toka-auth-service
```

You should see:
```
🚀 Starting Toka Collaborative Auth Service
📋 Configuration loaded:
  - GitHub Client ID: your_client_id
  - Redirect URI: http://localhost:3000/auth/callback
✅ Auth service initialized successfully
🌐 Starting web server...
📱 Visit http://localhost:3000/auth/login to test the OAuth flow
```

## Step 4: Test the OAuth Flow (1 minute)

1. **Open your browser** and go to: `http://localhost:3000/auth/login`
2. **Click "Login with GitHub"** - you'll be redirected to GitHub
3. **Authorize your app** - GitHub will redirect you back
4. **You're logged in!** 🎉

## Step 5: Explore the System (1 minute)

Try these endpoints:

- **Login page**: `http://localhost:3000/auth/login`
- **Check your session**: `http://localhost:3000/auth/session`
- **View your user info**: `http://localhost:3000/auth/user`
- **Logout**: `http://localhost:3000/auth/logout`

## 🎯 What You Just Built

You now have a **production-ready authentication system** that:

- ✅ **Handles GitHub OAuth** with security best practices
- ✅ **Maps your GitHub role** to Toka capabilities automatically
- ✅ **Manages secure sessions** with JWT tokens
- ✅ **Provides a beautiful login UI**
- ✅ **Logs all authentication events**

## 🔧 Troubleshooting

### "Environment variable required" error
```bash
# Make sure you set the environment variables
export GITHUB_CLIENT_ID=your_actual_client_id
export GITHUB_CLIENT_SECRET=your_actual_client_secret
```

### "OAuth callback mismatch" error
- Check that your GitHub App callback URL is exactly: `http://localhost:3000/auth/callback`

### "Permission denied" errors
- Make sure your GitHub App has the right permissions (see Step 1)

### Port already in use
```bash
# Use a different port
export SERVER_PORT=3001
cargo run --bin toka-auth-service
# Then visit http://localhost:3001/auth/login
```

## 🎉 Next Steps

Now that authentication works, you can:

1. **Integrate with your existing Toka services**
2. **Add more GitHub organizations/repos**
3. **Customize the permission mapping**
4. **Deploy to production** (see the main README)

## 🔒 Security Note

The JWT secret `dev-secret-for-testing` is only for testing. In production, use a strong random secret:

```bash
# Generate a secure secret
openssl rand -base64 32
```

## 📋 Quick Reference

**Environment Variables:**
- `GITHUB_CLIENT_ID` - Required: Your GitHub App Client ID
- `GITHUB_CLIENT_SECRET` - Required: Your GitHub App Client Secret  
- `REDIRECT_URI` - Default: `http://localhost:3000/auth/callback`
- `JWT_SECRET` - Default: `dev-secret-change-in-production`
- `GITHUB_ORGANIZATION` - Optional: Specific GitHub org to check membership
- `GITHUB_REPOSITORY` - Optional: Specific repo to check permissions
- `SERVER_PORT` - Default: `3000`

**Useful Commands:**
```bash
# Run the auth service
cargo run --bin toka-auth-service

# Run with custom port
SERVER_PORT=3001 cargo run --bin toka-auth-service

# Check logs
RUST_LOG=debug cargo run --bin toka-auth-service
```

That's it! You now have a working collaborative authentication system. 🚀 