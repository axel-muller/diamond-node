use std::{net::SocketAddr, sync::Arc};

use crate::{
    client::{BlockChainClient, EngineClient, ReservedPeersManagement},
    engines::hbbft::contracts::{
        staking::get_validator_internet_address,
        validator_set::{set_validator_internet_address, staking_by_mining_address},
    },
    ethereum::public_key_to_address::public_key_to_address,
};

use bytes::ToPretty;
use ethereum_types::Address;
use hbbft::NetworkInfo;

use super::{contracts::staking::get_pool_public_key, NodeId};


#[derive(Clone, Debug)]
struct ValidatorConnectionData {
    // mining_address: Address,
    staking_address: Address,
    socket_addr: SocketAddr,
    public_key: NodeId,
    peer_string: String,
    mining_address: Address
}

// impl ValidatorConnectionData {
// }

pub struct HbbftPeersManagement {
    is_syncing: bool,
    own_validator_address: Address,
    last_written_internet_address: Option<SocketAddr>,
    connected_current_pending_validators: Vec<ValidatorConnectionData>,
    connected_current_validators: Vec<ValidatorConnectionData>,
}

impl HbbftPeersManagement {
    pub fn new() -> Self {
        HbbftPeersManagement {
            is_syncing: false,
            own_validator_address: Address::zero(),
            last_written_internet_address: None,
            connected_current_pending_validators: Vec::new(),
            connected_current_validators: Vec::new()
        }
    }

    /// connections are not always required
    fn should_not_connect(&self, client: &dyn BlockChainClient) -> bool {

        // don't do any connections while the network is syncing.
        // the connection is not required yet, and might be outdated.
        // if we don't have a signing key, then we also do not need connections.
        
        return self.own_validator_address.is_zero() || client.is_major_syncing();
    }

    /// if we become a pending validator,
    /// we have to start to communicate with all other
    /// potential future validators.
    /// The transition phase for changing the validator
    /// gives us enough time, so the switch from
    pub fn connect_to_pending_validators(&mut self,  client_arc: &Arc<dyn EngineClient>, pending_validators: &Vec<Address>) -> Result<usize, String> {

        let block_chain_client = client_arc.as_full_client().ok_or("could not retrieve BlockChainClient for connect_to_pending_validators")?;
        if self.should_not_connect(block_chain_client) {
            // warn!(target: "Engine", "connect_to_pending_validators should_not_connect");
            return Ok(0);
        }
        let mut connected_current_pending_validators: Vec<ValidatorConnectionData> = Vec::new();

        // we need go get the nodeID from the smart contract
        for pending_validator_address in pending_validators.iter() {

            if let Some(connection) = self.is_address_connected(pending_validator_address) {
                connected_current_pending_validators.push(connection.clone());
            } else {
                if let Some(connected_validator) = self.connect_to_validator(client_arc.as_ref(), block_chain_client, pending_validator_address) {
                    connected_current_pending_validators.push(connected_validator);
                }
            }
            
        }
    
        // we overwrite here the data.
        // mahybe we should make sure that there are no connected_current_pending_validators
        debug_assert!(self.connected_current_pending_validators.len() == 0);
        self.connected_current_pending_validators = connected_current_pending_validators;

        return Ok(self.connected_current_pending_validators.len());

        
    }

    

    // if we boot up and figure out,
    // that we are a current valudator,
    // then we have to figure out the current devP2P endpoints
    // from the smart contract and connect to them.
    pub fn connect_to_current_validators(
        &mut self,
        network_info: &NetworkInfo<NodeId>,
        client_arc: &Arc<dyn EngineClient>
    ) {
        // todo: iterate over NodeIds, extract the address
        // we do not need to connect to ourself.
        // figure out the IP and port from the contracts
        
        let client = client_arc.as_ref();

        let block_chain_client = match client.as_full_client() {
            Some(full_client) => full_client,
            None => {
                error!(target: "Engine", "could not retrieve BlockChainClient for adding Internet Addresses.");
                return;
            }
        };

        if self.should_not_connect(block_chain_client) {
            // warn!("connect_to_current_validators should_not_connect" );
            return;
        }

        let ids: Vec<&NodeId> = network_info.validator_set().all_ids().collect();
        let start_time = std::time::Instant::now();

        for node in ids.iter() {
 
            let address = public_key_to_address(&node.0);
            if let Some(connected) = self.connect_to_validator(client, block_chain_client, &address) {
                self.connected_current_validators.push(connected);
            }
        }

        warn!(target: "engine", "gathering Endpoint internet adresses took {} ms", (std::time::Instant::now() - start_time).as_millis());
    }

    // if we drop out as a current validator,
    // as well a pending validator, we should drop
    // all reserved connections.
    pub fn disconnect_all_validators(&mut self) {
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
        error!("TODO: disconnect_old_pending_validators");
    }

    pub fn should_announce_own_internet_address(&self, client: &dyn BlockChainClient) -> bool {
        return !client.is_major_syncing() && self.last_written_internet_address.is_none();
    }

