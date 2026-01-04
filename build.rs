// Env toggles:
// - SPIDER_FP_ASSETS=1            => (re)generate blob + indexes into repo
// - SPIDER_FP_EXPAND_REFERRERS=1  => use merged_referrers.txt (large) instead of 20k list
// - REFERRERS_REFRESH=1           => rebuild merged_referrers.txt from optional inputs
// - REFERRERS_TRIM_TO_1M=1        => cap merged_referrers to 1,000,000 entries

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

const DOMAIN_LIST_REPO: &str = "https://github.com/spider-rs/domain-list.git";

fn norm_domain_from_any(s: &str) -> Option<String> {
    let mut t = s.trim().to_lowercase();
    if t.is_empty() {
        return None;
    }
    // remove scheme
    if let Some(rest) = t.strip_prefix("https://") {
        t = rest.to_string();
    } else if let Some(rest) = t.strip_prefix("http://") {
        t = rest.to_string();
    }
    // strip path/query/fragment
    if let Some((host, _)) = t.split_once('/') {
        t = host.to_string();
    }
    if let Some((host, _)) = t.split_once('?') {
        t = host.to_string();
    }
    if let Some((host, _)) = t.split_once('#') {
        t = host.to_string();
    }
    t = t.trim_matches('.').to_string();

    // quick sanity
    if t.len() < 3 || !t.contains('.') {
        return None;
    }
    // avoid spaces and obvious garbage
    if t.bytes().any(|b| b <= b' ' || b == b'\\' || b == b'"') {
        return None;
    }

    Some(t)
}

fn to_https_url(domain: &str) -> String {
    let mut s = String::with_capacity(8 + domain.len() + 1);
    s.push_str("https://");
    s.push_str(domain);
    s.push('/');
    s
}

/// Read lines from a "urls file" (may contain https://d/ or raw domains)
fn load_domains_from_lines_file(
    path: &Path,
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) -> io::Result<()> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    for line in reader.lines() {
        let line = line?;
        if let Some(dom) = norm_domain_from_any(&line) {
            if seen.insert(dom.clone()) {
                out.push(dom);
            }
        }
    }
    Ok(())
}

/// Read domains from Majestic CSV.
/// We try a few common column positions, otherwise scan all cells.
fn load_domains_from_majestic_csv(
    path: &Path,
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) -> io::Result<()> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            continue; // header
        }

        let cols: Vec<&str> = line.split(',').collect();
        let mut cand: Option<String> = None;

        for &idx in &[2usize, 1usize, 0usize] {
            if let Some(v) = cols.get(idx) {
                let v = v.trim().trim_matches('"');
                if let Some(dom) = norm_domain_from_any(v) {
                    cand = Some(dom);
                    break;
                }
            }
        }

        if cand.is_none() {
            for v in &cols {
                let v = v.trim().trim_matches('"');
                if let Some(dom) = norm_domain_from_any(v) {
                    cand = Some(dom);
                    break;
                }
            }
        }

        if let Some(dom) = cand {
            if seen.insert(dom.clone()) {
                out.push(dom);
            }
        }
    }

    Ok(())
}

/// Merge optional sources into assets/merged_referrers.txt (as https://domain/)
/// Order matters: earlier sources win if trimming.
fn maybe_refresh_merged_referrers(manifest_dir: &Path) -> io::Result<()> {
    let refresh = std::env::var("REFERRERS_REFRESH").ok().as_deref() == Some("1");
    if !refresh {
        return Ok(());
    }

    let assets = manifest_dir.join("assets");

    let tranco_urls = assets.join("tranco_1m_urls.txt"); // optional local file
    let majestic_csv = assets.join("majestic_million.csv"); // optional local file
    let existing = assets.join("merged_referrers.txt"); // current checked-in
    let out_path = assets.join("merged_referrers.txt");

    let trim_to_1m = std::env::var("REFERRERS_TRIM_TO_1M").ok().as_deref() == Some("1");

    let mut seen: HashSet<String> = HashSet::with_capacity(1_200_000);
    let mut merged: Vec<String> = Vec::with_capacity(1_200_000);

    // 1) Prefer Tranco first
    if tranco_urls.exists() {
        let _ = load_domains_from_lines_file(&tranco_urls, &mut seen, &mut merged);
    }

    // 2) Then Majestic
    if majestic_csv.exists() {
        let _ = load_domains_from_majestic_csv(&majestic_csv, &mut seen, &mut merged);
    }

    // 3) Then existing fallback pool
    if existing.exists() {
        let _ = load_domains_from_lines_file(&existing, &mut seen, &mut merged);
    }

    if trim_to_1m && merged.len() > 1_000_000 {
        merged.truncate(1_000_000);
    }

    // write atomically
    let tmp = assets.join("merged_referrers.txt.tmp");
    {
        let mut w = fs::File::create(&tmp)?;
        for d in merged {
            w.write_all(to_https_url(&d).as_bytes())?;
            w.write_all(b"\n")?;
        }
    }
    fs::rename(&tmp, &out_path)?;

    println!("cargo:rerun-if-changed={}", tranco_urls.display());
    println!("cargo:rerun-if-changed={}", majestic_csv.display());
    println!("cargo:rerun-if-changed={}", existing.display());
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)
        .unwrap_or_else(|e| panic!("failed to write temp file {}: {e}", tmp.display()));
    std::fs::rename(&tmp, path).unwrap_or_else(|e| {
        panic!(
            "failed to rename {} -> {}: {e}",
            tmp.display(),
            path.display()
        )
    });
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

