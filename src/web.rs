use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::{Deserialize, Serialize};

use std::io::Cursor;

use crate::commands::idgen::{
    generate_ids, validate_id_download_request, write_generated_ids, IdGenerateRequest, OutputType,
};
use crate::commands::imagetool::basic::convert::ImageFormatArg;
use crate::commands::jsonfmt::{format_json_text, MAX_INDENT};
use crate::commands::portscan::{scan_ports, PortScanRequest};
use crate::utils::areas::{all_cities, all_provinces, all_regions, Area, City, Province};

const INDEX_HTML: &str = include_str!("../static/index.html");
const IDGEN_HTML: &str = include_str!("../static/idgen.html");
const PORT_SCAN_HTML: &str = include_str!("../static/port-scan.html");
const JSONFMT_HTML: &str = include_str!("../static/jsonfmt.html");
const IMGTOOL_HTML: &str = include_str!("../static/imgtool.html");
const IDGEN_JS: &str = include_str!("../static/idgen.js");
const PORT_SCAN_JS: &str = include_str!("../static/port-scan.js");
const JSONFMT_JS: &str = include_str!("../static/jsonfmt.js");
const IMGTOOL_JS: &str = include_str!("../static/imgtool.js");
const STYLES_CSS: &str = include_str!("../static/styles.css");

#[derive(clap::Args)]
pub struct WebOpts {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Serialize)]
struct IdGenerateResponse<T> {
    records: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct JsonFmtRequest {
    input: String,
    indent: Option<usize>,
    sort: Option<bool>,
    compact: Option<bool>,
}

#[derive(Serialize)]
struct JsonFmtResponse {
    output: String,
    bytes: usize,
    lines: usize,
}

#[derive(Serialize)]
struct RegionOptionsResponse {
    provinces: Vec<Province>,
    cities: Vec<City>,
    regions: Vec<Area>,
}

#[derive(Deserialize)]
struct IdDownloadRequest {
    #[serde(flatten)]
    params: IdGenerateRequest,
    format: Option<OutputType>,
}

pub fn run_web(opts: WebOpts) -> anyhow::Result<()> {
    let addr = format!("{}:{}", opts.host, opts.port);
    let listener = TcpListener::bind(&addr)?;
    println!("rtoolkit web listening on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream) {
                    eprintln!("request failed: {}", error);
                }
            }
            Err(error) => eprintln!("connection failed: {}", error),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    if request_line.trim().is_empty() {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return write_json(
            &mut stream,
            400,
            &ErrorResponse {
                error: "bad request".to_string(),
            },
        );
    }

