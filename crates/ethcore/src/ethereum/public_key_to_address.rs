use ethereum_types::{Address, H512};
use tiny_keccak::Hasher;

/// Returns the Ethereum Address for the given ECDSA public key.
pub fn public_key_to_address(public_key: &H512) -> Address {
    let mut keccak = tiny_keccak::Keccak::v256();

    keccak.update(public_key.as_bytes());
    let mut hashed_pubkey: [u8; 32] = [0; 32];
    keccak.finalize(&mut hashed_pubkey);

    // Keccak256::<[u8; 32]>::keccak256(&self)
    // let hashed_pubkey = <dyn Keccak256<[u8; 32]>>::digest(&public_key.as_bytes());
    let mut address = [0u8; 20];

    address.copy_from_slice(&hashed_pubkey[12..]);
    Address::from(address)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use ethereum_types::{H512, Address};
    use crate::ethereum::public_key_to_address::public_key_to_address;

    #[test]
    fn test_public_key_to_address() {
        let public_key :H512 = H512::from_str("b461fffc33fed96f525b6791c9f628d92d9d06220d3150b46246ab2f107f4459139533435e39629a896d3e8fc9bbce1d63fc652429d1e2ca261cd21f119cae87").expect("invalid public key");
        let expected_address =
            Address::from_str("ffd50c8d035462d15c06413d9dff638bab201180").expect("invalid address");

        let address = public_key_to_address(&public_key);

        assert_eq!(address, expected_address);
    }
}