use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

use clap::ValueEnum;
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

#[derive(clap::Args)]
pub struct PortScanOpts {
    #[arg(
        short = 't',
        long = "target",
        value_name = "HOST",
        default_value = "127.0.0.1",
        help = "目标主机"
    )]
    target: Option<String>,
    #[arg(
        short = 'p',
        long = "port",
        value_name = "RANGE",
        default_value = "80",
        help = "目标端口, 例如 80 或 80-100"
    )]
    port: Option<String>,
    #[arg(
        short = 'c',
        long = "concurrency",
        value_name = "N",
        default_value = "100",
        help = "并发数"
    )]
    concurrency: Option<usize>,
    #[arg(
        long = "timeout",
        default_value = "1000",
        value_name = "MS",
        help = "超时时间(毫秒)"
    )]
    time_out: Option<u64>,
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FMT",
        default_value = "plain",
        help = "输出格式 plain|json"
    )]
    output: Option<String>,

    #[arg(
        value_enum,
        short = 's',
        long = "show",
        default_value_t = ShowType::All,
        value_name = "TYPE",
        help = "显示类型 all | open | closed"
    )]
    show_type: ShowType,
}

pub fn run_port_scan(opts: PortScanOpts) -> Result<(), PortScanError> {
    let request = PortScanRequest {
        target: opts.target,
        port: opts.port,
        concurrency: opts.concurrency,
        timeout_ms: opts.time_out,
    };

    let output = opts.output.unwrap_or_else(|| "plain".to_string());
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| PortScanError::RuntimeError(e.to_string()))?;
    let mut result = rt.block_on(async move { scan_ports(request).await })?;

    match opts.show_type {
        ShowType::Open => result.ports.retain(|p| p.open),
        ShowType::Closed => result.ports.retain(|p| !p.open),
        ShowType::All => {}
    }

    if output == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&result)
                .map_err(|e| PortScanError::RuntimeError(e.to_string()))?
        );
        return Ok(());
    }

    println!(
        "Scanning {} ports {} on {} (concurrency={}, timeout={}ms)",
        result.target, result.port_range, result.target, result.concurrency, result.timeout_ms
    );
    for port in &result.ports {
        if port.open {
            match (port.pid, port.command.as_deref()) {
                (Some(pid), Some(command)) => {
                    println!(
                        "[OPEN]  Port {:>5} is open (pid={}, command={})",
                        port.port, pid, command
                    )
                }
                (Some(pid), None) => {
                    println!("[OPEN]  Port {:>5} is open (pid={})", port.port, pid)
                }
                (None, _) => println!("[OPEN]  Port {:>5} is open", port.port),
            }
        } else {
            println!("[CLOSED] Port {:>5} is closed", port.port);
        }
    }
    println!("\nScan finished.");
    println!("Total ports scanned: {}", result.total);
    println!(
        "Open ports: {}  Closed ports: {}",
        result.open_count, result.closed_count
    );
    if !result.open_ports.is_empty() {
        println!("Open port list: {:?}", result.open_ports);
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum PortScanError {
    #[error("invalid port: {0}")]
    InvalidPort(String),
    #[error("port range is invalid: {0}")]
    InvalidPortRange(String),
    #[error("too many ports requested, maximum is 4096")]
    TooManyPorts,
    #[error("tokio runtime error: {0}")]
    RuntimeError(String),
    #[error("join error: {0}")]
    JoinError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ShowType {
    All,
    Open,
    Closed,
}

impl Default for ShowType {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Deserialize)]
pub struct PortScanRequest {
    pub target: Option<String>,
    pub port: Option<String>,
    pub concurrency: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PortStatus {
    pub port: u32,
    pub open: bool,
    pub pid: Option<u32>,
    pub command: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PortScanResult {
    pub target: String,
    pub port_range: String,
    pub concurrency: usize,
    pub timeout_ms: u64,
    pub total: usize,
    pub open_count: usize,
    pub closed_count: usize,
    pub open_ports: Vec<u32>,
    pub ports: Vec<PortStatus>,
}

pub async fn scan_ports(request: PortScanRequest) -> Result<PortScanResult, PortScanError> {
    let target = request
        .target
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = request
        .port
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "80".to_string());
    let concurrency = request.concurrency.unwrap_or(100).clamp(1, 1000);
    let timeout_ms = request.timeout_ms.unwrap_or(1000).clamp(50, 10_000);
    remote_scan(target, &port, concurrency, timeout_ms).await
}

pub async fn remote_scan(
    target: String,
    port: &str,
    concurrency: usize,
    timeout_ms: u64,
) -> Result<PortScanResult, PortScanError> {
    let (start, end) = parse_port_range(port)?;
    if end - start > 4095 {
        return Err(PortScanError::TooManyPorts);
    }

    let sem = Arc::new(Semaphore::new(concurrency));
    let mut tasks = FuturesUnordered::new();

    for port in start..=end {
        let permit = sem
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore acquire failed");
        let target_clone = target.clone();
        let to = Duration::from_millis(timeout_ms);

        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            let addr = format!("{}:{}", target_clone, port);
            let open = matches!(
                timeout(to, tokio::net::TcpStream::connect(&addr)).await,
                Ok(Ok(_))
            );
            PortStatus {
                port,
                open,
                pid: None,
                command: None,
            }
        }));
    }

    let mut ports = Vec::new();
    while let Some(join_res) = tasks.next().await {
        match join_res {
            Ok(status) => ports.push(status),
            Err(e) => return Err(PortScanError::JoinError(e.to_string())),
        }
    }

    ports.sort_by_key(|status| status.port);
    if is_local_target(&target) {
        let pid_map = local_tcp_listen_pids();
        let command_map = local_process_commands();
        for status in &mut ports {
            if status.open {
                status.pid = pid_map.get(&status.port).copied();
                status.command = status.pid.and_then(|pid| command_map.get(&pid).cloned());
            }
        }
    }
    let open_ports: Vec<u32> = ports
        .iter()
        .filter(|status| status.open)
        .map(|status| status.port)
        .collect();
    let total = ports.len();
    let open_count = open_ports.len();

    Ok(PortScanResult {
        target,
        port_range: port.to_string(),
        concurrency,
        timeout_ms,
        total,
        open_count,
        closed_count: total - open_count,
        open_ports,
        ports,
    })
}

