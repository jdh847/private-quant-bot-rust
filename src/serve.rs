use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct ServeRequest {
    pub root_dir: PathBuf,
    pub bind: String,
    pub prefer_latest: bool,
}

#[derive(Debug, Clone)]
pub struct ServeReport {
    pub root_dir: PathBuf,
    pub bind: String,
    pub base_url: String,
    pub default_doc_rel: Option<String>,
}

pub fn inspect_dashboard_server(req: &ServeRequest) -> Result<ServeReport> {
    let root_dir = fs::canonicalize(&req.root_dir)
        .with_context(|| format!("canonicalize root_dir {}", req.root_dir.display()))?;
    if !root_dir.is_dir() {
        return Err(anyhow!(
            "root_dir is not a directory: {}",
            root_dir.display()
        ));
    }
    let default_doc_rel = resolve_default_doc_rel(&root_dir, req.prefer_latest)?;
    Ok(ServeReport {
        root_dir,
        bind: req.bind.clone(),
        base_url: format!("http://{}/", req.bind),
        default_doc_rel,
    })
}

pub fn run_dashboard_server(req: &ServeRequest) -> Result<ServeReport> {
    let report = inspect_dashboard_server(req)?;
    let listener =
        TcpListener::bind(&report.bind).with_context(|| format!("bind to {} failed", req.bind))?;

    // Simple, single-threaded file server. This is only meant for local usage.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) =
                    handle_connection(stream, &report.root_dir, report.default_doc_rel.as_deref())
                {
                    eprintln!("serve warning: {err:#}");
                }
            }
            Err(err) => eprintln!("serve warning: accept failed: {err}"),
        }
    }

    #[allow(unreachable_code)]
    Ok(report)
}

fn handle_connection(
    mut stream: TcpStream,
    root: &Path,
    default_doc_rel: Option<&str>,
) -> Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).context("read request")?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().ok_or_else(|| anyhow!("empty request"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path_raw = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        return write_response(
            &mut stream,
            405,
            "Method Not Allowed",
            "text/plain; charset=utf-8",
            b"only GET/HEAD supported\n",
            method == "HEAD",
        );
    }

    let path_no_query = path_raw.split('?').next().unwrap_or("/");
    if path_no_query == "/" {
        if let Some(rel) = default_doc_rel {
            let location = format!("/{rel}");
            return write_redirect(&mut stream, &location);
        }
        return write_response(
            &mut stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"no dashboard.html found under root\n",
            method == "HEAD",
        );
    }

    let rel = path_no_query.trim_start_matches('/');
    if rel.is_empty() {
        return write_response(
            &mut stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"not found\n",
            method == "HEAD",
        );
    }

    let safe_rel = sanitize_rel_path(rel)?;
    let target = root.join(&safe_rel);
    let root_can = fs::canonicalize(root).context("canonicalize root")?;

    // canonicalize target only if it exists, otherwise respond 404 without leaking paths.
    let meta = match fs::metadata(&target) {
        Ok(m) => m,
        Err(_) => {
            return write_response(
                &mut stream,
                404,
                "Not Found",
                "text/plain; charset=utf-8",
                b"not found\n",
                method == "HEAD",
            );
        }
    };
    if meta.is_dir() {
        let index = target.join("index.html");
        if index.is_file() {
            return serve_file(&mut stream, &index, &root_can, method == "HEAD");
        }
        return write_response(
            &mut stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"directory listing disabled\n",
            method == "HEAD",
        );
    }

    serve_file(&mut stream, &target, &root_can, method == "HEAD")
}

fn serve_file(
    stream: &mut TcpStream,
    target: &Path,
    root_can: &Path,
    head_only: bool,
) -> Result<()> {
    let target_can =
        fs::canonicalize(target).with_context(|| format!("canonicalize {}", target.display()))?;
    if !target_can.starts_with(root_can) {
        return write_response(
            stream,
            403,
            "Forbidden",
            "text/plain; charset=utf-8",
            b"forbidden\n",
            head_only,
        );
    }

    let bytes = fs::read(&target_can).with_context(|| format!("read {}", target_can.display()))?;
    let ct = content_type_for_path(&target_can);
    write_response(stream, 200, "OK", ct, &bytes, head_only)
}

