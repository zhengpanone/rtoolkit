/// brightness 调整亮度
use std::path::PathBuf;

use clap::Args;

/// 图片裁剪
#[derive(Args, Debug)]
pub struct BrightnessArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum BrightnessError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl BrightnessArgs {
    pub fn run(self) -> Result<(), BrightnessError> {
        println!("{:#?}", self);
        Ok(())
    }
}
