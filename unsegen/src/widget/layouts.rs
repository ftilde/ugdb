use super::{
    Demand,
    Demand2D,
    RenderingHints,
    Widget,
};
use base::{
    Window,
    GraphemeCluster,
    StyleModifier,
};
use std::cmp::min;

#[derive(Clone)]
pub enum SeparatingStyle {
    None,
    AlternatingStyle(StyleModifier),
    Draw(GraphemeCluster)
}
impl SeparatingStyle {
    pub fn width(&self) -> u32 {
        match self {
            &SeparatingStyle::None => 0,
            &SeparatingStyle::AlternatingStyle(_) => 0,
            &SeparatingStyle::Draw(ref cluster) => cluster.width() as u32,
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            &SeparatingStyle::None => 0,
            &SeparatingStyle::AlternatingStyle(_) => 0,
            &SeparatingStyle::Draw(_) => 1,
        }
    }
}
pub fn layout_linearly(mut available_space: u32, separator_width: u32, demands: &[Demand]) -> Box<[u32]>
{
    //eprintln!("av {}, sep {}, dem, {:?}", available_space, separator_width, demands);

    let mut assigned_spaces = vec![0; demands.len()].into_boxed_slice();
    let mut unfinished = Vec::<usize>::new();

    for (i, demand) in demands.iter().enumerate() {
        let mut is_unfinished = false;
        if let Some(max_demand) = demand.max {
            if max_demand != demand.min {
                is_unfinished = true;
            }
        } else {
            is_unfinished = true;
        }

        let assigned_space = min(available_space, demand.min);
        available_space -= assigned_space;
        assigned_spaces[i] = assigned_space;

        if is_unfinished {
            unfinished.push(i);
        }

        let separator_width = if i == (demands.len()-1) { //Last element does not have a following separator
            0
        } else {
            separator_width
        };

        if available_space <= separator_width {
            return assigned_spaces;
        }
        available_space -= separator_width;
    }

    // equalize remaining
    loop {
        unfinished.sort_by(|&i, &j| assigned_spaces[i].cmp(&assigned_spaces[j]));
        let mut still_unfinished = Vec::<usize>::new();

        if unfinished.is_empty() {
            return assigned_spaces;
        }

        let mut planned_to_spend = 0;
        let mut planned_increased_space = 0;
        let mut num_equalized = 0;
        // Plan to equalize "ladder" as far as possible
        for (i, unfinished_index) in unfinished.iter().enumerate() {
            let new_space = assigned_spaces[*unfinished_index];
            let diff = new_space - planned_increased_space; //Sorted, so >= 0
            let increase_cost = (i as u32) * diff;
            if planned_to_spend + increase_cost > available_space {
                break;
            }
            num_equalized = i+1;
            planned_to_spend += increase_cost;
            planned_increased_space = new_space;
        }
        // Plan to distribute the remaining space equally (will be less than the last step on the
        // ladder!
        let left_to_spend = available_space - planned_to_spend;
        let per_widget_increase = left_to_spend / (num_equalized as u32);
        planned_increased_space += per_widget_increase;

        let min_space = assigned_spaces[unfinished[0]];
        if min_space == planned_increased_space {
            break;
        }
        debug_assert!(min_space < planned_increased_space, "Invalid planned increase");

        // Actually distribute (some of) the remaining space like planned
        for unfinished_index in unfinished {
            let assigned_space: &mut u32 = &mut assigned_spaces[unfinished_index];
            let increase = if let Some(max_demand) = demands[unfinished_index].max {
                if max_demand > planned_increased_space {
                    still_unfinished.push(unfinished_index);
                    planned_increased_space.checked_sub(*assigned_space).unwrap_or(0)
                } else {
                    max_demand - *assigned_space
                }
            } else {
                still_unfinished.push(unfinished_index);
                planned_increased_space.checked_sub(*assigned_space).unwrap_or(0)
            };
            *assigned_space += increase;
            available_space -= increase;
        }

        unfinished = still_unfinished;
    }

    for unfinished_index in unfinished {
        if available_space == 0 {
            break;
        }
        debug_assert!({
            let demand = demands[unfinished_index];
            demand.max.is_none() || demand.max.unwrap() > assigned_spaces[unfinished_index]
        }, "Invalid demand for unfinished");

        assigned_spaces[unfinished_index] += 1;
        available_space -= 1;
    }
    debug_assert!(available_space == 0, "Not all space distributed");

    assigned_spaces
}

fn draw_linearly<S, L, M, D>(window: Window,
                          widgets: &[(&Widget, RenderingHints)],
                          separating_style: &SeparatingStyle,
                          split: S,
                          window_length: L,
                          separator_length: M,
                          demand_dimension: D,
                          )
