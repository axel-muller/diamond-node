use client::traits::{EngineClient, TransactionRequest};
use engines::{
    hbbft::{
        contracts::{
            keygen_history::{
                engine_signer_to_synckeygen, get_current_key_gen_round, has_acks_of_address_data,
                key_history_contract, part_of_address, PublicWrapper, KEYGEN_HISTORY_ADDRESS,
            },
            staking::get_posdao_epoch,
            validator_set::{
                get_pending_validator_key_generation_mode, get_validator_pubkeys, KeyGenMode,
                ValidatorType,
            },
        },
        utils::bound_contract::CallError,
    },
    signer::EngineSigner,
};
use ethereum_types::{Address, U256};
use itertools::Itertools;
use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::Arc};
use types::ids::BlockId;

pub struct KeygenTransactionSender {
    last_keygen_mode: KeyGenMode,
    keygen_mode_counter: u64,
}

enum ShouldSendKeyAnswer {
    // no, we are not in this key gen phase.
    NoNotThisKeyGenMode,
    // no, we are waiting to send key later.
    NoWaiting,
    // yes, keys should be send now.
    Yes,
}

static KEYGEN_TRANSACTION_SEND_DELAY: u64 = 3;
static KEYGEN_TRANSACTION_RESEND_DELAY: u64 = 10;

impl KeygenTransactionSender {
    pub fn new() -> Self {
        KeygenTransactionSender {
            last_keygen_mode: KeyGenMode::Other,
            keygen_mode_counter: 0,
        }
    }

    fn should_send(
        &mut self,
        client: &dyn EngineClient,
        mining_address: &Address,
        mode_to_check: KeyGenMode,
    ) -> Result<ShouldSendKeyAnswer, CallError> {
        let keygen_mode = get_pending_validator_key_generation_mode(client, mining_address)?;
        if keygen_mode == mode_to_check {
            if self.last_keygen_mode == mode_to_check {
                self.keygen_mode_counter += 1;
                if self.keygen_mode_counter == KEYGEN_TRANSACTION_SEND_DELAY {
                    return Ok(ShouldSendKeyAnswer::Yes);
                } else if self.keygen_mode_counter > KEYGEN_TRANSACTION_SEND_DELAY {
                    // Part should have been sent already,
                    // give the chain time to include the transaction before trying a re-send.
                    if (self.keygen_mode_counter - KEYGEN_TRANSACTION_SEND_DELAY)
                        % KEYGEN_TRANSACTION_RESEND_DELAY
                        == 0
                    {
                        return Ok(ShouldSendKeyAnswer::Yes);
                    }
                } else {
                    return Ok(ShouldSendKeyAnswer::NoWaiting);
                }
            } else {
                self.last_keygen_mode = mode_to_check;
                self.keygen_mode_counter = 1;
                return Ok(ShouldSendKeyAnswer::NoWaiting);
            }
        }
        return Ok(ShouldSendKeyAnswer::NoNotThisKeyGenMode);
    }

    fn should_send_part(
        &mut self,
        client: &dyn EngineClient,
        mining_address: &Address,
    ) -> Result<ShouldSendKeyAnswer, CallError> {
        self.should_send(client, mining_address, KeyGenMode::WritePart)
    }

    fn should_send_ack(
        &mut self,
        client: &dyn EngineClient,
        mining_address: &Address,
    ) -> Result<ShouldSendKeyAnswer, CallError> {
        self.should_send(client, mining_address, KeyGenMode::WriteAck)
    }

