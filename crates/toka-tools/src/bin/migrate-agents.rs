//! Agent migration tool
//!
//! This tool migrates existing YAML agent configurations from the /agents directory
//! to the new Rust-based agent system in toka-tools.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::fs;
use serde_yaml;
use serde_json;

use toka_tools::agents::{AgentSystem, AgentSpec};
use toka_tools::core::ToolRegistry;

#[derive(Parser)]
#[command(name = "migrate-agents")]
#[command(about = "Migrate existing agent configurations to the new Rust system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate all agent configurations from /agents directory
    MigrateAll {
        /// Source directory containing agent configurations
        #[arg(short, long, default_value = "agents")]
        source: PathBuf,
        /// Target directory for migrated configurations
        #[arg(short, long, default_value = "crates/toka-tools/migrated")]
        target: PathBuf,
        /// Dry run - don't actually write files
        #[arg(long)]
        dry_run: bool,
    },
    /// Migrate a specific agent configuration
    MigrateOne {
        /// Source agent file
        source: PathBuf,
        /// Target file
        target: PathBuf,
        /// Dry run - don't actually write files
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate migrated agent configurations
    Validate {
        /// Directory containing migrated agent configurations
        #[arg(short, long, default_value = "crates/toka-tools/migrated")]
        directory: PathBuf,
    },
    /// Generate agent summary report
    Report {
        /// Source directory containing agent configurations
        #[arg(short, long, default_value = "agents")]
        source: PathBuf,
    },
}

#[derive(Debug, Clone)]
struct MigrationStats {
    total_agents: usize,
    migrated_successfully: usize,
    migration_errors: usize,
    validation_errors: usize,
    domains: HashMap<String, usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::MigrateAll { source, target, dry_run } => {
            migrate_all(source, target, dry_run).await?;
        }
        Commands::MigrateOne { source, target, dry_run } => {
            migrate_one(source, target, dry_run).await?;
        }
        Commands::Validate { directory } => {
            validate_migrated(directory).await?;
        }
        Commands::Report { source } => {
            generate_report(source).await?;
        }
    }
    
    Ok(())
}

async fn migrate_all(source: PathBuf, target: PathBuf, dry_run: bool) -> Result<()> {
    println!("🚀 Starting agent migration from {} to {}", source.display(), target.display());
    
    if !dry_run {
        fs::create_dir_all(&target).await?;
    }
    
    let mut stats = MigrationStats {
        total_agents: 0,
        migrated_successfully: 0,
        migration_errors: 0,
        validation_errors: 0,
        domains: HashMap::new(),
    };
    
    // Find all agent configurations
    let agent_files = discover_agent_files(&source).await?;
    stats.total_agents = agent_files.len();
    
    println!("📋 Found {} agent configurations to migrate", agent_files.len());
    
    for agent_file in agent_files {
        match migrate_agent_file(&agent_file, &target, dry_run).await {
            Ok(spec) => {
                stats.migrated_successfully += 1;
                *stats.domains.entry(format!("{:?}", spec.spec.domain)).or_insert(0) += 1;
                println!("✅ Migrated: {}", agent_file.display());
            }
            Err(e) => {
                stats.migration_errors += 1;
                println!("❌ Failed to migrate {}: {}", agent_file.display(), e);
            }
        }
    }
    
    print_migration_summary(&stats);
    
    if stats.migration_errors > 0 {
        println!("\n⚠️  Some agents failed to migrate. Please review the errors above.");
        std::process::exit(1);
    }
    
    println!("\n🎉 Migration completed successfully!");
    Ok(())
}

async fn migrate_one(source: PathBuf, target: PathBuf, dry_run: bool) -> Result<()> {
    println!("🔄 Migrating single agent: {} -> {}", source.display(), target.display());
    
    let spec = migrate_agent_file(&source, &target.parent().unwrap().to_path_buf(), dry_run).await?;
    
    println!("✅ Successfully migrated agent: {}", spec.metadata.name);
    println!("   Domain: {:?}", spec.spec.domain);
    println!("   Priority: {:?}", spec.spec.priority);
    println!("   Capabilities: {}", spec.capabilities.primary.len());
    
    Ok(())
}

async fn validate_migrated(directory: PathBuf) -> Result<()> {
    println!("🔍 Validating migrated agent configurations in {}", directory.display());
    
    // Initialize agent system for validation
    let registry = std::sync::Arc::new(ToolRegistry::new());
    let agent_system = AgentSystem::new(registry).await?;
    
    let mut total_validated = 0;
    let mut validation_errors = 0;
    
    let mut entries = fs::read_dir(&directory).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            total_validated += 1;
            
            match agent_system.load_agent_spec(&path).await {
                Ok(spec) => {
                    println!("✅ Valid: {} ({})", spec.metadata.name, path.file_name().unwrap().to_string_lossy());
                }
                Err(e) => {
                    validation_errors += 1;
                    println!("❌ Invalid: {} - {}", path.file_name().unwrap().to_string_lossy(), e);
                }
            }
        }
    }
    
    println!("\n📊 Validation Summary:");
    println!("   Total validated: {}", total_validated);
    println!("   Valid: {}", total_validated - validation_errors);
    println!("   Invalid: {}", validation_errors);
    
    if validation_errors > 0 {
        println!("\n⚠️  Some migrated agents have validation errors. Please review above.");
        std::process::exit(1);
    }
    
    println!("\n🎉 All migrated agents are valid!");
    Ok(())
}

