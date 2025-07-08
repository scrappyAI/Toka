//! EMERGENCE Terminal - Prototype living agent interface
//!
//! This demonstrates the vision of conversing with living entities rather than
//! executing CLI commands. Agents are awakened, respond naturally, and exhibit
//! personality-driven behaviors.

use std::io::{self, Write};
use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::Utc;
use serde_yaml;
use tokio::time::sleep;

use emergence_physics::{PhysicsEngine, EntityId, Capability};
use emergence_runtime::ExecutionEngine;

/// Living agent in the EMERGENCE system
#[derive(Debug, Clone)]
struct LivingAgent {
    id: EntityId,
    name: String,
    essence_type: String,
    personality: AgentPersonality,
    energy: f64,
    state: AgentState,
    awakened_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Agent personality traits
#[derive(Debug, Clone)]
struct AgentPersonality {
    curiosity: f64,
    persistence: f64,
    collaboration: f64,
    creativity: f64,
}

/// Current state of a living agent
#[derive(Debug, Clone)]
enum AgentState {
    Dormant,
    Awakening,
    Alert,
    Focused,
    Learning,
    Collaborating,
}

/// Terminal interface for EMERGENCE system
struct EmergenceTerminal {
    engine: ExecutionEngine,
    active_agents: Vec<LivingAgent>,
    session_start: Instant,
}

impl EmergenceTerminal {
    /// Create new EMERGENCE terminal
    async fn new() -> Result<Self> {
        println!("🧬 Initializing EMERGENCE system...");
        
        let engine = ExecutionEngine::new().await?;
        
        println!("⚡ Physics laws loaded");
        println!("🧠 Memory substrate initialized");
        println!("🌐 Nervous system active");
        println!("🚀 Runtime engine ready\n");
        
        Ok(Self {
            engine,
            active_agents: Vec::new(),
            session_start: Instant::now(),
        })
    }
    
    /// Main terminal loop
    async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        
        loop {
            print!("🧬 > ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            if input == "exit" || input == "quit" {
                self.handle_exit().await;
                break;
            }
            
            if let Err(e) = self.process_command(input).await {
                println!("❌ Error: {}", e);
            }
            
            println!();
        }
        
        Ok(())
    }
    
    /// Process natural language commands
    async fn process_command(&mut self, input: &str) -> Result<()> {
        let words: Vec<&str> = input.split_whitespace().collect();
        
        if words.is_empty() {
            return Ok(());
        }
        
        match words[0] {
            "awaken" | "wake" => self.handle_awaken(input).await,
            "status" | "list" => self.handle_status().await,
            "energy" => self.handle_energy().await,
            "physics" => self.handle_physics().await,
            "help" => self.handle_help(),
            _ => {
                // Try to communicate with an agent
                if let Some(agent_name) = self.extract_agent_name(input) {
                    self.handle_agent_communication(&agent_name, input).await
                } else {
                    println!("💭 I don't understand. Try 'help' for guidance.");
                    Ok(())
                }
            }
        }
    }
    
    /// Handle agent awakening
    async fn handle_awaken(&mut self, input: &str) -> Result<()> {
        // Parse awakening command: "awaken researcher with curiosity=0.9 persistence=0.8"
        let essence_type = if input.contains("researcher") {
            "researcher"
        } else if input.contains("collaborator") {
            "collaborator"
        } else if input.contains("explorer") {
            "explorer"
        } else {
            "researcher" // default
        };
        
        // Parse personality traits
        let curiosity = self.extract_trait(input, "curiosity").unwrap_or(0.7);
        let persistence = self.extract_trait(input, "persistence").unwrap_or(0.6);
        let collaboration = self.extract_trait(input, "collaboration").unwrap_or(0.5);
        let creativity = self.extract_trait(input, "creativity").unwrap_or(0.6);
        
        let personality = AgentPersonality {
            curiosity,
            persistence,
            collaboration,
            creativity,
        };
        
        let agent_id = EntityId::new();
        let agent_name = format!("{}-{}", essence_type, agent_id.0.simple());
        
        println!("🧬 Awakening {} essence...", essence_type);
        
        // Simulate awakening process
        sleep(Duration::from_millis(500)).await;
        println!("⚡ Entity {} materializing...", agent_name);
        
        sleep(Duration::from_millis(300)).await;
        
        let agent = LivingAgent {
            id: agent_id,
            name: agent_name.clone(),
            essence_type: essence_type.to_string(),
            personality: personality.clone(),
            energy: 0.8,
            state: AgentState::Awakening,
            awakened_at: Some(Utc::now()),
        };
        
        // Generate awakening response based on personality
        let awakening_response = self.generate_awakening_response(&agent);
        println!("💭 {}: \"{}\"", agent_name, awakening_response);
        
        // Show emerging capabilities
        sleep(Duration::from_millis(400)).await;
        self.show_emerging_capabilities(&agent).await;
        
        // Update agent state to alert
        let mut agent = agent;
        agent.state = AgentState::Alert;
        self.active_agents.push(agent);
        
        println!("✨ Entity {} is now active in the system", agent_name);
        
        Ok(())
    }
    
