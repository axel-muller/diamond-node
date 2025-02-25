use std::{
    borrow::Cow,
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicI64, Ordering},
};

use parking_lot::Mutex;

use crate::sync_io::SyncIo;

use super::ChainSync;

#[derive(Default)]
pub struct SyncPropagatorStatistics {
    logging_enabled: AtomicBool,
    propagated_blocks: AtomicI64,
    propagated_blocks_bytes: AtomicI64,

    node_statistics: Mutex<HashMap<String, SyncPropagatorNodeStatistics>>,
}

struct SyncPropagatorNodeStatistics {
    address: String,
    propagated_blocks: AtomicI64,
    propagated_blocks_bytes: AtomicI64,
}

impl SyncPropagatorStatistics {
    pub fn new() -> Self {
        SyncPropagatorStatistics {
            logging_enabled: AtomicBool::new(true),
            propagated_blocks: AtomicI64::new(0),
            propagated_blocks_bytes: AtomicI64::new(0),
            node_statistics: Mutex::new(HashMap::new()),
        }
    }

    pub fn logging_enabled(&self) -> bool {
        return self.logging_enabled.load(Ordering::Relaxed);
    }

    pub fn log_packet(&self, io: &mut dyn SyncIo, peer_id: usize, blocks: usize, bytes: usize) {
        if self.logging_enabled() {
            self.propagated_blocks
                .fetch_add(blocks as i64, Ordering::Relaxed);
            self.propagated_blocks_bytes
                .fetch_add(bytes as i64, Ordering::Relaxed);

            if let Some(peer_info) = io.peer_session_info(peer_id) {
                let mut node_statistics = self.node_statistics.lock();
                let node_statistics = node_statistics
                    .entry(peer_info.remote_address.clone())
                    .or_insert_with(|| SyncPropagatorNodeStatistics {
                        address: peer_info.remote_address,
                        propagated_blocks: AtomicI64::new(0),
                        propagated_blocks_bytes: AtomicI64::new(0),
                    });

                node_statistics
                    .propagated_blocks
                    .fetch_add(blocks as i64, Ordering::Relaxed);
                node_statistics
                    .propagated_blocks_bytes
                    .fetch_add(bytes as i64, Ordering::Relaxed);
            }
        }
    }
}
