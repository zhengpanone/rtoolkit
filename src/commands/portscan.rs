use std::sync::Arc;

use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

// 端口扫描实现
#[derive(clap::Args)]
pub struct PortScanOpts {
    /// 目标主机，留空为本机
    #[arg(
        short = 't',
        long = "target",
        value_name = "HOST",
        default_value = "127.0.0.1",
        help = "目标主机（留空为本机）"
    )]
    target: Option<String>,
    /// 端口范围，支持单端口，如 80，也支持范围，如 80-100
    #[arg(
        short = 'p',
        long = "port",
        value_name = "RANGE",
        default_value = "80",
        help = "目标端口",
        long_help = "目标端口（留空为 80)"
    )]
    port: Option<String>,
    /// 并发限制(只在远程扫描时生效)
    #[arg(
        short = 'c',
        long = "concurrency",
        value_name = "N",
        default_value = "100",
        help = "并发数",
        long_help = "并发数（留空为 100, 远程扫描时生效）"
    )]
    concurrency: Option<usize>,

    #[arg(
        long = "timeout",
        default_value = "1000",
        value_name = "MS",
        help = "超时时间(毫秒)",
        long_help = "超时时间（留空为 1000, 远程扫描时生效）"
    )]
    time_out: Option<u64>,
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FMT",
        default_value = "plain",
        help = "输出格式",
        long_help = "输出格式（留空为 plain, plain| json| csv）"
    )]
    output: Option<String>,
}

pub fn run_port_scan(opts: PortScanOpts) -> Result<(), PortScanError> {
    // 安全使用默认值（避免 unwrap panic）
    let target = opts.target.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = opts.port.unwrap_or_else(|| "80".to_string());
    let concurrency = opts.concurrency.unwrap_or(100);
    let timeout_ms = opts.time_out.unwrap_or(1000);
    let _output = opts.output.unwrap_or("plain".to_string());

    // 创建 tokio runtime 并执行异步扫描
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| PortScanError::RuntimeError(e.to_string()))?;
    rt.block_on(async move { remote_scan(target, &port, concurrency, timeout_ms).await })?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum PortScanError {
    #[error("invalid port: {0}")]
    InvalidPort(String),
    #[error("port range is invalid: {0}")]
    InvalidPortRange(String),
    #[error("tokio runtime error: {0}")]
    RuntimeError(String),
    #[error("join error: {0}")]
    JoinError(String),
}

pub async fn remote_scan(
    target: String,
    port: &str,
    concurrency: usize,
    timeout_ms: u64,
) -> Result<(), PortScanError> {
    let (start, end) = parse_port_range(port)?;
    println!(
        "Scanning {} ports {}-{} on {} (concurrency={}, timeout={}ms)",
        target, start, end, target, concurrency, timeout_ms
    );
    // 必须使用 Arc，否则 sem.clone() 不存在
    let sem = Arc::new(Semaphore::new(concurrency));

    let mut tasks = FuturesUnordered::new();

    for port in start..=end {
        // 获取一个可移动的 permit
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

            let is_open = match timeout(to, tokio::net::TcpStream::connect(&addr)).await {
                Ok(Ok(_stream)) => true,
                _ => false,
            };
            (port, is_open)
        }));
    }
    // 收集并打印开放端口
    // 统计
    let mut open_ports: Vec<u32> = Vec::new();
    let mut closed_count: usize = 0;
    let mut total: usize = 0;
    while let Some(join_res) = tasks.next().await {
        total += 1;
        match join_res {
            Ok((port_num, true)) => {
                println!("[OPEN]  Port {:>5} is open", port_num);
                open_ports.push(port_num);
            }
            Ok((port_num, false)) => {
                println!("[CLOSED] Port {:>5} is closed", port_num);
                closed_count += 1;
            }
            Err(e) => {
                eprintln!("[ERROR] task join error: {}", e);
                return Err(PortScanError::JoinError(e.to_string()));
            }
        }
    }

    println!("\nScan finished.");
    println!("Total ports scanned: {}", total);
    println!(
        "Open ports: {}  Closed ports: {}",
        open_ports.len(),
        closed_count
    );

    if !open_ports.is_empty() {
        println!("Open port list: {:?}", open_ports);
    }

    Ok(())
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
            .map_err(|_| PortScanError::InvalidPort(a.into()))?;
        if start == 0 || end == 0 || start > end {
            return Err(PortScanError::InvalidPortRange(s.into()));
        }
        Ok((start, end))
    } else {
        let port: u32 = s
            .parse()
            .map_err(|_| PortScanError::InvalidPort(s.into()))?;
        Ok((port, port))
    }
}
