use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;
use structopt::StructOpt;
pub use ya_client_model::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfferObj {
    offer: GolemBaseOffer,
    pushed_at: DateTime<Utc>,
    available: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Offers {
    offer_map: BTreeMap<String, OfferObj>,
}

#[derive(Clone)]
struct AppState {
    lock: Arc<tokio::sync::Mutex<Offers>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GolemBaseOffer {
    pub id: String,
    pub properties: Value,
    pub constraints: String,
    #[serde(rename = "providerId")]
    pub provider_id: NodeId,
    pub expiration: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, StructOpt, Clone)]
pub struct CliOptions {
    #[structopt(
        long = "http-port",
        help = "Port number of the server",
        default_value = "8080"
    )]
    pub http_port: u16,

    #[structopt(
        long = "http-addr",
        help = "Bind address of the server",
        default_value = "127.0.0.1"
    )]
    pub http_addr: String,

    #[structopt(
        long = "file-name",
        help = "Name of the file to store the queue",
        default_value = "data.json"
    )]
    pub file_name: String,
}

async fn list_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&GolemBaseOffer> = lock
        .offer_map
        .values()
        .map(|offer_obj| &offer_obj.offer)
        .collect();
    HttpResponse::Ok().json(offers)
}

async fn push_offer(data: web::Data<AppState>, item: String) -> impl Responder {
    let decode = serde_json::from_str::<GolemBaseOffer>(&item);
    let offer = match decode {
        Ok(offer) => offer,
        Err(e) => {
            log::error!("Error decoding offer: {}", e);
            return HttpResponse::BadRequest().body("Invalid offer format");
        }
    };

    let mut lock = data.lock.lock().await;
    if lock.offer_map.contains_key(&offer.id) {
        let id = &offer.id;
        return HttpResponse::Ok().body(format!("Offer {id} already registered"));
    }
    lock.offer_map.insert(
        offer.id.clone(),
        OfferObj {
            offer,
            pushed_at: Utc::now(),
            available: true,
        },
    );
    HttpResponse::Ok().body("Offer added to the queue")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();
    let args = CliOptions::from_args();
    // Load the queue from file or create a new one

    let app_state = AppState {
        lock: Arc::new(tokio::sync::Mutex::new(Offers::default())),
    };

    HttpServer::new(move || {
        //let auth = HttpAuthentication::with_fn(validator);

        log::info!("");

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .route("/provider/offer/new", web::post().to(push_offer))
            .route("/offers/list", web::get().to(list_offers))
    })
    .bind(format!("{}:{}", args.http_addr, args.http_port))?
    .workers(1)
    .run()
    .await
}
