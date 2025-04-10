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
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",  // JSON MIME type (structured text)
        Some("toml") => "text/plain",         // TOML MIME type (text format)
        Some("pdf") => "application/pdf", // PDF
        Some("epub") => "application/epub+zip", // epub
        Some("md") => "text/markdown",       // Markdown MIME type (text format)
        Some("txt") => "text/plain",         // Plain text
        Some("png") => "image/png",          // Image MIME type
        Some("jpg") | Some("jpeg") => "image/jpeg", // Image MIME type
        Some("gif") => "image/gif",          // Image MIME type
        Some("svg") => "image/svg+xml",      // Image MIME type
        Some("bmp") => "image/bmp",          // Image MIME type
        Some("tiff") | Some("tif") => "image/tiff", // Image MIME type
        Some("mp3") => "audio/mpeg",         // Audio MIME type
        Some("wav") => "audio/wav",          // Audio MIME type
        Some("ogg") => "audio/ogg",          // Audio MIME type
        Some("flac") => "audio/flac",        // Audio MIME type
        Some("mp4") => "video/mp4",          // Video MIME type
        Some("webm") => "video/webm",        // Video MIME type
        Some("avi") => "video/x-msvideo",    // Video MIME type
        Some("mov") => "video/quicktime",    // Video MIME type
        Some("mkv") => "video/x-matroska",   // Video MIME type
        _ => "application/octet-stream",     // Default (download)
    }
}

async fn directory_page(relative_path: &str, wwwroot: &str) -> Result<String, String> {
    let path = Path::new(wwwroot).join(relative_path);
    let host = config::get().unwrap().server.host;
    match fs::read_dir(&path).await {
        Ok(mut entries) => {
            eprintln!("PATH : {}", relative_path);
            let mut html = String::from("<html><meta charset=\"UTF-8\">");
            html.push_str(
                format!("<header><title>Fileboard - {}</title></header>",
                relative_path).as_str());
            html.push_str("<body>");
            html.push_str(
                format!("<body><h1>Directory listing for {} </h1><pre><strong>",
                relative_path).as_str());
            html.push_str("Name\t\t\t\t\tModified\t\t\t\t\tSize\t\t");
            html.push_str("<button id=\"uploadBtn\">Upload</button><hr>");
            if !relative_path.is_empty() {
                html.push_str("<a href=../>../</a><br>");
            }
            while let Ok(Some(entry)) = entries.next_entry().await {
                let mut file_name = entry.file_name().to_string_lossy().to_string();
                let encoded_file_name = encode(&file_name);
                let file_path = path.join(&file_name);

                // Get metadata
                let metadata = (fs::metadata(&file_path).await).ok();

                // Create the link
                let mut href = String::from("");
                /*
                if let Some(parent) = relative_path.parent() {
                    href.push_str(&parent.to_string_lossy());
                    href.push('/');
                }
                */
                href.push_str(&encoded_file_name);

                
                // datetime
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
                
                // long-ass span
                html.push_str("<span style=\"display:inline-block; width: 32ch;"); 
                html.push_str("overflow: hidden; text-overflow: ellipsis; white-space: nowrap;\">");
                html.push_str(&format!(
                    "<a href=\"{}\">{}</a></span>\t{}\t\t\t\t{}<br>",
                    href, file_name,
                    datetime,
                    size
                ));
            }
            let js_code = JsTemplate {
                host: format!("{}/{}", host, relative_path).as_str()
            }.render().unwrap();
            html.push_str("<hr></pre></body>");
            html.push_str(&format!("<script>{}</script>", js_code));
            html.push_str("</html>");
            Ok(html)
        }
        Err(_) => Err("Failed to read directory".to_string()),
    }
}

fn format_size(bytes: u64) -> String {
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