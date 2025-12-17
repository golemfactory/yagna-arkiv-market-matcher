use crate::state::OfferObj;
use crate::AppState;
use actix_web::web;
use std::collections::HashMap;
use std::time::Instant;

pub async fn download_offers_from_mirror(data: web::Data<AppState>) -> anyhow::Result<()> {
    let url = match std::env::var("OFFER_SOURCE_URL") {
        Ok(url) => url,
        Err(_) => {
            log::warn!("INITIAL_OFFERS_URL not set, skipping download offers");
            return Ok(());
        }
    };

    log::info!("Downloading initial offers from {}", url);

    let response = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to download offers: {}", e);
            return Err(e.into());
        }
    };

    let text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to read response body: {}", e);
            return Err(e.into());
        }
    };
    let perf_start = Instant::now();

    let offers: Vec<OfferObj> = match serde_json::from_str::<Vec<OfferObj>>(&text) {
        Ok(offers) => offers,
        Err(e) => {
            log::error!("Failed to parse offers: {}", e);
            return Err(e.into());
        }
    };

    if offers.is_empty() {
        log::warn!("No valid offers downloaded");
        return Ok(());
    }

    let mut lock = data.lock.lock().await;

    //build map of existing by provider_id
    let mut by_provider_id = HashMap::new();

    for offer in lock.offer_map.iter() {
        let res = by_provider_id.insert(offer.1.offer.provider_id, offer.1.clone());
        if res.is_some() {
            log::warn!(
                "Multiple existing offers from provider {}",
                offer.1.offer.provider_id
            );
        }
    }

    let mut added = 0;
    let mut removed = 0;
    let mut already_present = 0;
    let mut ignored = 0;
    for offer in offers {
        if lock.offer_map.contains_key(&offer.offer.id) {
            already_present += 1;
            continue;
        }
        let mut to_remove = None;

        if by_provider_id.contains_key(&offer.offer.provider_id) {
            let by_provider_offer = by_provider_id
                .get(&offer.offer.provider_id)
                .expect("Has to contain that");
            if by_provider_offer.offer.timestamp < offer.offer.timestamp {
                //great, new offer is newer than older one
                to_remove = Some(by_provider_offer.offer.id.clone());
                by_provider_id.insert(offer.offer.provider_id, offer.clone());
            } else {
                //skip, older offer
                ignored += 1;
                continue;
            }
        } else {
            by_provider_id.insert(offer.offer.provider_id, offer.clone());
        }

        if let Some(remove_id) = to_remove {
            lock.offer_map.remove(&remove_id);
            removed += 1;
        }
        lock.offer_map.insert(offer.offer.id.clone(), offer);
        added += 1;
    }
    if perf_start.elapsed().as_secs_f64() > 0.01 {
        log::warn!(
            "Insert offers took too long: {:.2} ms",
            perf_start.elapsed().as_secs_f64() * 1000.0
        );
    } else {
        log::info!(
            "Insert offers offer took: {:.2} ms",
            perf_start.elapsed().as_secs_f64() * 1000.0
        );
    }

    log::info!(
        "Loaded {} new offers, there was {} already existing, removed {} older offers, ignored {} outdated offers",
        added,
        already_present,
        removed,
        ignored
    );
    Ok(())
}
