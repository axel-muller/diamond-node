use ethereum_types::{H512, Address};
use tiny_keccak::{Hasher};

pub fn public_key_to_address(public_key: H512) -> Address {

    let mut keccak = tiny_keccak::Keccak::v256();

    keccak.update(public_key.as_bytes());
    let mut hashed_pubkey: [u8;64] = [0;64];
    keccak.finalize(&mut hashed_pubkey);

    
    // Keccak256::<[u8; 32]>::keccak256(&self)
    // let hashed_pubkey = <dyn Keccak256<[u8; 32]>>::digest(&public_key.as_bytes());
    let mut address = [0u8; 20];
    
    address.copy_from_slice(&hashed_pubkey[12..]);
    Address::from(address)
}


