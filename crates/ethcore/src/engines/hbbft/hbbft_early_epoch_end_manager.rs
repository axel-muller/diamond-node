use std::time::{Instant, Duration};
use super::NodeId;


pub(crate) struct EarlyEpochEndManager {

    /// The current epoch number.
    current_tracked_epoch_number: u64,

    /// epoch manager start up time.
    start_time: Instant,

    /// allowed devp2p warmup time.
    allowed_devp2p_warmup_time: Duration,

    /// public keys of all validators for this epoch.
    validators: Vec<NodeId>,

}


impl EarlyEpochEndManager { 

    // new
    pub fn new(allowed_devp2p_warmup_time: Duration) -> Self {
        Self {
            current_tracked_epoch_number: 0,
            start_time: Instant::now(),
            allowed_devp2p_warmup_time: allowed_devp2p_warmup_time,
            validators: Vec::new(),
        }
    }

    pub fn notify_new_epoch(&mut self, epoch: u64, validators: Vec<NodeId> ) {

        self.current_tracked_epoch_number = epoch;
        self.validators = validators;
        self.start_time = Instant::now();
    }

}