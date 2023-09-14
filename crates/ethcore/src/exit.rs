use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

#[derive(Debug)]
/// Status used to exit or restart the program.
pub struct ExitStatus {
    /// Whether the program panicked.
    panicking: bool,
    /// Whether the program should exit.
    should_exit: bool,
}

impl ExitStatus {
    pub fn new() -> Self {
        ExitStatus {
            panicking: false,
            should_exit: false,
        }
    }

    pub fn new_panicking() -> Self {
        ExitStatus {
            panicking: true,
            should_exit: true,
        }
    }

    pub fn new_should_exit() -> Self {
        ExitStatus {
            panicking: false,
            should_exit: true,
        }
    }

    /// Signals the overlaying system to perform a shutdown.
    pub fn do_shutdown(self: &mut Self) {
        self.should_exit = true;
    }

    pub fn should_exit(self: &Self) -> bool {
        return self.should_exit;
    }

    pub fn is_panicking(self: &Self) -> bool {
        return self.panicking;
    }
}

/// Shutdown Manager allows engines to signal the system to shutdown.
pub struct ShutdownManager {
    exit_mutex: Arc<(Mutex<ExitStatus>, Condvar)>,
}

impl ShutdownManager {
    /// get's a Null Object that does not interact with the system at all.
    pub fn null() -> Self {
        return ShutdownManager {
            exit_mutex: Arc::new((
                Mutex::new(ExitStatus {
                    panicking: false,
                    should_exit: false,
                }),
                Condvar::new(),
            )),
        };
    }

    pub fn new(mutex_original: &Arc<(Mutex<ExitStatus>, Condvar)>) -> Self {
        return ShutdownManager {
            exit_mutex: mutex_original.clone(),
        };
    }

    pub fn demand_shutdown(self: &Self) {
        todo!();
        //self.exit_mutex.lock().do_shutdown();
        //exit_mutex.
        // e.1.notify_all();
    }
}
