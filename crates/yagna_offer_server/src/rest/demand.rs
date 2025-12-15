use actix_web::{web, HttpResponse};
use ya_client_model::market::Demand;
use crate::model::demand::base::DemandSubscription;
use crate::state::{AppState, DemandObj};

pub async fn demand_new(data: web::Data<AppState>, item: String) -> HttpResponse {
    let decode = serde_json::from_str::<Demand>(&item);

    let demand = match decode {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding filter: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid filter format {}", e));
        }
    };
    let mut lock = data.demands.lock().await;

    if lock.demand_map.contains_key(&demand.demand_id) {
        return HttpResponse::Conflict().body("Demand with the same id already exists");
    }

    let _ = lock.demand_map.insert(demand.demand_id.clone(), DemandObj {
        demand: demand.clone(),
    });

    let subscription = DemandSubscription {
        demand_id: demand.demand_id,
        node_id: demand.requestor_id,
        valid_to: demand.timestamp + chrono::Duration::seconds(3600),
    };

    HttpResponse::Ok().json(subscription)
}