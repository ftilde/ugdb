pub use termion::event::{Event, Key};

pub struct InputEvent {
    event: Event,
}

impl InputEvent {
    pub fn chain(self) -> InputChain {
        InputChain {
            input: Some(self),
        }
    }
}

pub struct InputChain {
    input: Option<InputEvent>,
}

impl InputChain {
    pub fn chain<B: Behavior>(self, behavior: B) -> InputChain {
        if let Some(event) = self.input {
            InputChain {
                input: behavior.input(event),
            }
        } else {
            InputChain {
                input: None,
            }
        }
    }
}

pub trait Behavior {
    fn input(self, InputEvent) -> Option<InputEvent>;
}

pub struct ScrollBehavior<S: Scrollable> {
    scrollable: S,
    scroll_down_event: Event,
    scroll_up_event: Event,
}

impl<S: Scrollable> ScrollBehavior<S> {
    pub fn new(scrollable: S, scroll_down_event: Event, scroll_up_event: Event) -> Self {
        ScrollBehavior {
            scrollable: scrollable,
            scroll_down_event: scroll_down_event,
            scroll_up_event: scroll_up_event,
        }
    }
}

impl<S: Scrollable> Behavior for ScrollBehavior<S> {
    fn input(mut self, input: InputEvent) -> Option<InputEvent> {
        if input.event == self.scroll_up_event {
            self.scrollable.scroll_up();
            None
        } else if input.event == self.scroll_down_event {
            self.scrollable.scroll_down();
            None
        } else {
            Some(input)
        }
    }
}

pub trait Scrollable {
    fn scroll_down(&mut self);
    fn scroll_up(&mut self);
}