fn run(cmd: &mut Command) -> io::Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("command failed: {cmd:?} (status={status})"),
        ));
    }
    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success() || !s.success())
        .unwrap_or(false)
}

fn ensure_domain_list_repo() -> Option<PathBuf> {
    println!("cargo:rerun-if-env-changed=SPIDER_FP_DOMAIN_LIST_REV");
    println!("cargo:rerun-if-env-changed=SPIDER_FP_CLEANUP_DOMAIN_LIST");

    if !command_exists("git") {
        eprintln!("referrers: `git` not found; falling back to local assets/");
        return None;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").ok()?);
    let cache_dir = out_dir.join("spider_domain_list");

    let rev = std::env::var("SPIDER_FP_DOMAIN_LIST_REV").ok(); // optional pin

    // Clone if missing
    if !cache_dir.join(".git").exists() {
        let mut c = Command::new("git");
        c.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(DOMAIN_LIST_REPO)
            .arg(&cache_dir);

        if let Err(e) = run(&mut c) {
            eprintln!("referrers: git clone failed ({e}); falling back to local assets/");
            return None;
        }
    } else {
        // Best-effort update (shallow)
        let mut c = Command::new("git");
        c.current_dir(&cache_dir)
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin");

        if let Err(e) = run(&mut c) {
            eprintln!("referrers: git fetch failed ({e}); continuing with cached checkout");
        }
    }

    // Best-effort checkout of a specific rev (tag/sha/branch)
    if let Some(rev) = rev.as_deref() {
        let mut c = Command::new("git");
        c.current_dir(&cache_dir).arg("checkout").arg(rev);
        if let Err(e) = run(&mut c) {
            eprintln!("referrers: git checkout {rev} failed ({e}); using current HEAD");
        }
    }

    // Watch the repo directory for changes (best effort rerun triggers)
    // Cargo doesn't allow recursive rerun-if-changed; we at least rerun on build.rs changes + env.
    Some(cache_dir)
}

fn pick_merged_source(manifest: &Path) -> Option<PathBuf> {
    if let Some(repo) = ensure_domain_list_repo() {
        let p = repo.join("merged_referrers.txt");
        if p.exists() {
            return Some(p);
        }
        eprintln!(
            "referrers: domain-list repo missing {}; falling back to local assets/",
            p.display()
        );
    }

    let local = manifest.join("assets").join("merged_referrers.txt");
    if local.exists() {
        return Some(local);
    }
    None
}

fn pick_20k_source(manifest: &Path) -> PathBuf {
    if let Some(repo) = ensure_domain_list_repo() {
        let p = repo.join("referrers_20k_urls.txt");
        if p.exists() {
            return p;
        }
        eprintln!(
            "referrers: domain-list repo missing {}; falling back to local assets/",
            p.display()
        );
    }
    manifest.join("assets").join("referrers_20k_urls.txt")
}

/// Default is 20k.
/// Expand uses assets/merged_referrers.txt if present and SPIDER_FP_EXPAND_REFERRERS=1.
fn pick_referrers_input(manifest: &Path) -> (PathBuf, usize) {
    println!("cargo:rerun-if-env-changed=SPIDER_FP_EXPAND_REFERRERS");

    let cap_default = 20_000;
    let cap_expand = 1_000_000;

    let expand = std::env::var("SPIDER_FP_EXPAND_REFERRERS").ok().as_deref() == Some("1");

    // 1) If expand requested, prefer merged
    if expand {
        if let Some(p) = pick_merged_source(manifest) {
            return (p, cap_expand);
        }
        eprintln!(
            "referrers: expand requested but merged_referrers.txt unavailable; falling back to 20k"
        );
    }

    // 2) Default: try 20k
    let p20 = pick_20k_source(manifest);
    if p20.exists() {
        // quick empty check
        if let Ok(md) = std::fs::metadata(&p20) {
            if md.len() > 0 {
                return (p20, cap_default);
            }
        }
        eprintln!(
            "referrers: {} is empty; falling back to merged_referrers.txt capped to 20k",
            p20.display()
        );
    } else {
        eprintln!(
            "referrers: {} missing; falling back to merged_referrers.txt capped to 20k",
            p20.display()
        );
    }

    // 3) Fall back to merged even without expand (still cap to 20k)
    if let Some(pm) = pick_merged_source(manifest) {
        return (pm, cap_default);
    }

    // 4) Last resort: local assets (may be empty too)
    (
        manifest.join("assets").join("referrers_20k_urls.txt"),
        cap_default,
    )
}

/// Input: assets/high_quality_referrers.txt (FULL URLs, one per line)
/// Output:
///   - assets/hq_urls_blob.bin
///   - src/referrers_hq_index.rs
fn gen_hq_urls_to_repo() {
    let manifest = manifest_dir();

    let input_txt = manifest.join("assets").join("high_quality_referrers.txt");
    let out_blob = manifest.join("assets").join("hq_urls_blob.bin");
    let out_index = manifest.join("src").join("referrers_hq_index.rs");

    println!("cargo:rerun-if-changed={}", input_txt.display());

    if !input_txt.exists() {
        eprintln!(
            "referrers: skip HQ generation (missing {}); set SPIDER_FP_ASSETS=1 only when assets are present",
            input_txt.display()
        );
        return;
    }

    let text = match std::fs::read_to_string(&input_txt) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("referrers: failed to read {}: {e}", input_txt.display());
            return;
        }
    };

    let mut blob: Vec<u8> = Vec::with_capacity(text.len() + 4096);
    let mut offsets: Vec<u32> = Vec::new();
    let mut lens: Vec<u16> = Vec::new();

    for line in text.lines() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }

        // Normalize: must be URL-like, ensure trailing slash for bare hosts.
        let normalized = if !(s.starts_with("https://") || s.starts_with("http://")) {
            let mut tmp = String::with_capacity(8 + s.len() + 1);
            tmp.push_str("https://");
            tmp.push_str(s);
            tmp.push('/');
            tmp
        } else {
            let after_scheme = if let Some(rest) = s.strip_prefix("https://") {
                rest
            } else {
                s.strip_prefix("http://").unwrap_or("")
            };

            // Ensure trailing slash if it’s just scheme + host (no path)
            if !after_scheme.contains('/') {
                let mut tmp = String::with_capacity(s.len() + 1);
                tmp.push_str(s);
                tmp.push('/');
                tmp
            } else {
                s.to_string()
            }
        };

        let bytes = normalized.as_bytes();
        if bytes.contains(&0) || bytes.len() > u16::MAX as usize {
            continue;
        }

        let start = blob.len();
        if start > u32::MAX as usize {
            break;
        }

        offsets.push(start as u32);
        lens.push(bytes.len() as u16);
        blob.extend_from_slice(bytes);
        blob.push(0);
    }

    atomic_write(&out_blob, &blob);

    let mut rs = String::new();
    rs.push_str("// @generated by build.rs — DO NOT EDIT\n");
    rs.push_str(&format!("pub const HQ_LEN: usize = {};\n", offsets.len()));

    rs.push_str("pub static HQ_OFFSETS: &[u32] = &[\n");
    for chunk in offsets.chunks(1024) {
        rs.push_str("    ");
        for v in chunk {
            rs.push_str(&format!("{v},"));
        }
        rs.push('\n');
    }
    rs.push_str("];\n");

    rs.push_str("pub static HQ_LENS: &[u16] = &[\n");
    for chunk in lens.chunks(1024) {
        rs.push_str("    ");
        for v in chunk {
            rs.push_str(&format!("{v},"));
        }
        rs.push('\n');
    }
    rs.push_str("];\n");

    atomic_write(&out_index, rs.as_bytes());
}

