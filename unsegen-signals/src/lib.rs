extern crate libc;
extern crate unsegen;

use unsegen::input::{
    Behavior,
    Input,
    Event,
    Key,
    ToEvent,
};

use libc::{
    kill,
    getpid,
};

use std::collections::HashMap;

type CSig = libc::c_int;

pub trait Signal {
    fn to_sig() -> CSig;
    fn default_event() -> Event;
}

pub struct SIGINT;
pub struct SIGTSTP;
pub struct SIGQUIT;

impl Signal for SIGINT {
    fn to_sig() -> CSig {
        libc::SIGINT
    }
    fn default_event() -> Event {
        Event::Key(Key::Ctrl('c'))
    }
}

impl Signal for SIGTSTP {
    fn to_sig() -> CSig {
        libc::SIGTSTP
    }
    fn default_event() -> Event {
        Event::Key(Key::Ctrl('z'))
    }
}

impl Signal for SIGQUIT {
    fn to_sig() -> CSig {
        libc::SIGQUIT
    }
    fn default_event() -> Event {
        Event::Key(Key::Ctrl('\\'))
    }
}

// Passes all inputs through to the modelled terminal
pub struct SignalBehavior{
    mapping: HashMap<Event, CSig>,
}

impl SignalBehavior {
    pub fn new() -> Self {
        SignalBehavior {
            mapping: HashMap::new(),
        }
    }

    pub fn sig_on<S: Signal, E: ToEvent>(mut self, e: E) -> Self {
        self.mapping.insert(e.to_event(), S::to_sig());
        self
    }

    pub fn sig_default<S: Signal>(self) -> Self {
        self.sig_on::<S, Event>(S::default_event())
    }
}

impl<'a> Behavior for SignalBehavior {
    fn input(self, i: Input) -> Option<Input> {
        if let Some(sig) = self.mapping.get(&i.event) {
            unsafe { kill(getpid(), *sig); }
            None
        } else {
            Some(i)
        }
    }
}
