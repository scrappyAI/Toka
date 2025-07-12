# Toka Collaborative Authentication

**GitHub OAuth abstraction and role assignment for collaborative Toka workspace**

This crate provides a secure, web-based authentication system that enables humans and AI agents to collaborate seamlessly in the Toka workspace. It builds upon GitHub's OAuth2 flow to provide automatic role assignment and capability token generation.

## 🌟 Vision

Transform the Toka workspace into a **living laboratory** where:
- ✨ Users authenticate via GitHub with automatic role assignment
- 🔄 Non-technical users can easily plug in to experiment with branches/forks
- 🤖 Agentic collaboration surfaces ideas as formal proposals for merging
- 🎯 Value emerges as focused applications derived from the workspace
- 🔒 Clear security boundaries enable open-source collaboration at scale

## 🏗️ Architecture

The collaborative authentication system consists of:

- **OAuth Gateway**: Web-based GitHub OAuth2 flow with automatic user onboarding
- **Role Assignment Engine**: Maps GitHub permissions to Toka capability tokens
- **Permission Matrix**: Fine-grained access control for collaborative features
- **Session Management**: Secure session handling with automatic token refresh

### GitHub Role Mapping

| GitHub Role | Toka Capabilities |
|-------------|-------------------|
| **Owner** | Read, Fork, Experiment, Review, Write, Merge, Admin, Deploy |
| **Maintainer** | Read, Fork, Experiment, Review, Write, Merge |
| **Collaborator** | Read, Fork, Experiment, Review, Write |
| **Contributor** | Read, Fork, Experiment |
| **Public** | Read |

### Capability-Based Access Control

```rust
// Example: Check if user can create experiments
let can_experiment = checker
    .check_action(&user.capabilities, &Action::CreateExperiment)
    .await?;

if can_experiment {
    // User can create experiments
    create_experiment(&user, experiment_spec).await?;
}
```

## 🚀 Quick Start

### 1. Configuration

Create an `AuthConfig` with your GitHub app credentials:

```rust
use toka_collaborative_auth::{AuthService, AuthConfig};

let config = AuthConfig {
    github_client_id: "your-github-app-id".to_string(),
    github_client_secret: "your-github-app-secret".to_string(),
    redirect_uri: "http://localhost:3000/auth/callback".to_string(),
    jwt_secret: "your-jwt-secret".to_string(),
    organization: Some("your-org".to_string()),
    ..Default::default()
};
```

### 2. Start Authentication Service

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start the authentication service
    let auth_service = AuthService::new(config).await?;
    auth_service.start().await?;
    Ok(())
}
```

### 3. Web Endpoints

The service provides these endpoints:

- `GET /auth/login` - Start GitHub OAuth flow
- `GET /auth/callback` - OAuth callback handler
- `GET /auth/session` - Check current session
- `POST /auth/logout` - End session
- `GET /auth/user` - Get current user info
- `GET /auth/admin/users` - Admin: List all users
- `POST /auth/admin/capabilities` - Admin: Manage capabilities

## 🔧 Components

### OAuth Client (`oauth.rs`)

Handles GitHub OAuth2 flow with PKCE support:

```rust
let oauth_client = OAuthClient::new(
    github_client_id,
    github_client_secret,
    redirect_uri,
    storage.clone(),
    true, // Enable PKCE
).await?;

// Start OAuth flow
let auth_url = oauth_client.get_authorization_url(Some(redirect_url)).await?;
```

### GitHub Integration (`github.rs`)

Interfaces with GitHub API for user and organization data:

```rust
let github_client = GitHubClient::new(access_token);
let user = github_client.get_user().await?;
let role = github_client.determine_role(&organization, &repository).await?;
```

### Session Management (`session.rs`)

JWT-based session handling with automatic cleanup:

```rust
let session_manager = SessionManager::new(jwt_secret, storage.clone());
let session = session_manager.create_session(github_user, capabilities).await?;
```

### Permission System (`permissions.rs`)

Fine-grained capability checking:

```rust
let checker = PermissionChecker::new(role_mapping);
let allowed = checker.check_action(&capabilities, &Action::MergeProposal).await?;
```

### Storage Backends (`storage.rs`)

Multiple storage options:

```rust
// In-memory (development)
let storage = MemoryStorage::new();

