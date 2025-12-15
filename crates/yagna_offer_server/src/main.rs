pub mod model;
pub mod offers;
pub mod state;
pub mod rest;

use crate::model::offer::attributes::OfferFlatAttributes;
use crate::model::offer::base::GolemBaseOffer;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use structopt::StructOpt;
pub use ya_client_model::NodeId;
use crate::offers::download_initial_offers;
use crate::state::{AppState, Demands, OfferObj, Offers};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilterAttributes {
    exe_name: Option<String>,
    cpu_threads_min: Option<u32>,
    cpu_threads_max: Option<u32>,
    provider_group_min: Option<u32>,
    provider_group_max: Option<u32>,
    id_group_min: Option<u32>,
    id_group_max: Option<u32>,
    node_id: Option<NodeId>,
    subnet: Option<String>,
    cpu_architecture: Option<String>,
}

#[test]
fn test_filter_attributes() {
    use crate::model::offer::attributes::OfferFlatAttributes;

    let offer = "{\"id\":\"00082a0389918034011dbcc885bd3da086eaaa66dceef7e6784386842571854d\",\"properties\":{\"golem\":{\"com\":{\"payment\":{\"debit-notes\":{\"accept-timeout?\":240},\"platform\":{\"erc20-polygon-glm\":{\"address\":\"0xa3bde9e2ef344407afdc931c97fd33d506ec6545\"}},\"protocol\":{\"version\":3}},\"pricing\":{\"model\":{\"@tag\":\"linear\",\"linear\":{\"coeffs\":[1e-9,0.0,0.0]}}},\"scheme\":{\"@tag\":\"payu\",\"payu\":{\"debit-note\":{\"interval-sec?\":120},\"payment-timeout-sec?\":120}},\"usage\":{\"vector\":[\"golem.usage.cpu_sec\",\"golem.usage.duration_sec\"]}},\"inf\":{\"cpu\":{\"architecture\":\"x86_64\",\"cores\":14,\"threads\":1},\"mem\":{\"gib\":42.79507473111153},\"storage\":{\"gib\":3257.801303100586}},\"node\":{\"debug\":{\"subnet\":\"public\"},\"id\":{\"name\":\"brick-54\"},\"net\":{\"is-public\":false}},\"runtime\":{\"name\":\"ya-runtime-cruncher\",\"version\":\"0.1.0\"},\"srv\":{\"caps\":{\"multi-activity\":true,\"payload-manifest\":false}}}},\"constraints\":\"(&\\n  (golem.srv.comp.expiration>1765401640654)\\n  (golem.node.debug.subnet=public)\\n)\",\"providerId\":\"0xa3bde9e2ef344407afdc931c97fd33d506ec6545\",\"expiration\":\"2025-12-11T12:20:45.222028719Z\",\"timestamp\":\"2025-12-11T11:20:45.222028719Z\"}";

    let gbo = serde_json::from_str::<GolemBaseOffer>(offer).unwrap();
    let attributes = OfferFlatAttributes::from_gbo(&gbo);
    println!("Attributes: {:?}", attributes);
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

async fn get_if_available(data: web::Data<AppState>, item: String) -> impl Responder {
    let decode = serde_json::from_str::<FilterAttributes>(&item);
    let filer = match decode {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding filter: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid filter format {}", e));
        }
    };
    let mut lock = data.lock.lock().await;
    for (_id, offer_obj) in lock.offer_map.iter_mut() {
        if let Some(filter_exe_name) = &filer.exe_name {
            if &offer_obj.attributes.exe_name != filter_exe_name {
                continue;
            }
        }
        if let Some(filter_cpu_threads_min) = filer.cpu_threads_min {
            if offer_obj.attributes.cpu_threads < filter_cpu_threads_min {
                continue;
            }
        }
        if let Some(filter_cpu_threads_max) = filer.cpu_threads_max {
            if offer_obj.attributes.cpu_threads > filter_cpu_threads_max {
                continue;
            }
        }
        if let Some(filter_node_id) = &filer.node_id {
            if &offer_obj.offer.provider_id != filter_node_id {
                continue;
            }
        }
        if let Some(filter_subnet) = &filer.subnet {
            if &offer_obj.attributes.subnet != filter_subnet {
                continue;
            }
        }
        if let Some(filter_provider_group_min) = filer.provider_group_min {
            if offer_obj.attributes.node_id_group < filter_provider_group_min {
                continue;
            }
        }
        if let Some(filter_provider_group_max) = filer.provider_group_max {
            if offer_obj.attributes.node_id_group > filter_provider_group_max {
                continue;
            }
        }
        if let Some(filter_id_group_min) = filer.id_group_min {
            if offer_obj.attributes.offer_id_group < filter_id_group_min {
                continue;
            }
        }
        if let Some(filter_id_group_max) = filer.id_group_max {
            if offer_obj.attributes.offer_id_group > filter_id_group_max {
                continue;
            }
        }
        if let Some(filter_cpu_architecture) = &filer.cpu_architecture {
            if &offer_obj.attributes.cpu_architecture != filter_cpu_architecture {
                continue;
            }
        }

        if offer_obj.available {
            offer_obj.available = false;
            let offer = &offer_obj.offer;
            return HttpResponse::Ok().json(offer);
        }
    }
    HttpResponse::Ok().body("No available offers")
}

