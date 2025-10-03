use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MetricsSummaryInput {
    pub path: String,
    #[serde(default)]
    pub include_history: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MetricsSummaryOutput {
    pub complexity: MetricsResult,
    pub satd: MetricsResult,
    pub tdg: MetricsResult,
    pub cognitive: MetricsResult,
    pub summary: QualitySummary,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MetricsResult {
    pub passed: bool,
    pub value: String,
    pub threshold: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct QualitySummary {
    pub overall_grade: String,
    pub passed_checks: u32,
    pub total_checks: u32,
    pub recommendations: Vec<String>,
}

pub struct MetricsSummary;

#[async_trait::async_trait]
impl Handler for MetricsSummary {
    type Input = MetricsSummaryInput;
    type Output = MetricsSummaryOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Run all PMAT analyses
        let complexity_result = run_pmat_command(&["analyze", "complexity", "--max", "20", &input.path])?;
        let satd_result = run_pmat_command(&["analyze", "satd", "--max", "0", &input.path])?;
        let tdg_result = run_pmat_command(&["analyze", "tdg", "--min", "0.75", &input.path])?;
        let cognitive_result = run_pmat_command(&["analyze", "cognitive", "--max", "15", &input.path])?;

        // Note: include_history parameter reserved for future use (historical trend analysis)
        let _include_history = input.include_history;

        // Parse results (simplified - in production, parse actual JSON output)
        let complexity_passed = complexity_result.contains("PASS") || !complexity_result.contains("FAIL");
        let satd_passed = satd_result.contains("PASS") || !satd_result.contains("FAIL");
        let tdg_passed = tdg_result.contains("PASS") || !tdg_result.contains("FAIL");
        let cognitive_passed = cognitive_result.contains("PASS") || !cognitive_result.contains("FAIL");

        let passed_checks = [complexity_passed, satd_passed, tdg_passed, cognitive_passed]
            .iter()
            .filter(|&&x| x)
            .count() as u32;

        let overall_grade = calculate_grade(passed_checks, 4);
        let recommendations = generate_recommendations(
            complexity_passed,
            satd_passed,
            tdg_passed,
            cognitive_passed,
        );

        Ok(MetricsSummaryOutput {
            complexity: MetricsResult {
                passed: complexity_passed,
                value: extract_value(&complexity_result),
                threshold: "≤20".to_string(),
            },
            satd: MetricsResult {
                passed: satd_passed,
                value: extract_value(&satd_result),
                threshold: "0".to_string(),
            },
            tdg: MetricsResult {
                passed: tdg_passed,
                value: extract_value(&tdg_result),
                threshold: "≥0.75".to_string(),
            },
            cognitive: MetricsResult {
                passed: cognitive_passed,
                value: extract_value(&cognitive_result),
                threshold: "≤15".to_string(),
            },
            summary: QualitySummary {
                overall_grade,
                passed_checks,
                total_checks: 4,
                recommendations,
            },
        })
    }
}

fn run_pmat_command(args: &[&str]) -> Result<String> {
    let output = Command::new("pmat")
        .args(args)
        .output()
        .map_err(|e| pforge_runtime::Error::Handler(format!("Failed to run pmat: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn extract_value(output: &str) -> String {
    // Simplified extraction - in production, parse JSON
    output
        .lines()
        .find(|line| line.contains("value:") || line.contains("score:"))
        .map(|line| {
            line.split(':')
                .nth(1)
                .unwrap_or("unknown")
                .trim()
                .to_string()
        })
        .unwrap_or_else(|| "N/A".to_string())
}

fn calculate_grade(passed: u32, total: u32) -> String {
    let percentage = (passed as f64 / total as f64) * 100.0;
    match percentage as u32 {
        100 => "A+".to_string(),
        90..=99 => "A".to_string(),
        80..=89 => "B".to_string(),
        70..=79 => "C".to_string(),
        60..=69 => "D".to_string(),
        _ => "F".to_string(),
    }
}

fn generate_recommendations(
    complexity: bool,
    satd: bool,
    tdg: bool,
    cognitive: bool,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !complexity {
        recommendations.push(
            "Reduce cyclomatic complexity by breaking down complex functions".to_string(),
        );
    }
    if !satd {
        recommendations.push(
            "Remove or address Self-Admitted Technical Debt (SATD) comments".to_string(),
        );
    }
    if !tdg {
        recommendations.push("Improve Technical Debt Grade by addressing code quality issues".to_string());
    }
    if !cognitive {
        recommendations.push(
            "Simplify code to reduce cognitive complexity".to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push("All quality checks passed! Keep up the good work.".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_grade() {
        assert_eq!(calculate_grade(4, 4), "A+");  // 100%
        assert_eq!(calculate_grade(3, 4), "C");   // 75%
        assert_eq!(calculate_grade(2, 4), "F");   // 50%
        assert_eq!(calculate_grade(1, 4), "F");   // 25%
        assert_eq!(calculate_grade(0, 4), "F");   // 0%
    }

    #[test]
    fn test_generate_recommendations_all_pass() {
        let recs = generate_recommendations(true, true, true, true);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].contains("All quality checks passed"));
    }

    #[test]
    fn test_generate_recommendations_some_fail() {
        let recs = generate_recommendations(false, true, false, true);
        assert_eq!(recs.len(), 2);
        assert!(recs.iter().any(|r| r.contains("complexity")));
        assert!(recs.iter().any(|r| r.contains("Technical Debt Grade")));
    }
}
