use super::super::{
    Demand,
    Demand2D,
    LineIndex,
    LineStorage,
    MemoryLineStorage,
    RenderingHints,
    Widget,
};
use base::{
    Cursor,
    Window,
    WrappingMode,
};
use input::{
    Scrollable,
    OperationResult,
};

pub struct LogViewer {
    pub storage: MemoryLineStorage<String>,
    scrollback_position: Option<usize>,
    scroll_step: usize,
}

impl LogViewer {
    pub fn new() -> Self {
        LogViewer {
            storage: MemoryLineStorage::new(),
            scrollback_position: None,
            scroll_step: 1,
        }
    }

    fn current_line(&self) -> usize {
        self.scrollback_position.unwrap_or(self.storage.num_lines_stored().checked_sub(1).unwrap_or(0))
    }
}

impl Widget for LogViewer {
    fn space_demand(&self) -> Demand2D {
        Demand2D {
            width: Demand::at_least(1),
            height: Demand::at_least(1)
        }
    }
    fn draw(&mut self, mut window: Window, _: RenderingHints) {
        let height = window.get_height() as usize;
        if height == 0 {
            return;
        }

        // TODO: This does not work well when lines are wrapped, but we may want scrolling farther
        // than 1 line per event
        // self.scroll_step = ::std::cmp::max(1, height.checked_sub(1).unwrap_or(1));

        let y_start = height - 1;
        let mut cursor = Cursor::new(&mut window)
            .position(0, y_start as i32)
            .wrapping_mode(WrappingMode::Wrap);
        let end_line = LineIndex(self.current_line());
        let start_line = LineIndex(end_line.0.checked_sub(height).unwrap_or(0));
        for (_, line) in self.storage.view(start_line..(end_line+1)).rev() {
            let num_auto_wraps = cursor.num_expected_wraps(&line) as i32;
            cursor.move_by(0, -num_auto_wraps);
            cursor.writeln(&line);
            cursor.move_by(0, -num_auto_wraps-2);
        }
    }
}

impl Scrollable for LogViewer {
    fn scroll_forwards(&mut self) -> OperationResult {
        let current = self.current_line();
        let candidate = current + self.scroll_step;
        self.scrollback_position = if candidate < self.storage.num_lines_stored() {
            Some(candidate)
        } else {
            None
        };
        if self.scrollback_position.is_some() {
            Ok(())
        } else {
            Err(())
        }
    }
    fn scroll_backwards(&mut self) -> OperationResult {
        let current = self.current_line();
        let op_res = if current != 0 {
            Ok(())
        } else {
            Err(())
        };
        self.scrollback_position = Some(current.checked_sub(self.scroll_step).unwrap_or(0));
        op_res
    }
}
