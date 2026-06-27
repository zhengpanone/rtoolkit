use std::path::PathBuf;

use clap::Args;

/// 图片缩放
#[derive(Args, Debug)]
pub struct ResizeArgs {
    #[arg(short, long, value_name = "input.png", help = "输入图片文件")]
    pub input: PathBuf,
    #[arg(short, long, value_name = "output.png", help = "输出图片文件")]
    pub output: PathBuf,
    #[arg(short, long, value_name = "width", help = "缩放宽度")]
    pub width: u32,
    #[arg(short, long, value_name = "height", help = "缩放高度")]
    pub height: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum ResizeError {}

impl ResizeArgs {
    pub fn run(self) -> Result<(), ResizeError> {
        println!("{:#?}", self);
        Ok(())
    }
}
