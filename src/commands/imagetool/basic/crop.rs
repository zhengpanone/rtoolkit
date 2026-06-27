use std::path::PathBuf;

use clap::Args;

/// 图片裁剪
#[derive(Args, Debug)]
pub struct CropArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    output: PathBuf,
    #[arg(short, long, value_name = "x", help = "裁剪起始点x坐标")]
    pub x: u32,
    #[arg(short, long, value_name = "y", help = "裁剪起始点y坐标")]
    pub y: u32,
    #[arg(short, long, value_name = "width", help = "裁剪宽度")]
    pub width: u32,
    #[arg(short, long, value_name = "height", help = "裁剪高度")]
    pub height: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum CropError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid output: {0}")]
    InvalidOutput(String),
}

impl CropArgs {
    pub fn run(self) -> Result<(), CropError> {
        println!("{:#?}", self);
        Ok(())
    }
}
