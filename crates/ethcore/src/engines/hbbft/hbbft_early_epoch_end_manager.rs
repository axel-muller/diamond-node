use types::ids::BlockId;

use crate::client::BlockChainClient;
use std::time::{Duration, Instant};

use super::{hbbft_message_memorium::HbbftMessageMemorium, NodeId};

pub(crate) struct HbbftEarlyEpochEndManager {
    /// The current epoch number.
    current_tracked_epoch_number: u64,

    /// epoch manager start up time.
    start_time: Instant,

    start_block: u64,

    /// allowed devp2p warmup time.
    allowed_devp2p_warmup_time: Duration,

    /// public keys of all validators for this epoch.
    validators: Vec<NodeId>,

    /// current flagged validators, unordered list - no performance issue, since this can
    /// only grow up to 7 elements for a usual set of 25 nodes.
    flagged_validators: Vec<NodeId>,
}

impl HbbftEarlyEpochEndManager {
    /// creates a new EarlyEpochEndManager,
    /// if conditions are matching to create one.
    /// It is expected that this function is only called if the node is a validator.
    /// This prerequesite will be checked and if not met, panics.
    pub fn create_early_epoch_end_manager(
        allowed_devp2p_warmup_time: Duration,
        client: &dyn BlockChainClient,
        epoch_number: u64,
        epoch_start_block: u64,
    ) -> Option<HbbftEarlyEpochEndManager> {
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
            current_tracked_epoch_number: epoch_number,
            start_time: Instant::now(),
            start_block: epoch_start_block,
            allowed_devp2p_warmup_time,
            validators: Vec::new(),
            flagged_validators: Self::get_current_flagged_validators_from_contracts(client),
        };

        return Some(result);
    }

    /// notifies about a new epoch.
    /// This (re)inits the Manager, no early epoch end happened.
    pub fn notify_new_epoch(&mut self, epoch: u64, validators: Vec<NodeId>) {
        self.current_tracked_epoch_number = epoch;
        self.validators = validators;
        self.start_time = Instant::now();
    }

    /// retrieves the information from smart contracts which validators are currently flagged.
    fn get_current_flagged_validators_from_contracts(
        full_client: &dyn BlockChainClient,
    ) -> Vec<NodeId> {
        // todo: call smart contract.
        return Vec::new();
    }

    fn notify_about_missing_validator(
        &mut self,
        validator: &NodeId,
        full_client: &dyn BlockChainClient,
    ) /* -> result of contract call errr */
    {
        // let mining_address = match self.signer.read().as_ref() {
        //     Some(signer) => signer.address(),
        //     None => {
        //         // we do not have a signer on Full and RPC nodes.
        //         // here is a possible performance improvement:
        //         // this won't change during the lifetime of the application ?!
        //         return Ok(());
        //     }
        // };

        // todo: send transaction to smart contract about missing validator.

        self.flagged_validators.push(validator.clone());
        warn!(target: "engine", "TODO: early-epoch-end: notify about missing validator: {:?}", validator);
    }

    fn notify_about_validator_reconnect(
        &mut self,
        validator: &NodeId,
        full_client: &dyn BlockChainClient,
    ) {
        if let Some(index) = self.flagged_validators.iter().position(|x| x == validator) {
            self.flagged_validators.remove(index);
            warn!(target: "engine", "TODO: early-epoch-end: notify about reconnected validator: {:?}", validator);
        } else {
            error!(target: "engine", " Could not find reconnected validator in flagged validators.");
        }
    }

    /// decides on the memorium data if we should update to contract data.
    /// end executes them.
    pub fn decide(
        &mut self,
        memorium: &HbbftMessageMemorium,
        full_client: &dyn BlockChainClient,
    ) {
        // if devp2p warmup time is not over yet, we do not have to do anything.
        if self.start_time.elapsed() < self.allowed_devp2p_warmup_time {
            return;
        }

        if full_client.is_syncing() {
            // if we are syncing, we wont do any blaming.
            return;
        }

        //full_client.
        

        let block_num = if let Some(block) = full_client.block(BlockId::Latest) { 
            block.number()
        } else {
            error!(target:"engine", "could not retrieve latest block.");
            return;
        };

        let treshold: u64 = 10;

        if self.start_block + treshold < block_num {
            // not enought blocks have passed this epoch,
            // to judge other nodes.
            return;
        }

        //full_client.best_block_header()
        // get current state of missing validators from hbbftMemorium.
        if let Some(epoch_history) = memorium.get_staking_epoch_history(block_num) {
            for validator in &self.validators.clone() {
                if let Some(node_history) = epoch_history.get_history_for_node(validator) {
                    let last_sealing_message = node_history.get_sealing_message();

                    if last_sealing_message < block_num - treshold {
                        // we do not have to send notification, if we already did so.

                        if !self.flagged_validators.contains(validator) {
                            // this function will also add the validator to the list of flagged validators.
                            self.notify_about_missing_validator(&validator, full_client);
                        }
                    } else {
                        // this validator is OK.
                        // maybe it was flagged and we need to unflag it ?

                        if self.flagged_validators.contains(validator) {
                            self.notify_about_validator_reconnect(&validator, full_client);
                        }
                    }
                }
                // todo: if the systems switched from block based measurement to time based measurement.
            }
        }
        // nothing to do: no history yet.

        // note: We do not take care if hbbft message memorium might not have processed some of the messages yet,
        // since it is not important to do the decision based on the latest data, since the decide method will be called
        // again.
    }
}

/// testing early epoch stop manager.
#[cfg(test)]
mod tests {

    #[test]
    fn test_early_epoch_end() {

        // should
    }
}
