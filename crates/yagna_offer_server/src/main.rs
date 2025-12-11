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


struct FilterAttributes {
    execution_name: Option<String>,
    subnet: Option<String>,
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


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Properties {
    pub golem: GolemProperties,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GolemProperties {
    pub com: Com,
    pub inf: Inf,
    pub node: Node,
    pub runtime: Runtime,
    pub srv: Srv,
}

// --- Communication (com) ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Com {
    pub payment: Payment,
    pub pricing: Pricing,
    pub scheme: Scheme,
    pub usage: Usage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payment {
    #[serde(rename = "debit-notes")]
    pub debit_notes: DebitNotes,
    pub platform: Platform,
    pub protocol: Protocol,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DebitNotes {
    // The key in JSON literally contains the question mark
    #[serde(rename = "accept-timeout?")]
    pub accept_timeout: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Platform {
    #[serde(rename = "erc20-polygon-glm")]
    pub erc20_polygon_glm: Erc20PolygonGlm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Erc20PolygonGlm {
    pub address: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Protocol {
    pub version: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    pub model: PricingModel,
}

// The JSON uses "@tag" to determine which variant this is
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@tag")]
pub enum PricingModel {
    #[serde(rename = "linear")]
    Linear {
        linear: LinearPricing,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearPricing {
    pub coeffs: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@tag")]
pub enum Scheme {
    #[serde(rename = "payu")]
    Payu {
        payu: PayuScheme,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayuScheme {
    #[serde(rename = "debit-note")]
    pub debit_note: PayuDebitNote,
    #[serde(rename = "payment-timeout-sec?")]
    pub payment_timeout_sec: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayuDebitNote {
    #[serde(rename = "interval-sec?")]
    pub interval_sec: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub vector: Vec<String>,
}

// --- Infrastructure (inf) ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inf {
    pub cpu: Cpu,
    pub mem: Mem,
    pub storage: Storage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cpu {
    pub architecture: String,
    pub cores: u32,
    pub threads: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mem {
    pub gib: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Storage {
    pub gib: f64,
}

// --- Node (node) ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub debug: NodeDebug,
    pub id: NodeName,
    pub net: NodeNet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeDebug {
    pub subnet: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeName {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeNet {
    #[serde(rename = "is-public")]
    pub is_public: bool,
}

// --- Runtime (runtime) ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Runtime {
    pub name: String,
    pub version: String,
}

// --- Service (srv) ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Srv {
    pub caps: Caps,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caps {
    #[serde(rename = "multi-activity")]
    pub multi_activity: bool,
    #[serde(rename = "payload-manifest")]
    pub payload_manifest: bool,
}

#[test]
fn test_filter_attributes() {
    let offer = "{\"id\":\"00082a0389918034011dbcc885bd3da086eaaa66dceef7e6784386842571854d\",\"properties\":{\"golem\":{\"com\":{\"payment\":{\"debit-notes\":{\"accept-timeout?\":240},\"platform\":{\"erc20-polygon-glm\":{\"address\":\"0xa3bde9e2ef344407afdc931c97fd33d506ec6545\"}},\"protocol\":{\"version\":3}},\"pricing\":{\"model\":{\"@tag\":\"linear\",\"linear\":{\"coeffs\":[1e-9,0.0,0.0]}}},\"scheme\":{\"@tag\":\"payu\",\"payu\":{\"debit-note\":{\"interval-sec?\":120},\"payment-timeout-sec?\":120}},\"usage\":{\"vector\":[\"golem.usage.cpu_sec\",\"golem.usage.duration_sec\"]}},\"inf\":{\"cpu\":{\"architecture\":\"x86_64\",\"cores\":14,\"threads\":1},\"mem\":{\"gib\":42.79507473111153},\"storage\":{\"gib\":3257.801303100586}},\"node\":{\"debug\":{\"subnet\":\"public\"},\"id\":{\"name\":\"brick-54\"},\"net\":{\"is-public\":false}},\"runtime\":{\"name\":\"ya-runtime-cruncher\",\"version\":\"0.1.0\"},\"srv\":{\"caps\":{\"multi-activity\":true,\"payload-manifest\":false}}}},\"constraints\":\"(&\\n  (golem.srv.comp.expiration>1765401640654)\\n  (golem.node.debug.subnet=public)\\n)\",\"providerId\":\"0xa3bde9e2ef344407afdc931c97fd33d506ec6545\",\"expiration\":\"2025-12-11T12:20:45.222028719Z\",\"timestamp\":\"2025-12-11T11:20:45.222028719Z\"}";

    let gbo = serde_json::from_str::<GolemBaseOffer>(offer).unwrap();
    println!("GolemBaseOfferProperties: {}", serde_json::to_string(&gbo.properties).unwrap());

    let root = serde_json::from_value::<Properties>(gbo.properties).unwrap();

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

async fn get_if_available(data: web::Data<AppState>) -> impl Responder {
    let mut lock = data.lock.lock().await;
    for (_id, offer_obj) in lock.offer_map.iter_mut() {
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
    let offers: Vec<&GolemBaseOffer> = lock
        .offer_map
        .values()
        .map(|offer_obj| &offer_obj.offer)
        .collect();
    HttpResponse::Ok().json(offers)
}

async fn list_taken_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&GolemBaseOffer> = lock
        .offer_map
        .values()
        .filter(|offer_obj| !offer_obj.available)
        .map(|offer_obj| &offer_obj.offer)
        .collect();
    HttpResponse::Ok().json(offers)
}

async fn list_available_offers(data: web::Data<AppState>) -> impl Responder {
    let lock = data.lock.lock().await;
    let offers: Vec<&GolemBaseOffer> = lock
        .offer_map
        .values()
        .filter(|offer_obj| offer_obj.available)
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
            .route("/offer/take", web::get().to(get_if_available))
            .route(
                "/version",
                web::get().to(|| async { HttpResponse::Ok().body(env!("CARGO_PKG_VERSION")) }),
            )
    })
    .bind(format!("{}:{}", args.http_addr, args.http_port))?
    .workers(4)
    .run()
    .await
}
