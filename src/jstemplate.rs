use askama::Template;

#[derive(Template)]
#[template(path = "upload.js",escape = "none")]
pub struct JsTemplate<'a> {
    pub host: &'a str,
}