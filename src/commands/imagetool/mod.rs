use clap::{Args, Subcommand};

pub mod compress;
pub mod convert;
pub mod crop;
pub mod resize;
pub mod watermark;

#[derive(Args, Debug)]
pub struct ImageTool {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    #[command(about = "图片压缩")]
    Compress(compress::CompressArgs),
    #[command(about = "图片转换")]
    Convert(convert::ConvertArgs),

    #[command(about = "图片裁剪")]
    Crop(crop::CropArgs),

    #[command(about = "图片缩放")]
    Resize(resize::ResizeArgs),

    #[command(about = "图片水印")]
    Watermark(watermark::WatermarkArgs),
}

impl ImageTool {
    pub fn run(self) -> Result<(), ImageToolError> {
        match self.command {
            ImageCommand::Compress(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Convert(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Crop(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Resize(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Watermark(args) => {
                args.run()?;
                Ok(())
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ImageToolError {
    #[error("compress error: {0}")]
    CompressError(#[from] compress::CompressError),
    #[error("convert error: {0}")]
    ConvertError(#[from] convert::ConvertError),
    #[error("convert error: {0}")]
    CropError(#[from] crop::CropError),
    #[error("convert error: {0}")]
    ResizeError(#[from] resize::ResizeError),
    #[error("convert error: {0}")]
    WatermarkError(#[from] watermark::WatermarError),
}
