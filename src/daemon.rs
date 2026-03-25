use std::{fs, path::Path, thread, time::Duration};

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    config::BotConfig,
    data::CsvDataPortal,
    engine::{summarize_result, QuantBotEngine},
    output::write_outputs,
    paper_hints::{build_paper_hints, render_paper_hints_summary, PaperHintsDaemonInput},
};

#[derive(Debug, Clone)]
pub struct PaperDaemonRequest {
    pub cycles: usize,
    pub sleep_secs: u64,
    pub alert_drawdown_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct PaperDaemonReport {
    pub cycles_run: usize,
    pub alerts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DaemonState {
    updated_at_utc: String,
    last_cycle: usize,
    last_end_equity: f64,
    max_drawdown_observed: f64,
    alerts: usize,
}

pub fn run_paper_daemon(
    cfg: &BotConfig,
    data: &CsvDataPortal,
    output_dir: impl AsRef<Path>,
    req: &PaperDaemonRequest,
) -> Result<PaperDaemonReport> {
    if req.cycles == 0 {
        return Err(anyhow!("cycles must be > 0"));
    }
    if req.alert_drawdown_ratio < 0.0 {
        return Err(anyhow!("alert_drawdown_ratio must be >= 0"));
    }

    let out_dir = output_dir.as_ref();
    let cycle_dir = out_dir.join("paper_daemon_cycles");
    fs::create_dir_all(&cycle_dir)?;
    let alerts_path = out_dir.join("paper_daemon_alerts.log");

    let mut state = DaemonState {
        updated_at_utc: Utc::now().to_rfc3339(),
        last_cycle: 0,
        last_end_equity: 0.0,
        max_drawdown_observed: 0.0,
        alerts: 0,
    };

    for cycle in 1..=req.cycles {
        let result = QuantBotEngine::from_config_force_sim(cfg.clone(), data.clone()).run();
        let stats = summarize_result(&result);
        let per_cycle_output = cycle_dir.join(format!("cycle_{cycle:03}"));
        write_outputs(&per_cycle_output, &result)?;

        state.updated_at_utc = Utc::now().to_rfc3339();
        state.last_cycle = cycle;
        state.last_end_equity = stats.end_equity;
        if stats.max_drawdown > state.max_drawdown_observed {
            state.max_drawdown_observed = stats.max_drawdown;
        }

        if stats.max_drawdown >= req.alert_drawdown_ratio {
            state.alerts += 1;
            let line = format!(
                "{} cycle={} max_drawdown={:.6} threshold={:.6}\n",
                state.updated_at_utc, cycle, stats.max_drawdown, req.alert_drawdown_ratio
            );
            fs::write(
                &alerts_path,
                if alerts_path.exists() {
                    let mut prev = fs::read_to_string(&alerts_path)?;
                    prev.push_str(&line);
                    prev
                } else {
                    line
                },
            )?;
        }

        if cycle < req.cycles && req.sleep_secs > 0 {
            thread::sleep(Duration::from_secs(req.sleep_secs));
        }
    }

    fs::write(
        out_dir.join("paper_daemon_state.json"),
        serde_json::to_string_pretty(&state)?,
    )?;
    let summary = format!(
        "cycles_run={}\nalerts={}\nlast_end_equity={:.2}\nmax_drawdown_observed={:.6}\n",
        state.last_cycle, state.alerts, state.last_end_equity, state.max_drawdown_observed
    );
    fs::write(out_dir.join("paper_daemon_summary.txt"), summary)?;
    let research_summary = read_kv_file(&first_existing_path(&[
        out_dir.join("research_report_summary.txt"),
        out_dir
            .join("research_report")
            .join("research_report_summary.txt"),
    ]))?;
    let hints = build_paper_hints(
        &research_summary,
        Some(&PaperHintsDaemonInput {
            last_cycle: state.last_cycle,
            last_end_equity: state.last_end_equity,
            max_drawdown_observed: state.max_drawdown_observed,
            alerts: state.alerts,
        }),
        None,
    );
    fs::write(
        out_dir.join("paper_hints_summary.txt"),
        render_paper_hints_summary(&hints),
    )?;

    Ok(PaperDaemonReport {
        cycles_run: state.last_cycle,
        alerts: state.alerts,
    })
}

fn first_existing_path(paths: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
    paths.iter().find(|path| path.exists()).cloned()
}

fn read_kv_file(
    path: &Option<std::path::PathBuf>,
) -> Result<std::collections::HashMap<String, String>> {
    let Some(path) = path else {
        return Ok(std::collections::HashMap::new());
    };
    let text = fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{config::load_config, data::CsvDataPortal};

    use super::{run_paper_daemon, PaperDaemonRequest};

    #[test]
    fn daemon_generates_state_files() {
        let cfg = load_config("config/bot.toml").expect("load config");
        let data = CsvDataPortal::new(
            cfg.markets
                .values()
                .map(|m| (m.name.clone(), m.data_file.clone()))
                .collect(),
        )
        .expect("load data");

        let report = run_paper_daemon(
            &cfg,
            &data,
            "outputs_rust/test_daemon",
            &PaperDaemonRequest {
                cycles: 1,
                sleep_secs: 0,
                alert_drawdown_ratio: 0.01,
            },
        )
        .expect("run daemon");
        assert_eq!(report.cycles_run, 1);
        assert!(std::path::Path::new("outputs_rust/test_daemon/paper_hints_summary.txt").exists());
    }
}
