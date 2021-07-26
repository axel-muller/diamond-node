use hbbft::honey_badger::{self};

// use threshold_crypto::{SignatureShare};
use engines::hbbft::NodeId;
use hbbft::honey_badger::Message;
use std::{borrow::Borrow, collections::BTreeMap};

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

    fn on_message_received(&self, message: &HbMessage) {
		// todo implementation.
        // let epoch = message.epoch();
        // let Message(share) = message;
        // let bytes = share.to_bytes();
    }
}
