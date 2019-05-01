use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

struct CompletionState {
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
    fn current_line(&self) -> String {
        format!(
            "{}{}{}",
            &self.original[..self.cursor_pos],
            self.current_option(),
            &self.original[self.cursor_pos..],
        )
    }

    fn current_option(&self) -> &str {
        self.completion_options
            .get(self.current_option)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    fn num_options(&self) -> usize {
        self.completion_options.len() + 1
    }

    fn select_next_option(&mut self) {
        self.current_option = (self.current_option + 1) % self.num_options()
    }
    fn select_prev_option(&mut self) {
        self.current_option = if self.current_option == 0 {
            self.num_options() - 1
        } else {
            self.current_option - 1
        };
    }
}

trait Completer {
    fn complete(&self, original: &str, cursor_pos: usize) -> CompletionState;
}

struct CommandCompleter;

const GDB_COMMANDS: &[&str] = &["help", "break", "print"];

impl Completer for CommandCompleter {
    fn complete(&self, original: &str, cursor_pos: usize) -> CompletionState {
        let candidates = find_candidates(&original[..cursor_pos], GDB_COMMANDS);
        CompletionState::new(original.to_owned(), cursor_pos, candidates)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ExpressionTokenType {
    Atom,
    Arrow,
    Dot,
    LParen,
    RParen,
    Asterisk,
    Sep,
    String,
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

fn tokenize_espression(s: &str) -> Result<Vec<ExpressionToken>, TokenizeError> {
    let mut chars = s.chars().enumerate().peekable();
    let mut output = Vec::new();
    let is_atom_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    'outer: while let Some((i, c)) = chars.next() {
        let tok = |o: &mut Vec<ExpressionToken>, t: ExpressionTokenType| {
            o.push(ExpressionToken {
                ttype: t,
                pos: i..i + 1,
            });
        };
        match c {
            ' ' | '\t' | '\n' => {}
            '(' | '[' => tok(&mut output, ExpressionTokenType::LParen),
            ')' | ']' => tok(&mut output, ExpressionTokenType::RParen),
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
                                ttype: ExpressionTokenType::String,
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

//struct IdentifierCompleter;

#[derive(Debug, PartialEq)]
pub struct CompleterPath<'s> {
    path: Vec<&'s str>,
    incomplete: &'s str,
}

impl<'a> CompleterPath<'a> {
    pub fn from_str(s: &'a str) -> Self {
        let mut v = completable_path(s).into_iter();
        let incomplete = v.next().expect("complete_path yields at least one element");
        let path = v.rev().collect();

        CompleterPath { incomplete, path }
    }
}
fn completable_path(p: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut end = p.len();
    let mut begin = p.len();
    for (i, g) in p.grapheme_indices(true).rev() {
        let mut chars = g.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if c.is_alphabetic() {
                begin = i;
            } else if c == '.' {
                result.push(&p[begin..end]);
                begin = i;
                end = i;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result.push(&p[begin..end]);
    result
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

    #[test]
    fn test_completion_state() {
        let mut state =
            CompletionState::new("ba)".to_owned(), 2, vec!["r".to_owned(), "z".to_owned()]);
        assert_eq!(state.current_line(), "bar)");
        state.select_next_option();
        assert_eq!(state.current_line(), "baz)");
        state.select_next_option();
        assert_eq!(state.current_line(), "ba)");
        state.select_next_option();
        assert_eq!(state.current_line(), "bar)");
        state.select_prev_option();
        assert_eq!(state.current_line(), "ba)");
    }
    #[test]
    fn test_completion_state_empty() {
        let mut state = CompletionState::new("ba)".to_owned(), 2, vec![]);
        assert_eq!(state.current_line(), "ba)");
        state.select_next_option();
        assert_eq!(state.current_line(), "ba)");
        state.select_next_option();
        assert_eq!(state.current_line(), "ba)");
        state.select_next_option();
        assert_eq!(state.current_line(), "ba)");
        state.select_prev_option();
        assert_eq!(state.current_line(), "ba)");
    }
    #[test]
    fn test_identifier_completer() {
        let state = CommandCompleter.complete("he", 2);
        assert_eq!(state.current_line(), "help");
        assert_eq!(state.completion_options, vec!["lp"]);
    }
    #[test]
    fn test_completable_path() {
        assert_eq!(completable_path(""), vec![""]);
        assert_eq!(completable_path("foo.bar.baz"), vec!["baz", "bar", "foo"]);
        assert_eq!(
            completable_path("(fdf(foo.bar.baz"),
            vec!["baz", "bar", "foo"]
        );
        assert_eq!(
            completable_path("(fdf(foo.bar.baz"),
            vec!["baz", "bar", "foo"]
        );
        assert_eq!(
            completable_path("(fdf(foo.bar.baz."),
            vec!["", "baz", "bar", "foo"]
        );
        assert_eq!(completable_path("aäöüß"), vec!["aäöüß"]);
        assert_eq!(completable_path("f🦀aäöüßz"), vec!["aäöüßz"]);
        assert_eq!(completable_path("f aäöüßz"), vec!["aäöüßz"]);
    }
    #[test]
    fn test_completerpath() {
        assert_eq!(
            CompleterPath::from_str(""),
            CompleterPath {
                path: vec![],
                incomplete: ""
            }
        );
        assert_eq!(
            CompleterPath::from_str("foo.bar.baz"),
            CompleterPath {
                path: vec!["foo", "bar"],
                incomplete: "baz"
            }
        );
        assert_eq!(
            CompleterPath::from_str("(fdf(foo.bar.baz"),
            CompleterPath {
                path: vec!["foo", "bar"],
                incomplete: "baz"
            }
        );
        assert_eq!(
            CompleterPath::from_str("(fdf(foo.bar.baz."),
            CompleterPath {
                path: vec!["foo", "bar", "baz"],
                incomplete: ""
            }
        );
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
        let got = tokenize_espression(s).unwrap();
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
                (ExpressionTokenType::RParen, 4..5),
                (ExpressionTokenType::LParen, 5..6),
                (ExpressionTokenType::RParen, 6..7),
            ],
        );
        assert_eq_tokenize(
            "< \"foo\"",
            vec![
                (ExpressionTokenType::Sep, 0..1),
                (ExpressionTokenType::String, 2..7),
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
        assert_eq!(
            tokenize_espression(" asdf \" kldsj"),
            Err(TokenizeError::UnfinishedString)
        );
    }
}
