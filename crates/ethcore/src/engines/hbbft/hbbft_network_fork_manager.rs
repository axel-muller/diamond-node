use std::{collections::{BTreeMap, VecDeque}, sync::Arc};

use ethereum_types::Address;
use ethjson::spec::hbbft::HbbftNetworkFork;
use hbbft::{crypto::PublicKeySet, sync_key_gen::{Ack, Part, SyncKeyGen}, util::max_faulty, NetworkInfo};
use parking_lot::RwLock;

use crate::engines::{hbbft::contracts::keygen_history::{KeyPairWrapper, PublicWrapper}, EngineSigner};

use super::NodeId;

#[derive(Debug)]
struct HbbftFork {
    //    start_timestamp: u64,
    start_block: u64,

    // start epoch is set, if the fork has been started.
    start_epoch: Option<u64>,

    // end_block is set when the fork process is finished and the network operation has normaliced again.
    end_block: Option<u64>,

    validators: Vec<NodeId>,
    parts: Vec<Part>,
    acks: Vec<Ack>,
}

impl HbbftFork {
    pub fn from_definition(fork_definiton: &HbbftNetworkFork) -> HbbftFork {
        let parts = fork_definiton.parts.iter().map(|p| {
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
                error!(target:"engine", "hbbft-hardfork: could not interprete acks from spec: {:?}", a.as_slice());
                panic!("hbbft-hardfork: could not interprete acks from spec: {:?}", a.as_slice());
            }
        }).collect();

        
        let node_ids = fork_definiton.acks.iter().map(|h| {
            if let Ok(node_id) = bincode::deserialize( h.as_slice()) {
                node_id
            } else {
                error!(target:"engine", "hbbft-hardfork: could not interprete nodeIds from spec: {:?}", h.as_slice());
                panic!("hbbft-hardfork: could not interprete part from spec: {:?}", h.as_slice());
            }
        }).collect();

        HbbftFork {
            start_block: fork_definiton.block_number_start,
            start_epoch: None,
            end_block: fork_definiton.block_number_end,
            validators: node_ids,
            parts,
            acks,
        }
    }
}

/// Hbbft network fork manager.
/// This manager is responsible for managing the forks.
/// It allows cheap queries to see if a Fork is pending,
/// and stores information about a fork that is finished.
pub struct HbbftNetworkForkManager {

    /// a ordered list with upcomming forks.
    finished_forks: VecDeque<HbbftFork>,

    /// a ordered list with upcomming forks, including a fork that is in progress.
    /// see @is_currently_forking for more information.
    pending_forks: VecDeque<HbbftFork>,

    /// we cannot apply the RAI pattern because of the delayed Hbbft initialization
    /// this variable tracks if the fork manager is initialized or not.
    is_init: bool,

    own_id: NodeId
}

impl HbbftNetworkForkManager {

    /// Returns None if not forking
    /// Returns a List of Addresses that become the new validator set and
    /// declares the fork as active,
    pub fn should_fork(
        &mut self,
        last_block_number: u64,
        current_epoch: u64,
        signer_lock: Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
    ) -> Option<NetworkInfo<NodeId>> {
        // fields omitted

        if let Some(next_fork) = self.pending_forks.front_mut() {
            
            if next_fork.start_block == last_block_number {
               
               //let keys : PublicKeySet = PublicKeySet::
               let wrapper = KeyPairWrapper {
                    inner: signer_lock.clone(),
                };
                
                let mut rng = rand::thread_rng();
                let mut pub_keys_btree: BTreeMap<NodeId, PublicWrapper> = BTreeMap::new();

                for v in next_fork.validators.iter() {
                    pub_keys_btree.insert(v.clone(), PublicWrapper { 
                        inner: v.clone().0
                    });
                }
                
                let pub_keys: Arc<BTreeMap<NodeId, PublicWrapper>> = Arc::new(pub_keys_btree);
                let skg = match SyncKeyGen::new(self.own_id, wrapper, pub_keys, max_faulty(next_fork.validators.len()), &mut rng) {
                    Ok(s) => s.0,
                    Err(e) => {
                        error!(target: "engine", "hbbft-hardfork: could not create SyncKeyGen: {:?}", e);
                        panic!("hbbft-hardfork: could not create SyncKeyGen: {:?}", e);
                    }
                };
                
                if !skg.is_ready() {
                    error!(target: "engine", "hbbft-hardfork: missing parts for SyncKeyGen for fork {:?}", next_fork);
                    panic!("hbbft-hardfork: missing parts for SyncKeyGen for fork {:?}", next_fork);
                }


                let (pks, sks) = match skg.generate() {
                    Ok((p, s)) => (p, s),
                    Err(e) => {
                        error!(target: "engine", "hbbft-hardfork: could not generate keys for fork: {:?} {:?}", e, next_fork);
                        panic!("hbbft-hardfork: could not generate keys for fork: {:?} {:?}", e, next_fork);
                    }
                };

                let result = NetworkInfo::<NodeId>::new(
                    self.own_id,
                    sks,
                    pks,
                    next_fork.validators.clone()
                );

                return Some(result);

            } else if next_fork.start_block > last_block_number {

                // in the following blocks after the fork process was started,
                // it is possible for the network to have now ended the fork process.
                // we are checking if the current epoch is greater than the start epoch.

                if let Some(start_epoch) = next_fork.start_epoch {
                    if current_epoch == start_epoch + 1 {
                        next_fork.end_block = Some(last_block_number);

                        // the fork process is finished.
                        // we are moving the fork to the finished forks list.
                        
                        self.finished_forks.push_back(self.pending_forks.pop_front().unwrap());
                    }
                }
            } // else: we are just waiting for the fork to happen.
        }
        None

    }

    /// Initializes the fork Manager,
    /// with the information of the current block.
    /// the Fork Manager is able to determine when the next fork is pending.
    /// Forks that are already known to be finished,
    /// have to be declared as finished.
    pub fn initialize(
        &mut self,
        own_id: NodeId,
        startup_block_number: u64,
        mut fork_definition: Vec<HbbftNetworkFork>,
    ) {
        if self.is_init {
            panic!("HbbftNetworkForkManager is already initialized");
        }

        self.own_id = own_id;        

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

                self.pending_forks
                    .push_back(HbbftFork::from_definition(fork_def));
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
            finished_forks: VecDeque::new(),
            pending_forks: VecDeque::new(),
            is_init: false,
            own_id: NodeId::default(),
        }
    }
}



#[cfg(test)]
mod tests { 

    use super::*;
    use ethjson::spec::hbbft::HbbftNetworkFork;
    use hbbft::sync_key_gen::{Ack, Part};
    use ethereum_types::Address;

    #[test]
    fn test_should_fork() {
        // let mut fork_manager = HbbftNetworkForkManager::new();
        // let mut fork_definition = Vec::new();

        // let mut fork = HbbftNetworkFork {
        //     block_number_start: 10,
        //     block_number_end: Some(20),
        //     validators: vec![Address::from([0; 20])],
        //     parts: vec![bincode::serialize(&Part::new(0, 0)).unwrap()],
        //     acks: vec![bincode::serialize(&Ack::new(0, 0)).unwrap()],
        // };

        // fork_definition.push(fork);

        // fork_manager.initialize(5, fork_definition);

        // let result = fork_manager.should_fork(10, 0);
        // assert!(result.is_some());
    }


}