use axum::{
    body::Body,
    http::{HeaderValue, Request, Response},
    middleware::Next,
};
use tracing::Instrument;
use uuid::Uuid;

pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response<Body> {
    let request_id = Uuid::now_v7().to_string();
    req.extensions_mut().insert(request_id.clone());

    let span = tracing::info_span!("request", request_id = %request_id);
    let mut response = next.run(req).instrument(span).await;

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }
    response
}
