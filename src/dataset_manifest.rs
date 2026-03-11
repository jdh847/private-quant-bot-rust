use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::{load_config, BotConfig};

#[derive(Debug, Clone)]
pub struct DatasetManifestRequest {
    pub config_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetManifest {
    pub generated_at_utc: String,
    pub project_root: String,
    pub config_path: String,
    pub files: Vec<DatasetFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetFile {
    pub kind: String,
    pub market: Option<String>,
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub rows: Option<usize>,
    pub unique_symbols: Option<usize>,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
}

pub fn write_dataset_manifest(req: &DatasetManifestRequest) -> Result<DatasetManifest> {
    let cfg = load_config(&req.config_path)?;
    let project_root = detect_project_root(&req.config_path)?;
    let manifest = build_manifest(&cfg, &req.config_path, &project_root)?;

    if let Some(parent) = req.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create manifest parent {}", parent.display()))?;
        }
    }
    fs::write(
        &req.output_path,
        serde_json::to_string_pretty(&manifest).context("serialize manifest json failed")?,
    )
    .with_context(|| format!("write manifest {}", req.output_path.display()))?;

    Ok(manifest)
}

fn build_manifest(
    cfg: &BotConfig,
    config_path: &Path,
    project_root: &Path,
) -> Result<DatasetManifest> {
    let mut files = Vec::new();
    for market in cfg.markets.values() {
        let mut row = file_row(
            "market_data",
            Some(market.name.clone()),
            project_root,
            &market.data_file,
        )?;
        let stats = inspect_market_csv(&market.data_file).with_context(|| {
            format!(
                "inspect market csv failed: market={} path={}",
                market.name,
                market.data_file.display()
            )
        })?;
        row.rows = Some(stats.rows);
        row.unique_symbols = Some(stats.unique_symbols);
        row.first_date = stats.first_date.map(fmt_date);
        row.last_date = stats.last_date.map(fmt_date);
        files.push(row);

        if let Some(industry_file) = &market.industry_file {
            files.push(file_row(
                "industry_map",
                Some(market.name.clone()),
                project_root,
                industry_file,
            )?);
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(DatasetManifest {
        generated_at_utc: chrono::Utc::now().to_rfc3339(),
        project_root: ".".to_string(),
        config_path: config_path.display().to_string(),
        files,
    })
}

fn file_row(
    kind: &str,
    market: Option<String>,
    project_root: &Path,
    path: &Path,
) -> Result<DatasetFile> {
    let canon =
        fs::canonicalize(path).with_context(|| format!("canonicalize {}", path.display()))?;
    let bytes = fs::metadata(&canon)
        .with_context(|| format!("stat file {}", canon.display()))?
        .len();
    let buf = fs::read(&canon).with_context(|| format!("read file {}", canon.display()))?;
    let hash = Sha256::digest(&buf);
    let out_path = canon
        .strip_prefix(project_root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| canon.display().to_string());
    Ok(DatasetFile {
        kind: kind.to_string(),
        market,
        path: out_path,
        bytes,
        sha256: hex_lower(&hash),
        rows: None,
        unique_symbols: None,
        first_date: None,
        last_date: None,
    })
}

fn detect_project_root(config_path: &Path) -> Result<PathBuf> {
    let abs_config = fs::canonicalize(config_path)
        .with_context(|| format!("canonicalize config failed: {}", config_path.display()))?;
    for ancestor in abs_config.ancestors().skip(1) {
        if ancestor.join("Cargo.toml").is_file() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Ok(abs_config
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

struct MarketCsvStats {
    rows: usize,
    unique_symbols: usize,
    first_date: Option<NaiveDate>,
    last_date: Option<NaiveDate>,
}

fn inspect_market_csv(path: &Path) -> Result<MarketCsvStats> {
    let mut rdr =
        csv::Reader::from_path(path).with_context(|| format!("open csv {}", path.display()))?;
    let headers = rdr
        .headers()
        .context("read headers failed")?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let idx_date = headers
        .iter()
        .position(|h| h == "date")
        .ok_or_else(|| anyhow!("missing date column"))?;
    let idx_symbol = headers
        .iter()
        .position(|h| h == "symbol")
        .ok_or_else(|| anyhow!("missing symbol column"))?;

    let mut rows = 0usize;
    let mut symbols = HashSet::new();
    let mut first: Option<NaiveDate> = None;
    let mut last: Option<NaiveDate> = None;
    for rec in rdr.records() {
        let rec = rec?;
        rows += 1;
        if let Some(sym) = rec.get(idx_symbol) {
            symbols.insert(sym.to_string());
        }
        let Some(ds) = rec.get(idx_date) else {
            continue;
        };
        let Ok(d) = NaiveDate::parse_from_str(ds, "%Y-%m-%d") else {
            continue;
        };
        first = Some(first.map(|x| x.min(d)).unwrap_or(d));
        last = Some(last.map(|x| x.max(d)).unwrap_or(d));
    }
    Ok(MarketCsvStats {
        rows,
        unique_symbols: symbols.len(),
        first_date: first,
        last_date: last,
    })
}

fn fmt_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}
