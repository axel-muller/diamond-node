
//use hbbft::honey_badger::{self, MessageContent};
use hbbft::honey_badger::{self};
use parking_lot::RwLock;
use std::{collections::VecDeque, sync::mpsc::{self, Receiver, Sender}};

// use threshold_crypto::{SignatureShare};
use engines::hbbft::{sealing, NodeId};
// use hbbft::honey_badger::Message;
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Result, Value};

use std::{
    fs::{self, create_dir_all, File},
    io::Write,
    path::PathBuf,
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


pub(crate) enum SealMessageState {

    Good,
    Late(u64),
    Error(Box<sealing::Message>)
}

pub(crate) struct DispatchedSealMessage {
    message: Option<sealing::Message>

}

pub(crate) struct NodeBlockHistory {
    // sealing_messages: HashMap<SealMessageState>,
}

pub(crate) struct HbbftMessageMemorium {
    // future_messages_cache: BTreeMap<u64, Vec<(NodeId, HbMessage)>>,
    // signature_shares: BTreeMap<u64, Vec<(NodeId, HbMessage)>>,

    // decryption_shares: BTreeMap<u64, Vec<(NodeId, HbMessage)>>,
    //*
    // u64: epoch
    // NodeId: proposer
    // NodeId: node
    // HbMessage: message
    // */
    // agreements: BTreeMap<u64, Vec<(NodeId, NodeId, HbMessage)>>,
    message_tracking_id: u64,
    /// how many bad message should we keep on disk ?
    config_bad_message_evidence_reporting: u64,
    config_blocks_to_keep_on_disk: u64,
    config_block_to_keep_directory: String,
    last_block_deleted_from_disk: u64,
    dispatched_messages: VecDeque<HbMessage>,
    dispatched_seals: VecDeque<(sealing::Message, u64)>,
    //seal_event_good_receiver: Receiver<SealEventGood>,
    //seal_event_bad_receiver: Receiver<SealEventBad>,
}

pub(crate) struct HbbftMessageDispatcher {
    num_blocks_to_keep_on_disk: u64,
    thread: Option<std::thread::JoinHandle<Self>>,
    memorial: std::sync::Arc<RwLock<HbbftMessageMemorium>>,

    // seal_events_sender_good: Sender<SealEventGood>,
    // seal_events_sender_bad: Sender<SealEventBad>,
}

struct SealEventGood {
    node_id: NodeId,
    block_num: u64
}

struct SealEventBad {
    node_id: NodeId,
    block_num: u64
}



impl HbbftMessageDispatcher {
    pub fn new(num_blocks_to_keep_on_disk: u64, block_to_keep_directory: String) -> Self {

        let seal_event_channel_good = mpsc::channel::<SealEventGood>();
        let seal_event_channel_bad = mpsc::channel::<SealEventBad>();

        let mut result = HbbftMessageDispatcher {
            num_blocks_to_keep_on_disk,
            thread: None,
            memorial: std::sync::Arc::new(RwLock::new(HbbftMessageMemorium::new(
                num_blocks_to_keep_on_disk,
                block_to_keep_directory,
                // seal_event_channel_good.1,
                // seal_event_channel_bad.1,
            ))),
            //seal_events_sender_good: seal_event_channel_good.0,
            //seal_events_sender_bad: seal_event_channel_bad.0,
        };

        result.ensure_worker_thread();
        return result;
    }

    pub fn on_sealing_message_received(&mut self, message: &sealing::Message, epoch: u64) {

        if self.num_blocks_to_keep_on_disk > 0 { 
            self.memorial
                .write()
                .dispatched_seals
                .push_back((message.clone(), epoch));
        }
    }

    pub fn on_message_received(&mut self, message: &HbMessage) {

        if self.num_blocks_to_keep_on_disk > 0 {
            self.memorial
            .write()
            .dispatched_messages
            .push_back(message.clone());
        }
    }

    fn ensure_worker_thread(&mut self) {
        
        if self.thread.is_none() {
            // let mut memo = self;
            // let mut arc = std::sync::Arc::new(&self);
            let arc_clone = self.memorial.clone();
            self.thread = Some(std::thread::spawn(move || loop {
                let work_result = arc_clone.write().work_message();
                if !work_result {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
            }));
        }
    }

    pub fn report_seal_good(&mut self, node_id: &NodeId, block_num: u64) {
        let goodEvent = SealEventGood { node_id: node_id.clone(), block_num };
        // self.seal_events_good.push(goodEvent);
        
    }


    pub fn report_seal_bad(&mut self, node_id: &NodeId, block_num: u64) {

        let badEvent = SealEventBad { node_id: node_id.clone(), block_num };
        // self.seal_events_bad.push(badEvent);

    }

    pub fn free_memory(&mut self, _current_block: u64) {
        // TODO: make memorium freeing memory of ancient block.
    }
}

impl HbbftMessageMemorium {
    pub fn new(config_blocks_to_keep_on_disk: u64, block_to_keep_directory: String,/* seal_event_good_receiver: Receiver<SealEventGood>, seal_event_bad_receiver: Receiver<SealEventBad>,  */) -> Self {
        HbbftMessageMemorium {
            // signature_shares: BTreeMap::new(),
            // decryption_shares: BTreeMap::new(),
            // agreements: BTreeMap::new(),
            message_tracking_id: 0,
            config_bad_message_evidence_reporting: 0,
            config_blocks_to_keep_on_disk: config_blocks_to_keep_on_disk,
            config_block_to_keep_directory: block_to_keep_directory,
            last_block_deleted_from_disk: 0,
            dispatched_messages: VecDeque::new(),
            dispatched_seals: VecDeque::new(),
            // seal_event_good_receiver,
            // seal_event_bad_receiver,
        }
    }

    fn get_path_to_write(&self, epoch: u64) -> PathBuf {
        return PathBuf::from(format!("{}{}", self.config_block_to_keep_directory, epoch));
    }

    fn on_message_string_received(&mut self, message_json: String, epoch: u64) {
        self.message_tracking_id += 1;

        //don't pick up messages if we do not keep any.
        // and don't pick up old delayed messages for blocks already
        // decided to not to keep.
        if self.config_blocks_to_keep_on_disk > 0 && epoch > self.last_block_deleted_from_disk {
            let mut path_buf = self.get_path_to_write(epoch);
            if let Err(e) = create_dir_all(path_buf.as_path()) {
                warn!("Error creating key directory: {:?}", e);
                return;
            };

            path_buf.push(format!("{}.json", self.message_tracking_id));
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

            //figure out if we have to delete a old block
            // 1. protect against integer underflow.
            // 2. block is so new, that we have to trigger a cleanup
            if epoch > self.config_blocks_to_keep_on_disk
                && epoch > self.last_block_deleted_from_disk + self.config_blocks_to_keep_on_disk
            {
                let paths = fs::read_dir("data/messages/").unwrap();

                for dir_entry_result in paths {
                    //println!("Name: {}", path.unwrap().path().display())

                    match dir_entry_result {
                        Ok(dir_entry) => {
                            let path_buf = dir_entry.path();

                            if path_buf.is_dir() {
                                let dir_name = path_buf.file_name().unwrap().to_str().unwrap();

                                match dir_name.parse::<u64>() {
                                    Ok(dir_epoch) => {
                                        if dir_epoch <= epoch - self.config_blocks_to_keep_on_disk {
                                            match fs::remove_dir_all(path_buf.clone()) {
                                                Ok(_) => {
                                                    info!(target: "consensus", "deleted old message directory: {:?}", path_buf);
                                                }
                                                Err(e) => {
                                                    warn!(target: "consensus", "could not delete old directories reason: {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    self.last_block_deleted_from_disk = epoch;
                }
            }
        }
    }

    fn work_message(&mut self) -> bool {

        let mut had_worked = false;

        if let Some(message) = self.dispatched_messages.pop_front() {
            let epoch = message.epoch();
            // match message.content() {
            //     honey_badger::MessageContent::Subset(subset) => todo!(),
            //     honey_badger::MessageContent::DecryptionShare { proposer_id, share } => todo!(),
            // }
            match serde_json::to_string(&message) {
                Ok(json_string) => {
                    self.on_message_string_received(json_string, epoch);
                }
                Err(e) => {
                    // being unable to interprete a message, could result in consequences
                    // not being able to report missbehavior,
                    // or reporting missbehavior, where there was not a missbehavior.
                    error!(target: "consensus", "could not store hbbft message: {:?}", e);
                }
            }
            had_worked = true;
        }

        if let Some(seal) = self.dispatched_seals.pop_front() {
            match serde_json::to_string(&seal.0) {
                Ok(json_string) => {
                    self.on_message_string_received(json_string, seal.1);
                }
                Err(e) => {
                    // being unable to interprete a message, could result in consequences
                    // not being able to report missbehavior,
                    // or reporting missbehavior, where there was not a missbehavior.
                    error!(target: "consensus", "could not store seal message: {:?}", e);
                }
            }
            had_worked = true;
        }

        // if let Some(next) = self.seal_event_good_receiver.iter().next() {

        //     had_worked = true;
        // }

        return false;

        // let content = message.content();
        //match content {
        //    MessageContent::Subset(subset) => {}
        //    MessageContent::DecryptionShare { proposer_id, share } => {
        // debug!("got decryption share from {} {:?}", proposer_id, share);
        //        if !self.decryption_shares.contains_key(&epoch) {
        //            match self.decryption_shares.insert(epoch, Vec::new()) {
        //                None => {}
        //                Some(vec) => {
        //                    //Vec<(NodeId, message)
        //                }
        //            }
        //        }
        //    }
        //}
    }

    pub fn free_memory(&mut self, _current_block: u64) {
        // self.signature_shares.remove(&epoch);
    }
}
