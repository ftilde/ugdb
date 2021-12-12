use crate::gdbmi::commands::MiCommand;
use crate::gdbmi::output::{JsonValue, ResultClass};
use crate::Context;
use log::{error, info};
use std::ffi::OsString;
use std::ops::Range;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct CompletionState {
    original: String,
    cursor_pos: usize, //invariant: at grapheme cluster boundary
    completion_options: Vec<String>,
    current_option: usize, //invariant: in [0, options.size()], last is original
}

impl CompletionState {
    fn new(original: String, cursor_pos: usize, completion_options: Vec<String>) -> Self {
        CompletionState {
            original,
            cursor_pos,
            completion_options,
            current_option: 0,
        }
    }
    fn empty(original: String, cursor_pos: usize) -> Self {
        Self::new(original, cursor_pos, Vec::new())
    }
    pub fn current_line_parts(&self) -> (&str, &str, &str) {
        (
            &self.original[..self.cursor_pos],
            self.current_option(),
            &self.original[self.cursor_pos..],
        )
    }

    pub fn current_option(&self) -> &str {
        self.completion_options
            .get(self.current_option)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    fn num_options(&self) -> usize {
        self.completion_options.len() + 1
    }

    pub fn select_next_option(&mut self) {
        self.current_option = (self.current_option + 1) % self.num_options()
    }
    pub fn select_prev_option(&mut self) {
        self.current_option = if self.current_option == 0 {
            self.num_options() - 1
        } else {
            self.current_option - 1
        };
    }
}

pub trait Completer {
    fn complete(&mut self, original: &str, cursor_pos: usize) -> CompletionState;
}

struct CommandCompleter<'a> {
    binary_path: &'a Path,
    init_options: &'a [OsString],
}

fn gen_command_list(binary_path: &Path, init_options: &[OsString]) -> std::io::Result<Vec<String>> {
    let child = Command::new(binary_path)
        .args(init_options)
        .arg("-batch")
        .arg("-ex")
        .arg("help all")
        .stdout(Stdio::piped())
        .spawn()?;
    let gdb_output = child.wait_with_output()?;
    let gdb_output = String::from_utf8_lossy(&gdb_output.stdout);
    Ok(parse_command_names(&gdb_output))
}

fn parse_command_names(gdb_output: &str) -> Vec<String> {
    gdb_output
        .lines()
        .filter_map(|l| {
            let end = l.find(" -- ")?;
            let before_info = &l[..end];
            Some(before_info.split(","))
        })
        .flatten()
        .map(|l| l.trim().to_owned())
        .collect()
}

impl Completer for CommandCompleter<'_> {
    fn complete(&mut self, original: &str, cursor_pos: usize) -> CompletionState {
        // Possible optimization: Only generate command list once, but it does not appear to be a
        // real bottleneck so far.
        let candidates = match gen_command_list(self.binary_path, self.init_options) {
            Ok(commands) => find_candidates(&original[..cursor_pos], &commands),
            Err(e) => {
                error!("Failed to generate gdb command list: {}", e);
                Vec::new()
            }
        };
        CompletionState::new(original.to_owned(), cursor_pos, candidates)
    }
}

pub struct IdentifierCompleter<'a>(pub &'a mut Context);

struct VarObject {
    name: String,
    expr: Option<String>,
    typ: Option<String>,
}

impl VarObject {
    fn from_val(o: &JsonValue) -> Result<Self, String> {
        let name = if let Some(name) = o["name"].as_str() {
            name.to_string()
        } else {
            return Err(format!("Missing field 'name'"));
        };
        let expr = o["exp"].as_str().map(|s| s.to_owned());
        let typ = o["type"].as_str().map(|s| s.to_owned());
        Ok(VarObject { name, expr, typ })
    }
    fn create(p: &mut Context, expr: &str) -> Result<Self, String> {
        let res = p
            .gdb
            .mi
            .execute(MiCommand::var_create(None, expr, None))
            .map_err(|e| format!("{:?}", e))?;

        match res.class {
            ResultClass::Done => {}
            ResultClass::Error => return Err(format!("{}", res.results["msg"])),
            o => return Err(format!("Unexpected result class: {:?}", o)),
        }

        VarObject::from_val(&JsonValue::Object(res.results))
    }

