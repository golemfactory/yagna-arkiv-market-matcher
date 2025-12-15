use actix_web::{web, HttpResponse};
use ya_client_model::market::Demand;
use crate::model::demand::base::{DemandCancellation, DemandSubscription};
use crate::state::{AppState, DemandObj};




pub async fn list_demands(data: web::Data<AppState>) -> HttpResponse {
    let lock = data.demands.lock().await;
    let demands: Vec<&DemandObj> = lock.demand_map.values().collect();
    HttpResponse::Ok().json(demands)
}

pub async fn demand_cancel(data: web::Data<AppState>, item: String) -> HttpResponse {
    let decode = serde_json::from_str::<DemandCancellation>(&item);

    let cancellation = match decode {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding demand cancellation: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid cancellation format {}", e));
        }
    };

    let mut lock = data.demands.lock().await;
    if lock.demand_map.remove(&cancellation.demand_id).is_some() {
        HttpResponse::Ok().body("Demand cancelled successfully")
    } else {
        HttpResponse::NotFound().body("Demand not found")
    }
}

pub async fn demand_new(data: web::Data<AppState>, item: String) -> HttpResponse {
    let decode = serde_json::from_str::<DemandSubscription>(&item);

    let demand = match decode {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding demand: {}", e);
            log::error!("Received demand: {}", item);
            return HttpResponse::BadRequest().body(format!("Invalid filter format {}", e));
        }
    };
    let mut lock = data.demands.lock().await;

    if lock.demand_map.contains_key(&demand.id) {
        return HttpResponse::Conflict().body("Demand with the same id already exists");
    }

    // Remove existing demand from the same node
    lock.demand_map.retain(|_, v| v.demand.node_id != demand.node_id);

    let _ = lock.demand_map.insert(demand.id.clone(), DemandObj {
        demand: demand.clone(),
    });

    HttpResponse::Ok().json(demand)
}