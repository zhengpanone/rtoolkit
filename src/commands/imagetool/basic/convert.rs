use std::path::PathBuf;

use clap::{Args, ValueEnum};
use image::ImageFormat;

/// 图片格式转换
#[derive(Args, Debug)]
pub struct ConvertArgs {
    #[arg(
        short,
        long,
        value_name = "Input File or Directory",
        help = "输入图片文件或文件夹"
    )]
    input: PathBuf,
    #[arg(
        short,
        long,
        value_name = "Output File or Directory",
        help = "输出图片文件或文件夹"
    )]
    output: PathBuf,

    #[arg(
        short,
        long,
        num_args = 1..,
        value_delimiter = ',',
        value_name = "FORMAT",
        help = "输出图片格式,支持多个(如 --format png,jpg)"
    )]
    format: Vec<ImageFormatArg>,

    #[arg(
        long,
        value_name = "MODE",
        default_value = "none",
        help = "组织输出文件模式: none/ by-format / by-name"
    )]
    organize: OrganizeMode,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizeMode {
    /// 不创建子文件夹, 如 output/png、 output/webp
    None,
    /// 按格式创建子文件夹，如 output/png/、output/webp/
    ByFormat,
    /// 按原始文件名（不含扩展名）创建子文件夹，如 output/photo/
    ByName,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ImageFormatArg {
    Png,
    Jpg,
    Jpeg,
    Webp,
    Bmp,
    Gif,
    Tiff,
    Ico,
}

impl ImageFormatArg {
    fn to_image_format(self) -> ImageFormat {
        match self {
            ImageFormatArg::Png => ImageFormat::Png,
            ImageFormatArg::Jpg | ImageFormatArg::Jpeg => ImageFormat::Jpeg,
            ImageFormatArg::Webp => ImageFormat::WebP,
            ImageFormatArg::Bmp => ImageFormat::Bmp,
            ImageFormatArg::Gif => ImageFormat::Gif,
            ImageFormatArg::Tiff => ImageFormat::Tiff,
            ImageFormatArg::Ico => ImageFormat::Ico,
        }
    }

    fn ext(self) -> &'static str {
        match self {
            ImageFormatArg::Png => "png",
            ImageFormatArg::Jpg | ImageFormatArg::Jpeg => "jpg",
            ImageFormatArg::Webp => "webp",
            ImageFormatArg::Bmp => "bmp",
            ImageFormatArg::Gif => "gif",
            ImageFormatArg::Tiff => "tiff",
            ImageFormatArg::Ico => "ico",
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConvertError {
    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("单文件输入时必须通过 --format 指定至少一种输出格式")]
    NoFormatSpecified,

    #[error("无法识别的图片格式: {0}")]
    UnknownFormat(PathBuf),

    #[error("单文件转单一格式时输出可以是文件路径，但指定了多个格式")]
    AmbiguousOutput,

    #[error("Unsupport output format, Please use the specified extension name or use --format param : {0}")]
    UnsupportedOutputFormat(PathBuf),

    #[error("Unsupport format: {0}")]
    UnsupportedFormat(PathBuf),

    #[error("Read image error {path} : {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Save image error {path} : {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ConvertArgs {
    pub fn run(self) -> Result<(), ConvertError> {
        // check input file exists
        if !self.input.exists() {
            return Err(ConvertError::InputNotFound(self.input));
        }
        if self.input.is_dir() {
            self.run_dir()
        } else {
            self.run_file()
        }
    }
    /// 单一文件
    fn run_file(self) -> Result<(), ConvertError> {
        if self.format.is_empty() {
            // 没有指定 --format, 尝试从output扩展名推断
            let fmt =
                infer_format_from_path(&self.output).ok_or(ConvertError::NoFormatSpecified)?;
            convert_one(&self.input, &self.output, fmt)?;
        } else if self.format.len() == 1 && !self.output.is_dir() {
            // TODO
            // 单格式 + output 是文件路径
            let dest = resolve_output_path(
                &self.output,
                &self.input,
                self.format[0],
                self.organize,
                true,
            );
            convert_one(&self.input, &dest, self.format[0])?;
        } else {
            // 多格式, output 必须是目录
            for fmt in &self.format {
                let dest =
                    resolve_output_path(&self.output, &self.input, *fmt, self.organize, true);
                convert_one(&self.input, &dest, *fmt)?;
            }
        }
        Ok(())
    }

    /// 文件夹
    fn run_dir(self) -> Result<(), ConvertError> {
        let entries = collect_images(&self.input)?;
        if entries.is_empty() {
            println!("No image found in {}", self.input.display());
            return Ok(());
        }
        let formats: Vec<ImageFormatArg> = if self.format.is_empty() {
            vec![]
        } else {
            self.format.clone()
        };
        for src in &entries {
            if formats.is_empty() {
                let fmt = infer_format_from_path(src)
                    .ok_or_else(|| ConvertError::UnknownFormat(src.clone()))?;
                let dest = resolve_output_path(&self.output, src, fmt, self.organize, true);
                convert_one(src, &dest, fmt)?;
            }else{
                for fmt in &formats{
                    let dest = resolve_output_path(&self.output, src, *fmt, self.organize, true);
                    convert_one(src, &dest, *fmt)?;
                }
            }
        }
        Ok(())
    }
}

fn convert_one(src: &PathBuf, dest: &PathBuf, fmt: ImageFormatArg) -> Result<(), ConvertError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let img = image::open(src).map_err(|e| ConvertError::ReadError {
        path: src.clone(),
        source: e,
    })?;

    img.save_with_format(dest, fmt.to_image_format())
        .map_err(|e| ConvertError::WriteError {
            path: dest.clone(),
            source: e,
        })?;
    println!("{} -> {}", src.display(), dest.display());
    Ok(())
}

fn ext_to_image_format(ext: &str) -> Option<ImageFormatArg> {
    match ext.to_lowercase().as_str() {
        "png" => Some(ImageFormatArg::Png),
        "jpg" | "jpeg" => Some(ImageFormatArg::Jpeg),
        "webp" => Some(ImageFormatArg::Webp),
        "bmp" => Some(ImageFormatArg::Bmp),
        "gif" => Some(ImageFormatArg::Gif),
        "tiff" | "tif" => Some(ImageFormatArg::Tiff),
        "ico" => Some(ImageFormatArg::Ico),
        _ => None,
    }
}

fn infer_format_from_path(path: &PathBuf) -> Option<ImageFormatArg> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    ext_to_image_format(ext.as_str())
}

