use std::str::FromStr;
use std::sync::MutexGuard;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use ya_client_model::market::Demand;
use ya_client_model::NodeId;
use crate::model::demand::base::{DemandCancellation, DemandSubscription};
use crate::state::{AppState, DemandObj, Demands, Offers};

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
        offer_list: Default::default(),
    });

    HttpResponse::Ok().json(demand)
}

pub async fn take_offer_from_queue(data: web::Data<AppState>, demand_id: String) -> HttpResponse {
    let mut lock = data.demands.lock().await;
    let mut offers_lock = data.lock.lock().await;

    let mut get_demand = match lock.demand_map.contains_key(&demand_id) {
        true => lock.demand_map.get_mut(&demand_id),
        false => {
            let node_id = match NodeId::from_str(&demand_id) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid offer ID format or not found");
                }
            };
            let mut get_demand: Option<&mut DemandObj> = None;
            for (_, v) in lock.demand_map.iter_mut() {
                if v.demand.node_id == node_id {
                    get_demand = Some(v);
                    break;
                }
            }
            get_demand
        },
    };
    let demand_obj = match get_demand {
        Some(demand) => demand,
        None => {
            return HttpResponse::NotFound().body("Demand not found");
        }
    };
    match demand_obj.offer_list.pop_front() {
        Some(offer_id) => {
            let offer = offers_lock.offer_map.get(&offer_id);
            match offer {
                Some(offer) => HttpResponse::Ok().json(offer),
                None => HttpResponse::NotFound().body("Offer not found"),
            }
        }
        None => HttpResponse::NotFound().body("No offers available in the demand queue"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddOfferToDemand {
    pub demand_id: String,
    pub offer_id: String,
}

pub async fn add_offer_to_demand(data: web::Data<AppState>, body: String) -> HttpResponse {
    let decoded = serde_json::from_str::<AddOfferToDemand>(&body);
    let add_offer = match decoded {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding add offer to demand: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid format {}", e));
        }
    };
    let demand_id = add_offer.demand_id;
    let offer_id = add_offer.offer_id;

    let mut lock = data.demands.lock().await;
    let mut offers_lock = data.lock.lock().await;

    let mut offer = offers_lock.offer_map.get_mut(&offer_id);

    let offer = match offer {
        Some(offer) => offer,
        None => {
            return HttpResponse::NotFound().body("Offer not found");
        }
    };

    let mut get_demand = match lock.demand_map.contains_key(&demand_id) {
        true => lock.demand_map.get_mut(&demand_id),
        false => {
            let node_id = match NodeId::from_str(&demand_id) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid offer ID format or not found");
                }
            };
            let mut get_demand: Option<&mut DemandObj> = None;
            for (_, v) in lock.demand_map.iter_mut() {
                if v.demand.node_id == node_id {
                    get_demand = Some(v);
                    break;
                }
            }
            get_demand
        },
    };

    let demand_obj = match get_demand {
        Some(demand) => demand,
        None => {
            return HttpResponse::NotFound().body("Demand not found");
        }
    };
    if (!offer.available) {
        return HttpResponse::Conflict().body("Offer is not available");
    }
    offer.available = false;
    demand_obj.offer_list.push_back(offer.offer.id.clone());
    HttpResponse::Ok().body("Offer added to demand successfully")
}