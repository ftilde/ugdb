use super::{
    Demand,
    Widget,
};
use base::{
    Window,
    GraphemeCluster,
};
use std::cmp::{max, min};

#[derive(Clone)]
pub enum SeparatingStyle {
    None,
    //AlternateStyle(TextAttribute),
    Draw(GraphemeCluster)
}
impl SeparatingStyle {
    pub fn width(&self) -> u32 {
        match self {
            &SeparatingStyle::None => 0,
            &SeparatingStyle::Draw(ref cluster) => cluster.width() as u32,
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            &SeparatingStyle::None => 0,
            &SeparatingStyle::Draw(_) => 1,
        }
    }
}
pub fn layout_linearly(mut available_space: u32, separator_width: u32, demands: &[Demand]) -> Box<[u32]>
{
    let mut assigned_spaces = vec![0; demands.len()].into_boxed_slice();
    let mut max_max_demand = None;
    let mut num_unbounded = 0;
    for (i, demand) in demands.iter().enumerate() {
        if let Some(max_demand) = demand.max {
            max_max_demand = Some(max(max_max_demand.unwrap_or(0), max_demand));
        } else {
            num_unbounded += 1;
        }
        let assigned_space = min(available_space, demand.min);
        available_space -= assigned_space;
        assigned_spaces[i] = assigned_space;

        let separator_width = if i == (demands.len()-1) { //Last element does not have a following separator
            0
        } else {
            separator_width
        };

        if available_space <= separator_width {
            return assigned_spaces;
        }
        available_space -= separator_width;
        //println!("Step 1: {:?}, {:?} left", assigned_spaces, available_space);
    }

    if let Some(total_max_max_demand) = max_max_demand {
        for (i, demand) in demands.iter().enumerate() {
            let max_demand = demand.max.unwrap_or(total_max_max_demand);
            if max_demand > demand.min {
                let additional_assigned_space = min(available_space, max_demand - demand.min);
                available_space -= additional_assigned_space;
                assigned_spaces[i] += additional_assigned_space;

                //println!("Step 2: {:?}, {:?} left", assigned_spaces, available_space);
                if available_space == 0 {
                    return assigned_spaces;
                }
            }
        }
    }

    if num_unbounded == 0 {
        return assigned_spaces;
    }

    let left_over_space_per_unbounded_widget = available_space / num_unbounded; //Rounded down!

    for (i, _) in demands.iter().enumerate().filter(|&(_, w)| w.max.is_none()) {
        let additional_assigned_space = min(available_space, left_over_space_per_unbounded_widget);
        available_space -= additional_assigned_space;
        assigned_spaces[i] += additional_assigned_space;

        //println!("Step 3: {:?}, {:?} left", assigned_spaces, available_space);
        if available_space == 0 {
            return assigned_spaces;
        }
    }

    assigned_spaces
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

    pub fn space_demand(&self, widgets: &[&Widget]) -> (Demand, Demand) {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets {
            let (x, y) = w.space_demand();
            total_x = total_x + x;
            total_y = total_y.max(y);
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_x = total_x + Demand::exact(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw(&self, window: Window, widgets: &mut [&mut Widget]) {

        let separator_width = self.separating_style.width();
        let horizontal_demands: Vec<Demand> = widgets.iter().map(|w| w.space_demand().0.clone()).collect();
        let assigned_spaces = layout_linearly(window.get_width(), separator_width, horizontal_demands.as_slice());

        debug_assert!(widgets.len() == assigned_spaces.len(), "widgets and spaces len mismatch");

        let mut rest_window = window;
        let mut iter = widgets.iter_mut().zip(assigned_spaces.iter()).peekable();
        while let Some((&mut ref mut w, &pos)) = iter.next() {
            let (window, r) = rest_window.split_h(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), &SeparatingStyle::Draw(ref c)) = (iter.peek(), &self.separating_style) {
                if rest_window.get_width() > 0 {
                    let (mut window, r) = rest_window.split_h(c.width() as u32);
                    rest_window = r;
                    window.fill(c.clone());
                }
            }
        }
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

    pub fn space_demand(&self, widgets: &[&Widget]) -> (Demand, Demand) {
        let mut total_x = Demand::exact(0);
        let mut total_y = Demand::exact(0);
        let mut n_elements = 0;
        for w in widgets.iter() {
            let (x, y) = w.space_demand();
            total_x = total_x.max(x);
            total_y = total_y + y;
            n_elements += 1;
        }
        if let SeparatingStyle::Draw(_) = self.separating_style {
            total_y = total_y + Demand::exact(n_elements);
        }
        (total_x, total_y)
    }

    pub fn draw(&self, window: Window, widgets: &mut [&mut Widget]) {

        let separator_width = self.separating_style.height();
        let vertical_demands: Vec<Demand> = widgets.iter().map(|w| w.space_demand().1.clone()).collect();
        let assigned_spaces = layout_linearly(window.get_height(), separator_width, vertical_demands.as_slice());

        debug_assert!(widgets.len() == assigned_spaces.len(), "widgets and spaces len mismatch");

        let mut rest_window = window;
        let mut iter = widgets.iter_mut().zip(assigned_spaces.iter()).peekable();
        while let Some((&mut ref mut w, &pos)) = iter.next() {
            let (window, r) = rest_window.split_v(pos);
            rest_window = r;
            w.draw(window);
            if let (Some(_), &SeparatingStyle::Draw(ref c)) = (iter.peek(), &self.separating_style) {
                if rest_window.get_height() > 0 {
                    let (mut window, r) = rest_window.split_v(1);
                    rest_window = r;
                    window.fill(c.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use base::test::FakeTerminal;
    use super::*;


    struct FakeWidget {
        space_demand: (Demand, Demand),
        fill_char: char,
    }
    impl FakeWidget {
        fn new(space_demand: (Demand, Demand)) -> Self {
            FakeWidget {
                space_demand: space_demand,
                fill_char: '_',
            }
        }
        fn with_fill_char(space_demand: (Demand, Demand), fill_char: char) -> Self {
            FakeWidget {
                space_demand: space_demand,
                fill_char: fill_char,
            }
        }
    }
    impl Widget for FakeWidget {
        fn space_demand(&self) -> (Demand, Demand) {
            self.space_demand
        }
        fn draw(&mut self, mut window: Window) {
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
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::from_to(1, 5), Demand::from_to(1, 4)]), Box::new([3, 1]), "less for 1st");
    }

    #[test]
    fn test_layout_linearly_from_at_least() {
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(1)]), Box::new([2, 2]), "more for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(1), Demand::at_least(2)]), Box::new([1, 2]), "exact for both, devisor test");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(2), Demand::at_least(2)]), Box::new([2, 2]), "exact for both");
        assert_eq_boxed_slices(layout_linearly(4, 0, &[Demand::at_least(5), Demand::at_least(2)]), Box::new([4, 0]), "none for 2nd");
    }

    #[test]
    fn test_layout_linearly_mixed() {
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 7]), "exact, 2nd takes rest, no separator");
        assert_eq_boxed_slices(layout_linearly(10, 1, &[Demand::exact(3), Demand::at_least(1)]), Box::new([3, 6]), "exact, 2nd takes rest, separator");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::at_least(1)]), Box::new([2, 8]), "from_to, 2nd takes rest");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(1, 2), Demand::exact(3), Demand::at_least(1)]), Box::new([2, 3, 5]), "return paths: end");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(5, 6), Demand::exact(5), Demand::at_least(5)]), Box::new([5, 5, 0]), "return paths: first loop, 2nd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(4, 6), Demand::exact(4), Demand::at_least(3)]), Box::new([4, 4, 2]), "return paths: first loop, 3rd it, rest");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(4), Demand::at_least(3)]), Box::new([3, 4, 3]), "return paths: first loop, 3rd it, full");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(3, 6), Demand::exact(3), Demand::at_least(3)]), Box::new([4, 3, 3]), "return paths: second loop, 1st it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(3)]), Box::new([4, 2, 4]), "return paths: second loop, 3rd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::exact(3)]), Box::new([4, 2, 3]), "return paths: after second loop, 3rd it");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 4), Demand::exact(2), Demand::at_least(4)]), Box::new([4, 2, 4]), "return paths: third loop, after 1st item");
        assert_eq_boxed_slices(layout_linearly(10, 0, &[Demand::from_to(2, 3), Demand::at_least(2), Demand::at_least(2)]), Box::new([3, 3, 3]), "return paths: third loop, finished");
    }

    fn aeq_horizontal_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        assert_eq!(HorizontalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), solution);
    }
    #[test]
    fn test_horizontal_layout_space_demand() {
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::exact(1), Demand::exact(2))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::exact(2), Demand::exact(2)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 2), Demand::from_to(1, 3))), &FakeWidget::new((Demand::exact(1), Demand::exact(2)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_horizontal_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(1), Demand::exact(5)))], (Demand::at_least(4), Demand::at_least(5)));
    }
    fn aeq_horizontal_layout_draw(terminal_size: (usize, usize), mut widgets: Vec<&mut Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        HorizontalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets.as_mut_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_horizontal_layout_draw() {
        aeq_horizontal_layout_draw((4, 1), vec![&mut FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(2), Demand::exact(1)), '2')], "1122");
        aeq_horizontal_layout_draw((4, 1), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(1)), '2')], "1222");
        aeq_horizontal_layout_draw((4, 2), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::at_least(2), Demand::exact(2)), '2')], "1222 1222");
    }

    fn aeq_vertical_layout_space_demand(widgets: Vec<&Widget>, solution: (Demand, Demand)) {
        assert_eq!(VerticalLayout::new(SeparatingStyle::None).space_demand(widgets.as_slice()), solution);
    }
    #[test]
    fn test_vertical_layout_space_demand() {
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::exact(2), Demand::exact(1))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::exact(2), Demand::exact(2)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::from_to(1, 3), Demand::from_to(1, 2))), &FakeWidget::new((Demand::exact(2), Demand::exact(1)))], (Demand::from_to(2, 3), Demand::from_to(2, 3)));
        aeq_vertical_layout_space_demand(vec![&FakeWidget::new((Demand::at_least(3), Demand::at_least(3))), &FakeWidget::new((Demand::exact(5), Demand::exact(1)))], (Demand::at_least(5), Demand::at_least(4)));
    }
    fn aeq_vertical_layout_draw(terminal_size: (usize, usize), mut widgets: Vec<&mut Widget>, solution: &str) {
        let mut term = FakeTerminal::with_size(terminal_size);
        VerticalLayout::new(SeparatingStyle::None).draw(term.create_root_window(), widgets.as_mut_slice());
        assert_eq!(term, FakeTerminal::from_str(terminal_size, solution).expect("term from str"));
    }
    #[test]
    fn test_vertical_layout_draw() {
        aeq_vertical_layout_draw((1, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(2)), '2')], "1 1 2 2");
        aeq_vertical_layout_draw((1, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(1), Demand::at_least(2)), '2')], "1 2 2 2");
        aeq_vertical_layout_draw((2, 4), vec![&mut FakeWidget::with_fill_char((Demand::exact(1), Demand::exact(1)), '1'), &mut FakeWidget::with_fill_char((Demand::exact(2), Demand::at_least(2)), '2')], "11 22 22 22");
    }
}