/// 根据组织模式计算最终输出路径
///
/// - `output_root`     : CLI 传入的 --output 路径
/// - `src`             : 源文件路径(用于取文件名/stem)
/// - `fmt`             : 目标格式
/// - `mode`            : 组织模式
/// - `output_is_dir`   : output_root 是否应视为目录
fn resolve_output_path(
    output_root: &PathBuf,
    src: &PathBuf,
    fmt: ImageFormatArg,
    mode: OrganizeMode,
    output_is_dir: bool,
) -> PathBuf {
    if !output_is_dir {
        // 直接视为完整文件路径,应忽略组织模式
        return output_root.clone();
    }
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let filename = format!("{}.{}", stem, fmt.ext());

    match mode {
        OrganizeMode::None => output_root.join(&filename),
        OrganizeMode::ByFormat => output_root.join(fmt.ext()).join(&filename),
        OrganizeMode::ByName => output_root.join(stem.as_ref()).join(&filename),
    }
}

/// 递归收集目录下所有可识别的图片文件
fn collect_images(dir: &PathBuf) -> Result<Vec<PathBuf>, ConvertError> {
    let mut results = Vec::new();
    collect_images_recusive(dir, &mut results)?;
    results.sort();
    Ok(results)
}

fn collect_images_recusive(dir: &PathBuf, out: &mut Vec<PathBuf>) -> Result<(), ConvertError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_images_recusive(&path, out)?;
        } else if infer_format_from_path(&path).is_some() {
            out.push(path);
        }
    }
    Ok(())
}
