use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    idgen::{run_gen_id, IdOpts},
    portscan::{run_port_scan, PortScanOpts},
};

// 公共 Command trait + 注册函数
pub mod idgen;
pub mod portscan;

#[derive(Parser)]
#[command(name = "rtoolkit", version, about = "Rust Toolkit CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    #[command(about = "生成中国身份证号")]
    Idgen {
        /// 生成中国大陆 18 位身份证号（校验位符合 GB 11643 / MOD 11-2）
        #[command(flatten)]
        opts: IdOpts,
    },
    #[command(about = "端口扫描")]
    PortScan {
        #[command(flatten)]
        opts: PortScanOpts,
    },
}

pub fn build_cli() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Idgen { opts } => run_gen_id(opts)?,
        Commands::PortScan { opts } => run_port_scan(opts)?,
    };
    Ok(())
}
