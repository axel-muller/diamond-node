use ethereum_types::Address;
use stats::PrometheusMetrics;
use types::ids::BlockId;

use crate::{client::BlockChainClient, ethereum::public_key_to_address::public_key_to_address};
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
        validator_set: Vec<NodeId>,
        signing_address: &Address,
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

        let validators: Vec<NodeId> = validator_set
            .iter()
            .filter(|n| public_key_to_address(&n.0) != *signing_address)
            .cloned()
            .collect();

        // figure out if we have to retrieve the data from the smart contracts.
        // if the epoch start did just happen,
        // we do not have to retrieve the data from the smart contracts.

        let result = Self {
            current_tracked_epoch_number: epoch_number,
            start_time: Instant::now(),
            start_block: epoch_start_block,
            allowed_devp2p_warmup_time,
            validators: validators,
            flagged_validators: Self::get_current_flagged_validators_from_contracts(client),
        };

        return Some(result);
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
        mining_address: &Address,
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
        mining_address: &Address,
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
        mining_address: &Address,
    ) {
        // if devp2p warmup time is not over yet, we do not have to do anything.
        if self.start_time.elapsed() < self.allowed_devp2p_warmup_time {
            return;
        }

        if full_client.is_syncing() {
            // if we are syncing, we wont do any blaming.
            return;
        }

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
                // we need to exclude ourself.
                if let Some(node_history) = epoch_history.get_history_for_node(validator) {
                    let last_sealing_message = node_history.get_sealing_message();

                    if last_sealing_message < block_num - treshold {
                        // we do not have to send notification, if we already did so.

                        if !self.flagged_validators.contains(validator) {
                            // this function will also add the validator to the list of flagged validators.
                            self.notify_about_missing_validator(
                                &validator,
                                full_client,
                                mining_address,
                            );
                        }
                    } else {
                        // this validator is OK.
                        // maybe it was flagged and we need to unflag it ?

                        if self.flagged_validators.contains(validator) {
                            self.notify_about_validator_reconnect(
                                &validator,
                                full_client,
                                mining_address,
                            );
                        }
                    }
                } else {
                    // we do not have any history for this node.
                    if !self.flagged_validators.contains(validator) {
                        // this function will also add the validator to the list of flagged validators.
                        self.notify_about_missing_validator(
                            &validator,
                            full_client,
                            mining_address,
                        );
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

impl PrometheusMetrics for HbbftEarlyEpochEndManager {
    fn prometheus_metrics(&self, registry: &mut stats::PrometheusRegistry) {
        registry.register_gauge(
            "early_epoch_end_staking_epoch",
            "staking epoch information for early epoch end manager",
            self.current_tracked_epoch_number as i64,
        );

        registry.register_gauge(
            "early_epoch_end_num_flagged_validators",
            "number of validators flagged for missing communication",
            self.flagged_validators.len() as i64,
        );

        for v in self.validators.iter() {
            let is_flagged = self.flagged_validators.contains(v);
            let label_value = v.as_8_byte_string();
            registry.register_gauge_with_other_node_label(
                "early_epoch_end_flag",
                "node has flagged other_node [0-1]",
                label_value.as_str(),
                is_flagged as i64,
            );
        }
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
