pub mod jsonviewer;
pub mod linelabel;
pub mod lineedit;
pub mod promptline;
pub mod logviewer;
pub mod pager;
pub mod table;

pub use self::jsonviewer::*;
pub use self::linelabel::*;
pub use self::lineedit::*;
pub use self::promptline::*;
pub use self::logviewer::*;
pub use self::pager::*;
pub use self::table::*;

fn count_grapheme_clusters(text: &str) -> u32 {
    use ::unicode_segmentation::UnicodeSegmentation;
    text.graphemes(true).count() as u32
}
