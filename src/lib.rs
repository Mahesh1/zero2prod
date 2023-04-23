// use actix_web::dev::Server;
// use actix_web::{web, App, HttpResponse, HttpServer, Responder};
// use std::net::TcpListener;
// use serde::Deserialize;

pub mod configuration;
pub mod routes;
pub mod startup;

// async fn greet(req: HttpRequest) -> impl Responder {
//     let name = req.match_info().get("name").unwrap_or("World");
//     format!("Hello {:?}", name)
// }

// async fn health_check() -> impl Responder {
//     HttpResponse::Ok().finish()
//     // format!("byeeee {:?}", "now")
// }
//some comment
