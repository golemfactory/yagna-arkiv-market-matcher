use std::collections::BTreeMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::offer::attributes::OfferFlatAttributes;
use crate::model::offer::base::GolemBaseOffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferObj {
    pub offer: GolemBaseOffer,
    pub pushed_at: DateTime<Utc>,
    pub available: bool,
    pub attributes: OfferFlatAttributes,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Offers {
    pub offer_map: BTreeMap<String, OfferObj>,
}

#[derive(Clone)]
pub struct AppState {
    pub lock: Arc<tokio::sync::Mutex<Offers>>,
}