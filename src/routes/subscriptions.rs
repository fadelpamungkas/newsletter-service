use actix_web::{HttpResponse, Responder, web};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
