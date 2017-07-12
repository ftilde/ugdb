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

    fn from_bytes(slice: &[u8]) -> Self {
        let vec = SmallVec::from_slice(slice);
        GraphemeCluster {
            bytes: vec,
        }
    }

    pub(in base) fn from_str_unchecked<S: AsRef<str>>(string: S) -> Self {
        Self::from_bytes(&string.as_ref().as_bytes()[..])
    }

    pub(in base) fn empty() -> Self {
        Self::from_str_unchecked("")
    }

    pub(in base) fn merge_zero_width(&mut self, other: Self) {
        assert!(other.width() == 0, "Invalid merge");
        self.bytes.extend_from_slice(&other.bytes[..]);
    }

    pub fn space() -> Self {
        Self::from_str_unchecked(" ")
    }

    pub fn clear(&mut self) {
        *self = Self::space();
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
        self.graphemes.next().map(|s|
            // We trust the implementation of unicode_segmentation
            GraphemeCluster::from_str_unchecked(s)
        )
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
