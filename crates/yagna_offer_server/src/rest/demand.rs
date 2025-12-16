use crate::model::demand::base::{DemandCancellation, DemandSubscription};
use crate::state::{AppState, DemandObj};
use actix_web::{web, HttpResponse};
use anyhow::bail;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::str::FromStr;
use ya_client_model::NodeId;

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
    lock.demand_map
        .retain(|_, v| v.demand.node_id != demand.node_id);

    let _ = lock.demand_map.insert(
        demand.id.clone(),
        DemandObj {
            demand: demand.clone(),
            offer_list: Default::default(),
        },
    );

    HttpResponse::Ok().json(demand)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeOfferFromQueue {
    pub demand_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelOffer {
    pub id: String,
    pub properties: String,
    pub constraints: String,
    pub node_id: NodeId,
    // Database information telling if we are the owner of the Offer.
    // None means that we don't have this information yet (for example in case when
    // the Offer didn't come from our database).
    pub owned: Option<bool>,

    /// Creation time of Offer on Provider side.
    pub creation_ts: NaiveDateTime,
    /// Timestamp of adding this Offer to database.
    pub insertion_ts: Option<NaiveDateTime>,
    /// Time when Offer expires; set by Provider.
    pub expiration_ts: NaiveDateTime,
}
pub fn flatten(value: Value) -> Map<String, Value> {
    let mut map = Map::new();
    flatten_inner(String::new(), &mut map, value);
    map
}
pub const PROPERTY_TAG: &str = "@tag";

fn flatten_inner(prefix: String, result: &mut Map<String, Value>, value: Value) {
    match value {
        Value::Object(m) => {
            if m.is_empty() {
                // Important to keep this value in case we want to un-flatten later
                // and get the same structure.
                result.insert(prefix, Value::Object(Map::new()));
            } else {
                for (k, v) in m.into_iter() {
                    if k.as_str() == PROPERTY_TAG {
                        result.insert(prefix.clone(), v);
                        continue;
                    }
                    let p = match prefix.is_empty() {
                        true => k,
                        _ => format!("{}.{}", prefix, k),
                    };
                    flatten_inner(p, result, v);
                }
            }
        }
        v => {
            result.insert(prefix, v);
        }
    }
}

pub async fn take_offer_from_queue(data: web::Data<AppState>, body: String) -> HttpResponse {
    let decoded = serde_json::from_str::<TakeOfferFromQueue>(&body);
    let take_offer = match decoded {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding take offer from queue: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid format {}", e));
        }
    };
    let demand_id = take_offer.demand_id;

    let mut lock = data.demands.lock().await;
    let offers_lock = data.lock.lock().await;

    let get_demand = match lock.demand_map.contains_key(&demand_id) {
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
        }
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
                Some(offer) => {
                    let converted_offer = ModelOffer {
                        id: offer.offer.id.clone(),
                        properties: serde_json::to_string(&flatten(
                            serde_json::to_value(offer.offer.properties.clone()).unwrap(),
                        ))
                        .unwrap(),
                        constraints: offer.offer.constraints.clone(),
                        node_id: offer.offer.provider_id,
                        owned: None,
                        creation_ts: offer.offer.timestamp.naive_utc(),
                        insertion_ts: None,
                        expiration_ts: offer.offer.expiration.naive_utc(),
                    };

                    HttpResponse::Ok().json(converted_offer)
                }
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

    let offer = offers_lock.offer_map.get_mut(&offer_id);

    let offer = match offer {
        Some(offer) => offer,
        None => {
            return HttpResponse::NotFound().body("Offer not found");
        }
    };

    let get_demand = match lock.demand_map.contains_key(&demand_id) {
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
        }
    };

    let demand_obj = match get_demand {
        Some(demand) => demand,
        None => {
            return HttpResponse::NotFound().body("Demand not found");
        }
    };
    if offer.requestor_id.is_some() {
        return HttpResponse::Conflict().body("Offer is already taken");
    }
    offer.requestor_id = Some(demand_obj.demand.node_id);
    demand_obj.offer_list.push_back(offer.offer.id.clone());
    HttpResponse::Ok().body("Offer added to demand successfully")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickOfferToDemand {
    pub demand_id: String,
}