fn parse_port_range(s: &str) -> Result<(u32, u32), PortScanError> {
    let s = s.trim();
    if let Some((a, b)) = s.split_once('-') {
        let start: u32 = a
            .trim()
            .parse()
            .map_err(|_| PortScanError::InvalidPort(a.into()))?;
        let end: u32 = b
            .trim()
            .parse()
            .map_err(|_| PortScanError::InvalidPort(b.into()))?;
        validate_port(start, s)?;
        validate_port(end, s)?;
        if start > end {
            return Err(PortScanError::InvalidPortRange(s.into()));
        }
        Ok((start, end))
    } else {
        let port: u32 = s
            .parse()
            .map_err(|_| PortScanError::InvalidPort(s.into()))?;
        validate_port(port, s)?;
        Ok((port, port))
    }
}

fn validate_port(port: u32, raw: &str) -> Result<(), PortScanError> {
    if (1..=65535).contains(&port) {
        Ok(())
    } else {
        Err(PortScanError::InvalidPortRange(raw.into()))
    }
}

fn is_local_target(target: &str) -> bool {
    matches!(
        target.trim().to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1" | "0.0.0.0"
    )
}

fn local_tcp_listen_pids() -> HashMap<u32, u32> {
    #[cfg(windows)]
    {
        windows_tcp_listen_pids()
    }

    #[cfg(not(windows))]
    {
        HashMap::new()
    }
}

fn local_process_commands() -> HashMap<u32, String> {
    #[cfg(windows)]
    {
        windows_process_commands()
    }

    #[cfg(not(windows))]
    {
        HashMap::new()
    }
}

#[cfg(windows)]
fn windows_tcp_listen_pids() -> HashMap<u32, u32> {
    let mut pids = HashMap::new();
    let output = Command::new("netstat").args(["-ano", "-p", "tcp"]).output();
    let Ok(output) = output else {
        return pids;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 || !parts[0].eq_ignore_ascii_case("TCP") {
            continue;
        }
        if !parts[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        let Some(port) = parse_addr_port(parts[1]) else {
            continue;
        };
        let Ok(pid) = parts[4].parse::<u32>() else {
            continue;
        };
        pids.entry(port).or_insert(pid);
    }
    pids
}

#[cfg(windows)]
fn parse_addr_port(addr: &str) -> Option<u32> {
    addr.rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u32>().ok())
}

#[cfg(windows)]
fn windows_process_commands() -> HashMap<u32, String> {
    let mut commands = HashMap::new();
    let script = "Get-CimInstance Win32_Process | ForEach-Object { \"$($_.ProcessId)`t$($_.Name)`t$($_.CommandLine)\" }";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
    let Ok(output) = output else {
        return commands;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(pid_text) = parts.next() else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        let name = parts.next().unwrap_or("").trim();
        let command_line = parts.next().unwrap_or("").trim();
        let command = if command_line.is_empty() {
            name
        } else {
            command_line
        };
        if !command.is_empty() {
            commands.insert(pid, command.to_string());
        }
    }

    for (pid, name) in windows_tasklist_names() {
        commands.entry(pid).or_insert(name);
    }
    for (pid, name) in windows_get_process_names() {
        commands.entry(pid).or_insert(name);
    }
    commands
}

#[cfg(windows)]
fn windows_tasklist_names() -> HashMap<u32, String> {
    let mut names = HashMap::new();
    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output();
    let Ok(output) = output else {
        return names;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let fields = parse_csv_line(line);
        if fields.len() < 2 {
            continue;
        }
        let Ok(pid) = fields[1].parse::<u32>() else {
            continue;
        };
        if !fields[0].trim().is_empty() {
            names.insert(pid, fields[0].trim().to_string());
        }
    }
    names
}

#[cfg(windows)]
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

#[cfg(windows)]
fn windows_get_process_names() -> HashMap<u32, String> {
    let mut names = HashMap::new();
    let script = "Get-Process | ForEach-Object { \"$($_.Id)`t$($_.ProcessName)`t$($_.Path)\" }";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
    let Ok(output) = output else {
        return names;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(pid_text) = parts.next() else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        let name = parts.next().unwrap_or("").trim();
        let path = parts.next().unwrap_or("").trim();
        let command = if path.is_empty() { name } else { path };
        if !command.is_empty() {
            names.insert(pid, command.to_string());
        }
    }
    names
}
