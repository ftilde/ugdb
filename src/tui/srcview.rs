use unsegen::{
    Demand,
    FileLineStorage,
    HorizontalLayout,
    Key,
    ScrollBehavior,
    SeparatingStyle,
    Widget,
    Window,
};
use unsegen::widgets::{
    Pager,
    SyntectHighLighter,
};
use input::{
    Input,
};
use syntect::highlighting::{
    Theme,
};
use syntect::parsing::{
    SyntaxSet,
};
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use gdbmi;

#[derive(Debug)]
pub enum PagerShowError {
    CouldNotOpenFile(PathBuf, io::Error),
    LineDoesNotExist(usize),
}

pub struct SrcView<'a> {
    highlighting_theme: &'a Theme,
    syntax_set: SyntaxSet,
    file_viewer: Pager<FileLineStorage, SyntectHighLighter<'a>>,
    layout: HorizontalLayout,
}

impl<'a> SrcView<'a> {
    pub fn new(highlighting_theme: &'a Theme) -> Self {
        SrcView {
            highlighting_theme: highlighting_theme,
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            file_viewer: Pager::new(),
            layout: HorizontalLayout::new(SeparatingStyle::Draw('|')),
        }
    }
    pub fn show_in_file_viewer<P: AsRef<Path>>(&mut self, path: P, line: usize) -> Result<(), PagerShowError> {
        let need_to_reload = if let Some(ref content) = self.file_viewer.content {
            content.storage.get_file_path() != path.as_ref()
        } else {
            true
        };
        if need_to_reload {
            let path_ref = path.as_ref();
            try!{self.load_in_file_viewer(path_ref).map_err(|e| PagerShowError::CouldNotOpenFile(path_ref.to_path_buf(), e))};
        }
        self.file_viewer.go_to_line(line).map_err(|_| PagerShowError::LineDoesNotExist(line))
    }

    pub fn load_in_file_viewer<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file_storage = try!{FileLineStorage::new(path.as_ref())};
        let syntax = self.syntax_set.find_syntax_for_file(path.as_ref())
            .expect("file IS openable, see file storage")
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        self.file_viewer.load(file_storage, SyntectHighLighter::new(syntax, self.highlighting_theme));
        Ok(())
    }
    pub fn event(&mut self, event: Input, _ /*gdb*/: &mut gdbmi::GDB) {
        event.chain(ScrollBehavior::new(&mut self.file_viewer)
                    .forwards_on(Key::PageDown)
                    .backwards_on(Key::PageUp)
                   );
    }
}

impl<'a> Widget for SrcView<'a> {
    fn space_demand(&self) -> (Demand, Demand) {
        let widgets: Vec<&Widget> = vec![&self.file_viewer];
        self.layout.space_demand(widgets.as_slice())
    }
    fn draw(&mut self, window: Window) {
        let mut widgets: Vec<&mut Widget> = vec![&mut self.file_viewer];
        self.layout.draw(window, &mut widgets)
    }
}

