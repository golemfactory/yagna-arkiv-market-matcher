use std::collections::BTreeMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ya_client_model::market::Demand;
use crate::model::offer::attributes::OfferFlatAttributes;
use crate::model::offer::base::GolemBaseOffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandObj {
    pub demand: Demand,
}


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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Demands {
    pub demand_map: BTreeMap<String, DemandObj>,
}

#[derive(Clone)]
pub struct AppState {
    pub lock: Arc<tokio::sync::Mutex<Offers>>,
    pub demands: Arc<tokio::sync::Mutex<Demands>>,
}