async fn generate_report(source: PathBuf) -> Result<()> {
    println!("📊 Generating agent migration report for {}", source.display());
    
    let agent_files = discover_agent_files(&source).await?;
    let mut stats = MigrationStats {
        total_agents: agent_files.len(),
        migrated_successfully: 0,
        migration_errors: 0,
        validation_errors: 0,
        domains: HashMap::new(),
    };
    
    let mut domain_counts = HashMap::new();
    let mut priority_counts = HashMap::new();
    let mut version_counts = HashMap::new();
    
    for agent_file in &agent_files {
        if let Ok(content) = fs::read_to_string(agent_file).await {
            if let Ok(agent_yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                // Extract domain
                if let Some(domain) = agent_yaml.get("spec").and_then(|s| s.get("domain")).and_then(|d| d.as_str()) {
                    *domain_counts.entry(domain.to_string()).or_insert(0) += 1;
                }
                
                // Extract priority
                if let Some(priority) = agent_yaml.get("spec").and_then(|s| s.get("priority")).and_then(|p| p.as_str()) {
                    *priority_counts.entry(priority.to_string()).or_insert(0) += 1;
                }
                
                // Extract version
                if let Some(version) = agent_yaml.get("metadata").and_then(|m| m.get("version")).and_then(|v| v.as_str()) {
                    *version_counts.entry(version.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    
    println!("\n📈 Agent Configuration Report");
    println!("=" .repeat(50));
    println!("Total Agents: {}", stats.total_agents);
    
    println!("\n🏷️  Domain Distribution:");
    for (domain, count) in domain_counts {
        println!("   {}: {}", domain, count);
    }
    
    println!("\n⚡ Priority Distribution:");
    for (priority, count) in priority_counts {
        println!("   {}: {}", priority, count);
    }
    
    println!("\n🏷️  Version Distribution:");
    for (version, count) in version_counts {
        println!("   {}: {}", version, count);
    }
    
    println!("\n📂 Agent Files:");
    for agent_file in &agent_files {
        println!("   {}", agent_file.display());
    }
    
    Ok(())
}

async fn discover_agent_files(source: &Path) -> Result<Vec<PathBuf>> {
    let mut agent_files = Vec::new();
    
    if !source.exists() {
        return Ok(agent_files);
    }
    
    // Recursively find all YAML files in the source directory
    let mut stack = vec![source.to_path_buf()];
    
    while let Some(dir) = stack.pop() {
        let mut entries = fs::read_dir(&dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                // Skip orchestration and documentation files
                let filename = path.file_name().unwrap().to_string_lossy();
                if !filename.contains("orchestration") && !filename.contains("README") && !filename.contains("guide") {
                    agent_files.push(path);
                }
            }
        }
    }
    
    Ok(agent_files)
}

async fn migrate_agent_file(source: &Path, target_dir: &Path, dry_run: bool) -> Result<AgentSpec> {
    // Read the source agent configuration
    let content = fs::read_to_string(source).await?;
    let agent_yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    // Convert to AgentSpec (this is a simplified conversion)
    let spec = convert_yaml_to_agent_spec(agent_yaml)?;
    
    if !dry_run {
        // Create target directory
        fs::create_dir_all(target_dir).await?;
        
        // Write migrated configuration
        let target_file = target_dir.join(format!("{}.yaml", spec.metadata.name));
        let migrated_content = serde_yaml::to_string(&spec)?;
        fs::write(&target_file, migrated_content).await?;
        
        // Also write as JSON for compatibility
        let target_json = target_dir.join(format!("{}.json", spec.metadata.name));
        let json_content = serde_json::to_string_pretty(&spec)?;
        fs::write(&target_json, json_content).await?;
    }
    
    Ok(spec)
}

fn convert_yaml_to_agent_spec(yaml: serde_yaml::Value) -> Result<AgentSpec> {
    // This is a simplified conversion - in a real implementation, this would
    // handle all the schema transformations and validations
    let spec: AgentSpec = serde_yaml::from_value(yaml)?;
    Ok(spec)
}

fn print_migration_summary(stats: &MigrationStats) {
    println!("\n📊 Migration Summary");
    println!("=" .repeat(40));
    println!("Total agents: {}", stats.total_agents);
    println!("Successfully migrated: {}", stats.migrated_successfully);
    println!("Migration errors: {}", stats.migration_errors);
    println!("Validation errors: {}", stats.validation_errors);
    
    if !stats.domains.is_empty() {
        println!("\n🏷️  Migrated by domain:");
        for (domain, count) in &stats.domains {
            println!("   {}: {}", domain, count);
        }
    }
    
    let success_rate = if stats.total_agents > 0 {
        (stats.migrated_successfully as f64 / stats.total_agents as f64) * 100.0
    } else {
        0.0
    };
    
    println!("\n📈 Success rate: {:.1}%", success_rate);
}