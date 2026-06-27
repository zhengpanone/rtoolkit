use std::path::PathBuf;

use clap::Args;

/// 素描
#[derive(Args, Debug)]
pub struct SketchArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum SketchError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl SketchArgs {
    pub fn run(self) -> Result<(), SketchError> {
        println!("{:#?}", self);
        Ok(())
    }
}
