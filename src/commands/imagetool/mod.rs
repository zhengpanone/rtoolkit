use clap::{Args, Subcommand};

pub mod basic;
pub mod color;
pub mod filter;
pub mod watermark;

/// 图片工具
#[derive(Args, Debug)]
pub struct ImageTool {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    #[command(about = "基础编辑")]
    Basic(basic::BasicTool),
    #[command(about = "颜色调整")]
    Color(color::ColorTool),
    #[command(about = "滤镜")]
    Filter(filter::FilterTool),
    Watermark(watermark::WaterTool),
}

#[derive(thiserror::Error, Debug)]
pub enum ImageToolError {
    #[error("compress error: {0}")]
    BasicError(#[from] basic::BasicError),
    #[error("color error: {0}")]
    ColorError(#[from] color::ColorError),
    #[error("filter error: {0}")]
    FilterError(#[from] filter::FilterError),
    #[error("watermark error: {0}")]
    WatermarkError(#[from] watermark::WatermarkError),
}

impl ImageTool {
    pub fn run(self) -> Result<(), ImageToolError> {
        match self.command {
            ImageCommand::Basic(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Color(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Filter(arg) => {
                arg.run()?;
                Ok(())
            }
            ImageCommand::Watermark(args) => {
                args.run()?;
                Ok(())
            }
        }
    }
}
