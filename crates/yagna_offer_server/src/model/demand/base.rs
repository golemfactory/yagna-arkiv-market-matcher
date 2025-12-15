use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use ya_client_model::NodeId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandSubscription {
    pub id: String,
    pub properties: String,
    pub constraints: String,
    pub node_id: NodeId,
    /// Creation time of Demand on Requestor side.
    pub creation_ts: NaiveDateTime,
    /// Timestamp of adding this Demand to database.
    pub insertion_ts: Option<NaiveDateTime>,
    /// Time when Demand expires; set by Requestor.
    pub expiration_ts: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandCancellation {
    pub demand_id: String,
}
