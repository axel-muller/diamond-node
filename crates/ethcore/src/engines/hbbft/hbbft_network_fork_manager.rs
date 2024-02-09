use std::collections::VecDeque;

use ethereum_types::Address;
use ethjson::spec::hbbft::HbbftNetworkFork;
use hbbft::sync_key_gen::{Ack, Part};

struct HbbftFork {
//    start_timestamp: u64,
    start_block: u64,
//    end_timestamp: u64,
    end_block: Option<u64>,
    validators: Vec<Address>,
    parts: Vec<Part>,
    acks: Vec<Ack>,
}


impl HbbftFork {

    pub fn from_definition(fork_definiton: &HbbftNetworkFork) -> HbbftFork {

        let parts  = fork_definiton.parts.iter().map(|p| { 
            
            if let Ok(part) = bincode::deserialize( p.as_slice()) {
                part
            } else {
                error!(target:"engine", "hbbft-hardfork: could not interprete part from spec: {:?}", p.as_slice());
                panic!("hbbft-hardfork: could not interprete part from spec: {:?}", p.as_slice());
                
            }
        }).collect();

        let acks = fork_definiton.acks.iter().map(|a| { 
            
            if let Ok(ack) = bincode::deserialize( a.as_slice()) {
                ack
            } else {
                error!(target:"engine", "hbbft-hardfork: could not interprete part from spec: {:?}", a.as_slice());
                panic!("hbbft-hardfork: could not interprete part from spec: {:?}", a.as_slice());
            }
        }).collect();

        //bincode::deserialize(&serialized_part).unwrap();

        HbbftFork {
            start_block: fork_definiton.block_number_start,
            end_block: fork_definiton.block_number_end,
            validators: fork_definiton.validators.clone(),
            parts,
            acks
        }
    }
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

    /// we cannot apply the RAI pattern because of the delayed Hbbft initialization
    /// this variable tracks if the fork manager is initialized or not.
    is_init: bool,
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

    /// Initializes the fork Manager,
    /// with the information of the current block.
    /// the Fork Manager is able to determine when the next fork is pending.
    /// Forks that are already known to be finished,
    /// have to be declared as finished.
    pub fn initialize(&mut self, startup_block_number: u64, mut fork_definition:  Vec<HbbftNetworkFork>,) {
        if self.is_init {
            panic!("HbbftNetworkForkManager is already initialized");
        }

        fork_definition.sort_by_key(|fork| fork.block_number_start);

        // the fork definition can contain
        //  -  forks that are already finished
        //  -  forks that are pending

        // there is one corner case:
        // we could be in a current fork,
        // if there is a a fork defined,
        // that started in the past,
        // is ongoing, and the normal key generation did not proceed to a new block.

        // first of all, we are appending all forks that happened in the past and are considered finished.

        for fork_def in fork_definition.iter() {
            if let Some(end_block) = fork_def.block_number_end {
                // if the fork is known to be ended,
                // and the end is after current block,
                // we do not need to take care about this fork anymore.
                if end_block < startup_block_number {
                    debug!(target: "engine", "hbbft-hardfork: ignoring already finished fork {:?}", fork_def);
                    continue;
                }

                self.pending_forks.push_back(HbbftFork::from_definition(fork_def));
            }
        }

        // self.fork_definition.iter().filter(predicate).for_each(|fork| {
        //     self.pending_forks.push_back(HbbftFork {
        //         start_timestamp: 0,
        //         start_block: fork.block_number_start,
        //         is_finished: false,
        //         end_timestamp: 0,
        //         end_block: 0,
        //         validator_set: HbbftForkKeys {
        //             validators: fork.validators.clone(),
        //             parts: Vec::new(),
        //             acks: Vec::new(),
        //         },
        //     });
        // });
    }

    pub fn new() -> HbbftNetworkForkManager {
        HbbftNetworkForkManager {
            is_currently_forking: false,
            finished_forks: VecDeque::new(),
            pending_forks: VecDeque::new(),
            is_init: false,
        }
    }
}
