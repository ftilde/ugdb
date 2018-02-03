pub mod boxdrawing;

use base::{CursorTarget, Window};
use base::basic_types::*;
use widget::{Widget, ColDemand, RowDemand, Demand2D, RenderingHints};
use widget::layouts::{layout_linearly};
use input::{Behavior, Input, Navigatable, OperationResult};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::btree_map;
use std::convert::From;
use std::ops::Range;
use std::cmp::{min, max};
use self::boxdrawing::{LineSegment, LineType, LineCell};


pub trait Container<P: ?Sized> : Widget {
    fn input(&mut self, input: Input, parameters: &mut P) -> Option<Input>;
}

pub trait ContainerProvider {
    type Parameters;
    type Index: Clone + PartialEq;
    fn get<'a, 'b: 'a>(&'b self, index: &'a Self::Index) -> &'b Container<Self::Parameters>;
    fn get_mut<'a, 'b: 'a>(&'b mut self, index: &'a Self::Index) -> &'b mut Container<Self::Parameters>;
    const DEFAULT_CONTAINER: Self::Index;
}

pub struct ApplicationBehavior<'a, 'b, 'c, 'd: 'a, C: ContainerProvider + 'a + 'b>
where C::Parameters: 'c
{
    app: &'a mut Application<'d, C>,
    provider: &'b mut C,
    parameters: &'c mut C::Parameters,
}

impl<'a, 'b, 'c, 'd: 'a, C: ContainerProvider + 'a + 'b> Behavior for ApplicationBehavior<'a, 'b, 'c, 'd, C> {
    fn input(self, i: Input) -> Option<Input> {
        i.chain(|i| self.provider.get_mut(&self.app.active).input(i, self.parameters))
        .finish()
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub x_range: Range<ColIndex>,
    pub y_range: Range<RowIndex>,
}

impl Rectangle {
    fn width(&self) -> Width {
        (self.x_range.end - self.x_range.start).try_into_positive().expect("range invariant")
    }
    fn height(&self) -> Height {
        (self.y_range.end - self.y_range.start).try_into_positive().expect("range invariant")
    }

    fn slice_range_x(&self, range: Range<ColIndex>) -> Rectangle {
        debug_assert!(self.x_range.start <= range.start && range.end <= self.x_range.end, "Invalid slice argument");
        Rectangle {
            x_range: range,
            y_range: self.y_range.clone(),
        }
    }

    fn slice_range_y(&self, range: Range<RowIndex>) -> Rectangle {
        debug_assert!(self.y_range.start <= range.start && range.end <= self.y_range.end, "Invalid slice argument");
        Rectangle {
            x_range: self.x_range.clone(),
            y_range: range,
        }
    }

    fn slice_line_x(&self, x: ColIndex) -> HorizontalLine {
        debug_assert!(self.x_range.start <= x && x <= self.x_range.end, "Invalid slice argument");
        HorizontalLine {
            x: x,
            y_range: self.y_range.clone(),
        }
    }

    fn slice_line_y(&self, y: RowIndex) -> VerticalLine {
        debug_assert!(self.y_range.start <= y && y <= self.y_range.end, "Invalid slice argument");
        VerticalLine {
            x_range: self.x_range.clone(),
            y: y,
        }
    }

    fn is_near_border(&self, x: ColIndex, y: RowIndex, dir: LineSegment) -> bool {
        let x_l = self.x_range.start-1;
        let x_r = self.x_range.end;
        let y_l = self.y_range.start-1;
        let y_r = self.y_range.end;

        let west = x_l == x;
        let east = x_r == x;
        let north = y_l == y;
        let south = y_r == y;
        if east && dir == LineSegment::East || west && dir == LineSegment::West
            || north && dir == LineSegment::North || south && dir == LineSegment::South {

            false
        } else {
            (east || west) && (y_l <= y && y <= y_r) || (north || south) && (x_l <= x && x <= x_r)
        }
    }
}

pub struct HorizontalLine {
    pub x: ColIndex,
    pub y_range: Range<RowIndex>,
}

pub struct VerticalLine {
    pub x_range: Range<ColIndex>,
    pub y: RowIndex,
}

pub enum Line {
    Horizontal(HorizontalLine),
    Vertical(VerticalLine),
}

impl From<HorizontalLine> for Line {
    fn from(l: HorizontalLine) -> Self {
        Line::Horizontal(l)
    }
}

impl From<VerticalLine> for Line {
    fn from(l: VerticalLine) -> Self {
        Line::Vertical(l)
    }
}

