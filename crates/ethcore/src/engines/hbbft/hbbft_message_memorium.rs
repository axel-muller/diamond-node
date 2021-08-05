use hbbft::honey_badger::{self};

// use threshold_crypto::{SignatureShare};
use engines::hbbft::NodeId;
// use hbbft::honey_badger::Message;
use serde::{Deserialize, Serialize};
use serde_json::{json, Result, Value};
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fs::{self, create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

pub type HbMessage = honey_badger::Message<NodeId>;

/**
Hbbft Message Process
// Broadcast - Echo
// Broadcast - Value
// Agreement
// Decryption Share
// Broadcast - CanDecode
// Subset - Message (proposer)
// Broadcast - Ready
// SignatureShare
//
//
*/

pub(crate) struct HbbftMessageMemorium {
    // future_messages_cache: BTreeMap<u64, Vec<(NodeId, HbMessage)>>,
    signature_shares: BTreeMap<u64, Vec<(NodeId, HbMessage)>>,

    //*
    // u64: epoch
    // NodeId: proposer
    // NodeId: node
    // HbMessage: message
    // */
    agreements: BTreeMap<u64, Vec<(NodeId, NodeId, HbMessage)>>,

    message_tracking_id: u64,
}

impl HbbftMessageMemorium {
    pub fn new() -> Self {
        HbbftMessageMemorium {
            signature_shares: BTreeMap::new(),
            agreements: BTreeMap::new(),
            message_tracking_id: 0,
        }
    }

    pub fn on_message_string_received(&mut self, message_json: String, epoch: u64) {
        self.message_tracking_id += 1;
        let mut path_buf = PathBuf::from(format!(
            "data/messages/{}/message_{}.json",
            epoch, self.message_tracking_id
        ));
        if let Err(e) = create_dir_all(path_buf.as_path()) {
            warn!("Error creating key directory: {:?}", e);
            return;
        };

        path_buf.push(format!("{}", self.message_tracking_id));

        let path = path_buf.as_path();
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                warn!(target: "consensus", "Error creating hbbft memorial file: {:?}", e);
                return;
            }
        };

        if let Err(e) = file.write(message_json.as_bytes()) {
            warn!(target: "consensus", "Error writing hbbft memorial file: {:?}", e);
        }
    }

    pub fn on_message_received(&mut self, message: &HbMessage) {
        //performance: dispatcher pattern + multithreading could improve performance a lot.

        let epoch = message.epoch();

        match serde_json::to_string(message) {
            Ok(json_string) => {
                debug!(target: "consensus", "{}", json_string);
                self.on_message_string_received(json_string, epoch);
            }
            Err(e) => {
                error!(target: "consensus", "could not create json.");
            }
        }
    }

    pub fn free_epoch_memory(&mut self, epoch: u64) {
        self.signature_shares.remove(&epoch);
    }
}
