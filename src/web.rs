use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::Serialize;

use crate::commands::idgen::{generate_ids, IdGenerateRequest};
use crate::commands::portscan::{scan_ports, PortScanRequest};

const INDEX_HTML: &str = include_str!("../static/ocr.html");

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
    let path = parts[1];
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

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => write_html(&mut stream, INDEX_HTML),
        ("GET", "/api/health") => {
            write_json(&mut stream, 200, &serde_json::json!({ "status": "ok" }))
        }
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

fn write_json<T: Serialize>(stream: &mut TcpStream, status: u16, value: &T) -> anyhow::Result<()> {
    let body = serde_json::to_vec(value)?;
    write_response(stream, status, "application/json; charset=utf-8", &body)
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