pub trait Layout<C: ContainerProvider> {
    fn space_demand(&self, containers: &C) -> Demand2D;
    fn layout(&self, available_area: Rectangle, containers: &C) -> LayoutOutput<C::Index>;
}

pub struct LayoutOutput<I: Clone> {
    pub windows: Vec<(I, Rectangle)>,
    pub separators: Vec<Line>,
}

impl<I: Clone + PartialEq> LayoutOutput<I> {
    fn new() -> Self {
        LayoutOutput {
            windows: Vec::new(),
            separators: Vec::new(),
        }
    }
    fn add_child(&mut self, child: LayoutOutput<I>) {
        for (index, window) in child.windows {
            //self.windows.push((index, region.transform_to_outside_rectangle(window)));
            self.windows.push((index, window));
        }
        for separator in child.separators {
            //self.separators.push(region.transform_to_outside_line(separator));
            self.separators.push(separator);
        }
    }

    fn get_rect_with_index(&self, index: I) -> Option<Rectangle> {
        self.windows.iter().find(|&&(ref i, _)| { *i == index }).map(|&(_, ref w)| w.clone())
    }
}

pub struct Leaf<C: ContainerProvider> {
    container_index: C::Index,
}

impl<C: ContainerProvider> Leaf<C> {
    pub fn new(index: C::Index) -> Self {
        Leaf {
            container_index: index,
        }
    }
}

impl<C: ContainerProvider> Layout<C> for Leaf<C> {
    fn space_demand(&self, containers: &C) -> Demand2D {
        containers.get(&self.container_index).space_demand()
    }
    fn layout(&self, available_area: Rectangle, _: &C) -> LayoutOutput<C::Index> {
        let mut output = LayoutOutput::new();
        output.windows.push((self.container_index.clone(), available_area));
        output
    }
}

pub struct HSplit<'a, C: ContainerProvider> {
    elms: Vec<Box<Layout<C> + 'a>>,
}

impl<'a, C: ContainerProvider> HSplit<'a, C> {
    pub fn new(elms: Vec<Box<Layout<C> + 'a>>) -> Self {
        HSplit {
            elms: elms,
        }
    }
}

impl<'a, C: ContainerProvider> Layout<C> for HSplit<'a, C> {
    fn space_demand(&self, containers: &C) -> Demand2D {
        let mut total_x = ColDemand::exact(0);
        let mut total_y = RowDemand::exact(0);
        for e in self.elms.iter() {
            let demand2d = e.space_demand(containers);
            total_x = total_x + demand2d.width;
            total_y = total_y.max(demand2d.height);
        }
        total_x = total_x + ColDemand::exact(self.elms.len().checked_sub(1).unwrap_or(0));
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }
    fn layout(&self, available_area: Rectangle, containers: &C) -> LayoutOutput<C::Index> {
        let separator_length = Width::new(1).unwrap();
        let horizontal_demands: Vec<ColDemand> = self.elms.iter().map(|w| w.space_demand(containers).width).collect();
        let assigned_spaces = layout_linearly(available_area.width(), separator_length, horizontal_demands.as_slice());
        let mut output = LayoutOutput::new();
        let mut p = available_area.x_range.start;
        for (elm, space) in self.elms.iter().zip(assigned_spaces.into_iter()) {
            let elm_rect = available_area.slice_range_x(p..(p+*space));
            output.add_child(elm.layout(elm_rect, containers));
            p += *space;

            if p < available_area.x_range.end {
                output.separators.push(available_area.slice_line_x(p).into());
                p += 1
            }
        }
        output
    }
}

pub struct VSplit<'a, C: ContainerProvider> {
    elms: Vec<Box<Layout<C> + 'a>>,
}

impl<'a, C: ContainerProvider> VSplit<'a, C> {
    pub fn new(elms: Vec<Box<Layout<C> + 'a>>) -> Self {
        VSplit {
            elms: elms,
        }
    }
}

