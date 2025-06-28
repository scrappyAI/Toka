use super::{Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct FinancialData {
    transactions: Vec<Transaction>,
    metadata: ReportMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    date: String,
    amount: f64,
    category: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportMetadata {
    period_start: String,
    period_end: String,
    currency: String,
}

/// Tool for generating financial reports and summaries
pub struct ReportingTool {
    name: String,
    description: String,
    version: String,
}

impl ReportingTool {
    pub fn new() -> Self {
        Self {
            name: "reporting".to_string(),
            description: "Generate financial reports and summaries from transaction data"
                .to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn parse_data(&self, data: &str) -> Result<FinancialData> {
        serde_json::from_str(data).with_context(|| "Failed to parse financial data")
    }

    fn generate_summary(&self, data: &FinancialData) -> String {
        let mut total_income = 0.0;
        let mut total_expenses = 0.0;
        let mut category_totals: HashMap<String, f64> = HashMap::new();

        for tx in &data.transactions {
            if tx.amount > 0.0 {
                total_income += tx.amount;
            } else {
                total_expenses += tx.amount.abs();
            }
            *category_totals.entry(tx.category.clone()).or_insert(0.0) += tx.amount;
        }

        let net = total_income - total_expenses;
        let mut summary = format!(
            "Financial Summary\n\
             Period: {} to {}\n\
             Currency: {}\n\
             Total Income: {:.2}\n\
             Total Expenses: {:.2}\n\
             Net: {:.2}\n\n\
             Category Breakdown:\n",
            data.metadata.period_start,
            data.metadata.period_end,
            data.metadata.currency,
            total_income,
            total_expenses,
            net
        );

        for (category, amount) in category_totals {
            summary.push_str(&format!("{}: {:.2}\n", category, amount));
        }

        summary
    }
}

#[async_trait::async_trait]
impl Tool for ReportingTool {
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
        let data = params
            .args
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;

        // Parse financial data
        let financial_data = self.parse_data(data)?;

        // Generate summary
        let summary = self.generate_summary(&financial_data);

        Ok(ToolResult {
            success: true,
            output: summary,
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
        if !params.args.contains_key("data") {
            return Err(anyhow::anyhow!("Missing required parameter: data"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reporting_tool() -> Result<()> {
        let tool = ReportingTool::new();

        // Test data
        let test_data = r#"{
            "transactions": [
                {
                    "date": "2024-01-01",
                    "amount": 1000.00,
                    "category": "Salary",
                    "description": "Monthly salary"
                },
                {
                    "date": "2024-01-02",
                    "amount": -500.00,
                    "category": "Rent",
                    "description": "Monthly rent"
                }
            ],
            "metadata": {
                "period_start": "2024-01-01",
                "period_end": "2024-01-31",
                "currency": "USD"
            }
        }"#;

        let params = ToolParams {
            name: "reporting".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("data".to_string(), test_data.to_string());
                map
            },
        };

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Total Income: 1000.00"));
        assert!(result.output.contains("Total Expenses: 500.00"));
        assert!(result.output.contains("Net: 500.00"));

        Ok(())
    }
}
