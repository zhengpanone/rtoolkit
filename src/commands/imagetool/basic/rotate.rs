use std::path::PathBuf;

use clap::Args;

/// 图片旋转
/// flip 上下翻转
/// flop 左右翻转
#[derive(Args, Debug)]
pub struct RotateArgs {
    #[arg(value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,

    #[arg(
        short,
        long,
        value_name = "QUALITY",
        default_value_t = 80,
        help = "压缩质量，范围 0-100"
    )]
    quality: u8,
}

#[derive(thiserror::Error, Debug)]
pub enum RotateError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl RotateArgs {
    pub fn run(self) -> Result<(), RotateError> {
        println!("{:#?}", self);
        Ok(())
    }
}
