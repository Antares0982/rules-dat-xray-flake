use anyhow::{Context, Result};
use base64::engine::general_purpose;
use base64::Engine as _;
use clap::Parser;
use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

const VMESS_START: &str = "vmess://";
const VLESS_START: &str = "vless://";

const VMESS_TEMPLATE_DICT_STR: &str = r#"{
    "protocol": "vmess",
    "settings": {
        "vnext": [
            {
                "address": "to fill",
                "port": 443,
                "users": [
                    {
                        "id": "to fill",
                        "alterId": 0,
                        "security": "auto"
                    }
                ]
            }
        ]
    },
    "tag": "proxy",
    "streamSettings": {
        "network": "ws",
        "security": "tls",
        "wsSettings": {
            "path": "",
            "headers": {
                "Host": ""
            }
        },
        "tlsSettings": {
            "serverName": ""
        }
    }
}
"#;

const VLESS_TEMPLATE_DICT_STR: &str = r#"{
    "protocol": "vless",
    "settings": {
        "vnext": [
            {
                "address": "to fill",
                "port": 443,
                "users": [
                    {
                        "id": "to fill",
                        "encryption": "none",
                        "level": 0
                    }
                ]
            }
        ]
    },
    "tag": "proxy",
    "streamSettings": {
        "network": "ws",
        "security": "tls",
        "wsSettings": {
            "path": "to fill",
            "headers": {
                "Host": ""
            }
        },
        "tcpSettings": {},
        "kcpSettings": {},
        "httpSettings": {},
        "dsSettings": {},
        "quicSettings": {},
        "tlsSettings": {
            "serverName": ""
        }
    }
}
"#;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    raw: bool,
    #[arg(long)]
    headless: bool,
}

/// Normalize a human-friendly name by removing non-alphanumeric characters
/// and collapsing whitespace. Used to generate file-friendly names from
/// subscription metadata (e.g. `ps` fields).
fn remove_unknown_chars(s: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9\. ]").expect("compile name cleanup regex");
    let s = re.replace_all(s, " ");
    let re2 = Regex::new(r"\s+").expect("compile whitespace regex");
    re2.replace_all(&s, " ").trim().to_string()
}

/// Parse a JSON `Value` or string into a `u16` port number.
/// Returns an error if the value cannot be represented as a valid u16.
fn parse_port_value(v: &Value) -> Result<u16> {
    // Prefer numeric forms when available
    if let Some(n) = v.as_u64() {
        if n <= u16::MAX as u64 {
            return Ok(n as u16);
        } else {
            anyhow::bail!("port out of range: {}", n);
        }
    }
    if let Some(n) = v.as_i64() {
        if n >= 0 && (n as u64) <= u16::MAX as u64 {
            return Ok(n as u16);
        } else {
            anyhow::bail!("port out of range: {}", n);
        }
    }
    if let Some(s) = v.as_str() {
        if let Ok(p) = s.parse::<u16>() {
            return Ok(p);
        } else {
            anyhow::bail!("invalid port string: {}", s);
        }
    }
    anyhow::bail!("unsupported port value: {}", v)
}

/// Try to load a full xray config template from either a file located next to
/// the executable (`xray_template.json`) or from the path referenced by the
/// `XRAY_TEMPLATE` environment variable. If neither is present, returns
/// `Ok(None)` so the caller can decide how to proceed.
/// Load the total template only from the `XRAY_TEMPLATE` environment
/// variable. Previously the code also looked for `xray_template.json` next to
/// the executable; that behavior has been removed to simplify configuration.
fn load_total_template() -> Result<Option<Value>> {
    if let Ok(env_path) = env::var("XRAY_TEMPLATE") {
        let s = fs::read_to_string(&env_path).with_context(|| format!("reading {}", env_path))?;
        let v: Value = serde_json::from_str(&s)?;
        return Ok(Some(v));
    }
    Ok(None)
}

fn gen_config_json_dict(mut total: Value, insert_index: usize, proxy: Value) -> Value {
    if let Some(outbounds) = total.get_mut("outbounds") {
        if outbounds.is_array() {
            if let Some(arr) = outbounds.as_array_mut() {
                if insert_index < arr.len() {
                    arr[insert_index] = proxy;
                    return total;
                } else {
                    arr.insert(insert_index, proxy);
                    return total;
                }
            }
        }
    }
    // no proper outbounds, just return proxy as standalone
    proxy
}