fn write_redirect(stream: &mut TcpStream, location: &str) -> Result<()> {
    let resp = format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(resp.as_bytes())
        .context("write redirect")?;
    Ok(())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<()> {
    let headers = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .context("write headers")?;
    if !head_only {
        stream.write_all(body).context("write body")?;
    }
    Ok(())
}

fn sanitize_rel_path(rel: &str) -> Result<PathBuf> {
    // Minimal decode for spaces; dashboard fetch paths shouldn't need full decoding.
    let rel = rel.replace("%20", " ");
    let p = Path::new(&rel);
    for c in p.components() {
        match c {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(anyhow!("invalid path"));
            }
            _ => {}
        }
    }
    Ok(p.to_path_buf())
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn resolve_default_doc_rel(root: &Path, prefer_latest: bool) -> Result<Option<String>> {
    // 1) If root has LATEST_DASHBOARD.txt, use it (only if it points inside root).
    if prefer_latest {
        let latest = root.join("LATEST_DASHBOARD.txt");
        if latest.is_file() {
            let s = fs::read_to_string(&latest).context("read LATEST_DASHBOARD.txt")?;
            let p = PathBuf::from(s.trim());
            if p.is_absolute() {
                let root_can = fs::canonicalize(root).context("canonicalize root")?;
                if let Ok(p_can) = fs::canonicalize(&p) {
                    if p_can.starts_with(&root_can) {
                        if let Ok(rel) = p_can.strip_prefix(&root_can) {
                            return Ok(Some(
                                rel.to_string_lossy().trim_start_matches('/').to_string(),
                            ));
                        }
                    }
                }
            } else {
                // Relative paths are interpreted under root.
                let candidate = root.join(&p);
                if candidate.is_file() {
                    return Ok(Some(p.to_string_lossy().to_string()));
                }
            }
        }
    }

    // 2) root/dashboard.html
    let direct = root.join("dashboard.html");
    if direct.is_file() {
        return Ok(Some("dashboard.html".to_string()));
    }

    // 3) Find newest dashboard.html under root.
    let mut best: Option<(SystemTime, PathBuf)> = None;
    find_newest_dashboard(root, &mut best)?;
    Ok(best.and_then(|(_, p)| {
        p.strip_prefix(root)
            .ok()
            .map(|r| r.to_string_lossy().to_string())
    }))
}

fn find_newest_dashboard(dir: &Path, best: &mut Option<(SystemTime, PathBuf)>) -> Result<()> {
    for ent in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let ent = ent?;
        let path = ent.path();
        let meta = ent.metadata()?;
        if meta.is_dir() {
            find_newest_dashboard(&path, best)?;
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) != Some("dashboard.html") {
            continue;
        }
        let m = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let replace = match best {
            None => true,
            Some((best_m, _)) => m > *best_m,
        };
        if replace {
            *best = Some((m, path));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_default_doc_rel;
    use std::{fs, path::PathBuf};

    fn tmp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "private_quant_bot_test_{}_{}_{}",
            name,
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        p
    }

    #[test]
    fn resolve_prefers_latest_dashboard_file_when_inside_root() {
        let root = tmp_dir("latest");
        fs::create_dir_all(&root).unwrap();
        let run_dir = root.join("run_1");
        fs::create_dir_all(&run_dir).unwrap();
        let dash = run_dir.join("dashboard.html");
        fs::write(&dash, "<html/>").unwrap();
        fs::write(
            root.join("LATEST_DASHBOARD.txt"),
            dash.display().to_string(),
        )
        .unwrap();

        let rel = resolve_default_doc_rel(&root, true).unwrap().unwrap();
        assert!(rel.ends_with("run_1/dashboard.html"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_falls_back_to_root_dashboard() {
        let root = tmp_dir("root_dash");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("dashboard.html"), "<html/>").unwrap();

        let rel = resolve_default_doc_rel(&root, true).unwrap().unwrap();
        assert_eq!(rel, "dashboard.html");

        let _ = fs::remove_dir_all(&root);
    }
}