impl<'a, C: ContainerProvider> Layout<C> for VSplit<'a, C> {
    fn space_demand(&self, containers: &C) -> Demand2D {
        let mut total_x = ColDemand::exact(0);
        let mut total_y = RowDemand::exact(0);
        for e in self.elms.iter() {
            let demand2d = e.space_demand(containers);
            total_x = total_x.max(demand2d.width);
            total_y = total_y + demand2d.height;
        }
        total_y += RowDemand::exact(self.elms.len().checked_sub(1).unwrap_or(0));
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }
    fn layout(&self, available_area: Rectangle, containers: &C) -> LayoutOutput<C::Index> {
        let separator_length = Height::new(1).unwrap();
        let vertical_demands: Vec<RowDemand> = self.elms.iter().map(|w| w.space_demand(containers).height).collect();
        let assigned_spaces = layout_linearly(available_area.height(), separator_length, vertical_demands.as_slice());
        let mut output = LayoutOutput::new();
        let mut p = available_area.y_range.start;
        for (elm, space) in self.elms.iter().zip(assigned_spaces.into_iter()) {
            let elm_rect = available_area.slice_range_y(p..(p+*space));
            output.add_child(elm.layout(elm_rect, containers));
            p += *space;

            if p < available_area.y_range.end {
                output.separators.push(available_area.slice_line_y(p).into());
                p += 1
            }
        }
        output
    }
}

pub struct NavigatableApplication<'a, 'b, 'd: 'a, C: ContainerProvider + 'a + 'b> {
    app: &'a mut Application<'d, C>,
    provider: &'b mut C,
}

enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
}

fn raw_range<T: AxisDimension>(range: &Range<AxisIndex<T>>) -> Range<i32> {
    range.start.raw_value() .. range.end.raw_value()
}

impl<'a, 'b, 'd: 'a, C: ContainerProvider + 'a + 'b> NavigatableApplication<'a, 'b, 'd, C> {
    fn move_to(&mut self, direction: MovementDirection) -> OperationResult {
        let window_size = self.app.last_window_size.get();
        let window_rect = Rectangle { x_range: 0.into()..window_size.0.from_origin(), y_range: 0.into()..window_size.1.from_origin() };
        let layout_result = self.app.layout.layout(window_rect, self.provider);
        let active_rect = layout_result.get_rect_with_index(self.app.active.clone()).ok_or(())?;
        let best = layout_result.windows.iter().filter_map(|&(ref candidate_index, ref candidate_rect)| {
            if *candidate_index == self.app.active {
                return None;
            }
            let (smaller_adjacent, greater_adjacent, active_range, candidate_range) = match direction {
                MovementDirection::Up => {
                    (candidate_rect.y_range.end.raw_value(), active_rect.y_range.start.raw_value(),
                     raw_range(&active_rect.x_range), raw_range(&candidate_rect.x_range))
                },
                MovementDirection::Down => {
                    (active_rect.y_range.end.raw_value(), candidate_rect.y_range.start.raw_value(),
                     raw_range(&active_rect.x_range), raw_range(&candidate_rect.x_range))
                },
                MovementDirection::Left => {
                    (candidate_rect.x_range.end.raw_value(), active_rect.x_range.start.raw_value(),
                     raw_range(&active_rect.y_range), raw_range(&candidate_rect.y_range))
                },
                MovementDirection::Right => {
                    (active_rect.x_range.end.raw_value(), candidate_rect.x_range.start.raw_value(),
                     raw_range(&active_rect.y_range), raw_range(&candidate_rect.y_range))
                },
            };
            if smaller_adjacent < greater_adjacent && greater_adjacent - smaller_adjacent == 1 {
                // Rects are adjacent
                let overlap = min(active_range.end, candidate_range.end).checked_sub(max(active_range.start, candidate_range.start)).unwrap_or(0);
                Some((overlap, candidate_index))
            } else {
                None
            }
        }).max_by_key(|&(overlap, _)| overlap);

        if let Some((_, index)) = best {
            self.app.active = index.clone();
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<'a, 'b, 'd: 'a, C: ContainerProvider + 'a + 'b> Navigatable for NavigatableApplication<'a, 'b, 'd, C> {
    fn move_up(&mut self) -> OperationResult {
        self.move_to(MovementDirection::Up)
    }
    fn move_down(&mut self) -> OperationResult {
        self.move_to(MovementDirection::Down)
    }
    fn move_left(&mut self) -> OperationResult {
        self.move_to(MovementDirection::Left)
    }
    fn move_right(&mut self) -> OperationResult {
        self.move_to(MovementDirection::Right)
    }
}

pub struct Application<'a, C: ContainerProvider> {
    layout: Box<Layout<C> + 'a>,
    active: C::Index,
    last_window_size: Cell<(Width, Height)>,
}

struct LineCanvas {
    cells: BTreeMap<(ColIndex, RowIndex), LineCell>,
}

impl LineCanvas {
    fn new() -> Self {
        LineCanvas {
            cells: BTreeMap::new(),
        }
    }

