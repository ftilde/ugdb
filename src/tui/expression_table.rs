use self::json_ext::JsonValue;
use gdb_expression_parsing::parse_gdb_value;
use gdbmi::commands::MiCommand;
use gdbmi::output::ResultClass;
use gdbmi::ExecuteError;
use unsegen::base::{Color, GraphemeCluster, StyleModifier, Window};
use unsegen::container::Container;
use unsegen::input::{EditBehavior, Input, InputChain, Key, NavigateBehavior, ScrollBehavior};
use unsegen::widget::builtin::{Column, LineEdit, Table, TableRow};
use unsegen::widget::{Demand2D, RenderingHints, SeparatingStyle, Widget};
use unsegen_jsonviewer::{json_ext, JsonViewer};

use completion::{Completer, CompletionState, IdentifierCompleter};

pub struct ExpressionRow {
    expression: LineEdit,
    completion_state: Option<CompletionState>,
    result: JsonViewer,
}
impl ExpressionRow {
    fn new() -> Self {
        ExpressionRow {
            expression: LineEdit::new(),
            completion_state: None,
            result: JsonViewer::new(&JsonValue::Null),
        }
    }

    fn is_empty(&self) -> bool {
        self.expression.get().is_empty()
    }
}
impl TableRow for ExpressionRow {
    const COLUMNS: &'static [Column<ExpressionRow>] = &[
        Column {
            access: |r| &r.expression,
            access_mut: |r| &mut r.expression,
            behavior: |r, input| {
                input
                    .chain(
                        EditBehavior::new(&mut r.expression)
                            .left_on(Key::Left)
                            .right_on(Key::Right)
                            .up_on(Key::Up)
                            .down_on(Key::Down)
                            .delete_forwards_on(Key::Delete)
                            .delete_backwards_on(Key::Backspace)
                            .go_to_beginning_of_line_on(Key::Home)
                            .go_to_end_of_line_on(Key::End)
                            .clear_on(Key::Ctrl('c')),
                    )
                    .finish()
            },
        },
        Column {
            access: |r| &r.result,
            access_mut: |r| &mut r.result,
            behavior: |r, input| {
                input
                    .chain(
                        ScrollBehavior::new(&mut r.result)
                            .forwards_on(Key::PageDown)
                            .backwards_on(Key::PageUp)
                            .forwards_on(Key::Down)
                            .backwards_on(Key::Up)
                            .to_beginning_on(Key::Home)
                            .to_end_on(Key::End),
                    )
                    .chain(|evt: Input| {
                        if evt.matches(Key::Char(' ')) {
                            if r.result.toggle_active_element().is_ok() {
                                None
                            } else {
                                Some(evt)
                            }
                        } else {
                            Some(evt)
                        }
                    })
                    .finish()
            },
        },
    ];
}

pub struct ExpressionTable {
    table: Table<ExpressionRow>,
}

impl ExpressionTable {
    pub fn new() -> Self {
        let row_sep_style =
            SeparatingStyle::AlternatingStyle(StyleModifier::new().bg_color(Color::Black));
        let col_sep_style = SeparatingStyle::Draw(GraphemeCluster::try_from('│').unwrap());
        let focused_style = StyleModifier::new().bold(true);
        let mut table = Table::new(row_sep_style, col_sep_style, focused_style);
        table.rows_mut().push(ExpressionRow::new()); //Invariant: always at least one line
        ExpressionTable { table: table }
    }
    pub fn add_entry(&mut self, entry: String) {
        {
            let mut rows = self.table.rows_mut();
            match rows.last_mut() {
                Some(row) if row.is_empty() => {
                    row.expression.set(entry);
                }
                _ => {
                    let mut row = ExpressionRow::new();
                    row.expression.set(entry);
                    rows.push(row);
                }
            }
        }
        self.shrink_to_fit();
    }
    fn shrink_to_fit(&mut self) {
        let begin_of_empty_range = {
            let iter = self.table.rows().iter().enumerate().rev();
            let mut without_trailing_empty_rows = iter.skip_while(|&(_, r)| r.is_empty());
            if let Some((i, _)) = without_trailing_empty_rows.next() {
                i + 1
            } else {
                0
            }
        };
        let mut rows = self.table.rows_mut();
        rows.drain(begin_of_empty_range..);
        rows.push(ExpressionRow::new());
    }

    pub fn update_results(&mut self, p: ::UpdateParameters) {
        for row in self.table.rows_mut().iter_mut() {
            let expr = row.expression.get().to_owned();
            let result = if expr.is_empty() {
                JsonValue::Null
            } else {
                match p.gdb.mi.execute(MiCommand::data_evaluate_expression(expr)) {
                    Ok(res) => match res.class {
                        ResultClass::Error => res.results["msg"].clone(),
                        ResultClass::Done => {
                            let to_parse = res.results["value"].as_str().expect("value present");
                            match parse_gdb_value(to_parse) {
                                Ok(p) => p,
                                Err(_) => {
                                    JsonValue::String(format!("*Error parsing*: {}", to_parse))
                                }
                            }
                        }
                        other => panic!("unexpected result class: {:?}", other),
                    },
                    Err(ExecuteError::Busy) => {
                        return;
                    }
                    Err(ExecuteError::Quit) => {
                        panic!("GDB quit!");
                    }
                }
            };
            row.result.update(&result);
        }
    }
}

impl Widget for ExpressionTable {
    fn space_demand(&self) -> Demand2D {
        self.table.space_demand()
    }
    fn draw(&self, window: Window, hints: RenderingHints) {
        self.table.draw(window, hints);
    }
}

impl Container<::UpdateParametersStruct> for ExpressionTable {
    fn input(&mut self, input: Input, p: ::UpdateParameters) -> Option<Input> {
        let after_completion: InputChain = if let Some(r) = self.table.current_row_mut() {
            let set_completion = |completion_state: &Option<CompletionState>,
                                  expression: &mut LineEdit| {
                let completion = completion_state.as_ref().unwrap();
                let (begin, option, after) = completion.current_line_parts();
                expression.set(&format!("{}{}{}", begin, option, after));
                expression
                    .set_cursor_pos(begin.len() + option.len())
                    .unwrap();
            };
            let res = input
                .chain((&[Key::Ctrl('n'), Key::Char('\t')][..], || {
                    if let Some(s) = &mut r.completion_state {
                        s.select_next_option();
                    } else {
                        r.completion_state = Some(
                            IdentifierCompleter(p)
                                .complete(r.expression.get(), r.expression.cursor_pos()),
                        );
                    }
                    set_completion(&r.completion_state, &mut r.expression);
                }))
                .chain((Key::Ctrl('p'), || {
                    if let Some(s) = &mut r.completion_state {
                        s.select_prev_option();
                    } else {
                        r.completion_state = Some(
                            IdentifierCompleter(p)
                                .complete(r.expression.get(), r.expression.cursor_pos()),
                        );
                    }
                    set_completion(&r.completion_state, &mut r.expression);
                }))
                .finish();

            if res.is_some() {
                r.completion_state = None;
            }
            res.into()
        } else {
            input.into()
        };

        let res = after_completion
            .chain(
                NavigateBehavior::new(&mut self.table) //TODO: Fix this properly in lineedit
                    .down_on(Key::Char('\n')),
            )
            .chain(self.table.current_cell_behavior())
            .chain(
                NavigateBehavior::new(&mut self.table)
                    .up_on(Key::Up)
                    .down_on(Key::Down)
                    .left_on(Key::Left)
                    .right_on(Key::Right),
            )
            .finish();

        self.shrink_to_fit();
        self.update_results(p);
        res
    }
}
