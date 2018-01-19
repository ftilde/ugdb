pub use termion::event::{Event, Key, MouseEvent, MouseButton};
use termion::input::{EventsAndRaw, TermReadEventsAndRaw};
use std::collections::HashSet;

use std::io;

pub struct InputIter<R: io::Read> {
    inner: EventsAndRaw<R>,
}

impl<R: io::Read> Iterator for InputIter<R> {
    type Item = Result<Input, io::Error>;

    fn next(&mut self) -> Option<Result<Input, io::Error>> {
        self.inner.next().map(|tuple| tuple.map(|(event, raw)| Input { event: event, raw: raw }))
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Input {
    pub event: Event,
    pub raw: Vec<u8>,
}

impl Input {
    pub fn real_all<R: io::Read>(read: R) -> InputIter<R> {
        InputIter {
            inner: read.events_and_raw()
        }
    }

    pub fn chain<B: Behavior>(self, behavior: B) -> InputChain {
        let chain_begin = InputChain {
            input: Some(self),
        };
        chain_begin.chain(behavior)
    }

    pub fn matches<T: ToEvent>(&self, e: T) -> bool {
        self.event == e.to_event()
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

    pub fn finish(self) -> Option<Input> {
        self.input
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

impl<E: ToEvent, F: FnOnce()> Behavior for (E, F) {
    fn input(self, input: Input) -> Option<Input> {
        let (event, function) = self;
        if event.to_event() == input.event {
            function();
            None
        } else {
            Some(input)
        }
    }
}

pub type OperationResult = Result<(), ()>;
fn pass_on_if_err(res: OperationResult, input: Input) -> Option<Input> {
    if res.is_err() {
        Some(input)
    } else {
        None
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
    fn input(self, input: Input) -> Option<Input> {
        if self.forwards_on.contains(&input.event) {
            pass_on_if_err(self.scrollable.scroll_forwards(), input)
        } else if self.backwards_on.contains(&input.event) {
            pass_on_if_err(self.scrollable.scroll_backwards(), input)
        } else {
            Some(input)
        }
    }
}

pub trait Scrollable {
    fn scroll_backwards(&mut self) -> OperationResult;
    fn scroll_forwards(&mut self) -> OperationResult;
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
    fn input(self, input: Input) -> Option<Input> {
        if let Event::Key(Key::Char(c)) = input.event {
            pass_on_if_err(self.writable.write(c), input)
        } else {
            Some(input)
        }
    }
}

pub trait Writable {
    fn write(&mut self, c: char) -> OperationResult;
}

// NavigateBehavior ------------------------------------------------

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
    fn input(self, input: Input) -> Option<Input> {
        if self.up_on.contains(&input.event) {
            pass_on_if_err(self.navigatable.move_up(), input)
        } else if self.down_on.contains(&input.event) {
            pass_on_if_err(self.navigatable.move_down(), input)
        } else if self.left_on.contains(&input.event) {
            pass_on_if_err(self.navigatable.move_left(), input)
        } else if self.right_on.contains(&input.event) {
            pass_on_if_err(self.navigatable.move_right(), input)
        } else {
            Some(input)
        }
    }
}

pub trait Navigatable {
    fn move_up(&mut self) -> OperationResult;
    fn move_down(&mut self) -> OperationResult;
    fn move_left(&mut self) -> OperationResult;
    fn move_right(&mut self) -> OperationResult;
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
    fn input(self, input: Input) -> Option<Input> {
        if self.up_on.contains(&input.event) {
            pass_on_if_err(self.editable.move_up(), input)
        } else if self.down_on.contains(&input.event) {
            pass_on_if_err(self.editable.move_down(), input)
        } else if self.left_on.contains(&input.event) {
            pass_on_if_err(self.editable.move_left(), input)
        } else if self.right_on.contains(&input.event) {
            pass_on_if_err(self.editable.move_right(), input)
        } else if self.delete_symbol_on.contains(&input.event) {
            pass_on_if_err(self.editable.delete_symbol(), input)
        } else if self.remove_symbol_on.contains(&input.event) {
            pass_on_if_err(self.editable.remove_symbol(), input)
        } else if self.clear_on.contains(&input.event) {
            pass_on_if_err(self.editable.clear(), input)
        } else if let Event::Key(Key::Char(c)) = input.event {
            pass_on_if_err(self.editable.write(c), input)
        } else {
            Some(input)
        }
    }
}

pub trait Editable: Navigatable + Writable {
    fn delete_symbol(&mut self) -> OperationResult;
    fn remove_symbol(&mut self) -> OperationResult;
    fn clear(&mut self) -> OperationResult;
}
