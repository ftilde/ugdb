use base::GraphemeCluster;

#[derive(Copy, Clone, Debug)]
pub enum LineSegment {
    North,
    South,
    East,
    West,
}
impl LineSegment {
    fn to_u8(self) -> u8 {
        match self {
            LineSegment::North => { 0b00000001 }
            LineSegment::South => { 0b00000100 }
            LineSegment::East  => { 0b00010000 }
            LineSegment::West  => { 0b01000000 }
        }
    }
}
pub enum LineType {
    None,
    Thin,
    Thick,
}
impl LineType {
    fn to_u8(self) -> u8 {
        match self {
            LineType::None  => { 0b00 }
            LineType::Thin  => { 0b01 }
            LineType::Thick => { 0b10 }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LineCell {
    components: u8,
}

impl LineCell {
    pub fn empty() -> Self {
        LineCell {
            components: 0,
        }
    }

    pub fn to_grapheme_cluster(self) -> GraphemeCluster {
        GraphemeCluster::try_from(CELL_TO_CHAR[self.components as usize]).expect("CELL_TO_CHAR elements are single clusters")
    }

    pub fn set(&mut self, segment: LineSegment, ltype: LineType) -> &mut Self {
        let segment = segment.to_u8();
        let ltype = ltype.to_u8();
        let other_component_mask = !(segment * 0b11);
        self.components = (self.components & other_component_mask) | segment * ltype;
        self
    }
}

const CELL_TO_CHAR: [char; 256] = [
    ' ', '╵', '╹', '╳',
    '╷', '│', '╿', '╳',
    '╻', '╽', '┃', '╳',
    '╳', '╳', '╳', '╳',
    '╶', '└', '┖', '╳',
    '┌', '├', '┞', '╳',
    '┎', '┟', '┠', '╳',
    '╳', '╳', '╳', '╳',
    '╺', '┕', '┗', '╳',
    '┍', '┝', '┡', '╳',
    '┏', '┢', '┣', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╴', '┘', '┚', '╳',
    '┐', '┤', '┦', '╳',
    '┒', '┧', '┨', '╳',
    '╳', '╳', '╳', '╳',
    '─', '┴', '┸', '╳',
    '┬', '┼', '╀', '╳',
    '┰', '╁', '╂', '╳',
    '╳', '╳', '╳', '╳',
    '╼', '┶', '┺', '╳',
    '┮', '┾', '╄', '╳',
    '┲', '╆', '╊', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╸', '┙', '┛', '╳',
    '┑', '┥', '┩', '╳',
    '┓', '┪', '┫', '╳',
    '╳', '╳', '╳', '╳',
    '╾', '┵', '┹', '╳',
    '┭', '┽', '╃', '╳',
    '┱', '╅', '╉', '╳',
    '╳', '╳', '╳', '╳',
    '━', '┷', '┻', '╳',
    '┯', '┿', '╇', '╳',
    '┳', '╈', '╋', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
    '╳', '╳', '╳', '╳',
];