    fn children(&self, p: &mut Context) -> Result<Vec<Self>, String> {
        let res = p
            .gdb
            .mi
            .execute(MiCommand::var_list_children(&self.name, true, None))
            .map_err(|e| format!("{:?}", e))?;

        match res.class {
            ResultClass::Done => {}
            ResultClass::Error => return Err(format!("{}", res.results["msg"])),
            o => return Err(format!("Unexpected result class: {:?}", o)),
        }

        Ok(res.results["children"]
            .members()
            .map(|c| VarObject::from_val(c))
            .collect::<Result<Vec<_>, String>>()?)
    }

    fn collect_children_exprs(
        &self,
        p: &mut Context,
        output: &mut Vec<String>,
    ) -> Result<(), String> {
        // try to flatten public/private fields etc.
        let flatten_exprs = ["<anonymous union>", "<anonymous struct>"];

        for child in self.children(p)? {
            if child.typ.is_none()
                || child.expr.is_none()
                || flatten_exprs
                    .iter()
                    .any(|n| *n == child.expr.as_ref().unwrap())
            {
                // This is the case for pseudo children (like public, ...)
                child.collect_children_exprs(p, output)?;
            } else {
                if let Some(expr) = child.expr {
                    output.push(expr);
                }
            }
        }
        Ok(())
    }

    fn delete(self, p: &mut Context) -> Result<(), String> {
        let res = p
            .gdb
            .mi
            .execute(MiCommand::var_delete(self.name, true))
            .map_err(|e| format!("{:?}", e))?;

        match res.class {
            ResultClass::Done => {}
            ResultClass::Error => return Err(format!("{}", res.results["msg"])),
            o => return Err(format!("Unexpected result class: {:?}", o)),
        }
        Ok(())
    }
}

fn get_children(p: &mut Context, expr: &str) -> Result<Vec<String>, String> {
    let root = VarObject::create(p, &expr)?;

    let mut children = Vec::new();

    root.collect_children_exprs(p, &mut children)?;

    root.delete(p)?;

    Ok(children)
}

fn get_variables(p: &mut Context) -> Result<Vec<String>, String> {
    let res = p
        .gdb
        .mi
        .execute(MiCommand::stack_list_variables(None, None))
        .map_err(|e| format!("{:?}", e))?;

    match res.class {
        ResultClass::Done => {}
        ResultClass::Error => return Err(format!("{}", res.results["msg"])),
        o => return Err(format!("Unexpected result class: {:?}", o)),
    }

    Ok(res.results["variables"]
        .members()
        .map(|o| o["name"].as_str().unwrap().to_string())
        .collect::<Vec<_>>())
}

impl Completer for IdentifierCompleter<'_> {
    fn complete(&mut self, original: &str, cursor_pos: usize) -> CompletionState {
        let expr = if let Ok(e) = CompletableExpression::from_str(&original[..cursor_pos]) {
            e
        } else {
            return CompletionState::empty(original.to_owned(), cursor_pos);
        };
        let res = if expr.parent.is_empty() {
            get_variables(self.0)
        } else {
            get_children(self.0, &expr.parent)
        };
        let children = match res {
            Ok(c) => c,
            Err(e) => {
                info!("Could not complete identifier: {:?}", e);
                vec![]
            }
        };
        let candidates = find_candidates(&expr.prefix, children.as_slice());
        CompletionState::new(original.to_owned(), cursor_pos, candidates)
    }
}

