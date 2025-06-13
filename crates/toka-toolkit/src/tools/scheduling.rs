use super::{Tool, ToolParams, ToolResult, ToolMetadata};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ScheduledTask {
    id: String,
    task: String,
    scheduled_time: DateTime<Utc>,
    status: TaskStatus,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Tool for scheduling and managing financial tasks
#[derive(Clone)]
pub struct SchedulingTool {
    name: String,
    description: String,
    version: String,
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
}

impl SchedulingTool {
    pub fn new() -> Self {
        Self {
            name: "scheduling".to_string(),
            description: "Schedule and manage financial tasks with time-based execution".to_string(),
            version: "1.0.0".to_string(),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn parse_time(&self, time_str: &str) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(time_str)
            .map(|dt| dt.with_timezone(&Utc))
            .with_context(|| format!("Invalid time format: {}", time_str))
    }

    fn validate_schedule_time(&self, time: DateTime<Utc>) -> Result<()> {
        let now = Utc::now();
        if time < now {
            return Err(anyhow::anyhow!("Cannot schedule task in the past"));
        }
        if time > now + Duration::days(365) {
            return Err(anyhow::anyhow!("Cannot schedule task more than 1 year in advance"));
        }
        Ok(())
    }

    async fn get_next_task_id(&self) -> String {
        let tasks = self.tasks.read().await;
        format!("task_{}", tasks.len() + 1)
    }
}

#[async_trait::async_trait]
impl Tool for SchedulingTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let task = params.args.get("task")
            .ok_or_else(|| anyhow::anyhow!("Missing 'task' parameter"))?;
        let time = params.args.get("time")
            .ok_or_else(|| anyhow::anyhow!("Missing 'time' parameter"))?;

        // Parse and validate time
        let scheduled_time = self.parse_time(time)?;
        self.validate_schedule_time(scheduled_time)?;

        // Get next task ID
        let task_id = self.get_next_task_id().await;
        
        // Create task
        let scheduled_task = ScheduledTask {
            id: task_id.clone(),
            task: task.clone(),
            scheduled_time,
            status: TaskStatus::Pending,
        };

        // Store task
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, scheduled_task);
        }

        Ok(ToolResult {
            success: true,
            output: format!(
                "Task '{}' scheduled for {}",
                task,
                scheduled_time.to_rfc3339()
            ),
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("task") {
            return Err(anyhow::anyhow!("Missing required parameter: task"));
        }
        if !params.args.contains_key("time") {
            return Err(anyhow::anyhow!("Missing required parameter: time"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduling_tool() -> Result<()> {
        let tool = SchedulingTool::new();
        
        // Schedule a task for 1 hour from now
        let future_time = (Utc::now() + Duration::hours(1)).to_rfc3339();
        
        let params = ToolParams {
            name: "scheduling".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("task".to_string(), "Process monthly report".to_string());
                map.insert("time".to_string(), future_time);
                map
            },
        };

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("scheduled for"));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_schedule_time() -> Result<()> {
        let tool = SchedulingTool::new();
        
        // Try to schedule a task in the past
        let past_time = (Utc::now() - Duration::hours(1)).to_rfc3339();
        
        let params = ToolParams {
            name: "scheduling".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("task".to_string(), "Invalid task".to_string());
                map.insert("time".to_string(), past_time);
                map
            },
        };

        let result = tool.execute(&params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("past"));

        Ok(())
    }
} 