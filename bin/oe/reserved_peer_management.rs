use std::{net::SocketAddr, sync::Weak, collections::BTreeSet};

use ethcore::client::ReservedPeersManagement;

pub(crate) struct ReservedPeersWrapper {
    manage_network: Weak<dyn sync::ManageNetwork>,
    current_reserved_peers: BTreeSet<String>
}

impl ReservedPeersWrapper {
    pub fn new(manage_network: Weak<dyn sync::ManageNetwork>) -> Self {
        ReservedPeersWrapper { manage_network, current_reserved_peers: BTreeSet::new() }
    }
}

impl ReservedPeersManagement for ReservedPeersWrapper {
    fn add_reserved_peer(&mut self, peer: String) -> Result<(), String> {


        if self.current_reserved_peers.contains(&peer) {
            return Ok(());
        }

        match self.manage_network.upgrade() {
            Some(sync_arc) => sync_arc.add_reserved_peer(peer),
            None => Err("ManageNetwork instance not available.".to_string()),
        }
    }

       /// remove reserved peer
       fn remove_reserved_peer(&mut self, peer: String) -> Result<(), ()>  {
        if self.current_reserved_peers.contains(&peer) {
            match self.manage_network.upgrade() {
                Some(sync_arc) => {
                    let remove_result = sync_arc.remove_reserved_peer(peer);
                    return remove_result.map_err(|_e| ());
                },
                None => {
                    warn!("ManageNetwork instance not available.");
                    return Err(());
                }
            }
        }

        return Err(());
        
       }


    fn get_reserved_peers(&self) -> &BTreeSet<String> {
        &self.current_reserved_peers
    }

    fn disconnect_others_than(&mut self, keep_list: BTreeSet<String>) -> usize {

        let reserved_peers_to_disconnect : Vec<String> = self.current_reserved_peers.iter().filter_map(|p| if keep_list.contains(p) {None} else {Some(p.clone())}).collect();

        let mut disconnected = 0;
        for reserved_peer in reserved_peers_to_disconnect {
            if self.remove_reserved_peer(reserved_peer).is_ok() {
                disconnected += 1;
            }
        }

        return disconnected;
    }


    /// Returns the devp2p network endpoint IP and Port information that is used to communicate with other peers.
    fn get_devp2p_network_endpoint(&self) -> Option<SocketAddr> {
        match self.manage_network.upgrade() {
            Some(sync_arc) => sync_arc.get_devp2p_network_endpoint(),
            None => {
                warn!("ManageNetwork instance not available.");
                None
            }
        }
    }
}
