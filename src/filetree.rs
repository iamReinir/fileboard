use std::path::Path;
use std::time::SystemTime;
use askama::Template;
use chrono::format;
use chrono::DateTime;
use chrono::Local;
use hyper::body::Bytes;
use hyper::Response;
use hyper::header;
use hyper::StatusCode;
use http_body_util::combinators::BoxBody;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use urlencoding::encode;


use crate::config;
use crate::jstemplate::CssTemplate;
use crate::jstemplate::FileItem;
use crate::jstemplate::IndexTemplate;
use crate::jstemplate::JsTemplate;
use crate::config::{full, empty};

pub async fn serve_files(path: &str, wwwroot: &str) 
    -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
{
    let full_path = Path::new(wwwroot).join(path);
    let metadata = match fs::metadata(&full_path).await {
        Ok(meta) => meta,
        Err(_) => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            return Ok(not_found)
        }
    };

    if metadata.is_dir() {

        // Check for index.html. Serve that intead
        let index_path = full_path.join("index.html");
        if index_path.exists() && index_path.is_file() {
            match tokio::fs::read(index_path).await {
                Ok(contents) => {
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(full(contents))
                        .unwrap());
                }
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(empty())
                        .unwrap());
                }
            }
        }

        // Else, generate own index
        match directory_page(path, wwwroot).await {
            Ok(html) => return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(full(html))
                .unwrap()),
            Err(_) => return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty())
                .unwrap())
        }
    }
    
    let mime = guess_mime(path);
    match File::open(full_path).await {
        Ok(mut file) => {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await.unwrap();
            let bytes = Bytes::from(contents);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(full(bytes))
                .unwrap())
        }
        Err(_) => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::IM_A_TEAPOT;
            Ok(not_found)
        }
    }
}


fn guess_mime(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("toml") => "text/plain; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",

        Some("pdf") => "application/pdf",
        Some("epub") => "application/epub+zip",

        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("tiff") | Some("tif") => "image/tiff",

        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("flac") => "audio/flac",

        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",

        _ => "application/octet-stream",
    }
}

async fn directory_page(relative_path: &str, wwwroot: &str) -> Result<String, String> {
    let path = Path::new(wwwroot).join(relative_path);
    let host = config::get().unwrap().server.host;
    let css = CssTemplate{}.render().unwrap_or_default();
    match fs::read_dir(&path).await {
        Ok(mut entries) => {
            let mut datalist = Vec::<FileItem>::new();
            if !relative_path.is_empty() {
                datalist.push(FileItem { 
                    name: "../".to_string(),
                    link: "../".to_string(),
                    date: String::new(),
                    size: String::new() 
                });
            }
            while let Ok(Some(entry)) = entries.next_entry().await {
                // Get data
                let mut file_name = entry.file_name().to_string_lossy().to_string();
                let encoded_file_name = encode(&file_name);
                let file_path = path.join(&file_name);
                let metadata = (fs::metadata(&file_path).await).ok();
                let mut href = String::from("");
                href.push_str(&encoded_file_name);
                let (datetime, mut size) = metadata.as_ref()
                    .map(|meta| 
                        (meta.modified().unwrap_or(SystemTime::now()),
                        format_size(meta.len())))
                    .map(|(time, size)| 
                        (DateTime::<Local>::from(time).format("%Y-%m-%d %H:%M:%S").to_string(),
                        size))
                    .unwrap_or((String::new(), String::new()));
                if metadata.map(|meta| meta.is_dir()).unwrap_or(false) {
                    href.push('/');
                    file_name.push('/');
                    size = "-".to_string();
                }
                
                datalist.push(FileItem { 
                    name:file_name,
                    link: href, 
                    date: datetime,
                    size 
                });
            }
            let js_code = JsTemplate {
                host: format!("{}/{}", host, relative_path).as_str()
            }.render().unwrap();
            let html = IndexTemplate {
                items: datalist,
                path: relative_path,
                script: &js_code,
                style: &css
            }.render().unwrap_or_default();
            Ok(html)
        }
        Err(_) => Err("Failed to read directory".to_string()),
    }
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    // Keep 1 decimal place for non-integer values
    if size.fract() == 0.0 {
        format!("{:.0} {}", size, UNITS[unit])
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}