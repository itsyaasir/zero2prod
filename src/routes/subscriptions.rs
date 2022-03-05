#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

// Subscribe handler

use actix_web::{HttpResponse, web};

pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
