pub use termion::event::{Event, Key, MouseEvent};
use std::collections::HashSet;

#[derive(Eq, PartialEq, Clone)]
pub struct Input {
    pub event: Event,
}

impl Input {
    pub fn new(event: Event) -> Self {
        Input {
            event: event,
        }
    }
    pub fn chain<B: Behavior>(self, behavior: B) -> InputChain {
        let chain_begin = InputChain {
            input: Some(self),
        };
        chain_begin.chain(behavior)
    }
}

pub struct InputChain {
    input: Option<Input>,
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

pub trait ToEvent {
    fn to_event(self) -> Event;
}

impl ToEvent for Key {
    fn to_event(self) -> Event {
        Event::Key(self)
    }
}

impl ToEvent for MouseEvent {
    fn to_event(self) -> Event {
        Event::Mouse(self)
    }
}

impl ToEvent for Event {
    fn to_event(self) -> Event {
        self
    }
}

struct EventSet {
    events: HashSet<Event>,
}

impl EventSet {
    fn new() -> Self {
        EventSet {
            events: HashSet::new(),
        }
    }
    fn insert<E: ToEvent>(&mut self, event: E) {
        self.events.insert(event.to_event());
    }
    fn contains(&self, event: &Event) -> bool {
        self.events.contains(event)
    }
}

pub trait Behavior {
    fn input(self, Input) -> Option<Input>;
}

impl<F: FnOnce(Input)->Option<Input>> Behavior for F {
    fn input(self, input: Input) -> Option<Input> {
        self(input)
    }
}

// ScrollableBehavior -----------------------------------------------

pub struct ScrollBehavior<'a, S: Scrollable + 'a> {
    scrollable: &'a mut S,
    backwards_on: EventSet,
    forwards_on: EventSet,
}

impl<'a, S: Scrollable> ScrollBehavior<'a, S> {
    pub fn new(scrollable: &'a mut S) -> Self {
        ScrollBehavior {
            scrollable: scrollable,
            backwards_on: EventSet::new(),
            forwards_on: EventSet::new(),
        }
    }

    pub fn backwards_on<E: ToEvent>(mut self, event: E) -> Self {
        self.backwards_on.insert(event);
        self
    }
    pub fn forwards_on<E: ToEvent>(mut self, event: E) -> Self {
        self.forwards_on.insert(event);
        self
    }
}

impl<'a, S: Scrollable> Behavior for ScrollBehavior<'a, S> {
    fn input(mut self, input: Input) -> Option<Input> {
        if self.forwards_on.contains(&input.event) {
            self.scrollable.scroll_forwards();
            None
        } else if self.backwards_on.contains(&input.event) {
            self.scrollable.scroll_backwards();
            None
        } else {
            Some(input)
        }
    }
}

pub trait Scrollable {
    fn scroll_backwards(&mut self);
    fn scroll_forwards(&mut self);
}

// WriteBehavior ------------------------------------------

pub struct WriteBehavior<'a, W: Writable+'a> {
    writable: &'a mut W,
}
impl<'a, W: Writable + 'a> WriteBehavior<'a, W> {
    pub fn new(writable: &'a mut W) -> Self {
        WriteBehavior {
            writable: writable,
        }
    }
}

impl<'a, W: Writable + 'a> Behavior for WriteBehavior<'a, W> {
    fn input(mut self, input: Input) -> Option<Input> {
        if let Event::Key(Key::Char(c)) = input.event {
            self.writable.write(c);
            None
        } else {
            Some(input)
        }
    }
}

pub trait Writable {
    fn write(&mut self, c: char);
}

// NavigateBehavior ------------------------------------------------