    fn get_mut(&mut self, x: ColIndex, y: RowIndex) -> &mut LineCell {
        self.cells.entry((x, y)).or_insert(LineCell::empty())
    }

    fn into_iter(self) -> LineCanvasIter {
        LineCanvasIter {
            iter: self.cells.into_iter()
        }
    }
}

struct LineCanvasIter {
    iter: btree_map::IntoIter<(ColIndex, RowIndex), LineCell>,
}

impl Iterator for LineCanvasIter {
    type Item = (ColIndex, RowIndex, LineCell);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|((x,y),c)| (x,y,c))
    }
}

impl<'a, C: ContainerProvider> Application<'a, C> {
    pub fn from_layout(layout_root: Box<Layout<C> + 'a>) -> Self {
        Application {
            layout: layout_root,
            active: C::DEFAULT_CONTAINER.clone(),
            last_window_size: Cell::new((Width::new(100).unwrap(), Height::new(100).unwrap())),
        }
    }

    pub fn draw(&self, mut window: Window, provider: &mut C) {
        self.last_window_size.set((window.get_width(), window.get_height()));

        let window_rect = Rectangle { x_range: 0.into()..window.get_width().from_origin(), y_range: 0.into()..window.get_height().from_origin() };

        let layout_result = self.layout.layout(window_rect, provider);
        let active_rect = layout_result.get_rect_with_index(self.active.clone());

        for (index, rect) in layout_result.windows {
            provider.get_mut(&index).draw(window.create_subwindow(rect.x_range, rect.y_range), RenderingHints {
                active: index == self.active,
                ..Default::default()
            });
        }

        let get_line_type = |x, y, s| {
            if let &Some(ref active_rect) = &active_rect {
                if active_rect.is_near_border(x, y, s) {
                    LineType::Thick
                } else {
                    LineType::Thin
                }
            } else {
                LineType::Thin
            }
        };

        let mut line_canvas = LineCanvas::new();
        for line in layout_result.separators {
            match line {
                Line::Horizontal(HorizontalLine { x, y_range }) => {
                    line_canvas.get_mut(x, y_range.start-1)
                        .set(LineSegment::South, get_line_type(x, y_range.start-1, LineSegment::South));
                    // FIXME: Step trait stabilization
                    for y in y_range.start.raw_value() .. y_range.end.raw_value() {
                        let y = RowIndex::new(y);
                        line_canvas.get_mut(x, y)
                            .set(LineSegment::North, get_line_type(x, y, LineSegment::North))
                            .set(LineSegment::South, get_line_type(x, y, LineSegment::South));
                    }
                    line_canvas.get_mut(x, y_range.end)
                        .set(LineSegment::North, get_line_type(x, y_range.end, LineSegment::North));
                },
                Line::Vertical(VerticalLine { x_range, y }) => {
                    line_canvas.get_mut(x_range.start-1, y)
                        .set(LineSegment::East, get_line_type(x_range.start-1, y, LineSegment::East));
                    // FIXME: Step trait stabilization
                    for x in x_range.start.raw_value() .. x_range.end.raw_value() {
                        let x = ColIndex::new(x);
                        line_canvas.get_mut(x, y)
                            .set(LineSegment::East, get_line_type(x, y, LineSegment::East))
                            .set(LineSegment::West, get_line_type(x, y, LineSegment::West));
                    }
                    line_canvas.get_mut(x_range.end, y)
                        .set(LineSegment::West, get_line_type(x_range.end, y, LineSegment::West));
                },
            }
        }

        for (x, y, cell) in line_canvas.into_iter() {
            if let Some(styled_cluster) = window.get_cell_mut(x, y) {
                styled_cluster.grapheme_cluster = cell.to_grapheme_cluster();
            }
        }
    }

    pub fn navigatable<'b, 'c>(&'b mut self, provider: &'c mut C) -> NavigatableApplication<'b, 'c, 'a, C>
    {
        NavigatableApplication::<C> {
            app: self,
            provider: provider,
        }
    }

    pub fn active_container_behavior<'b, 'c, 'd>(&'b mut self, provider: &'c mut C, parameters: &'d mut C::Parameters) -> ApplicationBehavior<'b, 'c, 'd, 'a, C>
    {
        ApplicationBehavior {
            app: self,
            provider: provider,
            parameters: parameters,
        }
    }

    pub fn active(&self) -> C::Index {
        self.active.clone()
    }
}
