// Warning: Part of the Consensus protocol, changes need to produce *exactly* the same result or
// block verification will fail. Intentional breaking changes constitute a fork.

use ethereum_types::{Address, U256};
use std::collections::HashMap;
use types::transaction::SignedTransaction;

/// Combining an address with a random U256 seed using XOR, using big-endian byte ordering always.
fn address_xor_u256(address: &Address, seed: U256) -> Address {
    // Address bytes are always assuming big-endian order.
    let address_bytes = address.as_bytes();

    // Explicitly convert U256 to big endian order
    let mut seed_bytes = [0u8; 32];
    seed.to_big_endian(&mut seed_bytes);

    // Byte-wise XOR, constructing a new, big-endian array
    let mut result = [0u8; 20];
    for i in 0..20 {
        result[i] = address_bytes[i] ^ seed_bytes[i];
    }

    // Construct a new Address from the big-endian array
    Address::from(result)
}

/// transactions is expected to be free of duplicates. This is no guarantee that that transactions with the same nonce
/// but different content are present in the given transactions. There is also no guarantee the transactions are sorted
/// by nonce.
/// Avoid using implementations not under our control, avoid adding dependencies on crates or standard library functions
/// which may change in future versions.
/// The implementation needs to be both portable and deterministic.6
fn deterministic_transactions_shuffling(
    transactions: Vec<SignedTransaction>,
    seed: U256,
) -> Vec<SignedTransaction> {
    // Group transactions by sender.
    // * Walk the transactions from first to last
    // * Add unique senders to a vector in the order they appear in the transactions list
    // * Add transactions with unique nonce to a per-sender vector
    //   * Discard transactions with a nonce already existing in the list of transactions
    let mut txs_by_sender: HashMap<_, Vec<SignedTransaction>> = HashMap::new();
    for tx in transactions {
        let sender = tx.sender();
        let entry = txs_by_sender.entry(sender).or_insert_with(Vec::new);
        if entry
            .iter()
            .any(|existing_tx| existing_tx.tx().nonce == tx.tx().nonce)
        {
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
    senders.sort_by_key(|address| address_xor_u256(address, seed));

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

// Write a test function to test the address_xor_u256 function with known seeds and addresses and known XOR results
#[cfg(test)]
mod tests {
    use super::*;
    // Convert to bytes in big-endian order.
    fn u64_to_32_bytes_be(n: u64) -> [u8; 32] {
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&n.to_be_bytes());
        result
    }

    #[test]
    fn test_address_xor_u256() {
        let value_as_bytes = u64_to_32_bytes_be(0x1234567890abcdefu64);
        let address = Address::from_slice(&value_as_bytes[..20]);
        let seed = U256::from_big_endian(&value_as_bytes);
        let result = address_xor_u256(&address, seed);
        assert_eq!(
            result,
            Address::from_slice(&u64_to_32_bytes_be(0x1234567890abcdef ^ 0x1234567890abcdef)[..20])
        );
    }
}
