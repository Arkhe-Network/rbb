use serde::{Deserialize, Serialize};
use crate::evolution::desci_node_resource::{RoyaltySplit, FreeTier};

#[derive(Debug, Clone)]
pub enum DeSciCommand {
    Publish {
        title: String,
        abstract_text: Option<String>,
        components: Vec<String>,
        authors: Vec<String>,
        license: Option<String>,
        publish: bool,
        spdx_license: Option<String>,
        copyright_holder: Option<String>,
        software_version: Option<String>,
        derived_from: Option<String>,
        ai_generated: Option<bool>,
        training_data: Option<String>,
    },
    Update {
        node_id: String,
        spdx: Option<String>,
        copyright: Option<String>,
        software_version: Option<String>,
        derived_from: Option<String>,
        ai_generated: Option<bool>,
        training_data: Option<String>,
    },
    Royalties {
        node_id: String,
        price: String,
        currency: String,
        splits: Vec<(String, f32)>,
        picnic_basket: Option<String>,
        free_tier: Option<u32>,
    },
}
