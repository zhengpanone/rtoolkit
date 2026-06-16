use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    idgen::{run_gen_id, IdOpts},
    jsonfmt::{run_json_fmt, JsonFmtOpts},
    portscan::{run_port_scan, PortScanOpts},
};
use crate::web::{run_web, WebOpts};

// 公共 Command trait + 注册函数
pub mod idgen;
pub mod jsonfmt;
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
    #[command(name = "jsonfmt", alias = "json-fmt", about = "JSON 格式化")]
    JsonFmt {
        #[command(flatten)]
        opts: JsonFmtOpts,
    },
    #[command(about = "启动本地 Web 工作台")]
    Web {
        #[command(flatten)]
        opts: WebOpts,
    },
}

pub fn build_cli() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Idgen { opts } => run_gen_id(opts)?,
        Commands::PortScan { opts } => run_port_scan(opts)?,
        Commands::JsonFmt { opts } => run_json_fmt(opts)?,
        Commands::Web { opts } => run_web(opts)?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