where
    S: Fn(Window, u32) -> (Window, Window),
    L: Fn(&Window) -> u32,
    M: Fn(&SeparatingStyle) -> u32,
    D: Fn(Demand2D) -> Demand
{

    let separator_length = separator_length(separating_style);
    let horizontal_demands: Vec<Demand> = widgets.iter().map(|&(ref w,_)| demand_dimension(w.space_demand())).collect(); //TODO: rename
    let assigned_spaces = layout_linearly(window_length(&window), separator_length, horizontal_demands.as_slice());

    debug_assert!(widgets.len() == assigned_spaces.len(), "widgets and spaces len mismatch");

    let mut rest_window = window;
    let mut iter = widgets.iter().zip(assigned_spaces.iter()).enumerate().peekable();
    while let Some((i, (&(ref w, hint), &pos))) = iter.next() {
        let (mut window, r) = split(rest_window, pos);
        rest_window = r;
        if let (1, &SeparatingStyle::AlternatingStyle(modifier)) = (i%2, separating_style) {
            window.modify_default_style(&modifier);
        }
        window.clear(); // Fill background using new style
        w.draw(window, hint);
        if let (Some(_), &SeparatingStyle::Draw(ref c)) = (iter.peek(), separating_style) {
            if window_length(&rest_window) > 0 {
                let (mut window, r) = split(rest_window, separator_length);
                rest_window = r;
                window.fill(c.clone());
            }
        }
    }
}

pub struct HorizontalLayout {
    separating_style: SeparatingStyle,
}
impl HorizontalLayout {
    pub fn new(separating_style: SeparatingStyle) -> Self {
        HorizontalLayout {
            separating_style: separating_style,
        }
    }

    pub fn space_demand(&self, widgets: &[&Widget]) -> Demand2D {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets {
            let demand2d = w.space_demand();
            total_x = total_x + demand2d.width;
            total_y = total_y.max(demand2d.height);
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_x = total_x + Demand::exact(n_elements);
        }
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }

    pub fn draw(&self, window: Window, widgets: &[(&Widget, RenderingHints)]) {
        draw_linearly(window, widgets, &self.separating_style, |w, p| w.split_h(p).expect("valid split pos"), |w| w.get_width(), SeparatingStyle::width, |d| d.width);
    }
}

pub struct VerticalLayout {
    separating_style: SeparatingStyle,
}

impl VerticalLayout {
    pub fn new(separating_style: SeparatingStyle) -> Self {
        VerticalLayout {
            separating_style: separating_style,
        }
    }

    pub fn space_demand(&self, widgets: &[&Widget]) -> Demand2D {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets.iter() {
            let demand2d = w.space_demand();
            total_x = total_x.max(demand2d.width);
            total_y = total_y + demand2d.height;
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_y = total_y + Demand::exact(n_elements);
        }
        Demand2D {
            width: total_x,
            height: total_y,
        }
    }

    pub fn draw(&self, window: Window, widgets: &[(&Widget, RenderingHints)]) {
        draw_linearly(window, widgets, &self.separating_style, |w, p| w.split_v(p).expect("valid split pos"), |w| w.get_height(), SeparatingStyle::height, |d| d.height);
    }
}

#[cfg(test)]
mod test {
    // for fuzzing tests
    extern crate rand;
    use self::rand::Rng;

    use base::test::FakeTerminal;
    use super::*;

    struct FakeWidget {
        space_demand: Demand2D,
        fill_char: char,
    }
    impl FakeWidget {
        fn new(space_demand: (Demand, Demand)) -> Self {
            Self::with_fill_char(space_demand, '_')
        }
        fn with_fill_char(space_demand: (Demand, Demand), fill_char: char) -> Self {
            FakeWidget {
                space_demand: Demand2D { width: space_demand.0, height: space_demand.1 },
                fill_char: fill_char,
            }
        }
    }
    impl Widget for FakeWidget {
        fn space_demand(&self) -> Demand2D {
            self.space_demand
        }
        fn draw(&self, mut window: Window, _: RenderingHints) {
            window.fill(GraphemeCluster::try_from(self.fill_char).unwrap());
        }
    }


    fn assert_eq_boxed_slices<T: PartialEq+::std::fmt::Debug>(b1: Box<[T]>, b2: Box<[T]>, description: &str) {
        assert_eq!(b1, b2, "{}", description);
    }

