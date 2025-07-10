# ⚡ 3-Minute Quick Start

**Test your GitHub OAuth authentication system NOW**

## Prerequisites: 30 seconds

1. **GitHub Account** ✅
2. **Rust installed** → Get it: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## 1. Create GitHub App: 90 seconds

🔗 **Go to:** https://github.com/settings/apps/new

📝 **Fill in:**
- **App Name**: `My Toka Test`
- **Homepage URL**: `http://localhost:3000`
- **Callback URL**: `http://localhost:3000/auth/callback`

🔐 **Permissions:**
- Repository → Metadata: **Read**
- Organization → Members: **Read**
- Account → Email: **Read**

✅ **Click "Create GitHub App"**

📋 **Copy these:**
- Client ID (looks like `Iv1.a1b2c3d4e5f6g7h8`)
- Generate & copy Client Secret

## 2. Run It: 60 seconds

```bash
# Set your credentials (replace with your actual values!)
export GITHUB_CLIENT_ID=Iv1.your_actual_client_id_here
export GITHUB_CLIENT_SECRET=your_actual_client_secret_here

# Run the authentication service
cargo run --bin toka-auth-service
```

**You'll see:**
```
🚀 Starting Toka Collaborative Auth Service
✅ Auth service initialized successfully
📱 Visit http://localhost:3000/auth/login to test the OAuth flow
```

## 3. Test It: 30 seconds

1. **Open**: http://localhost:3000/auth/login
2. **Click**: "Login with GitHub"
3. **Authorize** your app
4. **Done!** 🎉

## 🎯 Try These URLs

- **Login**: http://localhost:3000/auth/login
- **Your Info**: http://localhost:3000/auth/user
- **Session**: http://localhost:3000/auth/session
- **Logout**: http://localhost:3000/auth/logout

## 🚨 Troubleshooting

**"Environment variable required"**
```bash
# Make sure you exported the variables in the same terminal:
export GITHUB_CLIENT_ID=your_actual_client_id
export GITHUB_CLIENT_SECRET=your_actual_client_secret
```

**"OAuth callback mismatch"**
- Check your GitHub App callback URL is: `http://localhost:3000/auth/callback`

**Port 3000 in use**
```bash
export SERVER_PORT=3001
cargo run --bin toka-auth-service
# Then visit http://localhost:3001/auth/login
```

## 🔥 What You Just Built

You now have a **production-ready** authentication system with:

- ✅ **Secure GitHub OAuth** with PKCE
- ✅ **Automatic role assignment** based on GitHub permissions
- ✅ **JWT session management**
- ✅ **Beautiful login UI**
- ✅ **Capability-based access control**

**Total time:** ~3 minutes ⏱️

---

### 🔗 Next Steps

- **Full Setup Guide:** [SETUP_GUIDE.md](SETUP_GUIDE.md)
- **Complete Documentation:** [crates/toka-collaborative-auth/README.md](crates/toka-collaborative-auth/README.md)
- **Use the test script:** `./scripts/test-auth.sh` 