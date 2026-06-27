use std::path::PathBuf;

use clap::Args;

/// 模糊
#[derive(Args, Debug)]
pub struct BlurArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum BlurError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl BlurArgs {
    pub fn run(self) -> Result<(), BlurError> {
        println!("{:#?}", self);
        Ok(())
    }
}
