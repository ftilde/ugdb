use std::str::CharIndices;

pub type Location = usize;
pub type TokenWithLocation = (Location, Token, Location);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LSquareBracket,
    RSquareBracket,
    LPointyBracket,
    RPointyBracket,
    Comma,
    Equals,
    String,
    Text,
    Newline,
}

#[derive(Copy, Clone)]
enum LexerState {
    Free,
    PendingOutput(TokenWithLocation),
    InString(Location),
    InStringEscapedChar(Location),
    InText(Location, Location),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexicalError {
    UnfinishedString { begin_index: Location },
}

pub struct Lexer<'input> {
    chars: CharIndices<'input>,
    state: LexerState,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            chars: input.char_indices(),
            state: LexerState::Free,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<TokenWithLocation, LexicalError>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.chars.next() {
            let (output, new_state) = match self.state {
                LexerState::Free => match c {
                    '"' => (None, LexerState::InString(i)),
                    '{' => (Some((i, Token::LBrace, i + 1)), LexerState::Free),
                    '}' => (Some((i, Token::RBrace, i + 1)), LexerState::Free),
                    '[' => (Some((i, Token::LSquareBracket, i + 1)), LexerState::Free),
                    ']' => (Some((i, Token::RSquareBracket, i + 1)), LexerState::Free),
                    '(' => (Some((i, Token::LBracket, i + 1)), LexerState::Free),
                    ')' => (Some((i, Token::RBracket, i + 1)), LexerState::Free),
                    '<' => (Some((i, Token::LPointyBracket, i + 1)), LexerState::Free),
                    '>' => (Some((i, Token::RPointyBracket, i + 1)), LexerState::Free),
                    ',' => (Some((i, Token::Comma, i + 1)), LexerState::Free),
                    '=' => (Some((i, Token::Equals, i + 1)), LexerState::Free),
                    '\n' => (Some((i, Token::Newline, i + 1)), LexerState::Free),
                    ' ' | '\t' => (None, LexerState::Free),
                    _ => (None, LexerState::InText(i, i + 1)),
                },
                LexerState::PendingOutput(output) => match c {
                    '"' => (Some(output), LexerState::InString(i)),
                    '{' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::LBrace, i + 1)),
                    ),
                    '}' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::RBrace, i + 1)),
                    ),
                    '[' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::LSquareBracket, i + 1)),
                    ),
                    ']' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::RSquareBracket, i + 1)),
                    ),
                    '(' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::LBracket, i + 1)),
                    ),
                    ')' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::RBracket, i + 1)),
                    ),
                    '<' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::LPointyBracket, i + 1)),
                    ),
                    '>' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::RPointyBracket, i + 1)),
                    ),
                    ',' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::Comma, i + 1)),
                    ),
                    '=' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::Equals, i + 1)),
                    ),
                    '\n' => (
                        Some(output),
                        LexerState::PendingOutput((i, Token::Newline, i + 1)),
                    ),
                    ' ' | '\t' => (Some(output), LexerState::Free),
                    _ => (Some(output), LexerState::InText(i, i + 1)),
                },
                LexerState::InString(begin) => match c {
                    '"' => (Some((begin, Token::String, i + 1)), LexerState::Free),
                    '\\' => (None, LexerState::InStringEscapedChar(begin)),
                    _ => (None, LexerState::InString(begin)),
                },
                LexerState::InStringEscapedChar(begin) => match c {
                    _ => (None, LexerState::InString(begin)),
                },
                LexerState::InText(begin, end) => match c {
                    '"' => (Some((begin, Token::Text, end)), LexerState::InString(i)),
                    '{' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::LBrace, i + 1)),
                    ),
                    '}' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::RBrace, i + 1)),
                    ),
                    '[' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::LSquareBracket, i + 1)),
                    ),
                    ']' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::RSquareBracket, i + 1)),
                    ),
                    '(' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::LBracket, i + 1)),
                    ),
                    ')' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::RBracket, i + 1)),
                    ),
                    '<' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::LPointyBracket, i + 1)),
                    ),
                    '>' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::RPointyBracket, i + 1)),
                    ),
                    ',' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::Comma, i + 1)),
                    ),
                    '=' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::Equals, i + 1)),
                    ),
                    '\n' => (
                        Some((begin, Token::Text, end)),
                        LexerState::PendingOutput((i, Token::Newline, i + 1)),
                    ),
                    ' ' | '\t' => (None, LexerState::InText(begin, end)),
                    _ => (None, LexerState::InText(begin, i + 1)),
                },
            };
            self.state = new_state;
            if let Some(output) = output {
                return Some(Ok(output));
            }
        }
        match self.state {
            LexerState::Free => None,
            LexerState::PendingOutput(output) => {
                self.state = LexerState::Free;
                Some(Ok(output))
            }
            LexerState::InString(begin) | LexerState::InStringEscapedChar(begin) => {
                self.state = LexerState::Free;
                Some(Err(LexicalError::UnfinishedString { begin_index: begin }))
            }
            LexerState::InText(begin, end) => {
                self.state = LexerState::Free;
                Some(Ok((begin, Token::Text, end)))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_eq_lexer_tokens(s: &'static str, expected_tokens: &[Token]) {
        let tokens = Lexer::new(s)
            .map(|t| t.map(|val| val.1))
            .collect::<Result<Vec<_>, LexicalError>>()
            .unwrap();
        assert_eq!(tokens.as_slice(), expected_tokens);
    }

    #[test]
    fn test_lexer_basic_success() {
        assert_eq_lexer_tokens("", &[]);
        assert_eq_lexer_tokens(
            "lj \"dlfj}[{}]=,  \\t \\\"\"    dfdf sadfad\n {{  []},   =\t\n\t",
            &[
                Token::Text,
                Token::String,
                Token::Text,
                Token::Newline,
                Token::LBrace,
                Token::LBrace,
                Token::LSquareBracket,
                Token::RSquareBracket,
                Token::RBrace,
                Token::Comma,
                Token::Equals,
                Token::Newline,
            ],
        );
    }

    fn assert_eq_lexer_error(s: &'static str, expected_error: LexicalError) {
        match Lexer::new(s)
            .map(|t| t.map(|val| val.1))
            .collect::<Result<Vec<_>, LexicalError>>()
        {
            Ok(tokens) => panic!("Unexpected successful lexing of \"{}\": {:?}", s, tokens),
            Err(err) => assert_eq!(err, expected_error),
        }
    }

    #[test]
    fn test_lexer_error() {
        assert_eq_lexer_error("\"", LexicalError::UnfinishedString { begin_index: 0 });
        assert_eq_lexer_error(
            "\"{,..\"\"",
            LexicalError::UnfinishedString { begin_index: 6 },
        );
    }
}