    /// Returns a collection of transactions the pending validator has to submit in order to
    /// complete the keygen history contract data necessary to generate the next key and switch to the new validator set.
    pub fn send_keygen_transactions(
        &mut self,
        client: &dyn EngineClient,
        signer: &Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
    ) -> Result<(), CallError> {
        // If we have no signer there is nothing for us to send.
        let address = match signer.read().as_ref() {
            Some(signer) => signer.address(),
            None => {
                warn!(target: "engine", "Could not send keygen transactions, because signer module could not be retrieved");
                return Err(CallError::ReturnValueInvalid);
            }
        };

        let full_client = client.as_full_client().ok_or(CallError::NotFullClient)?;

        // If the chain is still syncing, do not send Parts or Acks.
        if full_client.is_major_syncing() {
            debug!(target:"engine", "skipping sending key gen transaction, because we are syncing");
            return Ok(());
        }

        trace!(target:"engine", " get_validator_pubkeys...");

        let vmap = get_validator_pubkeys(&*client, BlockId::Latest, ValidatorType::Pending)?;
        let pub_keys: BTreeMap<_, _> = vmap
            .values()
            .map(|p| (*p, PublicWrapper { inner: p.clone() }))
            .collect();

        let pub_leys_arc = Arc::new(pub_keys);

        //let pub_key_len = pub_keys.len();
        // if synckeygen creation fails then either signer or validator pub keys are problematic.
        // Todo: We should expect up to f clients to write invalid pub keys. Report and re-start pending validator set selection.
        let (mut synckeygen, part) = engine_signer_to_synckeygen(signer, pub_leys_arc.clone())
            .map_err(|e| {
                warn!(target:"engine", "engine_signer_to_synckeygen pub keys count {:?} error {:?}", pub_leys_arc.len(), e);
                pub_leys_arc.iter().for_each(|(k, v)| {
                    warn!(target:"engine", "pub key {}", k);
                });
                CallError::ReturnValueInvalid
            })?;

        // If there is no part then we are not part of the pending validator set and there is nothing for us to do.
        let part_data = match part {
            Some(part) => part,
            None => {
                warn!(target:"engine", "no part to write.");
                return Err(CallError::ReturnValueInvalid);
            }
        };

        let upcoming_epoch = get_posdao_epoch(client, BlockId::Latest)? + 1;
        trace!(target:"engine", "preparing to send PARTS for upcoming epoch: {}", upcoming_epoch);

        // Check if we already sent our part.
        match self.should_send_part(client, &address)? {
            ShouldSendKeyAnswer::Yes => {
                let serialized_part = match bincode::serialize(&part_data) {
                    Ok(part) => part,
                    Err(e) => {
                        warn!(target:"engine", "could not serialize part: {:?}", e);
                        return Err(CallError::ReturnValueInvalid);
                    }
                };
                let serialized_part_len = serialized_part.len();
                let current_round = get_current_key_gen_round(client)?;
                let write_part_data = key_history_contract::functions::write_part::call(
                    upcoming_epoch,
                    current_round,
                    serialized_part,
                );

                // the required gas values have been approximated by
                // experimenting and it's a very rough estimation.
                // it can be further fine tuned to be just above the real consumption.
                // ACKs require much more gas,
                // and usually run into the gas limit problems.
                let gas: usize = serialized_part_len * 800 + 100_000;

                let part_transaction =
                    TransactionRequest::call(*KEYGEN_HISTORY_ADDRESS, write_part_data.0)
                        .gas(U256::from(gas))
                        .nonce(full_client.nonce(&address, BlockId::Latest).unwrap())
                        .gas_price(U256::from(10000000000u64));
                full_client
                    .transact_silently(part_transaction)
                    .map_err(|e| {
                        warn!(target:"engine", "could not transact_silently: {:?}", e);
                        CallError::ReturnValueInvalid
                    })?;

                trace!(target:"engine", "PART Transaction send.");
                return Ok(());
            }
            ShouldSendKeyAnswer::NoWaiting => {
                // we are waiting for parts to get written,
                // we do not need to continue any further with current key gen history.
                return Ok(());
            }
            ShouldSendKeyAnswer::NoNotThisKeyGenMode => {}
        }

        trace!(target:"engine", "checking for acks...");
        // Return if any Part is missing.
        let mut acks = Vec::new();
        for v in vmap.keys().sorted() {
            acks.push(
				match part_of_address(&*client, *v, &vmap, &mut synckeygen, BlockId::Latest) {
					Ok(part_result) => {
						match part_result {
							    Some(ack) => ack,
							    None => {
							        trace!(target:"engine", "could not retrieve part for {}", *v);
							        return Ok(());
							    }
							}
					}
					Err(err) => {
						error!(target:"engine", "could not retrieve part for {} call failed. Error: {:?}", *v, err);
						return Err(err);
					}
				}
            );
        }

        trace!(target:"engine", "has_acks_of_address_data: {:?}", has_acks_of_address_data(client, address));

        // Now we are sure all parts are ready, let's check if we sent our Acks.
        match self.should_send_ack(client, &address)? {
            ShouldSendKeyAnswer::Yes => {
                let mut serialized_acks = Vec::new();
                let mut total_bytes_for_acks = 0;

                for ack in acks {
                    let ack_to_push = match bincode::serialize(&ack) {
                        Ok(serialized_ack) => serialized_ack,
                        Err(_) => return Err(CallError::ReturnValueInvalid),
                    };
                    total_bytes_for_acks += ack_to_push.len();
                    serialized_acks.push(ack_to_push);
                }
                let current_round = get_current_key_gen_round(client)?;
                let write_acks_data = key_history_contract::functions::write_acks::call(
                    upcoming_epoch,
                    current_round,
                    serialized_acks,
                );

                // the required gas values have been approximated by
                // experimenting and it's a very rough estimation.
                // it can be further fine tuned to be just above the real consumption.
                let gas = total_bytes_for_acks * 850 + 200_000;
                trace!(target: "engine","acks-len: {} gas: {}", total_bytes_for_acks, gas);

                let acks_transaction =
                    TransactionRequest::call(*KEYGEN_HISTORY_ADDRESS, write_acks_data.0)
                        .gas(U256::from(gas))
                        .nonce(full_client.nonce(&address, BlockId::Latest).unwrap())
                        .gas_price(U256::from(10000000000u64));
                full_client
                    .transact_silently(acks_transaction)
                    .map_err(|_| CallError::ReturnValueInvalid)?;
            }
            _ => {}
        }

        Ok(())
    }
}
