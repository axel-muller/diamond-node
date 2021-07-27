use hbbft::honey_badger::{self};

// use threshold_crypto::{SignatureShare};
use engines::hbbft::NodeId;
use hbbft::honey_badger::Message;
use std::{borrow::Borrow, collections::BTreeMap};
use serde_json::{Result, Value};
use serde_json::json;
use serde::{Deserialize, Serialize};

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
}

impl HbbftMessageMemorium {
    pub fn new() -> Self {
        HbbftMessageMemorium {
            signature_shares: BTreeMap::new(),
            agreements: BTreeMap::new(),
        }
    }

	pub fn on_message_string_received(&self, message: String, epoch: u64) {

		// if (message.contains("")

	}

    pub fn on_message_received(&self, message: &HbMessage) {


		//performance: dispatcher pattern could improve performance a lot.
		let message_string = format!("{:?}", message);

		let epoch = message.epoch();

		match serde_json::to_string(message) {
			Ok(jsonString) => { debug!(target: "consensus", "{}", jsonString); }
			Err(e) => { error!(target: "consensus", "could not create json."); }
		}

		// how to figure out proposer ?

		self.on_message_string_received(message_string, epoch);


		//the details are imprisionated
		// todo implementation.


		// let Message(share) = message;
		// let bytes = share.to_bytes();
    }
}
