
#[cfg(feature = "server")]
use actix_files::Files;
#[cfg(feature = "server")]
use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};

// main.rs
#[cfg(feature = "server")]
use dotenv::dotenv;
use wasm_bindgen::prelude::wasm_bindgen;

// #[cfg(feature = "server")]
// async fn index() -> Result<HttpResponse> {
//     let html = include_str!("../static/index.html");
//     Ok(HttpResponse::Ok().content_type("text/html").body(html))
// }
// #[cfg(feature = "server")]
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     env_logger::init();
// 
//     HttpServer::new(|| {
//         App::new()
//             .wrap(Logger::default())
//             .route("/", web::get().to(index))
//             .service(Files::new("/static", "./static").show_files_listing())
//             .service(Files::new("/pkg", "./pkg").show_files_listing())
//     })
//         .bind("127.0.0.1:8080")?
//         .run()
//         .await
// }

// Initialize WASM module
#[cfg(feature = "wasm")]
fn main() {
    println!("Hello, world!");
}