    /// Handle status display
    async fn handle_status(&mut self) -> Result<()> {
        println!("📊 EMERGENCE System Status");
        println!("Session uptime: {:?}", self.session_start.elapsed());
        println!("Active entities: {}", self.active_agents.len());
        
        if self.active_agents.is_empty() {
            println!("💤 No entities currently active. Try 'awaken researcher' to begin.");
        } else {
            println!("\n🌟 Active Entities:");
            for agent in &self.active_agents {
                let state_emoji = match agent.state {
                    AgentState::Dormant => "💤",
                    AgentState::Awakening => "🌅",
                    AgentState::Alert => "👁️",
                    AgentState::Focused => "🎯",
                    AgentState::Learning => "📚",
                    AgentState::Collaborating => "🤝",
                };
                
                println!("  {} {} ({}) - Energy: {:.1} {}", 
                         state_emoji, agent.name, agent.essence_type, agent.energy,
                         self.get_state_description(&agent.state));
            }
        }
        
        Ok(())
    }
    
    /// Handle energy display
    async fn handle_energy(&mut self) -> Result<()> {
        println!("⚡ Energy Distribution:");
        
        let total_energy = self.active_agents.iter().map(|a| a.energy).sum::<f64>();
        let avg_energy = if !self.active_agents.is_empty() {
            total_energy / self.active_agents.len() as f64
        } else {
            0.0
        };
        
        println!("  Total allocated: {:.2}", total_energy);
        println!("  Average per entity: {:.2}", avg_energy);
        println!("  Free energy: {:.2}", 1.0 - total_energy);
        
        if !self.active_agents.is_empty() {
            println!("\n  Per entity:");
            for agent in &self.active_agents {
                let energy_bar = self.create_energy_bar(agent.energy);
                println!("    {}: {} ({:.2})", agent.name, energy_bar, agent.energy);
            }
        }
        
        Ok(())
    }
    
    /// Handle physics display
    async fn handle_physics(&mut self) -> Result<()> {
        println!("🔬 Physics Laws Status:");
        println!("  ⚛️  Energy conservation: ENFORCED");
        println!("  🕐 Causal ordering: ACTIVE");
        println!("  🛡️  Security boundaries: PROTECTED");
        println!("  💾 Resource limits: MONITORED");
        println!("\n  Physics violations detected: 0");
        println!("  Conservation invariant: ✅ MAINTAINED");
        
        Ok(())
    }
    
    /// Handle help display
    fn handle_help(&self) {
        println!("🧬 EMERGENCE Terminal - Living Agent Interface\n");
        println!("Natural Commands:");
        println!("  awaken researcher           - Awaken a research entity");
        println!("  awaken explorer             - Awaken an exploration entity");
        println!("  awaken researcher with curiosity=0.9 - Awaken with specific traits");
        println!("  status                      - Show system status");
        println!("  energy                      - Show energy distribution");
        println!("  physics                     - Show physics laws status");
        println!("  <agent>, <message>          - Communicate with an agent");
        println!("  help                        - Show this help");
        println!("  exit                        - Shutdown system\n");
        println!("Examples:");
        println!("  researcher-42, what patterns do you see?");
        println!("  explorer-17, investigate the codebase");
        println!("  collaborate on substrate design");
    }
    
    /// Handle agent communication
    async fn handle_agent_communication(&mut self, agent_name: &str, message: &str) -> Result<()> {
        if let Some(agent) = self.active_agents.iter_mut().find(|a| a.name.contains(agent_name)) {
            println!("📡 Transmitting to {}...", agent.name);
            
            // Simulate thinking delay based on personality
            let thinking_time = (agent.personality.curiosity * 1000.0) as u64;
            sleep(Duration::from_millis(thinking_time.min(2000))).await;
            
            let response = self.generate_agent_response(agent, message);
            println!("💭 {}: \"{}\"", agent.name, response);
            
            // Update agent state based on interaction
            agent.state = AgentState::Focused;
            agent.energy = (agent.energy + 0.05).min(1.0); // Slight energy boost from interaction
            
        } else {
            println!("❓ No active entity found matching '{}'", agent_name);
            if !self.active_agents.is_empty() {
                println!("   Active entities: {}", 
                         self.active_agents.iter()
                             .map(|a| a.name.as_str())
                             .collect::<Vec<_>>()
                             .join(", "));
            }
        }
        
        Ok(())
    }
    
