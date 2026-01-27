use anyhow::{anyhow, Result};
use clap::Parser;
use cliai::{Config, History, Orchestrator};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{timeout, Duration};

#[derive(Parser, Debug)]
#[command(name = "compare_runner")]
#[command(about = "Run scenarios in local vs cloud mode and save a report")]
struct Cli {
    /// Path to scenarios JSON file (defaults to tests/scenarios/real_world_50.json)
    #[arg(long)]
    scenarios: Option<PathBuf>,

    /// Output markdown file (defaults to reports/comparison_<timestamp>.md)
    #[arg(long)]
    out: Option<PathBuf>,

    /// Local model name to set before local run (optional).
    /// If not set, uses config.json's model.
    #[arg(long)]
    local_model: Option<String>,

    /// Cloud backend URL override (optional).
    #[arg(long)]
    backend_url: Option<String>,

    /// Cloud token override (optional). If not provided, uses config.json api_token.
    #[arg(long)]
    cloud_token: Option<String>,

    /// Override AI timeout for each run (ms). Default: 60000.
    #[arg(long, default_value_t = 60000)]
    timeout_ms: u64,

    /// Run only local mode (skip cloud).
    #[arg(long, default_value_t = false)]
    local_only: bool,

    /// Run only cloud mode (skip local).
    #[arg(long, default_value_t = false)]
    cloud_only: bool,

    /// Only run the first N scenarios (useful for quick smoke tests).
    #[arg(long)]
    max: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    id: u32,
    category: String,
    prompt: String,
}

#[derive(Debug)]
struct RunRow {
    id: u32,
    category: String,
    prompt: String,
    local_ms: u128,
    local_response: String,
    cloud_ms: Option<u128>,
    cloud_response: Option<String>,
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string()
}

fn read_scenarios(path: &PathBuf) -> Result<Vec<Scenario>> {
    let raw = fs::read_to_string(path)?;
    let scenarios: Vec<Scenario> = serde_json::from_str(&raw)?;
    if scenarios.len() != 50 {
        return Err(anyhow!(
            "Expected 50 scenarios, got {} from {}",
            scenarios.len(),
            path.display()
        ));
    }
    Ok(scenarios)
}

fn redact(s: &str) -> String {
    // Keep the report readable; avoid megabytes of logs.
    const LIMIT: usize = 2000;
    let trimmed = s.trim().to_string();
    if trimmed.len() <= LIMIT {
        return trimmed;
    }
    format!("{}… (truncated)", &trimmed[..LIMIT])
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let scenarios_path = cli
        .scenarios
        .unwrap_or_else(|| PathBuf::from("tests/scenarios/real_world_50.json"));
    let mut scenarios = read_scenarios(&scenarios_path)?;
    if let Some(max) = cli.max {
        scenarios.truncate(max);
    }

    let mut base_config = Config::load();
    if let Some(url) = cli.backend_url.clone() {
        base_config.backend_url = url;
    }
    if let Some(token) = cli.cloud_token.clone() {
        base_config.api_token = Some(token);
    }

    let out_path = cli.out.unwrap_or_else(|| {
        let _ = fs::create_dir_all("reports");
        PathBuf::from(format!("reports/comparison_{}.md", now_stamp()))
    });

    // Prepare configs
    let mut local_config = base_config.clone();
    local_config.use_cloud = false;
    local_config.ai_timeout = cli.timeout_ms;
    if let Some(m) = cli.local_model.clone() {
        local_config.model = m;
    }

    let mut cloud_config = base_config.clone();
    cloud_config.use_cloud = true;
    cloud_config.ai_timeout = cli.timeout_ms;
    if cloud_config.api_token.is_none() {
        if cli.local_only {
            // OK: user explicitly requested local-only.
        } else {
            return Err(anyhow!(
                "Cloud run requires a token. Provide --cloud-token or run `cliai login` first, or pass --local-only."
            ));
        }
    }

    let mut rows: Vec<RunRow> = Vec::with_capacity(scenarios.len());

    for (i, sc) in scenarios.iter().enumerate() {
        eprintln!(
            "Running {}/{} (#{}) {}",
            i + 1,
            scenarios.len(),
            sc.id,
            sc.prompt
        );

        // Local
        let (local_ms, local_response) = if cli.cloud_only {
            (0, "SKIPPED (cloud-only)".to_string())
        } else {
            let mut orch_local = Orchestrator::new(local_config.clone(), History::load());
            let start = Instant::now();
            let res = timeout(
                Duration::from_millis(cli.timeout_ms),
                orch_local.process(&sc.prompt),
            )
            .await;
            let ms = start.elapsed().as_millis();
            let txt = match res {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => format!("ERROR: {e}"),
                Err(_) => format!("TIMEOUT after {}ms", cli.timeout_ms),
            };
            (ms, txt)
        };

        // Cloud
        let (cloud_ms, cloud_response) = if cli.local_only {
            (None, None)
        } else {
            let mut orch_cloud = Orchestrator::new(cloud_config.clone(), History::load());
            let start = Instant::now();
            let res = timeout(
                Duration::from_millis(cli.timeout_ms),
                orch_cloud.process(&sc.prompt),
            )
            .await;
            let ms = start.elapsed().as_millis();
            let txt = match res {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => format!("ERROR: {e}"),
                Err(_) => format!("TIMEOUT after {}ms", cli.timeout_ms),
            };
            (Some(ms), Some(txt))
        };

        rows.push(RunRow {
            id: sc.id,
            category: sc.category.clone(),
            prompt: sc.prompt.clone(),
            local_ms,
            local_response: redact(&local_response),
            cloud_ms,
            cloud_response: cloud_response.map(|s| redact(&s)),
        });
    }

    // Write report
    let mut md = String::new();
    md.push_str("# CLIAI Local vs Cloud Comparison Report\n\n");
    md.push_str(&format!(
        "- Generated: {}\n- Scenarios: {}\n- Local model: `{}`\n- Cloud backend: `{}`\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        scenarios_path.display(),
        local_config.model,
        cloud_config.backend_url
    ));

    md.push_str("## Results (side-by-side)\n\n");
    for r in &rows {
        md.push_str(&format!("### {}. [{}]\n\n", r.id, r.category));
        md.push_str(&format!("**Prompt:** {}\n\n", r.prompt));
        md.push_str(&format!("**Local ({:}ms)**\n\n```\n{}\n```\n\n", r.local_ms, r.local_response));
        if let (Some(ms), Some(resp)) = (r.cloud_ms, r.cloud_response.as_deref()) {
            md.push_str(&format!("**Cloud ({:}ms)**\n\n```\n{}\n```\n\n", ms, resp));
        } else {
            md.push_str("**Cloud**\n\n```\nSKIPPED\n```\n\n");
        }
        md.push_str("---\n\n");
    }

    fs::write(&out_path, md)?;
    println!("Saved report to {}", out_path.display());
    Ok(())
}

