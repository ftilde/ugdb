use base::{Window};
use widget::{Widget, Demand, Demand2D, RenderingHints};
use widget::layouts::{layout_linearly};
use input::{Behavior, Input};
use std::ops::Range;
use std::convert::From;

pub trait Container<P: ?Sized> : Widget {
    fn input(&mut self, input: Input, parameters: &mut P) -> Option<Input>;
}

pub trait ContainerProvider {
    type Parameters;
    type Index: Clone;
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

impl<'a, 'b, 'c, 'd: 'a, C: ContainerProvider + 'a + 'b> ApplicationBehavior<'a, 'b, 'c, 'd, C> {
    pub fn new(app: &'a mut Application<'d, C>, provider: &'b mut C, parameters: &'c mut C::Parameters) -> Self {
        ApplicationBehavior {
            app: app,
            provider: provider,
            parameters: parameters,
        }
    }
}

impl<'a, 'b, 'c, 'd: 'a, C: ContainerProvider + 'a + 'b> Behavior for ApplicationBehavior<'a, 'b, 'c, 'd, C> {
    fn input(self, i: Input) -> Option<Input> {
        i.chain(|i| self.provider.get_mut(&self.app.active).input(i, self.parameters))
        .finish()
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub x_range: Range<u32>,
    pub y_range: Range<u32>,
}

impl Rectangle {
    fn width(&self) -> u32 {
        self.x_range.end - self.x_range.start
    }
    fn height(&self) -> u32 {
        self.y_range.end - self.y_range.start
    }

    fn slice_range_x(&self, range: Range<u32>) -> Rectangle {
        debug_assert!(self.x_range.start <= range.start && range.end <= self.x_range.end, "Invalid slice argument");
        Rectangle {
            x_range: range,
            y_range: self.y_range.clone(),
        }
    }

    fn slice_range_y(&self, range: Range<u32>) -> Rectangle {
        debug_assert!(self.y_range.start <= range.start && range.end <= self.y_range.end, "Invalid slice argument");
        Rectangle {
            x_range: self.x_range.clone(),
            y_range: range,
        }
    }

    fn slice_line_x(&self, x: u32) -> HorizontalLine {
        debug_assert!(self.x_range.start <= x && x <= self.x_range.end, "Invalid slice argument");
        HorizontalLine {
            x: x,
            y_range: self.y_range.clone(),
        }
    }

    fn slice_line_y(&self, y: u32) -> VerticalLine {
        debug_assert!(self.y_range.start <= y && y <= self.y_range.end, "Invalid slice argument");
        VerticalLine {
            x_range: self.x_range.clone(),
            y: y,
        }
    }
}

pub struct HorizontalLine {
    pub x: u32,
    pub y_range: Range<u32>,
}

pub struct VerticalLine {
    pub x_range: Range<u32>,
    pub y: u32,
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

impl<I: Clone> LayoutOutput<I> {
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
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        for e in self.elms.iter() {
            let demand2d = e.space_demand(containers);
            total_x = total_x + demand2d.width;
            total_y = total_y.max(demand2d.height);
        }
        total_x = total_x + Demand::exact(self.elms.len().checked_sub(1).unwrap_or(0) as u32);
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }
    fn layout(&self, available_area: Rectangle, containers: &C) -> LayoutOutput<C::Index> {
        let separator_length = 1;
        let horizontal_demands: Vec<Demand> = self.elms.iter().map(|w| w.space_demand(containers).width).collect();
        let assigned_spaces = layout_linearly(available_area.width(), separator_length, horizontal_demands.as_slice());
        let mut output = LayoutOutput::new();
        let mut p = available_area.x_range.start;
        for (elm, space) in self.elms.iter().zip(assigned_spaces.into_iter()) {
            let elm_rect = available_area.slice_range_x(p..(p+space));
            output.add_child(elm.layout(elm_rect, containers));
            p += space;

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
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        for e in self.elms.iter() {
            let demand2d = e.space_demand(containers);
            total_x = total_x.max(demand2d.width);
            total_y = total_y + demand2d.height;
        }
        total_y = total_y + Demand::exact(self.elms.len().checked_sub(1).unwrap_or(0) as u32);
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }
    fn layout(&self, available_area: Rectangle, containers: &C) -> LayoutOutput<C::Index> {
        let separator_length = 1;
        let vertical_demands: Vec<Demand> = self.elms.iter().map(|w| w.space_demand(containers).height).collect();
        let assigned_spaces = layout_linearly(available_area.height(), separator_length, vertical_demands.as_slice());
        let mut output = LayoutOutput::new();
        let mut p = available_area.y_range.start;
        for (elm, space) in self.elms.iter().zip(assigned_spaces.into_iter()) {
            let elm_rect = available_area.slice_range_y(p..(p+space));
            output.add_child(elm.layout(elm_rect, containers));
            p += space;

            if p < available_area.y_range.end {
                output.separators.push(available_area.slice_line_y(p).into());
                p += 1
            }
        }
        output
    }
}

pub struct Application<'a, C: ContainerProvider> {
    layout: Box<Layout<C> + 'a>,
    active: C::Index,
}

impl<'a, C: ContainerProvider> Application<'a, C> {
    pub fn from_layout(layout_root: Box<Layout<C> + 'a>) -> Self {
        Application {
            layout: layout_root,
            active: C::DEFAULT_CONTAINER.clone(),
        }
    }

    pub fn draw(&self, mut window: Window, provider: &mut C) {
        let window_rect = Rectangle { x_range: 0..window.get_width(), y_range: 0..window.get_height() };
        let layout_result = self.layout.layout(window_rect, provider);
        for (index, rect) in layout_result.windows {
            provider.get_mut(&index).draw(window.create_subwindow(rect.x_range, rect.y_range), RenderingHints::default() /* TODO */);
        }
        //TODO draw lines
    }
}
