use ethereum_types::Address;



pub struct HbbftPeersManagement {
    is_syncing: bool,
    own_address: Address
}

impl HbbftPeersManagement {

    pub fn new() -> Self {
        HbbftPeersManagement { is_syncing: false, own_address: Address::zero() }
    }

    /// if we become a pending validator, 
    /// we have to start to communicate with all other 
    /// potential future validators.
    /// The transition phase for changing the validator
    /// gives us enough time, so the switch from 
    pub fn connect_to_pending_validators(&mut self, pending_validators: &Vec<Address>) {

        error!("TODO: connect to pending validators: {:?}", pending_validators);
    }

    // if we boot up and figure out, 
    // that we are a current valudator,
    // then we have to figure out the current devP2P endpoints
    // from the smart contract and connect to them.
    pub fn connect_to_current_validators(&mut self) {
        error!("TODO: connect to current validators:");
    }

    // if we drop out as a current validator,
    // as well a pending validator, we should drop 
    // all reserved connections.
    pub  fn disconnect_all_validators(&mut self) {
        error!("TODO: disconnect all validators");
    }

    pub fn disconnect_pending_validators(&mut self) {

        // disconnect's can be done in any case, 
        // reguardless if we are syncing or not.

        error!("TODO: disconnect_pending_validators");
    }

    // if a key gen round fails,
    // we can disconnect from the failing validators, 
    // and only keep the connection to the current ones.
    fn disconnect_old_pending_validators(&mut self) {

    }

    pub fn set_is_syncing(&mut self, value: bool) {
        self.is_syncing = value;
    }

    pub fn set_own_address(&mut self, value: Address) {
        self.own_address = value;
    }

}