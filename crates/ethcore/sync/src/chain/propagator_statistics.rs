use std::{
    collections::HashMap,
};

use stats::PrometheusMetrics;

use crate::sync_io::SyncIo;

#[derive(Default)]
pub struct SyncPropagatorStatistics {
    logging_enabled: bool,
    logging_peer_details_enabled: bool,
    propagated_blocks: i64,
    propagated_blocks_bytes: i64,

    node_statistics: HashMap<String, SyncPropagatorNodeStatistics>,
}

struct SyncPropagatorNodeStatistics {
    address: String,
    propagated_blocks: i64,
    propagated_blocks_bytes: i64,
}

impl SyncPropagatorStatistics {
    pub fn new() -> Self {
        SyncPropagatorStatistics {
            logging_enabled: true,
            logging_peer_details_enabled: true,
            propagated_blocks: 0,
            propagated_blocks_bytes: 0,
            node_statistics: HashMap::new(),
        }
    }

    pub fn logging_enabled(&self) -> bool {
        return self.logging_enabled;
    }

    pub fn log_packet(&mut self, io: &mut dyn SyncIo, peer_id: usize, blocks: usize, bytes: usize) {
        if self.logging_enabled() {
            self.propagated_blocks += blocks as i64;
            self.propagated_blocks_bytes += bytes as i64;

            if self.logging_peer_details_enabled {
                if let Some(peer_info) = io.peer_session_info(peer_id) {
                    //let mut node_statistics = &self.node_statistics;
                    let node_statistics = self
                        .node_statistics
                        .entry(peer_info.remote_address.clone())
                        .or_insert_with(|| SyncPropagatorNodeStatistics {
                            address: peer_info.remote_address,
                            propagated_blocks: 0,
                            propagated_blocks_bytes: 0,
                        });

                    node_statistics
                        .propagated_blocks += blocks as i64;

                    node_statistics
                        .propagated_blocks_bytes += bytes as i64;
                }
            }
        }
    }
}

impl PrometheusMetrics for SyncPropagatorStatistics {
    fn prometheus_metrics(&self, registry: &mut stats::PrometheusRegistry) {
        registry.register_counter(
            "p2p_propagated_blocks",
            "blocks sent",
            self.propagated_blocks,
        );
        registry.register_counter(
            "p2p_propagated_blocks_bytes",
            "block byte sent",
            self.propagated_blocks_bytes,
        );
        //registry.register_counter("p2p_propagated_blocks", "", self.propagated_blocks_bytes.load(Ordering::Relaxed));

        self.node_statistics
            .iter()
            .for_each(|(address, node_statistics)| {
                registry.register_gauge_with_other_node_label(
                    "p2p_propagated_blocks_peer",
                    "# blocks to peer",
                    &node_statistics.address,
                    node_statistics.propagated_blocks,
                );
                registry.register_gauge_with_other_node_label(
                    "p2p_propagated_bytes_peer",
                    "bytes to peer",
                    &node_statistics.address,
                    node_statistics.propagated_blocks_bytes,
                );
            });
    }
}
