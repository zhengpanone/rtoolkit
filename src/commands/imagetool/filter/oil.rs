use std::path::PathBuf;

use clap::Args;

/// 油画
#[derive(Args, Debug)]
pub struct OilArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum OilError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl OilArgs {
    pub fn run(self) -> Result<(), OilError> {
        println!("{:#?}", self);
        Ok(())
    }
}
