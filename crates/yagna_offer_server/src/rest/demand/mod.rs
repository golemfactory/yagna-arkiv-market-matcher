pub mod add_offer_to_demand;
pub mod cancel_demand;
pub mod demand_new;
pub mod list_demands;
pub mod pick_offer_to_demand;
pub mod take_offer_from_queue;

use crate::rest::demand::pick_offer_to_demand::{local_pick_offer_to_demand, PickOfferToDemand};
use crate::state::{AppState, DemandObj};
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicI32, AtomicI64};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeOfferFromQueue {
    pub demand_id: String,
}

static NO_PICKED_OFFERS: AtomicI32 = AtomicI32::new(0);
static LAST_LOG_TIME: AtomicI64 = AtomicI64::new(0);
static LAST_CENTRAL_NET: AtomicI64 = AtomicI64::new(0);

pub async fn pick_offers_for_all_demands(data: web::Data<AppState>) {
    let demands: Vec<DemandObj> = {
        let lock = data.demands.lock().await;
        lock.demand_map.values().cloned().collect()
    };

    let mut central_nets = HashMap::new();
    {
        let _lock = data.offers_given_to_node.lock().await;
        for demand in demands.iter() {
            if let Some(net_address) = &demand.demand.central_net_address {
                central_nets.insert(net_address.clone(), true);
            }
        }
    }

    let mut central_nets: Vec<String> = central_nets.keys().cloned().collect();
    central_nets.sort();

    let current_net = LAST_CENTRAL_NET.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let net_selected = if central_nets.is_empty() {
        log::info!("No central nets found for picking offers");
        return;
    } else {
        let index = (current_net as usize) % central_nets.len();
        let net_id = &central_nets[index];
        log::info!("Picking offers for central net id: {}", net_id);
        net_id.clone()
    };

    let mut sort_by_given = Vec::new();
    {
        let lock = data.offers_given_to_node.lock().await;
        for demand in demands.iter() {
            if demand
                .demand
                .central_net_address
                .as_ref()
                .map(|cn| cn.as_str() != net_selected)
                .unwrap_or(true)
            {
                continue;
            }
            if let Some(count) = lock.get(&demand.demand.node_id.to_string()) {
                sort_by_given.push((demand.demand.id.clone(), *count));
            } else {
                sort_by_given.push((demand.demand.id.clone(), 0));
            }
        }
    }

    sort_by_given.sort_by_key(|k| k.1);

    let log_every_sec: f64 = env::var("LOG_EVERY_SEC")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10.0);

    let no_picked_offers = &NO_PICKED_OFFERS;
    if let Some(pair) = sort_by_given.first() {
        let pick_offer = PickOfferToDemand {
            demand_id: pair.0.clone(),
        };
        log::debug!(
            "Picking offer for node {}, that already received: {} offers",
            pair.0,
            pair.1
        );
        let last_log_time = LAST_LOG_TIME.load(std::sync::atomic::Ordering::SeqCst);
        let val = no_picked_offers.load(std::sync::atomic::Ordering::SeqCst);

        let current_time = chrono::Utc::now().timestamp_millis();
        if current_time - last_log_time > (log_every_sec * 1000.0) as i64 {
            log::info!(
                "Picked offers for {} demands so far, currently at node {} that received {} offers",
                val,
                pair.0,
                pair.1
            );
            LAST_LOG_TIME.store(current_time, std::sync::atomic::Ordering::SeqCst);
        }
        match local_pick_offer_to_demand(data.clone(), pick_offer, Some(net_selected)).await {
            Ok(found) => {
                if !found {
                    log::debug!("No available offers found to pick for demand {}", pair.0);
                } else {
                    no_picked_offers.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }
            Err(e) => {
                log::warn!("Failed to pick offer for demand: {}", e);
            }
        }
    }
}
