use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

static HTML: &str = include_str!("../studio/index.html");

pub fn start_studio() -> Result<(), String> {
    let addr = "127.0.0.1:7777";
    let listener = TcpListener::bind(addr)
        .map_err(|e| format!("Failed to start Mako Studio server: {}", e))?;

    println!();
    println!("  🌊  Mako Studio");
    println!("      http://localhost:7777");
    println!();
    println!("  Press Ctrl+C to stop.");
    println!();

    // Open browser automatically on Windows
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "http://localhost:7777"])
        .spawn();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| handle(s));
            }
            Err(_) => {}
        }
    }

    Ok(())
}

fn handle(stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    let mut reader = BufReader::new(&stream);

    // ── Read request line ─────────────────────────────────────
    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() {
        return;
    }
    let req_line = req_line.trim_end().to_string();

    // ── Read headers ──────────────────────────────────────────
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            content_length = lower
                .split(':')
                .nth(1)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }

    // ── Read body ─────────────────────────────────────────────
    let mut body = String::new();
    if content_length > 0 {
        let mut buf = vec![0u8; content_length.min(1_000_000)];
        let _ = reader.read_exact(&mut buf);
        body = String::from_utf8_lossy(&buf).to_string();
    }

    // ── Route ─────────────────────────────────────────────────
    let response = if req_line.starts_with("OPTIONS") {
        cors_preflight()
    } else if req_line.starts_with("GET /favicon") {
        "HTTP/1.1 204 No Content\r\n\r\n".to_string()
    } else if req_line.starts_with("GET /") {
        serve_html()
    } else if req_line.starts_with("POST /run") {
        run_code(&body)
    } else {
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
    };

    let mut writer = &stream;
    let _ = writer.write_all(response.as_bytes());
}

fn cors_headers() -> &'static str {
    "Access-Control-Allow-Origin: *\r\n\
     Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
     Access-Control-Allow-Headers: Content-Type\r\n"
}

fn cors_preflight() -> String {
    format!(
        "HTTP/1.1 204 No Content\r\n{}Content-Length: 0\r\n\r\n",
        cors_headers()
    )
}

fn serve_html() -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n{}Content-Length: {}\r\n\r\n{}",
        cors_headers(),
        HTML.len(),
        HTML
    )
}

fn run_code(code: &str) -> String {
    // Write code to a temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("mako_studio_run.mako");

    if let Err(e) = std::fs::write(&tmp_path, code) {
        return json_response(false, "", &format!("Failed to write temp file: {}", e));
    }

    // Run via the current executable
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("mako"));

    let output = std::process::Command::new(&exe)
        .arg("run")
        .arg(&tmp_path)
        .output();

    let _ = std::fs::remove_file(&tmp_path);

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let success = out.status.success();
            json_response(success, &stdout, &stderr)
        }
        Err(e) => json_response(false, "", &format!("Failed to execute Mako: {}", e)),
    }
}

fn json_response(success: bool, stdout: &str, stderr: &str) -> String {
    let json = format!(
        r#"{{"success":{},"stdout":{},"stderr":{}}}"#,
        success,
        json_string(stdout),
        json_string(stderr)
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\n\r\n{}",
        cors_headers(),
        json.len(),
        json
    )
}

fn json_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}
