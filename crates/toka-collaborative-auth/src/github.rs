//! GitHub API integration for user information and role detection

use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{AuthError, AuthResult, GitHubRole, GitHubUser};

/// GitHub API base URL
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub API client
#[derive(Debug, Clone)]
pub struct GitHubClient {
    /// HTTP client
    client: Client,
    /// Organization to check membership
    organization: Option<String>,
    /// Repository to check access
    repository: Option<String>,
}

/// GitHub user response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUserResponse {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub hireable: Option<bool>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub organization membership response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMembershipResponse {
    pub organization: GitHubOrganization,
    pub user: GitHubUserResponse,
    pub role: String, // "admin", "member"
    pub state: String, // "active", "pending"
}

/// GitHub organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOrganization {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: String,
}

/// GitHub repository collaborator response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCollaboratorResponse {
    pub permission: String, // "pull", "push", "admin"
    pub user: GitHubUserResponse,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub owner: GitHubUserResponse,
    pub permissions: Option<GitHubRepositoryPermissions>,
}

/// GitHub repository permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryPermissions {
    pub admin: bool,
    pub maintain: bool,
    pub push: bool,
    pub triage: bool,
    pub pull: bool,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(organization: Option<String>, repository: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("toka-collaborative-auth/1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            organization,
            repository,
        }
    }
    
    /// Get user information from GitHub
    pub async fn get_user(&self, access_token: &str) -> AuthResult<GitHubUserResponse> {
        debug!("Fetching user information from GitHub");
        
        let response = self.client
            .get(&format!("{}/user", GITHUB_API_BASE))
            .bearer_auth(access_token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("GitHub API error ({}): {}", status, body);
            return Err(AuthError::GitHub(format!("API error ({}): {}", status, body)));
        }
        
        let user: GitHubUserResponse = response.json().await?;
        debug!("Retrieved user: {} ({})", user.login, user.id);
        
        Ok(user)
    }
    
    /// Get user's primary email address
    pub async fn get_user_email(&self, access_token: &str) -> AuthResult<Option<String>> {
        debug!("Fetching user email from GitHub");
        
        let response = self.client
            .get(&format!("{}/user/emails", GITHUB_API_BASE))
            .bearer_auth(access_token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            warn!("Failed to fetch user emails ({})", status);
            return Ok(None);
        }
        
        let emails: Vec<GitHubEmail> = response.json().await?;
        
        // Find primary email
        let primary_email = emails.iter()
            .find(|email| email.primary)
            .map(|email| email.email.clone());
        
        debug!("Primary email: {:?}", primary_email);
        Ok(primary_email)
    }
    
    /// Determine user's role based on organization and repository access
    pub async fn determine_role(&self, access_token: &str, user: &GitHubUserResponse) -> AuthResult<GitHubRole> {
        debug!("Determining role for user: {}", user.login);
        
        // Check organization membership if configured
        if let Some(org) = &self.organization {
            match self.get_organization_membership(access_token, org, &user.login).await {
                Ok(Some(membership)) => {
                    debug!("User {} is {} in organization {}", user.login, membership.role, org);
                    return Ok(match membership.role.as_str() {
                        "admin" => GitHubRole::Owner,
                        "member" => {
                            // Check repository access for more specific role
                            if let Some(repo) = &self.repository {
                                self.determine_repository_role(access_token, org, repo, &user.login).await?
                            } else {
                                GitHubRole::Collaborator
                            }
                        }
                        _ => GitHubRole::Public,
                    });
                }
                Ok(None) => {
                    debug!("User {} is not a member of organization {}", user.login, org);
                }
                Err(e) => {
                    warn!("Failed to check organization membership: {}", e);
                }
            }
        }
        
        // Check repository access if configured
        if let Some(org) = &self.organization {
            if let Some(repo) = &self.repository {
                return self.determine_repository_role(access_token, org, repo, &user.login).await;
            }
        }
        
        // Default to public user
        debug!("User {} assigned public role", user.login);
        Ok(GitHubRole::Public)
    }
    
    /// Get organization membership for a user
    async fn get_organization_membership(
        &self, 
        access_token: &str, 
        org: &str, 
        username: &str
    ) -> AuthResult<Option<GitHubMembershipResponse>> {
        debug!("Checking organization membership for {} in {}", username, org);
        
        let response = self.client
            .get(&format!("{}/orgs/{}/memberships/{}", GITHUB_API_BASE, org, username))
            .bearer_auth(access_token)
            .send()
            .await?;
        
        match response.status().as_u16() {
            200 => {
                let membership: GitHubMembershipResponse = response.json().await?;
                Ok(Some(membership))
            }
            404 => Ok(None), // User is not a member
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(AuthError::GitHub(format!("Organization membership check failed ({}): {}", status, body)))
            }
        }
    }
    
    /// Determine role based on repository access
    async fn determine_repository_role(
        &self, 
        access_token: &str, 
        org: &str, 
        repo: &str, 
        username: &str
    ) -> AuthResult<GitHubRole> {
        debug!("Checking repository access for {} in {}/{}", username, org, repo);
        
        // First check if user has direct collaborator access
        match self.get_repository_collaborator(access_token, org, repo, username).await {
            Ok(Some(collaborator)) => {
                debug!("User {} has {} permission in {}/{}", username, collaborator.permission, org, repo);
                return Ok(match collaborator.permission.as_str() {
                    "admin" => GitHubRole::Maintainer,
                    "push" => GitHubRole::Collaborator,
                    "pull" => GitHubRole::Contributor,
                    _ => GitHubRole::Public,
                });
            }
            Ok(None) => {
                debug!("User {} is not a direct collaborator of {}/{}", username, org, repo);
            }
            Err(e) => {
                warn!("Failed to check repository collaborator status: {}", e);
            }
        }
        
        // Check if repository is public and user can access it
        match self.get_repository(access_token, org, repo).await {
            Ok(repository) => {
                if !repository.private {
                    debug!("Repository {}/{} is public, user {} gets contributor access", org, repo, username);
                    Ok(GitHubRole::Contributor)
                } else {
                    debug!("Repository {}/{} is private and user {} has no access", org, repo, username);
                    Ok(GitHubRole::Public)
                }
            }
            Err(_) => {
                debug!("User {} cannot access repository {}/{}", username, org, repo);
                Ok(GitHubRole::Public)
            }
        }
    }
    
    /// Get repository collaborator information
    async fn get_repository_collaborator(
        &self, 
        access_token: &str, 
        org: &str, 
        repo: &str, 
        username: &str
    ) -> AuthResult<Option<GitHubCollaboratorResponse>> {
        let response = self.client
            .get(&format!("{}/repos/{}/{}/collaborators/{}/permission", GITHUB_API_BASE, org, repo, username))
            .bearer_auth(access_token)
            .send()
            .await?;
        
        match response.status().as_u16() {
            200 => {
                let collaborator: GitHubCollaboratorResponse = response.json().await?;
                Ok(Some(collaborator))
            }
            404 => Ok(None), // User is not a collaborator
            _ => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(AuthError::GitHub(format!("Repository collaborator check failed ({}): {}", status, body)))
            }
        }
    }
    
    /// Get repository information
    async fn get_repository(&self, access_token: &str, org: &str, repo: &str) -> AuthResult<GitHubRepository> {
        let response = self.client
            .get(&format!("{}/repos/{}/{}", GITHUB_API_BASE, org, repo))
            .bearer_auth(access_token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::GitHub(format!("Repository fetch failed ({}): {}", status, body)));
        }
        
        let repository: GitHubRepository = response.json().await?;
        Ok(repository)
    }
    
    /// Create GitHubUser from API response and role
    pub async fn create_github_user(
        &self, 
        access_token: &str, 
        user_response: GitHubUserResponse
    ) -> AuthResult<GitHubUser> {
        // Get email if not provided in user response
        let email = if user_response.email.is_some() {
            user_response.email.clone()
        } else {
            self.get_user_email(access_token).await?
        };
        
        // Determine role
        let role = self.determine_role(access_token, &user_response).await?;
        
        info!("Created GitHubUser: {} with role: {:?}", user_response.login, role);
        
        Ok(GitHubUser {
            id: user_response.id,
            login: user_response.login,
            name: user_response.name,
            email,
            avatar_url: user_response.avatar_url,
            role,
        })
    }
}

/// GitHub email response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubEmail {
    email: String,
    verified: bool,
    primary: bool,
    visibility: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new(
            Some("test-org".to_string()),
            Some("test-repo".to_string())
        );
        
        assert_eq!(client.organization, Some("test-org".to_string()));
        assert_eq!(client.repository, Some("test-repo".to_string()));
    }
    
    #[test]
    fn test_github_client_no_org() {
        let client = GitHubClient::new(None, None);
        
        assert_eq!(client.organization, None);
        assert_eq!(client.repository, None);
    }
} 