    // handles the announcements of the internet address for other peers as blockchain transactions
    pub fn announce_own_internet_address(
        &mut self,
        block_chain_client: &dyn BlockChainClient,
        engine_client: &dyn EngineClient,
        node_address: &Address,
    ) -> Result<(), String> {
        // updates the nodes internet address if the information on the blockchain is outdated.

        // check if the stored internet address differs from our.
        // we do not need to do a special handling for 0.0.0.0, because
        // our IP is always different to that.

        warn!(target: "engine", "checking if internet address needs to be updated.");

        let current_endpoint = if let Some(peers_management) = block_chain_client
            .reserved_peers_management()
            .lock()
            .as_ref()
        {
            if let Some(endpoint) = peers_management.get_devp2p_network_endpoint() {
                endpoint
            } else {
                warn!(target: "engine", "devp2p endpoint not available.");
                return Ok(());
            }
        } else {
            error!(target: "engine", "Unable to lock reserved_peers_management");
            return Err("Unable to lock reserved_peers_management".to_string());
        };
        //let peers_management =

        warn!(target: "engine", "current Endpoint: {:?}", current_endpoint);

        // todo: we can improve performance,
        // by assuming that we are the only one who writes the internet address.
        // so we have to query this data only once, and then we can cache it.
        match get_validator_internet_address(engine_client, &node_address) {
            Ok(validator_internet_address) => {
                warn!(target: "engine", "stored validator address{:?}", validator_internet_address);
                if validator_internet_address.eq(&current_endpoint) {
                    // if the current stored endpoint is the same as the current endpoint,
                    // we don't need to do anything.
                    // but we cache the current endpoint, so we don't have to query the db again.
                    self.last_written_internet_address = Some(current_endpoint);
                    return Ok(());
                }

                match set_validator_internet_address(
                    block_chain_client,
                    &node_address,
                    &current_endpoint,
                ) {
                    Ok(()) => {
                        self.last_written_internet_address = Some(current_endpoint);
                        return Ok(());
                    }
                    Err(err) => {
                        error!(target: "engine", "unable to set validator internet address: {:?}", err);
                        return Err(format!(
                            "unable to set validator internet address: {:?}",
                            err
                        ));
                    }
                }
            }
            Err(err) => {
                error!(target: "engine", "unable to retrieve validator internet address: {:?}", err);
                return Err(format!(
                    "unable to retrieve validator internet address: {:?}",
                    err
                ));
            }
        }
    }

    pub fn set_is_syncing(&mut self, value: bool) {
        self.is_syncing = value;
    }

    pub fn set_validator_address(&mut self, value: Address) {
        self.own_validator_address = value;
    }

    fn connect_to_validator(&self, client: &dyn EngineClient, block_chain_client: &dyn BlockChainClient, mining_address: &Address) -> Option<ValidatorConnectionData> {

        // we do not connect to ourself.
        if mining_address == &self.own_validator_address {
            return None;
        }
        // self.own_validator_address
        match staking_by_mining_address(client, &mining_address) {
            Ok(staking_address) => {

                let node_id =match get_pool_public_key(client, &staking_address) {
                    Ok(pk) => { NodeId(pk) },
                    Err(e) => { 
                        error!("error calling get_pool_public_key: {:?}", e);
                        return None;
                    },
                };

                let result =  connect_to_validator_core(client, block_chain_client, staking_address, &node_id);
                if let Some(mut data) = result {
                    data.mining_address = *mining_address;
                }
            }
            Err(call_error) => {
                error!(target: "engine", "unable to ask for corresponding staking address for given mining address: {:?}", call_error);
            }
        };

        return None;        
    }

    fn is_address_connected(&self, mining_address: &Address) -> Option<&ValidatorConnectionData> {
        

        return self.connected_current_validators.iter().find(|x| x.mining_address == *mining_address);
    }
}

fn connect_to_validator_core(client: &dyn EngineClient, block_chain_client: &dyn BlockChainClient, staking_address: Address, node_id: &NodeId ) -> Option<ValidatorConnectionData> {

    if staking_address.is_zero() {
        // error!(target: "engine", "no IP Address found unable to ask for corresponding staking address for given mining address: {:?}", address);
        return None;
    }

    let socket_addr = match get_validator_internet_address(client, &staking_address)
    {
        Ok(socket_addr) => socket_addr,
        Err(error) => {
            error!(target: "engine", "unable to retrieve internet address for Node ( Public (NodeId): {:?} , staking address: {}, call Error: {:?}", node_id, staking_address, error);
            return None;
        }
    };

    if socket_addr.port() == 0 {
        // we interprate port 0 as NULL.
        return None;
    }
    let ip = socket_addr.to_string();

    warn!(target: "engine", "adding reserved peer: {:?}", ip);

    let mut guard = block_chain_client.reserved_peers_management().lock();

    if let Some(peers_management) =  guard.as_deref_mut() {

        let public_key = node_id.0.to_hex();
        let peer_string = format!("enode://{}@{}", public_key, ip);
        warn!(target: "engine", "adding reserved peer: {}", peer_string);
        if let Err(err) = peers_management.add_reserved_peer(peer_string.clone()) {
            warn!(target: "engine", "failed to adding reserved: {} : {}", peer_string, err);
        }

        return Some(ValidatorConnectionData {
            staking_address: staking_address,
            //mining_address: *address,
            socket_addr: socket_addr,
            peer_string,
            public_key: node_id.clone(),
            mining_address: Address::zero(), // all caller of this function will set this value.
        });
    } else {
        warn!(target: "engine", "no peers management");
        None
    }
    }
