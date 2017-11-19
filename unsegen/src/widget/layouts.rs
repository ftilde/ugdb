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
    let mut assigned_spaces = vec![0; demands.len()].into_boxed_slice();
    let mut num_unfinished = 0;
    let mut sum_of_unfinished = 0;
    for (i, demand) in demands.iter().enumerate() {
        let mut unfinished = false;
        if let Some(max_demand) = demand.max {
            if max_demand != demand.min {
                unfinished = true;
            }
        } else {
            unfinished = true;
        }

        let assigned_space = min(available_space, demand.min);
        available_space -= assigned_space;
        assigned_spaces[i] = assigned_space;

        if unfinished {
            num_unfinished += 1;
            sum_of_unfinished += assigned_space;
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
        //println!("Step 1: {:?}, {:?} left (unfinished? {})", assigned_spaces, available_space, unfinished);
    }

    // equalize remaining
    {
        if num_unfinished == 0 {
            return assigned_spaces;
        }

        let total_per_widget = (available_space + sum_of_unfinished) / num_unfinished; //Rounded down!
        if total_per_widget > 0 {
            for (i, demand) in demands.iter().enumerate() {
                let additional = if let Some(max_demand) = demand.max {
                    if assigned_spaces[i] == max_demand {
                        continue;
                    }
                    if total_per_widget < max_demand {
                        total_per_widget.checked_sub(assigned_spaces[i]).unwrap_or(0)
                    } else {
                        num_unfinished -= 1;
                        max_demand - assigned_spaces[i]
                    }
                } else {
                    total_per_widget.checked_sub(assigned_spaces[i]).unwrap_or(0)
                };
                assigned_spaces[i] += additional;
                available_space -= additional;
                //println!("Step 2: {:?}, {:?} left ({} total per widget)", assigned_spaces, available_space, total_per_widget);
            }
        }
    }

    // spend rest equally
    loop {
        if num_unfinished == 0 {
            return assigned_spaces;
        }

        let to_assign_per_widget = available_space / num_unfinished; //Rounded down!
        if to_assign_per_widget == 0 {
            break;
        }
        //println!("Starting Step 3: {:?} left, {:?} per widget", available_space, to_assign_per_widget);

        for (i, demand) in demands.iter().enumerate() {
            let additional = if let Some(max_demand) = demand.max {
                if assigned_spaces[i] == max_demand {
                    continue;
                }
                if assigned_spaces[i] + to_assign_per_widget < max_demand {
                    to_assign_per_widget
                } else {
                    num_unfinished -= 1;
                    max_demand - assigned_spaces[i]
                }
            } else {
                to_assign_per_widget
            };
            //println!("Step 3: {:?}, assigning {:?}, {:?} left", assigned_spaces, additional, available_space);
            assigned_spaces[i] += additional;
            available_space -= additional;
        }
    }

    // now: available_space / num_unfinished < 0!
    // => at most 1 space left per unfinished
    for (i, demand) in demands.iter().enumerate() {
        if available_space == 0 {
            break;
        }
        if demand.max.is_none() || demand.max.unwrap() > assigned_spaces[i] {
            assigned_spaces[i] += 1;
            available_space -= 1;
        }
        //println!("Step 4: {:?}, {:?} left", assigned_spaces, available_space);
    }

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
}
