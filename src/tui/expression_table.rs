use crate::gdb_expression_parsing::Format;
use crate::gdbmi::{commands::MiCommand, output::ResultClass, ExecuteError};
use crate::Context;
use unsegen::{
    base::{Color, GraphemeCluster, StyleModifier},
    container::Container,
    input::{EditBehavior, Input, Key, NavigateBehavior, ScrollBehavior},
    widget::{
        builtin::{Column, LineEdit, Table, TableRow},
        SeparatingStyle, Widget,
    },
};
use unsegen_jsonviewer::JsonViewer;

use crate::completion::{Completer, CompletionState, IdentifierCompleter};

pub struct ExpressionRow {
    expression: LineEdit,
    completion_state: Option<CompletionState>,
    result: JsonViewer,
    format: Option<crate::gdb_expression_parsing::Format>,
}

fn next_format(f: Option<Format>) -> Option<Format> {
    match f {
        None => Some(Format::Hex),
        Some(Format::Hex) => Some(Format::Decimal),
        Some(Format::Decimal) => Some(Format::Octal),
        Some(Format::Octal) => Some(Format::Binary),
        Some(Format::Binary) => None,
    }
}

impl ExpressionRow {
    fn new() -> Self {
        ExpressionRow {
            expression: LineEdit::new(),
            completion_state: None,
            result: JsonViewer::new(" "),
            format: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.expression.get().is_empty()
    }
    fn update_result(&mut self, p: &mut Context) {
        let expr = self.expression.get().to_owned();
        if expr.is_empty() {
            self.result.update(" ");
        } else {
            match p.gdb.mi.execute(MiCommand::data_evaluate_expression(expr)) {
                Ok(res) => match res.class {
                    ResultClass::Error => {
                        self.result.update(&res.results["msg"]);
                    }
                    ResultClass::Done => {
                        let to_parse = res.results["value"].as_str().expect("value present");
                        match crate::gdb_expression_parsing::parse_gdb_value(to_parse) {
                            Ok(n) => {
                                let v = crate::gdb_expression_parsing::Value {
                                    node: &n,
                                    format: self.format,
                                };
                                self.result.update(v);
                            }
                            Err(_) => {
                                self.result
                                    .update(format!("*Error parsing*: {}", to_parse).as_str());
                            }
                        }
                    }
                    other => panic!("unexpected result class: {:?}", other),
                },
                Err(ExecuteError::Busy) => {}
                Err(ExecuteError::Quit) => {
                    panic!("GDB quit!");
                }
            }
        }
    }
}
impl TableRow for ExpressionRow {
    type BehaviorContext = Context;
    const COLUMNS: &'static [Column<ExpressionRow>] = &[
        Column {
            access: |r| Box::new(r.expression.as_widget()),
            behavior: |r, input, p| {
                let mut format_changed = false;
                let prev_content = r.expression.get().to_owned();
                let set_completion =
                    |completion_state: &Option<CompletionState>, expression: &mut LineEdit| {
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
                    .chain((Key::Ctrl('f'), || {
                        r.format = next_format(r.format);
                        format_changed = true;
                    }))
                    .if_not_consumed(|| r.completion_state = None)
                    .chain((Key::Ctrl('w'), || {
                        match p.gdb.mi.execute(MiCommand::insert_watchpoing(
                            r.expression.get(),
                            crate::gdbmi::commands::WatchMode::Access,
                        )) {
                            Ok(o) => match o.class {
                                ResultClass::Done => {
                                    p.log(format!(
                                        "Inserted watchpoint for expression \"{}\"",
                                        r.expression.get()
                                    ));
                                }
                                ResultClass::Error => {
                                    p.log(format!(
                                        "Failed to set watchpoint: {}",
                                        o.results["msg"].as_str().unwrap(),
                                    ));
                                }
                                other => panic!("unexpected result class: {:?}", other),
                            },
                            Err(e) => {
                                p.log(format!("Failed to set watchpoint: {:?}", e));
                            }
                        }
                    }))
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
                    .finish();

                if r.expression.get() != prev_content || format_changed {
                    r.update_result(p);
                }
                res
            },
        },
        Column {
            access: |r| Box::new(r.result.as_widget()),
            behavior: |r, input, _| {
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
        let mut table = Table::new();
        table.rows_mut().push(ExpressionRow::new()); //Invariant: always at least one line
        ExpressionTable { table }
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

    pub fn update_results(&mut self, p: &mut Context) {
        for row in self.table.rows_mut().iter_mut() {
            row.update_result(p);
        }
    }
}

impl Container<Context> for ExpressionTable {
    fn input(&mut self, input: Input, p: &mut Context) -> Option<Input> {
        let res = input
            .chain(
                NavigateBehavior::new(&mut self.table) //TODO: Fix this properly in lineedit
                    .down_on(Key::Char('\n')),
            )
            .chain(self.table.current_cell_behavior(p))
            .chain(
                NavigateBehavior::new(&mut self.table)
                    .up_on(Key::Up)
                    .down_on(Key::Down)
                    .left_on(Key::Left)
                    .right_on(Key::Right),
            )
            .finish();
        self.shrink_to_fit();
        res
    }

    fn as_widget<'a>(&'a self) -> Box<dyn Widget + 'a> {
        Box::new(
            self.table
                .as_widget()
                .row_separation(SeparatingStyle::AlternatingStyle(
                    StyleModifier::new().bg_color(Color::Black),
                ))
                .col_separation(SeparatingStyle::Draw(
                    GraphemeCluster::try_from('│').unwrap(),
                ))
                .focused(StyleModifier::new().bold(true)),
        )
    }
}
