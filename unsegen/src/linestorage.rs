use std::ops::Range;
use std::cmp::{
    min,
};
use std::io;
use std::io::{
    BufReader,
    BufRead,
    SeekFrom,
    Seek,
};
use std::fmt;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
use std::ops:: {
    Add,
    AddAssign,
    Sub,
    SubAssign,
};
use ranges::{
    Bound,
    RangeArgument,
};
// Starting from 0, i.e., treating LineStorage like an array of lines
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct LineIndex(usize);
impl From<usize> for LineIndex {
    fn from(index: usize) -> Self {
        LineIndex(index)
    }
}
impl Into<usize> for LineIndex {
    fn into(self) -> usize {
        let LineIndex(index) = self;
        index
    }
}

impl From<LineNumber> for LineIndex {
    fn from(number: LineNumber) -> Self {
        let raw_number: usize = number.into();
        let raw_index = raw_number - 1;
        raw_index.into()
    }
}
impl Add<usize> for LineIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        let raw_index: usize = self.into();
        (raw_index + rhs).into()
    }
}
impl AddAssign<usize> for LineIndex {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}
impl Sub<usize> for LineIndex {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self {
        let raw_index: usize = self.into();
        (raw_index - rhs).into()
    }
}
impl SubAssign<usize> for LineIndex {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}
impl fmt::Display for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// Starting from 1, i.e., treating LineStorage like lines displayed in an editor
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct LineNumber(usize);
impl From<usize> for LineNumber {
    fn from(number: usize) -> Self {
        LineNumber(number)
    }
}
impl Into<usize> for LineNumber {
    fn into(self) -> usize {
        let LineNumber(number) = self;
        debug_assert!(number > 0, "Invalid LineNumber: Number == 0");
        number
    }
}
impl From<LineIndex> for LineNumber {
    fn from(index: LineIndex) -> Self {
        let raw_index: usize = index.into();
        let raw_number = raw_index + 1;
        raw_number.into()
    }
}
impl Add<usize> for LineNumber {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        let raw_number: usize = self.into();
        (raw_number + rhs).into()
    }
}
impl AddAssign<usize> for LineNumber {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}
impl Sub<usize> for LineNumber {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self {
        let raw_number: usize = self.into();
        debug_assert!(raw_number > rhs, "Overflowing sub on LineNumber: Result would be <= 0");
        (raw_number - rhs).into()
    }
}
impl SubAssign<usize> for LineNumber {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}
impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub trait LineStorage {
    type Line;
    fn view_line<'a, I: Into<LineIndex>>(&'a mut self, pos: I) -> Option<Self::Line>;

    fn view<'a, I: Into<LineIndex>, R: RangeArgument<I>>(&'a mut self, range: R) -> Box<DoubleEndedIterator<Item=(LineIndex, Self::Line)> + 'a>
        where Self: ::std::marker::Sized { // Not exactly sure, why this is needed... we only store a reference?!
        let start: LineIndex = match range.start() { // Always inclusive
            Bound::Unbound => 0.into(),
            Bound::Inclusive(i) => i.into(),
            Bound::Exclusive(i) => i.into()+1,
        };
        let end: LineIndex = match range.end() { // Always exclusive
            Bound::Unbound => {
                //This is not particularly nice, but what can you do...
                let u_start: usize = start.into();
                let mut end = start;
                for i in u_start.. {
                    end += 1;
                    if self.view_line(i).is_none() {
                        break;
                    }
                }
                end
            },
            Bound::Inclusive(i) => i.into()+1,
            Bound::Exclusive(i) => i.into(),
        };
        let urange = Range::<usize> {
            start: start.into(),
            end: end.into(),
        };
        Box::new(LineStorageIterator::<Self::Line, Self>::new(self, urange))
    }
}

struct LineStorageIterator<'a, I: 'a, L: 'a + LineStorage<Line=I>> {
    storage: &'a mut L,
    range: Range<usize>,
}