    let method = parts[0];
    let target = parts[1];
    let mut content_length = 0usize;
    let mut content_type_header = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
            if name.eq_ignore_ascii_case("content-type") {
                content_type_header = value.trim().to_string();
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let (path, query) = target.split_once('?').unwrap_or((target, ""));

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => write_html(&mut stream, INDEX_HTML),
        ("GET", "/idgen") | ("GET", "/idgen.html") => write_html(&mut stream, IDGEN_HTML),
        ("GET", "/port-scan") | ("GET", "/port-scan.html") => {
            write_html(&mut stream, PORT_SCAN_HTML)
        }
        ("GET", "/jsonfmt") | ("GET", "/jsonfmt.html") => write_html(&mut stream, JSONFMT_HTML),
        ("GET", "/imgtool") | ("GET", "/imgtool.html") => write_html(&mut stream, IMGTOOL_HTML),
        ("GET", "/idgen.js") => write_text(
            &mut stream,
            "application/javascript; charset=utf-8",
            IDGEN_JS,
        ),
        ("GET", "/port-scan.js") => write_text(
            &mut stream,
            "application/javascript; charset=utf-8",
            PORT_SCAN_JS,
        ),
        ("GET", "/jsonfmt.js") => write_text(
            &mut stream,
            "application/javascript; charset=utf-8",
            JSONFMT_JS,
        ),
        ("GET", "/imgtool.js") => write_text(
            &mut stream,
            "application/javascript; charset=utf-8",
            IMGTOOL_JS,
        ),
        ("GET", "/styles.css") => write_text(&mut stream, "text/css; charset=utf-8", STYLES_CSS),
        ("GET", "/api/health") => {
            write_json(&mut stream, 200, &serde_json::json!({ "status": "ok" }))
        }
        ("GET", "/api/regions") => write_json(
            &mut stream,
            200,
            &RegionOptionsResponse {
                provinces: all_provinces(),
                cities: all_cities(),
                regions: all_regions(),
            },
        ),
        ("POST", "/api/idgen") => {
            let payload: IdGenerateRequest = serde_json::from_slice(&body)?;
            match generate_ids(payload) {
                Ok(records) => write_json(&mut stream, 200, &IdGenerateResponse { records }),
                Err(error) => write_json(
                    &mut stream,
                    400,
                    &ErrorResponse {
                        error: error.to_string(),
                    },
                ),
            }
        }
        ("POST", "/api/idgen/download") => {
            let payload: IdDownloadRequest = serde_json::from_slice(&body)?;
            let format = payload.format.unwrap_or(OutputType::Text);
            match validate_id_download_request(&payload.params, format) {
                Ok(_) => {
                    write_download_header(&mut stream, format)?;
                    write_generated_ids(payload.params, format, &mut stream)?;
                    stream.flush()?;
                    Ok(())
                }
                Err(error) => write_json(
                    &mut stream,
                    400,
                    &ErrorResponse {
                        error: error.to_string(),
                    },
                ),
            }
        }
        ("GET", "/api/idgen/download") => {
            let payload = parse_id_download_query(query);
            let format = payload.format.unwrap_or(OutputType::Text);
            match validate_id_download_request(&payload.params, format) {
                Ok(_) => {
                    write_download_header(&mut stream, format)?;
                    write_generated_ids(payload.params, format, &mut stream)?;
                    stream.flush()?;
                    Ok(())
                }
                Err(error) => write_json(
                    &mut stream,
                    400,
                    &ErrorResponse {
                        error: error.to_string(),
                    },
                ),
            }
        }
        ("POST", "/api/portscan") => {
            let payload: PortScanRequest = serde_json::from_slice(&body)?;
            let rt = tokio::runtime::Runtime::new()?;
            match rt.block_on(scan_ports(payload)) {
                Ok(result) => write_json(&mut stream, 200, &result),
                Err(error) => write_json(
                    &mut stream,
                    400,
                    &ErrorResponse {
                        error: error.to_string(),
                    },
                ),
            }
        }
        ("POST", "/api/jsonfmt") => {
            let payload: JsonFmtRequest = serde_json::from_slice(&body)?;
            let indent = payload.indent.unwrap_or(2).min(MAX_INDENT);
            match format_json_text(
                &payload.input,
                indent,
                payload.sort.unwrap_or(false),
                payload.compact.unwrap_or(false),
            ) {
                Ok(output) => {
                    let lines = output.lines().count();
                    let bytes = output.len();
                    write_json(
                        &mut stream,
                        200,
                        &JsonFmtResponse {
                            output,
                            bytes,
                            lines,
                        },
                    )
                }
                Err(error) => write_json(
                    &mut stream,
                    400,
                    &ErrorResponse {
                        error: error.to_string(),
                    },
                ),
            }
        }
        ("POST", "/api/imgtool/convert") => {
            handle_img_convert(&mut stream, &body, &content_type_header)
        }
        _ => write_json(
            &mut stream,
            404,
            &ErrorResponse {
                error: "not found".to_string(),
            },
        ),
    }
}

