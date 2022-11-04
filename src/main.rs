use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use lapin::{
    options::*,
    types::FieldTable,
    Connection,
    ConnectionProperties
};
use std::{
    thread,
    collections::HashMap,
    sync::{Arc, Mutex}
};


mod websocket;
mod datasource;
use websocket::Ws;
use datasource::Source;


async fn index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<Mutex<HashMap<String, String>>>>
) -> Result<HttpResponse, Error> {
    let resp = ws::start(Ws {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Set env to force logging
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let data: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let receiver_data = data.clone();

    tokio::spawn(async move {
        let mut source = Source::new(receiver_data).await.unwrap();
        source.add_queue("data.protocol", "topic_queue", "topic_logs").await.unwrap();
        source.get_data()
    });


    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(data.clone()))
        .route("/ws/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}