impl<'a, I: 'a, L: 'a + LineStorage<Line=I>> LineStorageIterator<'a, I, L> {
    fn new(storage: &'a mut L, range: Range<usize>) -> Self {
        LineStorageIterator {
            storage: storage,
            range: range,
        }
    }
}
impl<'a, I: 'a, L: 'a + LineStorage<Line=I>> Iterator for LineStorageIterator<'a, I, L> {
    type Item = (LineIndex, I);
    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let item_index = self.range.start;
            self.range.start += 1;
            if let Some(line) = self.storage.view_line(item_index) {
                Some((item_index.into(), line)) //TODO: maybe we want to treat none differently here?
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, I: 'a, L: 'a + LineStorage<Line=I>> DoubleEndedIterator for LineStorageIterator<'a, I, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let item_index = self.range.end - 1;
            self.range.end -= 1;
            if let Some(line) = self.storage.view_line(item_index) {
                Some((item_index.into(), line)) //TODO: maybe we want to treat none differently here?
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct MemoryLineStorage<L> {
    pub lines: Vec<L>,
}

impl<L> MemoryLineStorage<L> {
    pub fn new() -> Self {
        MemoryLineStorage {
            lines: Vec::new(),
        }
    }

    pub fn num_lines_stored(&self) -> usize {
        return self.lines.len();
    }
}

impl<L: Default> MemoryLineStorage<L> {
    pub fn active_line_mut(&mut self) -> &mut L {
        if self.lines.is_empty() {
            self.lines.push(L::default());
        }
        return self.lines.last_mut().expect("last line");
    }
}


impl<L: Clone> LineStorage for MemoryLineStorage<L> {
    type Line = L;
    fn view_line<'a, I: Into<LineIndex>>(&'a mut self, pos: I) -> Option<L> {
        let upos: usize = pos.into().into();
        self.lines.get(upos).map(|s: &L| s.clone())
    }
}

pub type StringLineStorage = MemoryLineStorage<String>;

impl fmt::Write for StringLineStorage {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut s = s.to_owned();

        while let Some(newline_offset) = s.find('\n') {
            let mut line: String = s.drain(..(newline_offset+1)).collect();
            line.pop(); //Remove the \n
            self.active_line_mut().push_str(&line);
            self.lines.push(String::new());
        }
        self.active_line_mut().push_str(&s);
        Ok(())
    }
}

pub struct FileLineStorage {
    reader: BufReader<File>,
    line_seek_positions: Vec<usize>,
    file_path: PathBuf,
}
impl FileLineStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = try!{File::open(path.as_ref())};
        Ok(FileLineStorage {
            reader: BufReader::new(file),
            line_seek_positions: vec![0],
            file_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn get_file_path(&self) -> &Path {
        &self.file_path.as_path()
    }

    fn get_line(&mut self, index: usize) -> Option<String> {
        let mut buffer = Vec::new();

        loop {
            let current_max_index: usize = self.line_seek_positions[min(index, self.line_seek_positions.len()-1)];
            self.reader.seek(SeekFrom::Start(current_max_index as u64)).expect("seek to line pos");
            let n_bytes = self.reader.read_until(b'\n', &mut buffer).expect("read line");
            if n_bytes == 0 { //We reached EOF
                return None;
            }
            if index < self.line_seek_positions.len() { //We found the desired line
                let mut string = String::from_utf8_lossy(&buffer).into_owned();
                if string.as_str().bytes().last().unwrap_or(b'_') == b'\n' {
                    string.pop();
                }
                return Some(string);
            }
            self.line_seek_positions.push(current_max_index + n_bytes);
        }
    }
}

impl LineStorage for FileLineStorage {
    type Line = String;
    fn view_line<'a, I: Into<LineIndex>>(&'a mut self, pos: I) -> Option<String> {
        self.get_line(pos.into().into())
    }
}
