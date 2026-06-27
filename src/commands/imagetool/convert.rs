use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct ConvertArgs {
    #[arg(value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum ConvertError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl ConvertArgs {
    pub fn run(self) -> Result<(), ConvertError> {
        println!("{:#?}", self);
        Ok(())
    }
}
