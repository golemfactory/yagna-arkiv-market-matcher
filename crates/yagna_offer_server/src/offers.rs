use crate::state::OfferObj;
use crate::AppState;
use actix_web::web;

pub async fn download_initial_offers(data: web::Data<AppState>) -> anyhow::Result<()> {
    let url = match std::env::var("INITIAL_OFFERS_URL") {
        Ok(url) => url,
        Err(_) => {
            log::warn!("INITIAL_OFFERS_URL not set, skipping initial download");
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

    let mut added = 0;
    for offer in offers {
        if lock.offer_map.contains_key(&offer.offer.id) {
            continue;
        }

        lock.offer_map.insert(offer.offer.id.clone(), offer);
        added += 1;
    }

    log::info!("Loaded {} initial offers", added);
    Ok(())
}