    #[test]
    fn test_layout_linearly_exact() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(1), Demand::exact(2)]), Box::new([1, 2]), "some left");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(1), Demand::exact(3)]), Box::new([1, 3]), "exact");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(2), Demand::exact(3)]), Box::new([2, 2]), "less for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::exact(5), Demand::exact(3)]), Box::new([4, 0]), "none for 2nd");
    }

    #[test]
    fn test_layout_linearly_from_to() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 2), Demand::from_to(1, 2)]), Box::new([2, 2]), "both hit max");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 2), Demand::from_to(1, 3)]), Box::new([2, 2]), "less for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(5, 6), Demand::from_to(1, 4)]), Box::new([4, 0]), "nothing for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 5), Demand::from_to(1, 4)]), Box::new([2, 2]), "both not full");
    }

    #[test]
    fn test_layout_linearly_from_at_least() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(1)]), Box::new([2, 2]), "more for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(2)]), Box::new([2, 2]), "more for 1st, exact for 2nd");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(2), Demand::at_least(2)]), Box::new([2, 2]), "exact for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(5), Demand::at_least(2)]), Box::new([4, 0]), "none for 2nd");
    }

    #[test]
    fn test_layout_linearly_mixed() {
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 7]), "exact, 2nd takes rest, no separator");
        assert_eq_boxed_slices(layout_linearly(10, 1, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 6]), "exact, 2nd takes rest, separator");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::at_least(1)]), Box::new([2, 8]), "from_to, 2nd takes rest");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::exact(3), Demand::at_least(1)]),    Box::new([2, 3, 5]), "misc 1");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(5, 6), Demand::exact(5), Demand::at_least(5)]),    Box::new([5, 5, 0]), "misc 2");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(4, 6), Demand::exact(4), Demand::at_least(3)]),    Box::new([4, 4, 2]), "misc 3");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(4), Demand::at_least(3)]),    Box::new([3, 4, 3]), "misc 4");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(3), Demand::at_least(3)]),    Box::new([4, 3, 3]), "misc 5");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(3)]),    Box::new([4, 2, 4]), "misc 6");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::exact(3)]),       Box::new([4, 2, 3]), "misc 7");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(4)]),    Box::new([4, 2, 4]), "misc 8");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 3), Demand::at_least(2), Demand::at_least(2)]), Box::new([3, 4, 3]), "misc 9");

        assert_eq_boxed_slices(layout_linearly(82, 1, &[Demand::at_least(4), Demand::at_least(51)]), Box::new([30, 51]), "misc 10");
    }

    fn aeq_horizontal_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        let demand2d = Demand2D {
            width: solution.0,
            height: solution.1,
        };
        assert_eq!(HorizontalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), demand2d);
    }
    #[test]
    fn test_horizontal_layout_space_demand() {
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::exact(1), Demand::exact(2))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::exact(2), Demand::exact(2)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 2), Demand::from_to(1, 3))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(1), Demand::exact(5)))], (Demand::at_least(4), Demand::at_least(5)));
    }
    fn aeq_horizontal_layout_draw(terminal_size: (u32, u32), widgets: Vec<&Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        let widgets_with_hints: Vec<(&Widget, RenderingHints)> = widgets.into_iter().map(|w| (w, RenderingHints::default())).collect();
        HorizontalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets_with_hints.as_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_horizontal_layout_draw() {
        aeq_horizontal_layout_draw((4, 1), vec![&FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '1'), &FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '2')], "1122");
        aeq_horizontal_layout_draw((4, 1), vec![&FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(1)), '2')], "1222");
        aeq_horizontal_layout_draw((4, 2), vec![&FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(2)), '2')], "1222 1222");
        aeq_horizontal_layout_draw((8, 1), vec![&FakeWidget::with_fill_char((Demand::at_least(1), Demand::at_least(1)), '1'), &FakeWidget::with_fill_char((Demand::at_least(3), Demand::exact(3)), '2')], "11112222");
    }

    fn aeq_vertical_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        let demand2d = Demand2D {
            width: solution.0,
            height: solution.1,
        };
        assert_eq!(VerticalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), demand2d);
    }
    #[test]
    fn test_vertical_layout_space_demand() {
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::exact(2), Demand::exact(1))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::exact(2), Demand::exact(2)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 3), Demand::from_to(1, 2))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(5), Demand::exact(1)))], (Demand::at_least(5), Demand::at_least(4)));
    }
    fn aeq_vertical_layout_draw(terminal_size: (u32, u32), widgets: Vec<&Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        let widgets_with_hints: Vec<(&Widget, RenderingHints)> = widgets.into_iter().map(|w| (w, RenderingHints::default())).collect();
        VerticalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets_with_hints.as_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_vertical_layout_draw() {
        aeq_vertical_layout_draw((1, 4), vec![&FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '1'), &FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '2')], "1 1 2 2");
        aeq_vertical_layout_draw((1, 4), vec![&FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &FakeWidget::with_fill_char((Demand::exact(1), Demand::at_least(2)), '2')], "1 2 2 2");
        aeq_vertical_layout_draw((2, 4), vec![&FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &FakeWidget::with_fill_char((Demand::exact(2), Demand::at_least(2)), '2')], "11 22 22 22");
        aeq_vertical_layout_draw((1, 8), vec![&FakeWidget::with_fill_char((Demand::at_least(2), Demand::at_least(2)), '1'), &FakeWidget::with_fill_char((Demand::at_least(1), Demand::at_least(1)), '2')], "1 1 1 1 2 2 2 2");
    }

    #[test]
    fn fuzz_layout_linearly() {
        let fuzz_iterations = 10000;
        let max_widgets = 10;
        let max_space = 1000;
        let max_separator_size = 5;

        let mut rng = rand::thread_rng();
        for _ in 0..fuzz_iterations {
            let mut demands = Vec::new();
            for _ in 0..max_widgets {
                let min = rng.gen_range(0, max_space);
                let max = if rng.gen() {
                    Some(rng.gen_range(min, max_space))
                } else {
                    None
                };
                demands.push(Demand {
                    min: min,
                    max: max,
                });
            }

            let space = rng.gen_range(0, max_space);
            let separator_size = rng.gen_range(0, max_separator_size);
            layout_linearly(space, separator_size, demands.as_slice());
        }
    }
}