fn handle_img_convert(
    stream: &mut TcpStream,
    body: &[u8],
    content_type: &str,
) -> anyhow::Result<()> {
    // 从 content-type header 解析 boundary
    let boundary = content_type
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            if part.to_lowercase().starts_with("boundary=") {
                Some(part[9..].trim_matches('"').to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();
    if boundary.is_empty() {
        return write_json(
            stream,
            400,
            &ErrorResponse {
                error: "无法解析 multipart boundary".to_string(),
            },
        );
    }

    let (file_data, format_str, filename) =
        match parse_multipart(body, &boundary) {
            Ok(data) => data,
            Err(e) => {
                return write_json(
                    stream,
                    400,
                    &ErrorResponse {
                        error: format!("解析上传文件失败: {}", e),
                    },
                );
            }
        };

    let format = match format_str.to_lowercase().as_str() {
        "png" => ImageFormatArg::Png,
        "jpg" | "jpeg" => ImageFormatArg::Jpg,
        "webp" => ImageFormatArg::Webp,
        "bmp" => ImageFormatArg::Bmp,
        "gif" => ImageFormatArg::Gif,
        "tiff" | "tif" => ImageFormatArg::Tiff,
        "ico" => ImageFormatArg::Ico,
        _ => {
            return write_json(
                stream,
                400,
                &ErrorResponse {
                    error: format!("不支持的目标格式: {}", format_str),
                },
            );
        }
    };

    // 读取并转换图片
    let img = match image::load_from_memory(&file_data) {
        Ok(img) => img,
        Err(e) => {
            return write_json(
                stream,
                400,
                &ErrorResponse {
                    error: format!("图片读取失败: {}", e),
                },
            );
        }
    };

    let mut output_buf = Cursor::new(Vec::new());
    let image_format = match format {
        ImageFormatArg::Png => image::ImageFormat::Png,
        ImageFormatArg::Jpg | ImageFormatArg::Jpeg => image::ImageFormat::Jpeg,
        ImageFormatArg::Webp => image::ImageFormat::WebP,
        ImageFormatArg::Bmp => image::ImageFormat::Bmp,
        ImageFormatArg::Gif => image::ImageFormat::Gif,
        ImageFormatArg::Tiff => image::ImageFormat::Tiff,
        ImageFormatArg::Ico => image::ImageFormat::Ico,
    };

    img.write_to(&mut output_buf, image_format)
        .map_err(|e| anyhow::anyhow!("图片写入失败: {}", e))?;

    let output_data = output_buf.into_inner();
    let output_ext = format_str.to_lowercase();

    // 生成输出文件名
    let output_name = if let Some(stem) = filename.split('.').next() {
        format!("{}.{}", stem, output_ext)
    } else {
        format!("output.{}", output_ext)
    };

    let content_type_str = match output_ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "tiff" | "tif" => "image/tiff",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    };

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        content_type_str,
        output_name,
        output_data.len()
    )?;
    stream.write_all(&output_data)?;
    stream.flush()?;

    Ok(())
}

