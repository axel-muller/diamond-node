use ethereum_types::Address;



pub struct HbbftPeersManagement {

}

impl HbbftPeersManagement {

    pub fn new() -> Self {
        HbbftPeersManagement {}
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

    // if a key gen round fails,
    // we can disconnect from the failing validators, 
    // and only keep the connection to the current ones.
    fn disconnect_old_pending_validators(&mut self) {

    }

}