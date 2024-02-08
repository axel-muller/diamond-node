use std::collections::VecDeque;

use ethereum_types::Address;
use hbbft::sync_key_gen::{Ack, Part};

struct HbbftForkKeys {
    validators: Vec<Address>,
    parts: Vec<Part>,
    acks: Vec<Ack>,
}

struct HbbftFork {
    start_timestamp: u64,
    start_block: u64,
    is_finished: bool,
    end_timestamp: u64,
    end_block: u64,
    validator_set: HbbftForkKeys,
}

/// Hbbft network fork manager.
/// This manager is responsible for managing the forks.
/// It allows cheap queries to see if a Fork is pending,
/// and stores information about a fork that is finished.
pub struct HbbftNetworkForkManager {
    /// If a fork is currently in progress, this is true.
    is_currently_forking: bool,

    /// a ordered list with upcomming forks.
    finished_forks: VecDeque<HbbftFork>,

    /// a ordered list with upcomming forks, including a fork that is in progress.
    /// see @is_currently_forking for more information.
    pending_forks: VecDeque<HbbftFork>,
}

impl HbbftNetworkForkManager {
    /// Returns None if not forking
    /// Returns a List of Addresses that become the new validator set and
    /// declares the fork as active,
    pub fn should_fork(
        &mut self,
        last_block_number: u64,
        last_block_time_stamp: u64,
    ) -> Option<Vec<Address>> {
        // fields omitted

        None
    }
}
