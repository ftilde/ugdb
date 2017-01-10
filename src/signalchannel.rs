
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use nix::sys::signal;

struct TakeableChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: Option<mpsc::Receiver<T>>,
}
impl<T> TakeableChannel<T> {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        TakeableChannel {
            sender: sender,
            receiver: Some(receiver),
        }
    }
}

#[derive(Debug)]
pub enum SignalReceiverSetupError {
    Taken,
    StaticLockPoisoned,
    SigActionFailed(::nix::Error)
}

lazy_static! {
    static ref SIGNAL_CHANNEL_INIT: Arc<Mutex<TakeableChannel<signal::Signal>>> = Arc::new(Mutex::new(TakeableChannel::new()));
}

extern fn handle_resize(signo: ::nix::c_int) {
    SIGNAL_CHANNEL_INIT.lock().expect("Lock poisoned").sender.send(signal::Signal::from_c_int(signo).expect("signal from signo")).expect("Sent resize event");
}

pub fn setup_signal_receiver() -> Result<mpsc::Receiver<signal::Signal>, SignalReceiverSetupError> {
    // TODO: offer to specify a signal set to be received
    let sig_action = signal::SigAction::new(
        signal::SigHandler::Handler(handle_resize),
        signal::SaFlags::empty(),
        signal::SigSet::empty());
    let sigaction_res = unsafe { signal::sigaction(signal::SIGWINCH, &sig_action) };
    if let Err(err) = sigaction_res {
        return Err(SignalReceiverSetupError::SigActionFailed(err))
    }
    if let Ok(ref mut channel_guard) = SIGNAL_CHANNEL_INIT.lock() {
        channel_guard.receiver.take().ok_or(SignalReceiverSetupError::Taken)
    } else {
        Err(SignalReceiverSetupError::StaticLockPoisoned)
    }
}
