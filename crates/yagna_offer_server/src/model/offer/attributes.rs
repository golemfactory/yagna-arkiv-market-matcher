use crate::model::offer::base::GolemBaseOffer;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct OfferFlatAttributes {
    exe_name: String,
    subnet: String,
    cpu_architecture: String,
    cpu_threads: u32,
    node_id: String,
    node_id_group: u32,
    offer_id_group: u32,
}
static STATE: Mutex<(u64, u64)> = Mutex::new((0, 0));
// (last_hour, random_value)

//every hour new random number is generated
pub fn get_static_random() -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let hour = secs / 3600;
    let mut state = STATE.lock().unwrap();

    if state.0 != hour {
        // Generate a fresh random value for this hour
        let value = rand::thread_rng().gen();
        *state = (hour, value);
    }
    state.1
}

impl OfferFlatAttributes {
    pub fn from_gbo(gbo: &GolemBaseOffer) -> Self {
        let node_id = gbo.provider_id.to_string();

        let string_to_hash = format!("{}{}", get_static_random(), node_id);

        let sha256_hash = sha2::Sha256::digest(string_to_hash.as_bytes());
        let node_id_group = u32::from_be_bytes([
            sha256_hash[0],
            sha256_hash[1],
            sha256_hash[2],
            sha256_hash[3],
        ]) % 1000;

        let offer_id_hash = format!("{}{}", get_static_random(), gbo.id);
        let offer_id_hash = sha2::Sha256::digest(offer_id_hash.as_bytes());

        let offer_id_group = u32::from_be_bytes([
            offer_id_hash[0],
            offer_id_hash[1],
            offer_id_hash[2],
            offer_id_hash[3],
        ]) % 1000;

        OfferFlatAttributes {
            node_id,
            node_id_group,
            exe_name: gbo.properties.golem.runtime.name.clone(),
            subnet: gbo
                .properties
                .golem
                .node
                .debug
                .as_ref()
                .map(|f| f.subnet.clone())
                .unwrap_or_else(|| "public".to_string()),
            cpu_architecture: gbo.properties.golem.inf.cpu.architecture.clone(),
            cpu_threads: gbo.properties.golem.inf.cpu.threads,
            offer_id_group,
        }
    }
}
