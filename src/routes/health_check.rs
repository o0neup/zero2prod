use actix_web::HttpResponse;
use uuid::Uuid;

pub async fn health_check() -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Doing Health Check.",
        %request_id
    );
    let _request_span_guard = request_span.enter();
    tracing::info!("request_id {} - Received healthcheck request!", request_id);
    HttpResponse::Ok().finish()
}
