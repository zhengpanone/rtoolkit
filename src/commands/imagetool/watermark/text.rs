use std::path::PathBuf;

use clap::Args;

/// 文字水印
#[derive(Args, Debug)]
pub struct TextArgs {
    #[arg(value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum TextError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl TextArgs {
    pub fn run(self) -> Result<(), TextError> {
        println!("{:#?}", self);
        Ok(())
    }
}
