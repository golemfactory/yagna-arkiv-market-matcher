use crate::state::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;

pub async fn clean_old_offers(data: web::Data<AppState>) {
    let mut lock = data.lock.lock().await;
    let now = Utc::now();
    lock.offer_map.retain(|_id, offer_obj| {
        offer_obj.offer.expiration > (now - chrono::Duration::minutes(60))
    });
}

pub async fn delete_all_offers(data: web::Data<AppState>) -> HttpResponse {
    let mut lock = data.lock.lock().await;
    lock.offer_map.clear();
    HttpResponse::Ok().body("All offers deleted successfully")
}
