use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    name: String,
    email: String,
}

pub async fn subscribe(_form: web::Form<SubscriptionFormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
