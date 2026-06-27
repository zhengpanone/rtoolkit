use std::path::PathBuf;

use clap::Args;

/// contrast 调整对比度
#[derive(Args, Debug)]
pub struct ContrastArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum ContrastError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl ContrastArgs {
    pub fn run(self) -> Result<(), ContrastError> {
        println!("{:#?}", self);
        Ok(())
    }
}
