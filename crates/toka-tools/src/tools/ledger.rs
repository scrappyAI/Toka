use super::{Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    id: String,
    date: String,
    amount: f64,
    description: String,
    transaction_type: String,
}

/// Tool for matching and reconciling financial transactions
pub struct LedgerTool {
    name: String,
    description: String,
    version: String,
}

impl LedgerTool {
    pub fn new() -> Self {
        Self {
            name: "ledger".to_string(),
            description: "Match and reconcile credits and debits in financial transactions"
                .to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn parse_transactions(&self, data: &str) -> Result<Vec<Transaction>> {
        serde_json::from_str(data).with_context(|| "Failed to parse transaction data")
    }

    fn match_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Vec<(Transaction, Transaction)> {
        let mut matches = Vec::new();
        let mut credits: HashMap<String, Transaction> = HashMap::new();
        let mut debits: HashMap<String, Transaction> = HashMap::new();

        // Sort transactions into credits and debits
        for tx in transactions {
            match tx.transaction_type.as_str() {
                "credit" => {
                    credits.insert(tx.id.clone(), tx);
                }
                "debit" => {
                    debits.insert(tx.id.clone(), tx);
                }
                _ => continue,
            }
        }

        // Match transactions based on amount and date proximity
        for (_, credit) in credits {
            for (_, debit) in debits.iter() {
                if (credit.amount - debit.amount).abs() < 0.01 {
                    matches.push((credit.clone(), debit.clone()));
                    break;
                }
            }
        }

        matches
    }
}

#[async_trait::async_trait]
impl Tool for LedgerTool {
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

        // Parse transactions
        let transactions = self.parse_transactions(data)?;

        // Match transactions
        let matches = self.match_transactions(transactions);

        // Generate summary
        let summary = format!("Found {} matching pairs of transactions", matches.len());

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
    async fn test_ledger_tool() -> Result<()> {
        let tool = LedgerTool::new();

        // Test data with matching credit and debit
        let test_data = r#"[
            {
                "id": "1",
                "date": "2024-01-01",
                "amount": 100.00,
                "description": "Test credit",
                "transaction_type": "credit"
            },
            {
                "id": "2",
                "date": "2024-01-01",
                "amount": 100.00,
                "description": "Test debit",
                "transaction_type": "debit"
            }
        ]"#;

        let params = ToolParams {
            name: "ledger".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("data".to_string(), test_data.to_string());
                map
            },
        };

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Found 1 matching pairs"));

        Ok(())
    }
}