/// Input: assets/referrers_20k_urls.txt (default) OR assets/merged_referrers.txt (expand)
/// Lines may be full URLs or raw domains; we normalize to domains and store FULL URL strings "https://{domain}/".
/// Output:
///   - assets/domains_blob.bin
///   - src/referrers_domains_index.rs
fn gen_domains_to_repo() {
    let manifest = manifest_dir();
    let (input_txt, cap) = pick_referrers_input(&manifest);

    println!("cargo:rerun-if-changed={}", input_txt.display());
    // Watch both known inputs so toggling env works without touching files.
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("referrers_20k_urls.txt")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("merged_referrers.txt")
            .display()
    );

    if !input_txt.exists() {
        eprintln!(
            "referrers: skip domains generation (missing {}); set SPIDER_FP_ASSETS=1 only when assets are present",
            input_txt.display()
        );
        return;
    }

    let out_blob = manifest.join("assets").join("domains_blob.bin");
    let out_index = manifest.join("src").join("referrers_domains_index.rs");

    // Load + dedupe + cap
    let f = match fs::File::open(&input_txt) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("referrers: failed to open {}: {e}", input_txt.display());
            return;
        }
    };
    let reader = BufReader::new(f);

    let mut seen: HashSet<String> = HashSet::with_capacity(cap.saturating_mul(2));
    let mut domains: Vec<String> = Vec::with_capacity(cap);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Some(dom) = norm_domain_from_any(&line) {
            if seen.insert(dom.clone()) {
                domains.push(dom);
                if domains.len() >= cap {
                    break;
                }
            }
        }
    }

    if domains.is_empty() {
        eprintln!(
            "WARNING: referrers input {} produced 0 domains; falling back to google.com",
            input_txt.display()
        );
        domains.push("google.com".to_string());
    }

    eprintln!(
        "referrers: using {} -> {} unique domains (cap={})",
        input_txt.display(),
        domains.len(),
        cap
    );

    // Build blob/index of FULL URL strings "https://{domain}/\0"
    let mut blob: Vec<u8> = Vec::with_capacity(domains.len() * 24);
    let mut offsets: Vec<u32> = Vec::with_capacity(domains.len());
    let mut lens: Vec<u16> = Vec::with_capacity(domains.len());

    for d in &domains {
        let url_len = 8 + d.len() + 1; // https:// + domain + /
        if url_len > u16::MAX as usize {
            continue;
        }
        let start = blob.len();
        if start > u32::MAX as usize {
            break;
        }

        offsets.push(start as u32);
        lens.push(url_len as u16);

        blob.extend_from_slice(b"https://");
        blob.extend_from_slice(d.as_bytes());
        blob.push(b'/');
        blob.push(0);
    }

    atomic_write(&out_blob, &blob);

    let mut rs = String::new();
    rs.push_str("// @generated by build.rs — DO NOT EDIT\n");
    rs.push_str(&format!(
        "pub const DOMAINS_LEN: usize = {};\n",
        offsets.len()
    ));

    rs.push_str("pub static DOMAINS_OFFSETS: &[u32] = &[\n");
    for chunk in offsets.chunks(1024) {
        rs.push_str("    ");
        for v in chunk {
            rs.push_str(&format!("{v},"));
        }
        rs.push('\n');
    }
    rs.push_str("];\n");

    rs.push_str("pub static DOMAINS_LENS: &[u16] = &[\n");
    for chunk in lens.chunks(1024) {
        rs.push_str("    ");
        for v in chunk {
            rs.push_str(&format!("{v},"));
        }
        rs.push('\n');
    }
    rs.push_str("];\n");

    atomic_write(&out_index, rs.as_bytes());
}