pub async fn pick_offer_to_demand(data: web::Data<AppState>, body: String) -> HttpResponse {
    let decoded = serde_json::from_str::<PickOfferToDemand>(&body);

    let add_offer = match decoded {
        Ok(filer) => filer,
        Err(e) => {
            log::error!("Error decoding pick offer to demand: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid format {}", e));
        }
    };
    let demand_id = add_offer.demand_id;

    let mut lock = data.demands.lock().await;
    let mut offers_lock = data.lock.lock().await;

    let get_demand = match lock.demand_map.contains_key(&demand_id) {
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
        }
    };

    let demand_obj = match get_demand {
        Some(demand) => demand,
        None => {
            return HttpResponse::NotFound().body("Demand not found");
        }
    };
    let mut selected_offer_id = None;
    for offer_pair in offers_lock.offer_map.iter_mut() {
        let offer = offer_pair.1;
        if offer.requestor_id.is_none() {
            selected_offer_id = Some(offer);
            break;
        }
    }

    let offer = match selected_offer_id {
        Some(offer) => offer,
        None => {
            return HttpResponse::NotFound().body("No available offers found");
        }
    };

    offer.requestor_id = Some(demand_obj.demand.node_id);
    demand_obj.offer_list.push_back(offer.offer.id.clone());
    HttpResponse::Ok().body("Offer added to demand successfully")
}

pub async fn local_pick_offer_to_demand(
    data: web::Data<AppState>,
    pick_offer_to_demand: PickOfferToDemand,
) -> anyhow::Result<()> {
    let demand_id = pick_offer_to_demand.demand_id;

    let mut lock = data.demands.lock().await;
    let mut offers_lock = data.lock.lock().await;
    let mut given_lock = data.offers_given_to_node.lock().await;

    let get_demand = match lock.demand_map.contains_key(&demand_id) {
        true => lock.demand_map.get_mut(&demand_id),
        false => {
            let node_id = match NodeId::from_str(&demand_id) {
                Ok(id) => id,
                Err(_) => {
                    bail!("Invalid offer ID format or not found");
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
        }
    };

    let demand_obj = match get_demand {
        Some(demand) => demand,
        None => {
            bail!("Demand not found");
        }
    };
    let mut selected_offer_id = None;
    for offer_pair in offers_lock.offer_map.iter_mut() {
        let offer = offer_pair.1;
        if offer.requestor_id.is_none() {
            selected_offer_id = Some(offer);
            break;
        }
    }

    let offer = match selected_offer_id {
        Some(offer) => offer,
        None => {
            bail!("No available offers found");
        }
    };

    offer.requestor_id = Some(demand_obj.demand.node_id);
    demand_obj.offer_list.push_back(offer.offer.id.clone());
    let val = given_lock
        .get(&demand_obj.demand.node_id.to_string())
        .cloned();
    match val {
        Some(count) => {
            given_lock.insert(demand_obj.demand.node_id.to_string(), count + 1);
        }
        None => {
            given_lock.insert(demand_obj.demand.node_id.to_string(), 1);
        }
    }
    Ok(())
}

pub async fn pick_offers_for_all_demands(data: web::Data<AppState>) {
    let demands: Vec<DemandObj> = {
        let lock = data.demands.lock().await;
        lock.demand_map.values().cloned().collect()
    };

    let mut sort_by_given = Vec::new();
    {
        let lock = data.offers_given_to_node.lock().await;
        for demand in demands.iter() {
            if let Some(count) = lock.get(&demand.demand.node_id.to_string()) {
                sort_by_given.push((demand.demand.id.clone(), *count));
            } else {
                sort_by_given.push((demand.demand.id.clone(), 0));
            }
        }
    }

    sort_by_given.sort_by_key(|k| k.1);

    if let Some(pair) = sort_by_given.first() {
        let pick_offer = PickOfferToDemand {
            demand_id: pair.0.clone(),
        };
        log::info!(
            "Picking offer for node {}, that already received: {} offers",
            pair.0,
            pair.1
        );
        match local_pick_offer_to_demand(data.clone(), pick_offer).await {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to pick offer for demand: {}", e);
            }
        }
    }
}
