
use calamine::{Data, DataType, Error, RangeDeserializerBuilder, Reader, Xls, open_workbook};
use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};

mod lib;
mod secrets;

async fn index() -> Result<HttpResponse> {
    let html = include_str!("../static/index.html");
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .service(Files::new("/static", "./static").show_files_listing())
            .service(Files::new("/pkg", "./pkg").show_files_listing())
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
