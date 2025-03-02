// Warning: Part of the Consensus protocol, changes need to produce *exactly* the same result or
// block verification will fail. Intentional breaking changes constitute a fork.

use std::collections::HashMap;
use ethereum_types::{Address, U256};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use types::transaction::SignedTransaction;

/// Combining an address with a random U256 seed using XOR
fn U256_xor_address(address: &Address, seed: U256) -> u64
{
    address.to_low_u64_ne() ^ seed.low_u64()
}

/// transactions is expected to be free of duplicates. This is no guarantee that that transactions with the same nonce
/// but different content are present in the given transactions. There is also no guarantee the transactions are sorted
/// by nonce.
/// Avoid using implementations not under our control, avoid adding dependencies on crates or standard library functions
/// which may change in future versions.
/// The implementation needs to be both portable and deterministic.6
fn deterministic_transactions_shuffling(transactions: Vec<SignedTransaction>, seed: U256) -> Vec<SignedTransaction> {

    // Group transactions by sender.
    // * Walk the transactions from first to last
    // * Add unique senders to a vector in the order they appear in the transactions list
    // * Add transactions with unique nonce to a per-sender vector
    //   * Discard transactions with a nonce already existing in the list of transactions
    let mut txs_by_sender: HashMap<_, Vec<SignedTransaction>> = HashMap::new();
    for tx in transactions {
        let sender = tx.sender();
        let entry = txs_by_sender.entry(sender).or_insert_with(Vec::new);
        if entry.iter().any(|existing_tx| existing_tx.tx().nonce == tx.tx().nonce) {
            // Duplicate nonce found, ignore this transaction.
            continue;
        }
        entry.push(tx);
    }

    // For each sender, sort their transactions by nonce (lowest first).
    // Nonces are expected to be unique at this point, guaranteeing portable
    // and deterministic results independent from the sorting algorithm as long as
    // the sorting algorithm works and is implemented correctly.
    for txs in txs_by_sender.values_mut() {
        txs.sort_by_key(|tx| tx.tx().nonce);
    }

    // Randomly shuffle the list of senders in the order they appear in the transactions list.
    // Use a portable and deterministic random number generator where we control the exact implementation.
    // * Seed the random number generator with the given seed value, identical for all validators
    // * Use the random number generator to pick a sender in the unique sender list
    // * Remove the sender from the sender list
    //   * Assure sender removal does not change the sequence of senders
    // * Add the removed sender to a new list
    // * Keep moving randomly selected senders until the original sender list is empty.
    let mut senders: Vec<_> = txs_by_sender.keys().cloned().collect();
    senders.sort_by_key(|address| U256_xor_address(address, seed));

    // Create the final transaction list by iterating over the randomly shuffled senders.
    let mut final_transactions = Vec::new();
    for sender in senders {
        if let Some(mut sender_txs) = txs_by_sender.remove(&sender) {
            // Each sender's transactions are already sorted by nonce.
            final_transactions.append(&mut sender_txs);
        }
    }

    final_transactions
}

// Write a test function to test the address_xor_U256 function with known seeds and addresses and known XOR results
#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::H160;

    #[test]
    fn test_address_xor_U256() {
        let address = H160::from_low_u64_ne(0x1234567890abcdefu64);
        let address_u64 = address.to_low_u64_ne();
        let seed = U256::from(0x1234567890abcdefu64);
        let seed_u64 = seed.low_u64();
        let result = U256_xor_address(&address, seed);
        assert_eq!(result, 0x1234567890abcdef ^ 0x1234567890abcdef);
    }
}