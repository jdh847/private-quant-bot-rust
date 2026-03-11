use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Serialize;

use crate::{config::load_config, data::CsvDataPortal};

#[derive(Debug, Clone)]
pub struct DoctorRequest {
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDoctorRow {
    pub market: String,
    pub allocation: f64,
    pub currency: String,
    pub fx_to_base: f64,
    pub industry_map_symbols: usize,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    pub last_day_bars: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub config_path: PathBuf,
    pub paper_only: bool,
    pub broker_mode: String,
    pub dates: usize,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    pub markets: Vec<MarketDoctorRow>,
}

pub fn run_doctor(req: &DoctorRequest) -> Result<DoctorReport> {
    let cfg = load_config(&req.config_path)?;
    let market_files = cfg
        .markets
        .values()
        .map(|m| (m.name.clone(), m.data_file.clone()))
        .collect::<Vec<_>>();
    let portal = CsvDataPortal::new(market_files).context("load market csv data failed")?;

    let dates = portal.trading_dates();
    let (first_date, last_date) = match (dates.first().copied(), dates.last().copied()) {
        (Some(f), Some(l)) => (Some(f), Some(l)),
        _ => (None, None),
    };

    let mut rows = Vec::new();
    for m in cfg.markets.values() {
        let (m_first, m_last, last_day_bars) = market_span(&portal, &dates, &m.name);
        rows.push(MarketDoctorRow {
            market: m.name.clone(),
            allocation: m.allocation,
            currency: m.currency.clone(),
            fx_to_base: m.fx_to_base,
            industry_map_symbols: m.industry_map.len(),
            first_date: m_first.map(fmt_date),
            last_date: m_last.map(fmt_date),
            last_day_bars,
        });
    }
    rows.sort_by(|a, b| a.market.cmp(&b.market));

    Ok(DoctorReport {
        config_path: req.config_path.clone(),
        paper_only: cfg.broker.paper_only,
        broker_mode: cfg.broker.mode,
        dates: dates.len(),
        first_date: first_date.map(fmt_date),
        last_date: last_date.map(fmt_date),
        markets: rows,
    })
}

fn fmt_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn market_span(
    portal: &CsvDataPortal,
    all_dates: &[NaiveDate],
    market: &str,
) -> (Option<NaiveDate>, Option<NaiveDate>, usize) {
    let mut first: Option<NaiveDate> = None;
    for d in all_dates {
        if !portal.bars_for(*d, market).is_empty() {
            first = Some(*d);
            break;
        }
    }
    let mut last: Option<NaiveDate> = None;
    let mut last_bars = 0usize;
    for d in all_dates.iter().rev() {
        let bars = portal.bars_for(*d, market);
        if !bars.is_empty() {
            last = Some(*d);
            last_bars = bars.len();
            break;
        }
    }
    (first, last, last_bars)
}
