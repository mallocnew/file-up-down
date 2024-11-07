use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use futures::StreamExt;
use std::{io::Write, time::{SystemTime, UNIX_EPOCH}};

#[get("/download")]
async fn download(req: HttpRequest) -> HttpResponse {
    let query = req.query_string();

    // 解析查询参数
    let mut params = query.split('&');

    // 查找名为 "filename" 的参数
    let filename_param = params.find(|param| param.starts_with("filename="));

    // 如果找到了 "filename" 参数，提取文件名
    let filename = if let Some(param) = filename_param {
        let filename = param.split('=').nth(1).unwrap();
        filename.to_string()
    } else {
        return HttpResponse::BadRequest().body("Missing filename parameter");
    };
    let filename = "./data/".to_owned() + filename.as_str();
    let file = fs::NamedFile::open(filename).unwrap();
    file.into_response(&req)
}

// upload file from client to server by form data
#[post("/upload")]
async fn upload(req: HttpRequest, bytes: web::Payload) -> Result<HttpResponse, Error> {
    let mut multipart = Multipart::new(req.headers(), bytes);

    let name_id = human_id::id("", true);
    let name_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Give me your time machine")
        .as_micros();

    let uploaded_name = format!("{}-{:?}", name_id, name_time);
    let mut upload_file = std::fs::File::create(format!("data/{}", uploaded_name))?;

    while let Some(chunk) = multipart.next().await {
        let mut chunk = chunk?;
        for chunk_content in chunk.next().await {
            let content = chunk_content.ok().unwrap_or_default();
            upload_file.write(&content)?;
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(uploaded_name))
}

#[get("/")]
async fn index() -> HttpResponse {
    // get file list of ./data/ and return html with download url
    let mut html = String::from("<html><body><ul>");
    let files = std::fs::read_dir("./data/").unwrap();
    for file in files {
        let file = file.unwrap();
        let filename = file.file_name().into_string().unwrap();
        html.push_str(&format!(
            "<li><a href=\"/download?filename={}\">{}</a></li>",
            filename, filename
        ));
    }
    html.push_str("</ul></body></html>");
    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set log level
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| App::new().service(index).service(download).service(upload))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}