fn parse_multipart(
    body: &[u8],
    boundary: &str,
) -> Result<(Vec<u8>, String, String), String> {
    let delimiter = format!("--{}", boundary);
    let delimiter_bytes = delimiter.as_bytes();
    let end_delimiter = format!("--{}--", boundary);
    let end_delimiter_bytes = end_delimiter.as_bytes();

    let mut file_data = Vec::new();
    let mut format_val = String::new();
    let mut filename = String::from("image");

    // 找第一个 boundary
    let first_boundary = match find_bytes(body, delimiter_bytes) {
        Some(pos) => pos,
        None => return Err("找不到 boundary".to_string()),
    };

    let mut pos = first_boundary;

    // 循环处理每个 part
    loop {
        // 检查是否是结束 boundary
        if pos + end_delimiter_bytes.len() <= body.len()
            && &body[pos..pos + end_delimiter_bytes.len()] == end_delimiter_bytes
        {
            break;
        }
        // 跳过 boundary 行
        pos = match find_bytes(&body[pos..], b"\r\n") {
            Some(nl) => pos + nl + 2,
            None => break,
        };

        // 解析 headers
        let headers_end = match find_bytes(&body[pos..], b"\r\n\r\n") {
            Some(end) => pos + end,
            None => break,
        };
        let headers_raw = &body[pos..headers_end];
        let headers_str = String::from_utf8_lossy(headers_raw);

        // 解析 name 和 filename
        let mut current_name = String::new();
        let mut current_filename = String::new();
        for line in headers_str.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("content-disposition") {
                if let Some(name_start) = line.find("name=\"") {
                    let after_name = &line[name_start + 6..];
                    if let Some(name_end) = after_name.find('"') {
                        current_name = after_name[..name_end].to_string();
                    }
                }
                if let Some(fn_start) = line.find("filename=\"") {
                    let after_fn = &line[fn_start + 10..];
                    if let Some(fn_end) = after_fn.find('"') {
                        current_filename = after_fn[..fn_end].to_string();
                    }
                }
            }
        }

        pos = headers_end + 4; // 跳过 \r\n\r\n

        // 找下一个 boundary
        let data_end = match find_bytes(&body[pos..], delimiter_bytes) {
            Some(end) => pos + end,
            None => match find_bytes(&body[pos..], end_delimiter_bytes) {
                Some(end) => pos + end,
                None => body.len(),
            },
        };

        // 去掉末尾的 \r\n
        let mut data = &body[pos..data_end];
        if data.ends_with(b"\r\n") {
            data = &data[..data.len() - 2];
        }

        match current_name.as_str() {
            "file" => {
                file_data = data.to_vec();
                if !current_filename.is_empty() {
                    filename = current_filename;
                }
            }
            "format" => {
                format_val = String::from_utf8_lossy(data).trim().to_string();
            }
            _ => {}
        }

        // 移到下一个 boundary
        pos = data_end;
    }

    if file_data.is_empty() {
        return Err("未找到上传文件".to_string());
    }
    if format_val.is_empty() {
        return Err("未指定目标格式".to_string());
    }

    Ok((file_data, format_val, filename))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn write_html(stream: &mut TcpStream, html: &str) -> anyhow::Result<()> {
    write_response(stream, 200, "text/html; charset=utf-8", html.as_bytes())
}

fn write_text(stream: &mut TcpStream, content_type: &str, body: &str) -> anyhow::Result<()> {
    write_response(stream, 200, content_type, body.as_bytes())
}

fn parse_id_download_query(query: &str) -> IdDownloadRequest {
    let mut request = IdGenerateRequest {
        count: None,
        region: None,
        birth: None,
        min_birth: None,
        max_birth: None,
        gender: None,
    };
    let mut format = None;

    for pair in query.split('&').filter(|item| !item.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = url_decode(value);
        match key {
            "count" => request.count = value.parse().ok(),
            "region" => request.region = non_empty(value),
            "birth" => request.birth = non_empty(value),
            "min_birth" => request.min_birth = non_empty(value),
            "max_birth" => request.max_birth = non_empty(value),
            "gender" => request.gender = serde_json::from_str(&format!("\"{}\"", value)).ok(),
            "format" => format = serde_json::from_str(&format!("\"{}\"", value)).ok(),
            _ => {}
        }
    }

    IdDownloadRequest {
        params: request,
        format,
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    output.push(byte);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn write_json<T: Serialize>(stream: &mut TcpStream, status: u16, value: &T) -> anyhow::Result<()> {
    let body = serde_json::to_vec(value)?;
    write_response(stream, status, "application/json; charset=utf-8", &body)
}

fn write_download_header(stream: &mut TcpStream, format: OutputType) -> anyhow::Result<()> {
    let (content_type, ext) = match format {
        OutputType::Text => ("text/plain; charset=utf-8", "txt"),
        OutputType::Csv => ("text/csv; charset=utf-8", "csv"),
        OutputType::Json => ("application/json; charset=utf-8", "json"),
        OutputType::Excel => (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        ),
    };
    let filename = format!("idgen.{}", ext);
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nConnection: close\r\n\r\n",
        content_type,
        filename
    )?;
    stream.flush()?;
    Ok(())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> anyhow::Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        status_text,
        content_type,
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}