fn gen_assets_if_enabled() {
    if std::env::var("SPIDER_FP_ASSETS").ok().as_deref() == Some("1") {
        gen_hq_urls_to_repo();
        gen_domains_to_repo();
    }
}

#[cfg(not(feature = "dynamic-versions"))]
fn main() {
    println!("cargo:rustc-cfg=build_script_ran");

    // Keep chrome fallback copy behavior
    if let Ok(out_path) = std::env::var("OUT_DIR") {
        let out_path = Path::new(&out_path).join("chrome_versions.rs");
        let fallback_path = "chrome_versions.rs.fallback";
        let _ = std::fs::copy(fallback_path, &out_path);
    } else {
        println!("out dir does not exist");
    }
    println!("cargo:rerun-if-changed=build/chrome_versions.rs.fallback");

    let manifest = manifest_dir();

    // Optional refresh of merged_referrers.txt
    let _ = maybe_refresh_merged_referrers(&manifest);

    // Ensure builds rerun if inputs change even when SPIDER_FP_ASSETS isn't set.
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("high_quality_referrers.txt")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("referrers_20k_urls.txt")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("merged_referrers.txt")
            .display()
    );

    gen_assets_if_enabled();
}

#[cfg(feature = "dynamic-versions")]
fn main() {
    println!("cargo:rustc-cfg=build_script_ran");

    let manifest = manifest_dir();

    // Optional refresh of merged_referrers.txt
    let _ = maybe_refresh_merged_referrers(&manifest);

    // Ensure rebuild triggers
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("high_quality_referrers.txt")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("referrers_20k_urls.txt")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("merged_referrers.txt")
            .display()
    );

    gen_assets_if_enabled();

    // Existing chrome dynamic versions logic (unchanged)
    use std::fs::{copy, rename, File};
    use std::io::BufWriter;

    if let Ok(out_path) = std::env::var("OUT_DIR") {
        let generated_path = format!("{}/chrome_versions.rs", out_path);
        let tmp_path = format!("{}/chrome_versions.rs.tmp", out_path);
        let fallback_path = "chrome_versions.rs.fallback"; // repo root

        let result = (|| -> Option<(BTreeMap<String, Vec<String>>, String)> {
            let known_json: serde_json::Value = reqwest::blocking::get(
                "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json",
            )
            .ok()?
            .json()
            .ok()?;

            let mut versions_by_major: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for entry in known_json["versions"].as_array()? {
                let ver = entry["version"].as_str()?;
                let major = ver.split('.').next()?;
                versions_by_major
                    .entry(major.to_string())
                    .or_default()
                    .push(ver.to_string());
            }

            let last_json: serde_json::Value = reqwest::blocking::get(
                "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json",
            )
            .ok()?
            .json()
            .ok()?;

            let latest_full = last_json["channels"]["Stable"]["version"]
                .as_str()?
                .to_string();

            Some((versions_by_major, latest_full))
        })();

        match result {
            Some((versions_by_major, latest_full)) => {
                {
                    let mut file = BufWriter::new(File::create(&tmp_path).unwrap());
                    writeln!(file, "use phf::{{phf_map, Map}};").unwrap();
                    writeln!(file, "/// Map of Chrome major version to all known good full versions. Generated at build time.").unwrap();
                    writeln!(
                        file,
                        "/// The \"latest\" key points to the current stable Chrome full version."
                    )
                    .unwrap();
                    writeln!(file, "pub static CHROME_VERSIONS_BY_MAJOR: Map<&'static str, &'static [&'static str]> = phf_map! {{").unwrap();
                    writeln!(file, "    \"latest\" => &[\"{}\"],", latest_full).unwrap();
                    for (major, versions) in &versions_by_major {
                        let quoted_versions: Vec<String> =
                            versions.iter().map(|v| format!("\"{}\"", v)).collect();
                        writeln!(
                            file,
                            "    \"{}\" => &[{}],",
                            major,
                            quoted_versions.join(", ")
                        )
                        .unwrap();
                    }
                    writeln!(file, "}};").unwrap();
                    file.flush().unwrap();
                }

                if let Err(e) = rename(&tmp_path, &generated_path) {
                    eprintln!("{:?}", e)
                }
                if let Err(e) = copy(&generated_path, fallback_path) {
                    eprintln!("{:?}", e)
                }
            }
            None => {
                eprintln!(
                    "WARNING: Failed to fetch or parse Chrome version lists; using fallback file."
                );
                if Path::new(fallback_path).exists() {
                    if let Err(e) = copy(fallback_path, &generated_path) {
                        eprintln!("{:?}", e)
                    }
                } else {
                    panic!("No fallback file found and failed to download new data!");
                }
            }
        }
    }

    println!("cargo:rerun-if-changed=build/chrome_versions.rs.fallback");
}
