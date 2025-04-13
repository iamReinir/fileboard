use askama::Template;

#[derive(Template)]
#[template(path = "upload.js",escape = "none")]
pub struct JsTemplate<'a> {
    pub host: &'a str,
}

#[derive(Template)]
#[template(path = "styles.css",escape = "none")]
pub struct CssTemplate {
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub path: &'a str,
    pub items: Vec<FileItem>,
    pub script: &'a str,
    pub style: &'a str,
}

pub struct FileItem {
    pub name: String,
    pub link: String,
    pub date: String,
    pub size: String
}