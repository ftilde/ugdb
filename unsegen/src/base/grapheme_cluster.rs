use unicode_segmentation::{
    UnicodeSegmentation,
    Graphemes,
};
use smallvec::SmallVec;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct GraphemeCluster {
    // Invariant: the contents of bytes is always valid utf8!
    bytes: SmallVec<[u8; 16]>,
}

impl GraphemeCluster {
    pub fn as_str<'a>(&'a self) -> &'a str {
        // This is safe because bytes is always valid utf8.
        unsafe {
            ::std::str::from_utf8_unchecked(&self.bytes)
        }
    }

    unsafe fn from_bytes(slice: &[u8]) -> Self {
        let vec = SmallVec::from_slice(slice);
        GraphemeCluster {
            bytes: vec,
        }
    }

    pub unsafe fn from_str_unchecked<S: AsRef<str>>(string: S) -> Self {
        Self::from_bytes(&string.as_ref().as_bytes()[..])
    }

    //TODO: use pub(base) once pub(restricted) is stable
    pub unsafe fn empty() -> Self {
        Self::from_str_unchecked("")
    }

    pub fn space() -> Self {
        unsafe {
            Self::from_str_unchecked(" ")
        }
    }

    pub fn try_from(text: char) -> Result<Self, GraphemeClusterError> {
        Self::from_str(text.to_string().as_ref())
    }

    pub fn all_from_str<'a>(string: &'a str) -> GraphemeClusterIter<'a> {
        GraphemeClusterIter::new(string)
    }

    pub fn width(&self) -> usize {
        ::unicode_width::UnicodeWidthStr::width(self.as_str())
    }
}

pub struct GraphemeClusterIter<'a> {
    graphemes: Graphemes<'a>,
}

impl<'a> GraphemeClusterIter<'a> {
    fn new(string: &'a str) -> Self {
        GraphemeClusterIter {
            graphemes: string.graphemes(true),
        }
    }
}

impl<'a> Iterator for GraphemeClusterIter<'a> {
    type Item = GraphemeCluster;
    fn next(&mut self) -> Option<Self::Item> {
        self.graphemes.next().map(|s| unsafe {
            // We trust the implementation of unicode_segmentation
            GraphemeCluster::from_str_unchecked(s)
        })
    }
}

#[derive(Debug)]
pub enum GraphemeClusterError {
    MultipleGraphemeClusters,
    NoGraphemeCluster,
}

/*
impl TryFrom<char> for GraphemeCluster {
    type Err = GraphemeClusterError;
    fn try_from(text: char) -> Result<Self, Self::Err> {
        let mut clusters = text.graphemes(true);
        let res = if let Some(cluster) = clusters.next() {
            Self::from_str_unchecked(cluster)
        } else {
            Err(GraphemeClusterError::NoGraphemeCluster);
        };
    }
}
*/

impl FromStr for GraphemeCluster {
    type Err = GraphemeClusterError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut clusters = GraphemeCluster::all_from_str(text);
        let res = clusters.next().ok_or(GraphemeClusterError::NoGraphemeCluster);
        if clusters.next().is_none() {
            res
        } else {
            Err(GraphemeClusterError::MultipleGraphemeClusters)
        }
    }
}