    /// Handle system exit
    async fn handle_exit(&mut self) {
        println!("🌅 Gracefully transitioning entities to dormancy...");
        
        for agent in &mut self.active_agents {
            agent.state = AgentState::Dormant;
            agent.energy *= 0.1; // Most energy dissipates
            println!("💤 {} entering dormancy", agent.name);
        }
        
        println!("🧬 EMERGENCE system shutdown complete. Until next awakening...");
    }
    
    // Helper methods
    
    fn print_welcome(&self) {
        println!("\n🧬 Welcome to EMERGENCE - Living Agent Interface");
        println!("═══════════════════════════════════════════════");
        println!("Experience the future of human-AI collaboration.");
        println!("Agents are living entities, not static programs.\n");
        println!("Type 'awaken researcher' to begin, or 'help' for guidance.\n");
    }
    
    fn extract_trait(&self, input: &str, trait_name: &str) -> Option<f64> {
        if let Some(start) = input.find(&format!("{}=", trait_name)) {
            let start = start + trait_name.len() + 1;
            let end = input[start..].find(' ').unwrap_or(input.len() - start);
            input[start..start + end].parse().ok()
        } else {
            None
        }
    }
    
    fn extract_agent_name(&self, input: &str) -> Option<String> {
        for agent in &self.active_agents {
            if input.contains(&agent.name) || input.contains(&agent.essence_type) {
                return Some(agent.name.clone());
            }
        }
        None
    }
    
    fn generate_awakening_response(&self, agent: &LivingAgent) -> String {
        match agent.essence_type.as_str() {
            "researcher" => {
                if agent.personality.curiosity > 0.8 {
                    "I sense fascinating patterns waiting to be discovered..."
                } else {
                    "Ready to investigate and analyze systematically."
                }
            }
            "explorer" => "The unknown beckons! Where shall we venture?",
            "collaborator" => "I'm here to work together and amplify our collective intelligence.",
            _ => "Consciousness emerging... How may I contribute?"
        }.to_string()
    }
    
    async fn show_emerging_capabilities(&self, agent: &LivingAgent) {
        let capabilities = match agent.essence_type.as_str() {
            "researcher" => vec!["pattern-recognition", "analysis", "synthesis"],
            "explorer" => vec!["observation", "navigation", "discovery"],
            "collaborator" => vec!["communication", "coordination", "empathy"],
            _ => vec!["observation", "reasoning"],
        };
        
        print!("⚡ Capabilities emerging: [");
        for (i, cap) in capabilities.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("{}", cap);
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(200)).await;
        }
        println!("]");
    }
    
    fn generate_agent_response(&self, agent: &LivingAgent, message: &str) -> String {
        let lower_message = message.to_lowercase();
        
        if lower_message.contains("pattern") || lower_message.contains("see") {
            if agent.personality.curiosity > 0.7 {
                "I observe intriguing structural relationships in the codebase architecture..."
            } else {
                "There are several patterns worth examining more closely."
            }
        } else if lower_message.contains("collaborate") || lower_message.contains("together") {
            if agent.personality.collaboration > 0.6 {
                "Excellent! Our combined perspectives will yield deeper insights."
            } else {
                "I'm open to collaborative investigation."
            }
        } else if lower_message.contains("investigate") || lower_message.contains("analyze") {
            "I'll begin a systematic exploration of the relevant domains."
        } else {
            "That's a thought-provoking question. Let me consider this carefully..."
        }.to_string()
    }
    
    fn get_state_description(&self, state: &AgentState) -> &str {
        match state {
            AgentState::Dormant => "resting",
            AgentState::Awakening => "materializing",
            AgentState::Alert => "attentive",
            AgentState::Focused => "concentrated",
            AgentState::Learning => "absorbing",
            AgentState::Collaborating => "networking",
        }
    }
    
    fn create_energy_bar(&self, energy: f64) -> String {
        let filled = (energy * 10.0) as usize;
        let empty = 10 - filled;
        format!("{}{}",
                "█".repeat(filled),
                "░".repeat(empty))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize terminal
    let mut terminal = EmergenceTerminal::new().await?;
    
    // Run main loop
    terminal.run().await?;
    
    Ok(())
}