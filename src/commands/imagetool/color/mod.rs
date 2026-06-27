use clap::{Args, Subcommand};

pub mod brightness;
pub mod contrast;
// 颜色调整
// gamma         Gamma 校正
// saturation    饱和度
// hue           色相
// grayscale     灰度
// invert        颜色反转
// sepia         怀旧色
// TODO

/// 颜色调整
#[derive(Args, Debug)]
pub struct ColorTool {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    #[command(about = "图片压缩")]
    Compress(brightness::BrightnessArgs),
    #[command(about = "图片转换")]
    Convert(contrast::ContrastArgs),
}

impl ColorTool {
    pub fn run(self) -> Result<(), ColorError> {
        match self.command {
            ImageCommand::Compress(args) => {
                args.run()?;
                Ok(())
            }
            ImageCommand::Convert(args) => {
                args.run()?;
                Ok(())
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ColorError {
    #[error("compress error: {0}")]
    ContrastError(#[from] contrast::ContrastError),
    #[error("convert error: {0}")]
    BrightnessError(#[from] brightness::BrightnessError),
}
