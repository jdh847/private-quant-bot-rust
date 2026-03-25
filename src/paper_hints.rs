use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PaperHintsDaemonInput {
    pub last_cycle: usize,
    pub last_end_equity: f64,
    pub max_drawdown_observed: f64,
    pub alerts: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PaperHintsCompareInput {
    pub winner: String,
    pub research_changes: usize,
    pub top_research_keys: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PaperHintsReport {
    pub stance: String,
    pub headline: String,
    pub watch_markets: Vec<String>,
    pub bullets: Vec<String>,
}

pub fn build_paper_hints(
    research_summary: &HashMap<String, String>,
    daemon: Option<&PaperHintsDaemonInput>,
    compare: Option<&PaperHintsCompareInput>,
) -> PaperHintsReport {
    let top_market = value_or_dash(research_summary.get("top_regime_leader_market"));
    let top_bucket = value_or_dash(research_summary.get("top_regime_leader_bucket"));
    let top_factor = value_or_dash(research_summary.get("top_regime_leader_factor"));
    let rotation_factor = value_or_dash(research_summary.get("current_rotation_leader_factor"));
    let rotation_date = value_or_dash(research_summary.get("current_rotation_date"));
    let rotation_switches = research_summary
        .get("rotation_switches")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let transition_market = value_or_dash(research_summary.get("latest_regime_transition_market"));
    let transition_date = value_or_dash(research_summary.get("latest_regime_transition_date"));
    let transition_from =
        value_or_dash(research_summary.get("latest_regime_transition_from_bucket"));
    let transition_to = value_or_dash(research_summary.get("latest_regime_transition_to_bucket"));

    let mut watch_markets = Vec::<String>::new();
    for market in [top_market.as_str(), transition_market.as_str()] {
        if market != "-" && !watch_markets.iter().any(|item| item == market) {
            watch_markets.push(market.to_string());
        }
    }

    let compare_changes = compare.map(|item| item.research_changes).unwrap_or(0);
    let daemon_alerts = daemon.map(|item| item.alerts).unwrap_or(0);
    let stance = if daemon_alerts > 0 || compare_changes >= 8 {
        "RISK"
    } else if transition_market != "-" || compare_changes > 0 || rotation_switches > 1 {
        "WATCH"
    } else {
        "HEALTHY"
    }
    .to_string();

    let headline = if transition_market != "-" && rotation_factor != "-" {
        format!(
            "paper-only: {transition_market} shifted {transition_from} -> {transition_to}; watch {rotation_factor}"
        )
    } else if top_market != "-" && top_factor != "-" {
        format!("paper-only: {top_market} leader is {top_factor} in {top_bucket}")
    } else if let Some(daemon) = daemon {
        format!(
            "paper-only: daemon cycle={} alerts={} last_end_equity={:.2}",
            daemon.last_cycle, daemon.alerts, daemon.last_end_equity
        )
    } else {
        "paper-only: waiting for research signals".to_string()
    };

    let mut bullets = Vec::new();
    if top_market != "-" {
        bullets.push(format!(
            "leader: {top_market} / {top_bucket} / {top_factor}"
        ));
    }
    if rotation_factor != "-" {
        bullets.push(format!(
            "rotation: {rotation_factor} on {rotation_date} (switches={rotation_switches})"
        ));
    }
    if transition_market != "-" {
        bullets.push(format!(
            "transition: {transition_market} {transition_from} -> {transition_to} on {transition_date}"
        ));
    }
    if let Some(compare) = compare {
        let top_keys = if compare.top_research_keys.is_empty() {
            "-".to_string()
        } else {
            compare.top_research_keys.join(",")
        };
        bullets.push(format!(
            "compare: winner={} research_changes={} top={}",
            if compare.winner.is_empty() {
                "-"
            } else {
                compare.winner.as_str()
            },
            compare.research_changes,
            top_keys
        ));
    }
    if let Some(daemon) = daemon {
        bullets.push(format!(
            "daemon: alerts={} max_drawdown_observed={:.2}%",
            daemon.alerts,
            daemon.max_drawdown_observed * 100.0
        ));
    }
    if bullets.is_empty() {
        bullets.push("paper-only: no actionable signals yet".to_string());
    }

    PaperHintsReport {
        stance,
        headline,
        watch_markets,
        bullets,
    }
}

pub fn render_paper_hints_summary(report: &PaperHintsReport) -> String {
    let markets = if report.watch_markets.is_empty() {
        "-".to_string()
    } else {
        report.watch_markets.join("|")
    };
    let mut out = format!(
        "stance={}\nheadline={}\nwatch_markets={}\nbullets_count={}\n",
        report.stance,
        report.headline,
        markets,
        report.bullets.len()
    );
    for (idx, bullet) in report.bullets.iter().enumerate() {
        out.push_str(&format!("bullet_{}={}\n", idx + 1, bullet));
    }
    out
}

fn value_or_dash(value: Option<&String>) -> String {
    value
        .map(|item| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        build_paper_hints, render_paper_hints_summary, PaperHintsCompareInput,
        PaperHintsDaemonInput,
    };

    #[test]
    fn paper_hints_prioritize_transition_and_compare() {
        let mut research = HashMap::new();
        research.insert("top_regime_leader_market".to_string(), "US".to_string());
        research.insert(
            "top_regime_leader_bucket".to_string(),
            "trend_up_low_vol".to_string(),
        );
        research.insert(
            "top_regime_leader_factor".to_string(),
            "momentum".to_string(),
        );
        research.insert(
            "current_rotation_leader_factor".to_string(),
            "volume".to_string(),
        );
        research.insert(
            "current_rotation_date".to_string(),
            "2026-01-04".to_string(),
        );
        research.insert("rotation_switches".to_string(), "2".to_string());
        research.insert(
            "latest_regime_transition_market".to_string(),
            "JP".to_string(),
        );
        research.insert(
            "latest_regime_transition_from_bucket".to_string(),
            "trend_down_low_vol".to_string(),
        );
        research.insert(
            "latest_regime_transition_to_bucket".to_string(),
            "trend_down_high_vol".to_string(),
        );
        research.insert(
            "latest_regime_transition_date".to_string(),
            "2026-01-05".to_string(),
        );

        let report = build_paper_hints(
            &research,
            Some(&PaperHintsDaemonInput {
                alerts: 0,
                last_cycle: 3,
                last_end_equity: 1010000.0,
                max_drawdown_observed: 0.05,
            }),
            Some(&PaperHintsCompareInput {
                winner: "candidate".to_string(),
                research_changes: 3,
                top_research_keys: vec![
                    "top_regime_leader_market".to_string(),
                    "current_rotation_leader_factor".to_string(),
                ],
            }),
        );

        assert_eq!(report.stance, "WATCH");
        assert!(report.headline.contains("JP shifted"));
        assert!(report.watch_markets.iter().any(|item| item == "US"));
        assert!(report.watch_markets.iter().any(|item| item == "JP"));
        let summary = render_paper_hints_summary(&report);
        assert!(summary.contains("watch_markets=US|JP"));
        assert!(summary.contains("bullet_1="));
    }
}
