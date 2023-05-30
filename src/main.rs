use actix_middleware_ed25519_authentication::AuthenticatorBuilder;
use actix_web::{
    body::BoxBody, http::header::ContentType, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

#[derive(Deserialize)]
struct Interaction {
    // id: String,
    r#type: InteractionType,
}

#[derive(Deserialize_repr, PartialEq)]
#[repr(u8)]
enum InteractionType {
    Ping = 1,
    ApplicationCommand,
    MessageComponent,
    ApplicationCommandAutocomplete,
    ModalSubmit,
}

#[derive(Serialize)]
struct InteractionResponse {
    r#type: InteractionResponseType,
}

#[derive(Serialize)]
enum InteractionResponseType {
    Pong = 1,
}

impl Responder for InteractionResponse {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[post("/")]
async fn hello(payload: web::Json<Interaction>) -> impl Responder {
    if payload.r#type == InteractionType::Ping {
        InteractionResponse {
            r#type: InteractionResponseType::Pong,
        }
    } else {
        InteractionResponse {
            r#type: InteractionResponseType::Pong,
        }
    }
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let public_key = dotenvy::var("DISCORD_PUBLIC_KEY").expect("Expect public key");
        App::new()
            .wrap(AuthenticatorBuilder::new().public_key(&public_key).build())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
