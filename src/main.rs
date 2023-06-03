use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use middleware::Ed25519Authentication;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

mod middleware;

#[derive(Deserialize)]
struct InteractionRequest {
    r#type: InteractionRequestType,
}

#[derive(Deserialize_repr, PartialEq)]
#[repr(u8)]
enum InteractionRequestType {
    Ping = 1,
}

#[derive(Serialize)]
struct InteractionResponse {
    r#type: InteractionResponseType,
}

#[derive(Serialize_repr)]
#[repr(u8)]
enum InteractionResponseType {
    Pong = 1,
}

#[post("/interactions")]
async fn interactions(body: web::Json<InteractionRequest>) -> impl Responder {
    if body.r#type == InteractionRequestType::Ping {
        HttpResponse::Ok().json(InteractionResponse {
            r#type: InteractionResponseType::Pong,
        })
    } else {
        HttpResponse::BadRequest().body("Invalid interaction type")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Ed25519Authentication::new())
            .service(interactions)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