/*
pub struct NavigateBehavior<'a, N: Navigatable + 'a> {
    navigatable: &'a mut N,
    up_on: EventSet,
    down_on: EventSet,
    left_on: EventSet,
    right_on: EventSet,
}

impl<'a, N: Navigatable + 'a> NavigateBehavior<'a, N> {
    pub fn new(navigatable: &'a mut N) -> Self {
        NavigateBehavior {
            navigatable: navigatable,
            up_on: EventSet::new(),
            down_on: EventSet::new(),
            left_on: EventSet::new(),
            right_on: EventSet::new(),
        }
    }

    pub fn up_on<E: ToEvent>(mut self, event: E) -> Self {
        self.up_on.insert(event);
        self
    }
    pub fn down_on<E: ToEvent>(mut self, event: E) -> Self {
        self.down_on.insert(event);
        self
    }
    pub fn left_on<E: ToEvent>(mut self, event: E) -> Self {
        self.left_on.insert(event);
        self
    }
    pub fn right_on<E: ToEvent>(mut self, event: E) -> Self {
        self.right_on.insert(event);
        self
    }
}

impl<'a, N: Navigatable + 'a> Behavior for NavigateBehavior<'a, N> {
    fn input(mut self, input: Input) -> Option<Input> {
        if self.up_on.contains(&input.event) {
            self.navigatable.move_up();
            None
        } else if self.down_on.contains(&input.event) {
            self.navigatable.move_down();
            None
        } else if self.left_on.contains(&input.event) {
            self.navigatable.move_left();
            None
        } else if self.right_on.contains(&input.event) {
            self.navigatable.move_right();
            None
        } else {
            Some(input)
        }
    }
}
*/

pub trait Navigatable {
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn move_left(&mut self);
    fn move_right(&mut self);
}

// EditBehavior ---------------------------------------------------------

pub struct EditBehavior<'a, E: Editable+'a> {
    editable: &'a mut E,
    up_on: EventSet,
    down_on: EventSet,
    left_on: EventSet,
    right_on: EventSet,
    delete_symbol_on: EventSet,
    remove_symbol_on: EventSet,
    clear_on: EventSet,
}

impl<'a, E: Editable> EditBehavior<'a, E> {
    pub fn new(editable: &'a mut E) -> Self {
        EditBehavior {
            editable: editable,
            up_on: EventSet::new(),
            down_on: EventSet::new(),
            left_on: EventSet::new(),
            right_on: EventSet::new(),
            delete_symbol_on: EventSet::new(),
            remove_symbol_on: EventSet::new(),
            clear_on: EventSet::new(),
        }
    }

    pub fn up_on<T: ToEvent>(mut self, event: T) -> Self {
        self.up_on.insert(event);
        self
    }
    pub fn down_on<T: ToEvent>(mut self, event: T) -> Self {
        self.down_on.insert(event);
        self
    }
    pub fn left_on<T: ToEvent>(mut self, event: T) -> Self {
        self.left_on.insert(event);
        self
    }
    pub fn right_on<T: ToEvent>(mut self, event: T) -> Self {
        self.right_on.insert(event);
        self
    }
    pub fn delete_symbol_on<T: ToEvent>(mut self, event: T) -> Self {
        self.delete_symbol_on.insert(event);
        self
    }
    pub fn remove_symbol_on<T: ToEvent>(mut self, event: T) -> Self {
        self.remove_symbol_on.insert(event);
        self
    }
    pub fn clear_on<T: ToEvent>(mut self, event: T) -> Self {
        self.clear_on.insert(event);
        self
    }
}

impl<'a, E: Editable> Behavior for EditBehavior<'a, E> {
    fn input(mut self, input: Input) -> Option<Input> {
        if self.up_on.contains(&input.event) {
            self.editable.move_up();
            None
        } else if self.down_on.contains(&input.event) {
            self.editable.move_down();
            None
        } else if self.left_on.contains(&input.event) {
            self.editable.move_left();
            None
        } else if self.right_on.contains(&input.event) {
            self.editable.move_right();
            None
        } else if self.delete_symbol_on.contains(&input.event) {
            self.editable.delete_symbol();
            None
        } else if self.remove_symbol_on.contains(&input.event) {
            self.editable.remove_symbol();
            None
        } else if self.clear_on.contains(&input.event) {
            self.editable.clear();
            None
        } else if let Event::Key(Key::Char(c)) = input.event {
            self.editable.write(c);
            None
        } else {
            Some(input)
        }
    }
}

pub trait Editable: Navigatable + Writable {
    fn delete_symbol(&mut self);
    fn remove_symbol(&mut self);
    fn clear(&mut self);
}
