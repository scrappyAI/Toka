use super::{Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::tools::resolve_uri_to_path;

/// Represents the standardized data format for all ingested data
#[derive(Debug, Serialize, Deserialize)]
pub struct StandardizedData {
    pub schema_version: String,
    pub source_format: String,
    pub ingestion_timestamp: u64,
    pub metadata: HashMap<String, String>,
    pub columns: Vec<ColumnInfo>,
    pub data: Vec<HashMap<String, serde_cbor::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub semantic_type: String, // temporal, numeric, descriptive, etc.
    pub is_required: bool,
}

/// Tool for importing and validating Data
pub struct IngestionTool {
    name: String,
    description: String,
    version: String,
}

impl IngestionTool {
    pub fn new() -> Self {
        Self {
            name: "ingestion".to_string(),
            description: "Import and validate data for financial processing".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    async fn validate_data(&self, path: &std::path::Path) -> Result<()> {
        // Validate path and read metadata as bytes, avoiding assumptions about file type
        let metadata = tokio::fs::metadata(path)
            .await
            .with_context(|| format!("Failed to access file metadata: {}", path.display()))?;

        // Validate is regular file
        if !metadata.is_file() {
            return Err(anyhow::anyhow!("Path is not a regular file: {}", path.display()));
        }

        // Check file size isn't empty or too large (100MB limit)
        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("File is empty: {}", path.display()));
        }
        if metadata.len() > 100_000_000 {
            return Err(anyhow::anyhow!("File exceeds 100MB limit: {}", path.display()));
        }

        // Validate file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file extension"))?
            .to_lowercase();

        // Allow common financial data formats
        match extension.as_str() {
            "csv" | "tsv" | "xls" | "xlsx" | "json" => Ok(()),
            _ => Err(anyhow::anyhow!(
                "Unsupported file format. Supported formats: CSV, TSV, XLS, XLSX, JSON"
            )),
        }
    }

    async fn read_data(&self, path: &std::path::Path) -> Result<String> {
        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        Ok(contents)
    }

    fn infer_column_type(&self, value: &str) -> String {
        // Try parsing as different types and return the most specific type
        if value.parse::<i64>().is_ok() {
            "integer".to_string()
        } else if value.parse::<f64>().is_ok() {
            "float".to_string()
        } else if chrono::DateTime::parse_from_rfc3339(value).is_ok() {
            "datetime".to_string()
        } else if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() {
            "date".to_string()
        } else {
            "string".to_string()
        }
    }

    fn infer_semantic_type(&self, header: &str, _sample_value: &str) -> String {
        let header_lower = header.to_lowercase();

        if header_lower.contains("date")
            || header_lower.contains("time")
            || header_lower
                .matches(r"(?i)(dt|timestamp|created_at|updated_at)")
                .count()
                > 0
        {
            "temporal".to_string()
        } else if header_lower.contains("amount")
            || header_lower.contains("value")
            || header_lower.contains("price")
            || header_lower.contains("cost")
            || header_lower
                .matches(r"(?i)(qty|quantity|total|sum|balance|num\w*)")
                .count()
                > 0
        {
            "numeric".to_string()
        } else if header_lower.contains("desc")
            || header_lower.contains("description")
            || header_lower.contains("name")
            || header_lower.contains("details")
            || header_lower.contains("notes")
            || header_lower.contains("memo")
            || header_lower
                .matches(r"(?i)(comment|title|label|tag|category)")
                .count()
                > 0
        {
            "descriptive".to_string()
        } else {
            "unknown".to_string()
        }
    }

    async fn convert_to_standardized(
        &self,
        contents: &str,
        source_format: &str,
    ) -> Result<StandardizedData> {
        // Create reader and get headers
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(contents.as_bytes());

        let headers = reader
            .headers()
            .with_context(|| "Failed to read CSV headers")?
            .clone(); // Clone headers to avoid borrow issues

        // Read all records
        let mut records = Vec::new();
        let mut first_row = None;

        for result in reader.records() {
            let record = result.with_context(|| "Failed to read CSV record")?;
            if first_row.is_none() {
                first_row = Some(record.clone());
            }
            let mut row_data = HashMap::new();
            for (header, value) in headers.iter().zip(record.iter()) {
                row_data.insert(
                    header.to_string(),
                    serde_cbor::Value::Text(value.to_string()),
                );
            }
            records.push(row_data);
        }

        // Infer column types and create column info
        let mut columns = Vec::new();
        if let Some(first_row) = first_row {
            for (header, value) in headers.iter().zip(first_row.iter()) {
                let data_type = self.infer_column_type(value);
                let semantic_type = self.infer_semantic_type(header, value);
                columns.push(ColumnInfo {
                    name: header.to_string(),
                    data_type,
                    semantic_type,
                    is_required: true, // Could be made configurable
                });
            }
        }

        // Create standardized data structure
        let standardized = StandardizedData {
            schema_version: "1.0".to_string(),
            source_format: source_format.to_string(),
            ingestion_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::new(), // Could be populated with additional metadata
            columns,
            data: records,
        };

        Ok(standardized)
    }
}

#[async_trait::async_trait]
impl Tool for IngestionTool {
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
        let raw_path = params.args.get("path").ok_or_else(|| anyhow::anyhow!("'path' arg required"))?;
        let path_buf = resolve_uri_to_path(raw_path);
        self.validate_data(&path_buf).await?;
        let data = self.read_data(&path_buf).await?;

        // Get file extension for source format
        let source_format = path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Convert to standardized format
        let standardized = self
            .convert_to_standardized(&data, &source_format)
            .await?;

        // Serialize to CBOR
        let cbor_data =
            serde_cbor::to_vec(&standardized).context("Failed to serialize data to CBOR")?;

        // Save CBOR data to file with .cbor extension
        let output_path = path_buf.with_extension("cbor");
        tokio::fs::write(&output_path, cbor_data)
            .await
            .with_context(|| format!("Failed to write CBOR data to {}", output_path.display()))?;

        Ok(ToolResult {
            success: true,
            output: format!(
                "Successfully converted {} to standardized CBOR format. Output: {}",
                path_buf.display(),
                output_path.display()
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
        if !params.args.contains_key("path") {
            return Err(anyhow::anyhow!("Missing required parameter: path"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_ingestion_tool() -> Result<()> {
        // Create a temporary CSV file with .csv extension
        let mut temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();
        let csv_path = temp_path.with_extension("csv");

        // Write CSV content
        writeln!(temp_file, "date,amount,description")?;
        writeln!(temp_file, "2024-01-01,100.00,Test transaction")?;

        // Move the file to have .csv extension
        fs::rename(temp_path, &csv_path)?;

        let tool = IngestionTool::new();
        let params = ToolParams {
            name: "ingestion".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("path".to_string(), csv_path.to_str().unwrap().to_string());
                map
            },
        };

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Successfully converted"));
        assert!(result.output.contains(".cbor"));

        // Verify CBOR file was created
        let cbor_path = csv_path.with_extension("cbor");
        assert!(cbor_path.exists());

        // Verify CBOR can be deserialized
        // Explicit type annotation prevents inference to a slice and avoids unsized-local errors.
        let cbor_data: Vec<u8> = tokio::fs::read(&cbor_path).await?;
        let _standardized: StandardizedData =
            serde_cbor::from_slice(&cbor_data).context("Failed to deserialize CBOR data")?;

        Ok(())
    }
}