async fn list_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&OfferObj> = lock.offer_map.values().collect();
    HttpResponse::Ok().json(offers)
}

async fn list_taken_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&OfferObj> = lock
        .offer_map
        .values()
        .filter(|offer_obj| !offer_obj.available)
        .collect();
    HttpResponse::Ok().json(offers)
}

async fn list_available_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&OfferObj> = lock
        .offer_map
        .values()
        .filter(|offer_obj| offer_obj.available)
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
    let attributes = OfferFlatAttributes::from_gbo(&offer);
    lock.offer_map.insert(
        offer.id.clone(),
        OfferObj {
            offer,
            pushed_at: Utc::now(),
            available: true,
            attributes,
        },
    );
    HttpResponse::Ok().body("Offer added to the queue")
}

async fn clean_old_offers(data: web::Data<AppState>) {
    let mut lock = data.lock.lock().await;
    let now = Utc::now();
    lock.offer_map.retain(|_id, offer_obj| {
        offer_obj.offer.expiration > (now - chrono::Duration::minutes(60))
    });
}

fn clean_old_offers_periodically(data: web::Data<AppState>) {
    let interval = tokio::time::Duration::from_secs(60);
    let data_clone = data.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            clean_old_offers(data_clone.clone()).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();
    let args = CliOptions::from_args();
    // Load the queue from file or create a new one

    let app_state = AppState {
        lock: Arc::new(tokio::sync::Mutex::new(Offers::default())),
        demands: Arc::new(tokio::sync::Mutex::new(Demands::default())),
    };
    log::info!("Downloading initial offers...");
    let _ = download_initial_offers(web::Data::new(app_state.clone())).await;

    clean_old_offers_periodically(web::Data::new(app_state.clone()));

    log::info!(
        "Starting Offer Server at {}:{}",
        &args.http_addr,
        &args.http_port
    );
    HttpServer::new(move || {
        //let auth = HttpAuthentication::with_fn(validator);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .route("/provider/offer/new", web::post().to(push_offer))
            .route("/offers/list", web::get().to(list_offers))
            .route("/offers/list/taken", web::get().to(list_taken_offers))
            .route(
                "/offers/list/available",
                web::get().to(list_available_offers),
            )
            .route("/offer/take", web::post().to(get_if_available))
            .route(
                "/version",
                web::get().to(|| async { HttpResponse::Ok().body(env!("CARGO_PKG_VERSION")) }),
            )
            .route("/requestor/demand/new", web::post().to(|| async {
                HttpResponse::Ok().body("Not implemented")
            }))
    })
    .bind(format!("{}:{}", args.http_addr, args.http_port))?
    .workers(4)
    .run()
    .await
}
