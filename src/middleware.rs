use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::HeaderValue,
    Error, HttpMessage,
};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use futures_util::{future::LocalBoxFuture, FutureExt};

#[derive(Clone, Debug)]
pub struct Ed25519Authentication {
    public_key: String,
}

impl Ed25519Authentication {
    pub fn new() -> Self {
        Self {
            public_key: dotenvy::var("DISCORD_PUBLIC_KEY").expect("DISCORD_PUBLIC_KEY not set"),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Ed25519Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Ed25519AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Ed25519AuthenticationMiddleware {
            service: Rc::new(service),
            data: Rc::new(self.clone()),
        }))
    }
}

pub struct Ed25519AuthenticationMiddleware<S> {
    service: Rc<S>,
    data: Rc<Ed25519Authentication>,
}

impl<S, B> Service<ServiceRequest> for Ed25519AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let data = self.data.clone();
        let srv = self.service.clone();
        async move {
            match authenticate_request(&mut req, &*data).await {
                Ok(_) => srv.call(req).await,
                Err(e) => Err(ErrorUnauthorized(e)),
            }
        }
        .boxed_local()
    }
}

async fn authenticate_request(
    req: &mut ServiceRequest,
    data: &Ed25519Authentication,
) -> Result<(), String> {
    let empty_header = HeaderValue::from_static("");
    let body = req.extract::<String>().await.unwrap();
    let signature = &hex::decode(
        req.headers()
            .get("X-Signature-Ed25519")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let signature = match Signature::from_bytes(signature) {
        Ok(signature) => signature,
        Err(e) => return Err(format!("Invalid signature: {}", e)),
    };
    let timestamp = req
        .headers()
        .get("X-Signature-Timestamp")
        .unwrap_or(&empty_header)
        .to_str()
        .unwrap();
    let message = timestamp.to_string() + &body;
    let message = message.as_bytes();
    let Ed25519Authentication { public_key } = data;
    let public_key = &hex::decode(public_key).unwrap();
    let public_key = match PublicKey::from_bytes(public_key) {
        Ok(public_key) => public_key,
        Err(e) => return Err(format!("Invalid public key: {}", e)),
    };
    match public_key.verify(message, &signature) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Invalid signature: {}", e)),
    }
}
