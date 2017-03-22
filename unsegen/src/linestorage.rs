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
use std::fs::{File};
use std::path::{
    Path,
    PathBuf,
};

pub trait LineStorage {
    fn view_line<'a>(&'a mut self, pos: usize) -> Option<String>;

    fn view<'a>(&'a mut self, range: Range<usize>) -> Box<DoubleEndedIterator<Item=(usize, String)> + 'a>
        where Self: ::std::marker::Sized { // Not exactly sure, why this is needed... we only store a reference?!
        Box::new(LineStorageIterator::new(self, range))
    }
}

struct LineStorageIterator<'a, L: 'a + LineStorage> {
    storage: &'a mut L,
    range: Range<usize>,
}

impl<'a, L: 'a + LineStorage> LineStorageIterator<'a, L> {
    fn new(storage: &'a mut L, range: Range<usize>) -> Self {
        LineStorageIterator {
            storage: storage,
            range: range,
        }
    }
}
impl<'a, L: 'a + LineStorage> Iterator for LineStorageIterator<'a, L> {
    type Item = (usize, String);
    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let item_index = self.range.start;
            self.range.start += 1;
            if let Some(line) = self.storage.view_line(item_index) {
                Some((item_index, line)) //TODO: maybe we want to treat none differently here?
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, L: 'a + LineStorage> DoubleEndedIterator for LineStorageIterator<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range.start < self.range.end {
            let item_index = self.range.end - 1;
            self.range.end -= 1;
            if let Some(line) = self.storage.view_line(item_index) {
                Some((item_index, line)) //TODO: maybe we want to treat none differently here?
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct MemoryLineStorage {
    lines: Vec<String>,
}

impl MemoryLineStorage {
    pub fn new() -> Self {
        MemoryLineStorage {
            lines: Vec::new(),
        }
    }

    pub fn active_line_mut(&mut self) -> &mut String {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        return self.lines.last_mut().expect("last line");
    }

    pub fn num_lines_stored(&self) -> usize {
        return self.lines.len();
    }
}

impl ::std::fmt::Write for MemoryLineStorage {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
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

impl LineStorage for MemoryLineStorage {
    fn view_line<'a>(&'a mut self, pos: usize) -> Option<String> {
        self.lines.get(pos).map(|s| s.clone())
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
    fn view_line<'a>(&'a mut self, pos: usize) -> Option<String> {
        self.get_line(pos)
    }
}


