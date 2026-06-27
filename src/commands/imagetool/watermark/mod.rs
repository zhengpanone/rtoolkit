use clap::{Args, Subcommand};
pub mod image;
pub mod text;

/// 水印
#[derive(Args, Debug)]
pub struct WaterTool {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    #[command(about = "图片水印")]
    Image(image::ImageArgs),
    #[command(about = "文字水印")]
    Text(text::TextArgs),
}

impl WaterTool {
    pub fn run(self) -> Result<(), WatermarkError> {
        match self.command {
            ImageCommand::Image(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Text(args) => {
                args.run()?;
                Ok(())
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WatermarkError {
    #[error("compress error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("convert error: {0}")]
    TextError(#[from] text::TextError),
}
