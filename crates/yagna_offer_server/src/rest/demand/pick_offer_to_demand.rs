use crate::state::{AppState, DemandObj};
use actix_web::{web, HttpResponse};
use anyhow::bail;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;
use std::ops::Sub;
use std::str::FromStr;
use std::time::Instant;
use ya_client_model::NodeId;

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

    let mut name_filter = "";
    let mut selected_offer_id = None;

    //@todo, this works only in TESTNET !!, fix until go to prod
    if let Some(addr) = demand_obj.demand.central_net_address.as_ref() {
        name_filter = addr.split(".").next().unwrap_or("N/A");
    }

    for offer_pair in offers_lock.offer_map.iter_mut() {
        let offer = offer_pair.1;

        if !offer
            .offer
            .properties
            .golem
            .node
            .id
            .name
            .contains(name_filter)
        {
            continue;
        }

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
    central_net_filter: Option<String>,
) -> anyhow::Result<bool> {
    let perf_start = Instant::now();
    {
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
        // most recent, unexpired, and unassigned offer from the collection.
        // The use of newest_one as a baseline timestamp ensures that only the most recent valid
        // offer is chosen during the iteration.
        let mut selected_offer_id = None;
        let mut newest_one = Utc::now().sub(chrono::Duration::days(365 * 100));
        for offer_pair in offers_lock.offer_map.iter_mut() {
            let offer = offer_pair.1;
            if offer.offer.expiration.naive_utc() < Utc::now().naive_utc() {
                // expired
                continue;
            }
            if offer.requestor_id.is_some() {
                // already assigned
                continue;
            }
            let name_group = offer
                .attributes
                .node_name
                .split("-")
                .next()
                .unwrap_or("N/A");


            //used in integration tests
            let group = env::var("OFFER_GROUP").ok();

            if let Some(group) = group.as_ref() {
                if !offer.attributes.node_name.contains(group) {
                    continue;
                }
            } else {
                if let Some(central_net_filter) = central_net_filter.as_ref() {
                    if !central_net_filter.contains("127.0.0.1")
                        && !central_net_filter.contains(name_group)
                    {
                        continue;
                    }
                }
            }

            if offer.offer.timestamp > newest_one && offer.offer.timestamp < Utc::now() {
                // new good candidate
                newest_one = offer.offer.timestamp;
                selected_offer_id = Some(offer);
            }
        }

        let offer = match selected_offer_id {
            Some(offer) => offer,
            None => {
                return Ok(false);
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
    }
    if perf_start.elapsed().as_secs_f64() > 0.01 {
        log::warn!(
            "Pick offer took too long: {:.2} ms",
            perf_start.elapsed().as_secs_f64() * 1000.0
        );
    } else {
        log::debug!(
            "Pick offer took: {:.2} ms",
            perf_start.elapsed().as_secs_f64() * 1000.0
        );
    }
    Ok(true)
}