// SQLite (production)
let storage = SQLiteStorage::new("auth.db").await?;
```

## 🔒 Security Features

### OAuth2 Security
- **PKCE Support**: Prevents authorization code interception
- **State Validation**: CSRF protection for OAuth flow
- **Secure Redirects**: Validates redirect URLs

### JWT Security
- **HS256 Signing**: Cryptographically signed tokens
- **Expiration**: Automatic token expiration and refresh
- **Revocation**: Immediate session invalidation

### Capability Tokens
- **Fine-grained Permissions**: Specific actions require specific capabilities
- **Role-based Access**: GitHub roles map to Toka capabilities
- **Audit Logging**: All authentication events are logged

## 🏭 Production Setup

### GitHub App Configuration

1. Create a GitHub App with these permissions:
   - **Repository permissions**: Read access to metadata and code
   - **Organization permissions**: Read access to members
   - **User permissions**: Read access to profile

2. Set OAuth callback URL: `https://your-domain.com/auth/callback`

### Environment Variables

```bash
# Required
GITHUB_CLIENT_ID=your_github_app_id
GITHUB_CLIENT_SECRET=your_github_app_secret
JWT_SECRET=your_secure_jwt_secret
REDIRECT_URI=https://your-domain.com/auth/callback

# Optional
GITHUB_ORGANIZATION=your-org
REPOSITORY=your-repo
DATABASE_URL=sqlite:auth.db
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

### Docker Deployment

```dockerfile
FROM rust:1.70-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p toka-collaborative-auth

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/toka-auth-service /usr/local/bin/
EXPOSE 3000
CMD ["toka-auth-service"]
```

## 🔗 Integration with Toka

### Agent Runtime Integration

```rust
use toka_collaborative_auth::{SessionClaims, AuthError};
use axum::extract::State;

async fn agent_endpoint(
    Extension(claims): Extension<SessionClaims>,
    State(app_state): State<AppState>,
) -> Result<Json<AgentResponse>, AuthError> {
    // User is authenticated - proceed with agent interaction
    let agent_response = app_state
        .agent_runtime
        .execute_task(claims.github_user.id, task)
        .await?;
    
    Ok(Json(agent_response))
}
```

### Capability Middleware

```rust
use toka_collaborative_auth::middleware::{require_capability, Action};

let app = Router::new()
    .route("/api/experiments", post(create_experiment))
    .layer(require_capability(Action::CreateExperiment))
    .route("/api/merge", post(merge_proposal))
    .layer(require_capability(Action::MergeProposal));
```

## 🧪 Testing

Run the test suite:

```bash
cargo test -p toka-collaborative-auth
```

Integration tests require GitHub credentials:

```bash
# Set test environment
export TEST_GITHUB_CLIENT_ID=your_test_app_id
export TEST_GITHUB_CLIENT_SECRET=your_test_app_secret

cargo test -p toka-collaborative-auth --features integration-tests
```

## 📈 Monitoring

The service provides Prometheus metrics:

- `auth_sessions_total` - Total active sessions
- `auth_login_attempts` - Login attempt counter
- `auth_token_refresh_total` - Token refresh counter
- `auth_permission_checks` - Permission check counter

Access metrics at `http://localhost:3000/metrics`

## 🛠️ Development

### Local Development

```bash
# Start local auth service
cargo run -p toka-collaborative-auth

# Use ngrok for GitHub OAuth callback
ngrok http 3000
# Update GitHub app callback URL to ngrok URL
```

### Testing OAuth Flow

1. Visit `http://localhost:3000/auth/login`
2. Complete GitHub OAuth flow
3. Check session at `http://localhost:3000/auth/session`

## 🤝 Contributing

This crate is part of the broader Toka collaborative ecosystem roadmap:

- ✅ **Phase 1**: GitHub OAuth abstraction (Complete)
- 🚧 **Phase 2**: Collaborative UI for agent-human interactions
- 📋 **Phase 3**: Agent-driven proposal generation system
- 📋 **Phase 4**: Sandbox framework for safe experimentation
- 📋 **Phase 5**: Value extraction and application derivation

## 📄 License

Licensed under either of Apache License, Version 2.0 or MIT license at your option. 