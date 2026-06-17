use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::{Deserialize, Serialize};

use crate::commands::idgen::{
    generate_ids, validate_id_download_request, write_generated_ids, IdGenerateRequest, OutputType,
};
use crate::commands::jsonfmt::{format_json_text, MAX_INDENT};
use crate::commands::portscan::{scan_ports, PortScanRequest};
use crate::utils::areas::{all_cities, all_provinces, all_regions, Area, City, Province};

const INDEX_HTML: &str = include_str!("../static/index.html");
const IDGEN_HTML: &str = include_str!("../static/idgen.html");
const PORT_SCAN_HTML: &str = include_str!("../static/port-scan.html");
const JSONFMT_HTML: &str = include_str!("../static/jsonfmt.html");
const IDGEN_JS: &str = include_str!("../static/idgen.js");
const PORT_SCAN_JS: &str = include_str!("../static/port-scan.js");
const JSONFMT_JS: &str = include_str!("../static/jsonfmt.js");
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
        _ => write_json(
            &mut stream,
            404,
            &ErrorResponse {
                error: "not found".to_string(),
            },
        ),
    }
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
