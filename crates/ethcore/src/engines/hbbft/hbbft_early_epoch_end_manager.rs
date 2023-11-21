use std::time::{Instant, Duration};
use crate::client::BlockChainClient;

use super::{NodeId, hbbft_message_memorium::HbbftMessageMemorium};


pub(crate) struct EarlyEpochEndManager {

    /// The current epoch number.
    current_tracked_epoch_number: u64,

    /// epoch manager start up time.
    start_time: Instant,

    /// allowed devp2p warmup time.
    allowed_devp2p_warmup_time: Duration,

    /// public keys of all validators for this epoch.
    validators: Vec<NodeId>,

    /// current flagged validators
    flagged_validators: Vec<NodeId>,
}


impl EarlyEpochEndManager { 

    /// creates a new EarlyEpochEndManager,
    /// if conditions are matching to create one.
    /// It is expected that this function is only called if the node is a validator.
    /// This prerequesite will be checked and if not met, panics.
    pub fn create_early_epoch_end_manager(allowed_devp2p_warmup_time: Duration, client: &dyn BlockChainClient) -> Option<EarlyEpochEndManager> {


        if client.is_syncing() {
            // if we are syncing, we do not need to create an early epoch end manager yet.
            // if we are syncing as a validator, and it is really this epoch,
            // this way the creation of the early epoch end manager is created in a subsequent call,
            // when we are at the tip of the chain, and get the correct state for 
            // - flagged validators
            // - start_time
            // The whole window for the devp2p warmup time is granted in this case,
            // therefore this node won't flag anyone in the near future.
            return None;
        }

        // figure out if we have to retrieve the data from the smart contracts.
        // if the epoch start did just happen, 
        // we do not have to retrieve the data from the smart contracts.

        
        let result = Self {
            current_tracked_epoch_number: 0,
            start_time: Instant::now(),
            allowed_devp2p_warmup_time,
            validators: Vec::new(),
            flagged_validators: Vec::new(),
        };

        return Some(result);

    }

    /// notifies about a new epoch.
    /// This (re)inits the Manager, no early epoch end happened.
    pub fn notify_new_epoch(&mut self, epoch: u64, validators: Vec<NodeId> ) {

        self.current_tracked_epoch_number = epoch;
        self.validators = validators;
        self.start_time = Instant::now();
    }

    /// retrieves the information from smart contracts which validators are currently flagged.
    fn get_current_flagged_validators_from_contracts() -> Vec<NodeId> {

        // todo: call smart contract.
        return Vec::new();
    }

    fn notify_about_missing_validator(&mut self, validator: NodeId, full_client: &dyn BlockChainClient) { 

        // todo: send transaction to smart contract about missing validator.
    }

    /// decides on the memorium data if we should update to contract data.
    pub fn decide(memorium: &HbbftMessageMemorium) {


        // note: We do not take care if hbbft message memorium might not have processed some of the messages yet,
        // since it is not important to do the decision based on the latest data, since the decide method will be called
        // again.
    }

}