use clap::{Args, Subcommand};

pub mod blur;
pub mod edge;
pub mod emboss;
pub mod oil;
pub mod pixelate;
pub mod sharpen;
pub mod sketch;

/// 滤镜
#[derive(Args, Debug)]
pub struct FilterTool {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    #[command(about = "模糊")]
    Blur(blur::BlurArgs),
    #[command(about = "边缘检测")]
    Edge(edge::EdgeArgs),
    #[command(about = "浮雕")]
    Emboss(emboss::EmbossArgs),
    #[command(about = "油画")]
    Oil(oil::OilArgs),
    #[command(about = "像素化")]
    Pixelate(pixelate::PixelateArgs),
    #[command(about = "锐化")]
    Sharpen(sharpen::SharpenArgs),
    #[command(about = "素描")]
    Sketch(sketch::SketchArgs),
}

impl FilterTool {
    pub fn run(self) -> Result<(), FilterError> {
        match self.command {
            ImageCommand::Blur(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Edge(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Emboss(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Oil(arg) => {
                arg.run()?;
                Ok(())
            }
            ImageCommand::Pixelate(arg) => {
                arg.run()?;
                Ok(())
            }
            ImageCommand::Sharpen(arg) => {
                arg.run()?;
                Ok(())
            }
            ImageCommand::Sketch(arg) => {
                arg.run()?;
                Ok(())
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FilterError {
    #[error("blur error: {0}")]
    BlurError(#[from] blur::BlurError),
    #[error("edge error: {0}")]
    EdgeError(#[from] edge::EdgeError),
    #[error("emboss error: {0}")]
    EmbossError(#[from] emboss::EmbossError),
    #[error("oil error: {0}")]
    OilError(#[from] oil::OilError),
    #[error("pixelate error: {0}")]
    PixelateError(#[from] pixelate::PixelateError),
    #[error("sharpen error: {0}")]
    SharpenError(#[from] sharpen::SharpenError),
    #[error("sketch error: {0}")]
    SketchError(#[from] sketch::SketchError),
}
