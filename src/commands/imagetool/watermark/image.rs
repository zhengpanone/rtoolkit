use std::path::PathBuf;

use clap::Args;

/// 图片水印
#[derive(Args, Debug)]
pub struct ImageArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum ImageError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl ImageArgs {
    pub fn run(self) -> Result<(), ImageError> {
        println!("{:#?}", self);
        Ok(())
    }
}