pub struct CmdlineCompleter<'a>(pub &'a mut Context);
impl Completer for CmdlineCompleter<'_> {
    fn complete(&mut self, original: &str, cursor_pos: usize) -> CompletionState {
        if original[..cursor_pos].find(' ').is_some() {
            // gdb command already typed, try to complete identifier in expression
            IdentifierCompleter(self.0).complete(original, cursor_pos)
        } else {
            // First "word" in command line, complete gdb command
            CommandCompleter {
                binary_path: self.0.gdb.mi.binary_path(),
                init_options: self.0.gdb.mi.init_options(),
            }
            .complete(original, cursor_pos)
        }
        //TODO: path completer? not sure how to distinguish between IdentifierCompleter and path
        //completer. Maybe based on gdb command...
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ExpressionTokenType {
    Atom,
    Arrow,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Asterisk,
    Sep,
}

#[derive(Clone, PartialEq, Debug)]
struct ExpressionToken {
    ttype: ExpressionTokenType,
    pos: Range<usize>,
}

#[derive(Debug, PartialEq)]
enum TokenizeError {
    UnfinishedString,
}

fn tokenize_expression(s: &str) -> Result<Vec<ExpressionToken>, TokenizeError> {
    let mut chars = s.chars().enumerate().peekable();
    let mut output = Vec::new();
    let is_atom_char = |c: char| c.is_alphanumeric() || c == '_';
    'outer: while let Some((i, c)) = chars.next() {
        let tok = |o: &mut Vec<ExpressionToken>, t: ExpressionTokenType| {
            o.push(ExpressionToken {
                ttype: t,
                pos: i..i + 1,
            });
        };
        match c {
            ' ' | '\t' | '\n' => {}
            '(' => tok(&mut output, ExpressionTokenType::LParen),
            '[' => tok(&mut output, ExpressionTokenType::LBracket),
            ')' => tok(&mut output, ExpressionTokenType::RParen),
            ']' => tok(&mut output, ExpressionTokenType::RBracket),
            '*' => tok(&mut output, ExpressionTokenType::Asterisk),
            '.' => tok(&mut output, ExpressionTokenType::Dot),
            '-' => match chars.peek().map(|(_, c)| *c) {
                Some('>') => {
                    let _ = chars.next();
                    output.push(ExpressionToken {
                        ttype: ExpressionTokenType::Arrow,
                        pos: i..i + 2,
                    });
                }
                Some('-') => {
                    let _ = chars.next();
                    output.push(ExpressionToken {
                        ttype: ExpressionTokenType::Sep,
                        pos: i..i + 1,
                    });
                    output.push(ExpressionToken {
                        ttype: ExpressionTokenType::Sep,
                        pos: i + 1..i + 2,
                    });
                }
                Some(_) | None => {
                    tok(&mut output, ExpressionTokenType::Sep);
                }
            },
            '"' => {
                let mut escaped = false;
                let start = i;
                while let Some((i, c)) = chars.next() {
                    match (c, escaped) {
                        ('"', false) => {
                            output.push(ExpressionToken {
                                ttype: ExpressionTokenType::Atom,
                                pos: start..i + 1,
                            });
                            continue 'outer;
                        }
                        (_, true) => escaped = false,
                        ('\\', false) => escaped = true,
                        (_, false) => {}
                    }
                }
                return Err(TokenizeError::UnfinishedString);
            }
            c if is_atom_char(c) => {
                let start = i;
                let mut prev_i = i;
                loop {
                    match chars.peek().cloned() {
                        Some((i, c)) if is_atom_char(c) => {
                            let _ = chars.next();
                            prev_i = i;
                        }
                        Some(_) | None => {
                            output.push(ExpressionToken {
                                ttype: ExpressionTokenType::Atom,
                                pos: start..prev_i + 1,
                            });
                            break;
                        }
                    }
                }
            }
            _ => tok(&mut output, ExpressionTokenType::Sep),
        }
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompletableExpression {
    parent: String,
    prefix: String,
}

impl CompletableExpression {
    fn from_str(s: &str) -> Result<Self, TokenizeError> {
        let mut tokens = tokenize_expression(s)?.into_iter().rev().peekable();
        let prefix;
        let mut needs_deref = false;
        if let Some(ExpressionToken {
            ttype: ExpressionTokenType::Atom,
            pos,
        }) = tokens.peek().cloned()
        {
            if pos.end == s.len() {
                prefix = &s[pos];
                let _ = tokens.next();
            } else {
                prefix = "";
            }
        } else {
            prefix = "";
        }
        let parent_end;
        match tokens.next() {
            Some(ExpressionToken {
                ttype: ExpressionTokenType::Dot,
                pos,
            }) => parent_end = pos.start,
            Some(ExpressionToken {
                ttype: ExpressionTokenType::Arrow,
                pos,
            }) => {
                parent_end = pos.start;
                needs_deref = true;
            }
            _ => {
                return Ok(CompletableExpression {
                    parent: "".to_owned(),
                    prefix: prefix.to_owned(),
                });
            }
        }

        // Now we need to find the beginning of parent!
        let mut paren_level = 0i32;
        let mut bracket_level = 0i32;
        let mut parent_begin = None;
        let mut active_atom = false;
        let mut prev_token_begin = s.len();
        for token in tokens {
            match token.ttype {
                ExpressionTokenType::RParen => {
                    paren_level += 1;
                    active_atom = false;
                }
                ExpressionTokenType::RBracket => {
                    bracket_level += 1;
                    active_atom = false;
                }
                ExpressionTokenType::LParen => {
                    paren_level -= 1;
                    active_atom = false;
                    if paren_level < 0 {
                        parent_begin = Some(prev_token_begin);
                        break;
                    }
                }
                ExpressionTokenType::LBracket => {
                    bracket_level -= 1;
                    active_atom = false;
                    if bracket_level < 0 {
                        parent_begin = Some(prev_token_begin);
                        break;
                    }
                }
                ExpressionTokenType::Dot | ExpressionTokenType::Arrow => {
                    active_atom = false;
                }
                t if paren_level == 0 && bracket_level == 0 => {
                    if t != ExpressionTokenType::Atom || active_atom {
                        parent_begin = Some(prev_token_begin);
                        break;
                    } else {
                        active_atom = true;
                    }
                }
                _ => {}
            }
            prev_token_begin = token.pos.start;
        }
        let parent_begin = parent_begin.unwrap_or(0);
        let parent = &s[parent_begin..parent_end];
        let parent = if needs_deref {
            format!("*({})", parent)
        } else {
            parent.to_owned()
        };

        Ok(CompletableExpression {
            parent,
            prefix: prefix.to_owned(),
        })
    }
}

fn find_candidates<'a, S: AsRef<str>>(prefix: &str, candidates: &'a [S]) -> Vec<String> {
    candidates
        .iter()
        .filter_map(|candidate| {
            if candidate.as_ref().starts_with(prefix) {
                Some(candidate.as_ref()[prefix.len()..].to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;

    fn current_line(state: &CompletionState) -> String {
        let parts = state.current_line_parts();
        format!("{}{}{}", parts.0, parts.1, parts.2)
    }
    #[test]
    fn test_completion_state() {
        let mut state =
            CompletionState::new("ba)".to_owned(), 2, vec!["r".to_owned(), "z".to_owned()]);
        assert_eq!(current_line(&state), "bar)");
        state.select_next_option();
        assert_eq!(current_line(&state), "baz)");
        state.select_next_option();
        assert_eq!(current_line(&state), "ba)");
        state.select_next_option();
        assert_eq!(current_line(&state), "bar)");
        state.select_prev_option();
        assert_eq!(current_line(&state), "ba)");
    }
    #[test]
    fn test_completion_state_empty() {
        let mut state = CompletionState::new("ba)".to_owned(), 2, vec![]);
        assert_eq!(current_line(&state), "ba)");
        state.select_next_option();
        assert_eq!(current_line(&state), "ba)");
        state.select_next_option();
        assert_eq!(current_line(&state), "ba)");
        state.select_next_option();
        assert_eq!(current_line(&state), "ba)");
        state.select_prev_option();
        assert_eq!(current_line(&state), "ba)");
    }
    #[test]
    fn test_gdb_help_parser() {
        let input = "
tsave -- Save the trace data to a file.
while-stepping -- Specify single-stepping behavior at a tracepoint.

Command class: user-defined

myadder -- User-defined.

Unclassified commands

add-inferior -- Add a new inferior.
help, h -- Print list of commands.
function _any_caller_is -- Check all calling function's names.
        ";
        let got = parse_command_names(input);
        let expected = [
            "tsave",
            "while-stepping",
            "myadder",
            "add-inferior",
            "help",
            "h",
            "function _any_caller_is",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        assert_eq!(got, expected);
    }
    #[test]
    fn test_command_completer() {
        let state = CommandCompleter {
            binary_path: Path::new("gdb"),
            init_options: &[],
        }
        .complete("he", 2);
        assert_eq!(current_line(&state), "help");
        assert_eq!(state.completion_options, vec!["lp"]);
    }
    #[test]
    fn test_find_candidates() {
        assert_eq!(find_candidates::<&str>("", &[]), Vec::<&str>::new());
        assert_eq!(find_candidates::<&str>("foo", &[]), Vec::<&str>::new());
        assert_eq!(
            find_candidates("", &["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
            vec!["foo", "bar", "baz"]
        );
        assert_eq!(
            find_candidates(
                "ba",
                &["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
            ),
            vec!["r", "z"]
        );
        assert_eq!(
            find_candidates(
                "baf",
                &["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
            ),
            Vec::<&str>::new()
        );
    }

    fn assert_eq_tokenize(s: &str, v: Vec<(ExpressionTokenType, Range<usize>)>) {
        let expected = v
            .into_iter()
            .map(|(t, r)| ExpressionToken { ttype: t, pos: r })
            .collect::<Vec<_>>();
        let got = tokenize_expression(s).unwrap();
        assert_eq!(got, expected);
    }
    #[test]
    fn test_tokenize_expression() {
        assert_eq_tokenize("", Vec::new());
        assert_eq_tokenize(
            "123 asdf",
            vec![
                (ExpressionTokenType::Atom, 0..3),
                (ExpressionTokenType::Atom, 4..8),
            ],
        );
        assert_eq_tokenize(
            " * * ,, .  ",
            vec![
                (ExpressionTokenType::Asterisk, 1..2),
                (ExpressionTokenType::Asterisk, 3..4),
                (ExpressionTokenType::Sep, 5..6),
                (ExpressionTokenType::Sep, 6..7),
                (ExpressionTokenType::Dot, 8..9),
            ],
        );
        assert_eq_tokenize(
            "(  (][)",
            vec![
                (ExpressionTokenType::LParen, 0..1),
                (ExpressionTokenType::LParen, 3..4),
                (ExpressionTokenType::RBracket, 4..5),
                (ExpressionTokenType::LBracket, 5..6),
                (ExpressionTokenType::RParen, 6..7),
            ],
        );
        assert_eq_tokenize(
            "< \"foo\"",
            vec![
                (ExpressionTokenType::Sep, 0..1),
                (ExpressionTokenType::Atom, 2..7),
            ],
        );
        assert_eq_tokenize(
            "-->",
            vec![
                (ExpressionTokenType::Sep, 0..1),
                (ExpressionTokenType::Sep, 1..2),
                (ExpressionTokenType::Sep, 2..3),
            ],
        );
        assert_eq_tokenize(
            "->-",
            vec![
                (ExpressionTokenType::Arrow, 0..2),
                (ExpressionTokenType::Sep, 2..3),
            ],
        );
        assert_eq_tokenize(
            "foo->bar",
            vec![
                (ExpressionTokenType::Atom, 0..3),
                (ExpressionTokenType::Arrow, 3..5),
                (ExpressionTokenType::Atom, 5..8),
            ],
        );
        assert_eq!(
            tokenize_expression(" asdf \" kldsj"),
            Err(TokenizeError::UnfinishedString)
        );
    }
    fn assert_eq_completable_expression(s: &str, parent: &str, prefix: &str) {
        let got = CompletableExpression::from_str(s).unwrap();
        let expected = CompletableExpression {
            parent: parent.to_owned(),
            prefix: prefix.to_owned(),
        };
        assert_eq!(got, expected);
    }
    #[test]
    fn test_completable_expression() {
        assert_eq_completable_expression("", "", "");
        assert_eq_completable_expression("foo.bar", "foo", "bar");
        assert_eq_completable_expression("foo->bar", "*(foo)", "bar");
        assert_eq_completable_expression("(foo[2]->bar", "*(foo[2])", "bar");
        assert_eq_completable_expression("][foo(1,23).", "foo(1,23)", "");
        assert_eq_completable_expression("][foo(1,23)", "", "");
        assert_eq_completable_expression("foo + b", "", "b");
        assert_eq_completable_expression("\"ldkf\" f", "", "f");
        assert_eq_completable_expression("  foo", "", "foo");
        assert_eq_completable_expression("foo ", "", "");
        assert_eq_completable_expression("f foo[2].f", "foo[2]", "f");
        assert_eq_completable_expression("f \"foo\"[2].f", "\"foo\"[2]", "f");
    }
}
