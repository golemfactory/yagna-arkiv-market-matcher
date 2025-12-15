use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ya_client_model::NodeId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandSubscription {
    pub demand_id: String,
    pub node_id: NodeId,
    pub valid_to: DateTime<Utc>,




}