fn parse_vmess_link(s: &str) -> Result<(Value, String)> {
    let mut raw = s.to_string();
    if raw.starts_with(VMESS_START) {
        raw = raw[VMESS_START.len()..].to_string();
    }
    // try base64 decode
    let decoded = match general_purpose::STANDARD.decode(&raw) {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(_) => raw.clone(),
    };
    // try parse JSON
    let parsed: Value =
        serde_json::from_str(&decoded).with_context(|| "failed to parse vmess JSON")?;

    // fill template
    let mut template: Value = serde_json::from_str(VMESS_TEMPLATE_DICT_STR)?;
    if let Some(settings) = template.get_mut("settings") {
        if let Some(vnext) = settings.get_mut("vnext").and_then(|v| v.as_array_mut()) {
            if let Some(first) = vnext.get_mut(0) {
                if let Some(add) = parsed.get("add").and_then(|v| v.as_str()) {
                    first["address"] = Value::String(add.to_string());
                }
                if let Some(port_val) = parsed.get("port") {
                    // Strictly require a u16 port. Fail if not representable.
                    let port_u16 = parse_port_value(port_val)
                        .with_context(|| format!("invalid port in vmess link: {}", port_val))?;
                    first["port"] = Value::from(port_u16);
                }
                if let Some(users) = first.get_mut("users").and_then(|u| u.as_array_mut()) {
                    if let Some(user0) = users.get_mut(0) {
                        if let Some(id) = parsed.get("id").and_then(|v| v.as_str()) {
                            user0["id"] = Value::String(id.to_string());
                        }
                        if let Some(aid) = parsed.get("aid") {
                            user0["alterId"] = aid.clone();
                        }
                    }
                }
            }
        }
    }
    if let Some(ss) = template.get_mut("streamSettings") {
        if let Some(net) = parsed.get("net").and_then(|v| v.as_str()) {
            ss["network"] = Value::String(net.to_string());
            if net == "ws" {
                if let Some(ws) = ss.get_mut("wsSettings") {
                    if let Some(path) = parsed.get("path").and_then(|v| v.as_str()) {
                        ws["path"] = Value::String(path.to_string());
                    }
                    if let Some(headers) = ws.get_mut("headers") {
                        if let Some(host) = parsed.get("host").and_then(|v| v.as_str()) {
                            headers["Host"] = Value::String(host.to_string());
                        }
                    }
                }
            } else {
                // copy unknown settings into <net>Settings if exists
                let settingname = format!("{}Settings", net);
                if let Some(setting_obj) = ss.get_mut(&settingname) {
                    if setting_obj.is_object() {
                        if let Some(map) = setting_obj.as_object_mut() {
                            for (k, v) in parsed.as_object().into_iter().flat_map(|m| m.clone()) {
                                let known = [
                                    "v", "ps", "add", "port", "id", "aid", "net", "type", "tls",
                                    "sni",
                                ];
                                if !known.contains(&k.as_str()) {
                                    map.insert(k, v);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(tls) = parsed.get("tls").and_then(|v| v.as_str()) {
            ss["security"] = Value::String(tls.to_string());
            if tls == "tls" {
                if let Some(host) = parsed.get("host").and_then(|v| v.as_str()) {
                    if let Some(tlssettings) = ss.get_mut("tlsSettings") {
                        tlssettings["serverName"] = Value::String(host.to_string());
                    }
                }
            }
        }
    }

    // Friendly name for the generated file. Fall back to `unnamed` if no
    // `ps` field exists.
    let name = parsed
        .get("ps")
        .and_then(|v| v.as_str())
        .map(|s| remove_unknown_chars(s))
        .unwrap_or_else(|| "unnamed".to_string());

    Ok((template, name))
}

fn parse_vless_link(s: &str) -> Result<(Value, String)> {
    let mut raw = s.to_string();
    if raw.starts_with(VLESS_START) {
        raw = raw[VLESS_START.len()..].to_string();
    }
    // percent-decode
    let decoded = percent_decode_str(&raw).decode_utf8_lossy().to_string();

    let re = Regex::new(r"(?P<id>.*?)@(?P<add>.*?):(?P<port>[0-9]+)\?(?P<params>.*)").unwrap();
    let caps = re
        .captures(&decoded)
        .with_context(|| format!("invalid vless string: {}", decoded))?;
    let id = caps.name("id").unwrap().as_str();
    let add = caps.name("add").unwrap().as_str();
    let port = caps.name("port").unwrap().as_str();
    let params = caps.name("params").unwrap().as_str();

    // Convert query string (after `?`) into a small map for named params.
    let mut params_map = serde_json::Map::new();
    for part in params.split('&') {
        let mut kv = part.splitn(2, '=');
        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
            params_map.insert(k.to_string(), Value::String(v.to_string()));
        }
    }

    let mut template: Value = serde_json::from_str(VLESS_TEMPLATE_DICT_STR)?;
    if let Some(settings) = template.get_mut("settings") {
        if let Some(vnext) = settings.get_mut("vnext").and_then(|v| v.as_array_mut()) {
            if let Some(first) = vnext.get_mut(0) {
                first["address"] = Value::String(add.to_string());
                // Port captured from the vless regex must be a valid u16.
                let port_u16 = port
                    .parse::<u16>()
                    .with_context(|| format!("invalid vless port: {}", port))?;
                first["port"] = Value::from(port_u16);
                if let Some(users) = first.get_mut("users").and_then(|u| u.as_array_mut()) {
                    if let Some(user0) = users.get_mut(0) {
                        user0["id"] = Value::String(id.to_string());
                        if let Some(encryption) = params_map.get("encryption") {
                            user0["encryption"] = encryption.clone();
                        }
                    }
                }
            }
        }
    }
    if let Some(ss) = template.get_mut("streamSettings") {
        if let Some(typ) = params_map.get("type").and_then(|v| v.as_str()) {
            ss["network"] = Value::String(typ.to_string());
            if typ == "ws" {
                if let Some(ws) = ss.get_mut("wsSettings") {
                    if let Some(path) = params_map.get("path").and_then(|v| v.as_str()) {
                        ws["path"] = Value::String(path.to_string());
                    }
                    if let Some(headers) = ws.get_mut("headers") {
                        if let Some(host) = params_map.get("host").and_then(|v| v.as_str()) {
                            headers["Host"] = Value::String(host.to_string());
                        }
                    }
                }
            } else {
                let settingname = format!("{}Settings", typ);
                if let Some(setting_obj) = ss.get_mut(&settingname) {
                    if setting_obj.is_object() {
                        if let Some(map) = setting_obj.as_object_mut() {
                            for (k, v) in params_map.iter() {
                                let known = ["id", "add", "port", "encryption", "type", "security"];
                                if !known.contains(&k.as_str()) {
                                    map.insert(k.clone(), v.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(security) = params_map.get("security").and_then(|v| v.as_str()) {
            ss["security"] = Value::String(security.to_string());
            if security == "tls" {
                if let Some(sni) = params_map.get("sni").and_then(|v| v.as_str()) {
                    if let Some(tlssettings) = ss.get_mut("tlsSettings") {
                        tlssettings["serverName"] = Value::String(sni.to_string());
                    }
                }
            }
        }
    }

    // Use `headerType` as a hint for the name when available.
    let name = params_map
        .get("headerType")
        .and_then(|v| v.as_str())
        .map(|s| remove_unknown_chars(s))
        .unwrap_or_else(|| "unnamed".to_string());

    Ok((template, name))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load template strictly from XRAY_TEMPLATE env var.
    let total_template = get_total_template(&args)?;

    // Validate XRAY_CONF_DIR and subscription file
    let xray_conf_dir =
        env::var("XRAY_CONF_DIR").context("Please set XRAY_CONF_DIR in environment variables")?;
    let subscription = read_subscription_url(&xray_conf_dir)?;
    let folder = ensure_subscriptions_folder(&xray_conf_dir)?;

    // Fetch subscription content and split into lines
    let client = Client::builder().user_agent("xray_sub/0.1.0").build()?;
    let responses = fetch_subscription_lines(&client, &subscription)?;

    let processed_files = process_and_write(&responses, &args, &total_template, &folder)?;

    if processed_files.is_empty() {
        anyhow::bail!("No valid link found");
    }

    cleanup_folder(&folder, &processed_files)?;

    Ok(())
}

/// Load total template unless `--raw` is set. Mirrors previous behavior.
fn get_total_template(args: &Args) -> Result<Option<Value>> {
    if args.raw {
        return Ok(None);
    }
    match load_total_template()? {
        Some(t) => Ok(Some(t)),
        None => {
            eprintln!(
                "Please set the environment variable XRAY_TEMPLATE to your xray config template"
            );
            std::process::exit(1);
        }
    }
}

/// Read subscription URL from `XRAY_CONF_DIR/sub_url.txt`.
fn read_subscription_url(xray_conf_dir: &str) -> Result<String> {
    let sub_file = PathBuf::from(xray_conf_dir).join("sub_url.txt");
    if !sub_file.exists() {
        eprintln!("set subscription url to $XRAY_CONF_DIR/sub_url.txt");
        std::process::exit(1);
    }
    let subscription = fs::read_to_string(&sub_file)?.trim().to_string();
    Ok(subscription)
}

/// Ensure `subscriptions/` exists and is a directory.
fn ensure_subscriptions_folder(xray_conf_dir: &str) -> Result<PathBuf> {
    let folder = PathBuf::from(xray_conf_dir).join("subscriptions");
    if !folder.exists() {
        fs::create_dir_all(&folder)?;
    }
    if !folder.is_dir() {
        eprintln!("{} is not a directory", folder.display());
        std::process::exit(1);
    }
    Ok(folder)
}

/// Download subscription and return non-empty lines. Handles optional base64 decoding.
fn fetch_subscription_lines(client: &Client, subscription: &str) -> Result<Vec<String>> {
    let response = client
        .get(subscription)
        .timeout(std::time::Duration::from_secs(10))
        .send()?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to get subscription");
    }
    let text = response.text()?;
    let responses: Vec<String> = match general_purpose::STANDARD.decode(&text) {
        Ok(b) => String::from_utf8_lossy(&b)
            .to_string()
            .split('\n')
            .map(|s| s.to_string())
            .collect(),
        Err(_) => text.split('\n').map(|s| s.to_string()).collect(),
    };
    if responses.is_empty() {
        anyhow::bail!("No valid links found");
    }
    Ok(responses)
}

/// Process each subscription line, write files, and return the set of written files.
fn process_and_write(
    responses: &[String],
    args: &Args,
    total_template: &Option<Value>,
    folder: &PathBuf,
) -> Result<HashSet<PathBuf>> {
    let mut processed_files: HashSet<PathBuf> = HashSet::new();
    for row in responses.iter() {
        let row = row.trim();
        if row.is_empty() {
            continue;
        }
        println!("Processing: {}", row);
        if !(row.starts_with(VMESS_START) || row.starts_with(VLESS_START)) {
            continue;
        }
        if row.starts_with(VMESS_START) {
            match parse_vmess_link(row) {
                Ok((vmess_dict, name)) => {
                    let out_json = if args.raw || total_template.is_none() {
                        vmess_dict
                    } else {
                        gen_config_json_dict(total_template.clone().unwrap(), 0, vmess_dict)
                    };
                    let fname = folder.join(format!("{}.json", name));
                    println!("Writing to: \"{}\"", fname.display());
                    let f = fs::File::create(&fname)?;
                    serde_json::to_writer_pretty(f, &out_json)?;
                    processed_files.insert(fname);
                }
                Err(e) => {
                    eprintln!("Error occurred when processing {}: {}", row, e);
                }
            }
            continue;
        }
        if row.starts_with(VLESS_START) {
            match parse_vless_link(row) {
                Ok((vless_dict, name)) => {
                    let out_json = if args.raw || total_template.is_none() {
                        vless_dict
                    } else {
                        gen_config_json_dict(total_template.clone().unwrap(), 0, vless_dict)
                    };
                    let fname = folder.join(format!("VLESS_{}.json", name));
                    println!("Writing to: \"{}\"", fname.display());
                    let f = fs::File::create(&fname)?;
                    serde_json::to_writer_pretty(f, &out_json)?;
                    processed_files.insert(fname);
                }
                Err(e) => {
                    eprintln!("Error occurred when processing {}: {}", row, e);
                }
            }
            continue;
        }
    }
    Ok(processed_files)
}

/// Remove files in `folder` that were not produced in this run.
fn cleanup_folder(folder: &PathBuf, processed_files: &HashSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(folder)? {
        let p = entry?.path();
        if !processed_files.contains(&p) {
            println!("Removing: {}", p.display());
            let _ = fs::remove_file(&p);
        }
    }
    Ok(())
}
