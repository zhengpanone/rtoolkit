use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use rtoolkit::{generate_id, parse_date, Gender, IdError};

#[derive(Parser)]
#[command(name = "rtoolkit", version, about = "Rust Toolkit CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "身份证", long_about = "身份证生成器")]
    Id {
        /// 生成中国大陆 18 位身份证号（校验位符合 GB 11643 / MOD 11-2）
        #[command(flatten)]
        opts: IdOpts,
    },
}

#[derive(clap::Args)]
struct IdOpts {
    #[arg(
        short = 'n',
        long,
        default_value_t = 1,
        help = "生成数量",
        long_help = "生成身份证号数量"
    )]
    count: u32,
    /// 指定6位地区码
    #[arg(long = "region", help = "地区")]
    region: Option<String>,
    /// 出生日期
    #[arg(long = "birth", help = "出生日期")]
    birth: Option<String>,

    /// 随机生日的最小日期（含）
    #[arg(long, default_value = "1970-01-01")]
    min_birth: String,

    /// 随机生日的最大日期（含）
    #[arg(long, default_value = "2010-12-31")]
    max_birth: String,

    /// 性别（male 奇数、female 偶数、 any随机）
    #[arg(value_enum,long = "gender", default_value_t = Gender::Any, help = "性别")]
    gender: Gender,
}

fn main() -> Result<(), IdError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Id { opts } => run_id(opts)?,
    };
    Ok(())
}

fn run_id(opts: IdOpts) -> Result<(), IdError> {
    let region = opts.region.as_deref();
    let min_date: NaiveDate = parse_date(&opts.min_birth)?;
    let max_date: NaiveDate = parse_date(&opts.max_birth)?;
    let fixed_birth = match opts.birth {
        Some(b) => Some(parse_date(&b)?),
        None => None,
    };

    for _ in 0..opts.count {
        let id = generate_id(region, fixed_birth, min_date, max_date, opts.gender)?;
        println!("{}", id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
