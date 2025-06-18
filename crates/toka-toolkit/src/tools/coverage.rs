//! Coverage-related tools
//!
//! 1. `coverage-json` — Runs `make coverage-json` and returns path to JSON report.
//! 2. `coverage-analyse` — Consumes the JSON report and outputs a minimal ranked list
//!    of low-coverage files plus overall coverage percentage.  This is an early scaffold
//!    meant for agent consumption; the parsing logic can be improved incrementally.

use crate::tools::{Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use crate::tools::resolve_uri_to_path;

// -----------------------------------------------------------------------------
// CoverageJsonTool
// -----------------------------------------------------------------------------

pub struct CoverageJsonTool;

impl CoverageJsonTool {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Tool for CoverageJsonTool {
    fn name(&self) -> &str {
        "coverage-json"
    }

    fn description(&self) -> &str {
        "Generate llvm-cov JSON report and return its file path"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn execute(&self, _params: &ToolParams) -> Result<ToolResult> {
        // Run `make coverage-json`
        let status = Command::new("make").arg("coverage-json").status()?;
        if !status.success() {
            anyhow::bail!("`make coverage-json` failed with status: {:?}", status);
        }

        Ok(ToolResult {
            success: true,
            output: "coverage/llvm-cov.json".to_string(),
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, _params: &ToolParams) -> Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// CoverageAnalysisTool
// -----------------------------------------------------------------------------

pub struct CoverageAnalysisTool;

impl CoverageAnalysisTool {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_json(coverage_path: &Path) -> Result<CoverageSummary> {
        let data = std::fs::read_to_string(coverage_path)
            .with_context(|| format!("Failed to read {:?}", coverage_path))?;
        let report: LlvmCovReport = serde_json::from_str(&data)?;
        Ok(CoverageSummary::from_report(report))
    }
}

#[async_trait]
impl Tool for CoverageAnalysisTool {
    fn name(&self) -> &str {
        "coverage-analyse"
    }

    fn description(&self) -> &str {
        "Analyse llvm-cov JSON and output a ranked list of low-coverage files"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let path_str = params
            .args
            .get("path")
            .map(String::as_str)
            .unwrap_or("coverage/llvm-cov.json");
        let path = resolve_uri_to_path(path_str);
        let summary = Self::parse_json(path.as_path())?;
        let output = serde_json::to_string_pretty(&summary)?;
        Ok(ToolResult {
            success: true,
            output,
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, _params: &ToolParams) -> Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Simple structures for analysis output
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct CoverageSummary {
    overall_line_coverage: f64,
    worst_files: Vec<FileCoverage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileCoverage {
    path: String,
    line_coverage: f64,
}

impl CoverageSummary {
    fn from_report(report: LlvmCovReport) -> Self {
        // overall
        let overall = report.data.get(0).and_then(|d| d.totals.as_ref());
        let overall_pct = overall
            .map(|t| percent(t.lines.covered, t.lines.count))
            .unwrap_or(0.0);

        // files
        let mut files: Vec<FileCoverage> = report
            .data
            .iter()
            .flat_map(|d| d.files.iter())
            .map(|f| FileCoverage {
                path: f.filename.clone(),
                line_coverage: percent(f.summary.lines.covered, f.summary.lines.count),
            })
            .collect();
        files.sort_by(|a, b| a.line_coverage.partial_cmp(&b.line_coverage).unwrap());
        files.truncate(10);

        Self {
            overall_line_coverage: overall_pct,
            worst_files: files,
        }
    }
}

fn percent(covered: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (covered as f64) * 100.0 / (total as f64)
    }
}

// -----------------------------------------------------------------------------
// Minimal subset of llvm-cov JSON structures (only what we need)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct LlvmCovReport {
    data: Vec<ReportData>,
}

#[derive(Debug, Deserialize)]
struct ReportData {
    totals: Option<Summary>,
    files: Vec<FileData>,
}

#[derive(Debug, Deserialize)]
struct FileData {
    filename: String,
    summary: Summary,
}

#[derive(Debug, Deserialize)]
struct Summary {
    lines: Count,
}

#[derive(Debug, Deserialize)]
struct Count {
    count: u64,
    covered: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_percent_function() {
        assert_eq!(percent(0, 0), 0.0);
        assert_eq!(percent(5, 10), 50.0);
        assert_eq!(percent(3, 4), 75.0);
    }

    #[test]
    fn test_summary_from_report() {
        // Synthetic minimal report JSON
        let fake = json!({
            "data": [
                {
                    "totals": {
                        "lines": { "count": 10, "covered": 7 }
                    },
                    "files": [
                        { "filename": "a.rs", "summary": { "lines": { "count": 10, "covered": 7 } } },
                        { "filename": "b.rs", "summary": { "lines": { "count": 20, "covered": 10 } } }
                    ]
                }
            ]
        });

        let report: LlvmCovReport = serde_json::from_value(fake).unwrap();
        let summary = CoverageSummary::from_report(report);

        // Overall coverage 7/10 = 70 %
        assert!((summary.overall_line_coverage - 70.0).abs() < f64::EPSILON);
        // Worst file should be b.rs (50%)
        assert_eq!(summary.worst_files[0].path, "b.rs");
    }
} 