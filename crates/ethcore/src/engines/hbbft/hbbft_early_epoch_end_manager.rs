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