use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use middleware::Ed25519Authentication;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

mod middleware;

#[derive(Deserialize, Debug)]
struct InteractionRequest {
    r#type: InteractionRequestType,
    member: Option<Member>,
}

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
enum InteractionRequestType {
    Ping = 1,
    ApplicationCommand,
}

#[derive(Deserialize, Debug, Default)]
struct Member {
    user: Option<User>,
}

#[derive(Deserialize, Debug, Default)]
struct User {
    username: String,
}

#[derive(Serialize, Default)]
struct InteractionResponse {
    r#type: InteractionResponseType,
    data: Option<InteractionResponseData>,
}

#[derive(Serialize_repr, Default)]
#[repr(u8)]
enum InteractionResponseType {
    #[default]
    Pong = 1,
    ChannelMessageWithSource = 4,
}

#[derive(Serialize)]
struct InteractionResponseData {
    content: Option<String>,
}

#[post("/interactions")]
async fn interactions(body: web::Json<InteractionRequest>) -> impl Responder {
    match body.r#type {
        InteractionRequestType::Ping => HttpResponse::Ok().json(InteractionResponse {
            r#type: InteractionResponseType::Pong,
            data: None,
        }),
        InteractionRequestType::ApplicationCommand => {
            let username = match &body.member {
                Some(member) => match &member.user {
                    Some(user) => &user.username,
                    None => "",
                },
                None => "",
            };
            HttpResponse::Ok().json(InteractionResponse {
                r#type: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!("Hello, {username}")),
                }),
            })
        }
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
