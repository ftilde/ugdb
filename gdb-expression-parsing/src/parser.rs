// auto-generated: "lalrpop 0.14.0"
use super::lexer;
use super::lexer::{Span};
use super::ast::{Node, build_vec};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;

mod __parse__Value {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens)]

    use super::super::lexer;
    use super::super::lexer::{Span};
    use super::super::ast::{Node, build_vec};
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    use super::__ToTriple;
    #[allow(dead_code)]
    pub enum __Symbol<>
     {
        Term_22_2c_22(lexer::Token),
        Term_22_3d_22(lexer::Token),
        Term_22_5b_22(lexer::Token),
        Term_22_5d_22(lexer::Token),
        Term_22_7b_22(lexer::Token),
        Term_22_7d_22(lexer::Token),
        TermString(lexer::Token),
        TermText(lexer::Token),
        Termerror(__lalrpop_util::ErrorRecovery<lexer::Location, lexer::Token, lexer::LexicalError>),
        Nt_28_3cMapElement_3e_20_22_2c_22_29((Span, Node)),
        Nt_28_3cMapElement_3e_20_22_2c_22_29_2a(::std::vec::Vec<(Span, Node)>),
        Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(::std::vec::Vec<(Span, Node)>),
        Nt_28_3cValue_3e_20_22_2c_22_29(Node),
        Nt_28_3cValue_3e_20_22_2c_22_29_2a(::std::vec::Vec<Node>),
        Nt_28_3cValue_3e_20_22_2c_22_29_2b(::std::vec::Vec<Node>),
        Nt_40L(lexer::Location),
        Nt_40R(lexer::Location),
        NtArray(Node),
        NtKey(Span),
        NtMap(Node),
        NtMapElement((Span, Node)),
        NtScalar(Span),
        NtTextOrString(Span),
        NtTextOrString_2b(::std::vec::Vec<Span>),
        NtValue(Node),
        Nt____Value(Node),
    }
    const __ACTION: &'static [i32] = &[
        // State 0
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 1
        -51, 0, 0, -51, 0, -51, 9, 10, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 0, 4, 13, 5, 0, 6, 7, 8,
        // State 4
        0, 0, 4, 0, 5, 16, 17, 18, 8,
        // State 5
        -37, 0, 0, -37, 0, -37, -37, -37, 0,
        // State 6
        -38, 0, 0, -38, 0, -38, -38, -38, 0,
        // State 7
        -52, 0, 0, -52, 0, -52, 0, 0, 0,
        // State 8
        -39, 0, 0, -39, 0, -39, -39, -39, 0,
        // State 9
        -40, 0, 0, -40, 0, -40, -40, -40, 0,
        // State 10
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 11
        20, 0, 0, 21, 0, 0, 0, 0, 0,
        // State 12
        -41, 0, 0, -41, 0, -41, 0, 0, 0,
        // State 13
        0, 0, 4, 0, 5, 0, 23, 24, 8,
        // State 14
        25, 0, 0, 0, 0, 26, 0, 0, 0,
        // State 15
        -44, 0, 0, -44, 0, -44, 0, 0, 0,
        // State 16
        -37, 27, 0, 0, 0, -37, -37, -37, 0,
        // State 17
        -38, 28, 0, 0, 0, -38, -38, -38, 0,
        // State 18
        29, 0, 0, 30, 0, 0, 0, 0, 0,
        // State 19
        0, 0, -15, 0, -15, 0, -15, -15, -15,
        // State 20
        -42, 0, 0, -42, 0, -42, 0, 0, 0,
        // State 21
        31, 0, 0, 0, 0, 32, 0, 0, 0,
        // State 22
        -37, 33, 0, 0, 0, -37, -37, -37, 0,
        // State 23
        -38, 34, 0, 0, 0, -38, -38, -38, 0,
        // State 24
        0, 0, -8, 0, -8, 0, -8, -8, -8,
        // State 25
        -49, 0, 0, -49, 0, -49, 0, 0, 0,
        // State 26
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 27
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 28
        0, 0, -16, 0, -16, 0, -16, -16, -16,
        // State 29
        -43, 0, 0, -43, 0, -43, 0, 0, 0,
        // State 30
        0, 0, -11, 0, -11, 0, -11, -11, -11,
        // State 31
        -50, 0, 0, -50, 0, -50, 0, 0, 0,
        // State 32
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 33
        0, 0, 4, 0, 5, 0, 6, 7, 8,
        // State 34
        39, 0, 0, 0, 0, 40, 0, 0, 0,
        // State 35
        41, 0, 0, 0, 0, 42, 0, 0, 0,
        // State 36
        43, 0, 0, 0, 0, 44, 0, 0, 0,
        // State 37
        45, 0, 0, 0, 0, 46, 0, 0, 0,
        // State 38
        0, 0, -6, 0, -6, 0, -6, -6, -6,
        // State 39
        -45, 0, 0, -45, 0, -45, 0, 0, 0,
        // State 40
        0, 0, -7, 0, -7, 0, -7, -7, -7,
        // State 41
        -47, 0, 0, -47, 0, -47, 0, 0, 0,
        // State 42
        0, 0, -9, 0, -9, 0, -9, -9, -9,
        // State 43
        -46, 0, 0, -46, 0, -46, 0, 0, 0,
        // State 44
        0, 0, -10, 0, -10, 0, -10, -10, -10,
        // State 45
        -48, 0, 0, -48, 0, -48, 0, 0, 0,
    ];
    const __EOF_ACTION: &'static [i32] = &[
        // State 0
        0,
        // State 1
        -51,
        // State 2
        -53,
        // State 3
        0,
        // State 4
        0,
        // State 5
        -37,
        // State 6
        -38,
        // State 7
        -52,
        // State 8
        -39,
        // State 9
        -40,
        // State 10
        0,
        // State 11
        0,
        // State 12
        -41,
        // State 13
        0,
        // State 14
        0,
        // State 15
        -44,
        // State 16
        0,
        // State 17
        0,
        // State 18
        0,
        // State 19
        0,
        // State 20
        -42,
        // State 21
        0,
        // State 22
        0,
        // State 23
        0,
        // State 24
        0,
        // State 25
        -49,
        // State 26
        0,
        // State 27
        0,
        // State 28
        0,
        // State 29
        -43,
        // State 30
        0,
        // State 31
        -50,
        // State 32
        0,
        // State 33
        0,
        // State 34
        0,
        // State 35
        0,
        // State 36
        0,
        // State 37
        0,
        // State 38
        0,
        // State 39
        -45,
        // State 40
        0,
        // State 41
        -47,
        // State 42
        0,
        // State 43
        -46,
        // State 44
        0,
        // State 45
        -48,
    ];
    const __GOTO: &'static [i32] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 2, 12, 0,
        // State 4
        0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 15, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 19, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 22, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 22
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 35, 0,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 36, 0,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 37, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 38, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    fn __expected_tokens(__state: usize) -> Vec<::std::string::String> {
        const __TERMINAL: &'static [&'static str] = &[
            r###"",""###,
            r###""=""###,
            r###""[""###,
            r###""]""###,
            r###""{""###,
            r###""}""###,
            r###"String"###,
            r###"Text"###,
        ];
        __ACTION[(__state * 9)..].iter().zip(__TERMINAL).filter_map(|(&state, terminal)| {
            if state == 0 {
                None
            } else {
                Some(terminal.to_string())
            }
        }).collect()
    }
    #[allow(dead_code)]
    pub fn parse_Value<
        __TOKEN: __ToTriple<Error=lexer::LexicalError>,
        __TOKENS: IntoIterator<Item=__TOKEN>,
    >(
        __tokens0: __TOKENS,
    ) -> Result<Node, __lalrpop_util::ParseError<lexer::Location, lexer::Token, lexer::LexicalError>>
    {
        let __tokens = __tokens0.into_iter();
        let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
        let mut __states = vec![0_i32];
        let mut __symbols = vec![];
        let mut __integer;
        let mut __lookahead;
        let __last_location = &mut Default::default();
        '__shift: loop {
            __lookahead = match __tokens.next() {
                Some(Ok(v)) => v,
                None => break '__shift,
                Some(Err(e)) => return Err(__lalrpop_util::ParseError::User { error: e }),
            };
            *__last_location = __lookahead.2.clone();
            __integer = match __lookahead.1 {
                lexer::Token::Comma if true => 0,
                lexer::Token::Equals if true => 1,
                lexer::Token::LSquareBracket if true => 2,
                lexer::Token::RSquareBracket if true => 3,
                lexer::Token::LBrace if true => 4,
                lexer::Token::RBrace if true => 5,
                lexer::Token::String if true => 6,
                lexer::Token::Text if true => 7,
                _ => {
                    let __state = *__states.last().unwrap() as usize;
                    let __error = __lalrpop_util::ParseError::UnrecognizedToken {
                        token: Some(__lookahead),
                        expected: __expected_tokens(__state),
                    };
                    return Err(__error);
                }
            };
            '__inner: loop {
                let __state = *__states.last().unwrap() as usize;
                let __action = __ACTION[__state * 9 + __integer];
                if __action > 0 {
                    let __symbol = match __integer {
                        0 => match __lookahead.1 {
                            __tok @ lexer::Token::Comma => __Symbol::Term_22_2c_22((__tok)),
                            _ => unreachable!(),
                        },
                        1 => match __lookahead.1 {
                            __tok @ lexer::Token::Equals => __Symbol::Term_22_3d_22((__tok)),
                            _ => unreachable!(),
                        },
                        2 => match __lookahead.1 {
                            __tok @ lexer::Token::LSquareBracket => __Symbol::Term_22_5b_22((__tok)),
                            _ => unreachable!(),
                        },
                        3 => match __lookahead.1 {
                            __tok @ lexer::Token::RSquareBracket => __Symbol::Term_22_5d_22((__tok)),
                            _ => unreachable!(),
                        },
                        4 => match __lookahead.1 {
                            __tok @ lexer::Token::LBrace => __Symbol::Term_22_7b_22((__tok)),
                            _ => unreachable!(),
                        },
                        5 => match __lookahead.1 {
                            __tok @ lexer::Token::RBrace => __Symbol::Term_22_7d_22((__tok)),
                            _ => unreachable!(),
                        },
                        6 => match __lookahead.1 {
                            __tok @ lexer::Token::String => __Symbol::TermString((__tok)),
                            _ => unreachable!(),
                        },
                        7 => match __lookahead.1 {
                            __tok @ lexer::Token::Text => __Symbol::TermText((__tok)),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };
                    __states.push(__action - 1);
                    __symbols.push((__lookahead.0, __symbol, __lookahead.2));
                    continue '__shift;
                } else if __action < 0 {
                    if let Some(r) = __reduce(__action, Some(&__lookahead.0), &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                        if r.is_err() {
                            return r;
                        }
                        return Err(__lalrpop_util::ParseError::ExtraToken { token: __lookahead });
                    }
                } else {
                    let mut __err_lookahead = Some(__lookahead);
                    let mut __err_integer: Option<usize> = Some(__integer);
                    match __error_recovery( &mut __tokens, &mut __states, &mut __symbols, __last_location, &mut __err_lookahead, &mut __err_integer, ::std::marker::PhantomData::<()>) {
                        Err(__e) => return Err(__e),
                        Ok(Some(__v)) => return Ok(__v),
                        Ok(None) => (),
                    }
                    match (__err_lookahead, __err_integer) {
                        (Some(__l), Some(__i)) => {
                            __lookahead = __l;
                            __integer = __i;
                            continue '__inner;
                        }
                        _ => break '__shift,
                    }
                }
            }
        }
        loop {
            let __state = *__states.last().unwrap() as usize;
            let __action = __EOF_ACTION[__state];
            if __action < 0 {
                if let Some(r) = __reduce(__action, None, &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                    return r;
                }
            } else {
                let mut __err_lookahead = None;
                let mut __err_integer: Option<usize> = None;
                match __error_recovery( &mut __tokens, &mut __states, &mut __symbols, __last_location, &mut __err_lookahead, &mut __err_integer, ::std::marker::PhantomData::<()>) {
                    Err(__e) => return Err(__e),
                    Ok(Some(__v)) => return Ok(__v),
                    Ok(None) => (),
                }
            }
        }
    }
    fn __error_recovery<
        __I,
    >(
        __tokens: &mut __I,
        __states: &mut ::std::vec::Vec<i32>,
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>,
        __last_location: &mut lexer::Location,
        __opt_lookahead: &mut Option<(lexer::Location, lexer::Token, lexer::Location)>,
        __opt_integer: &mut Option<usize>,
        _: ::std::marker::PhantomData<()>,
    ) -> Result<Option<Node>, __lalrpop_util::ParseError<lexer::Location, lexer::Token, lexer::LexicalError>> where
      __I: Iterator<Item = Result<(lexer::Location, lexer::Token, lexer::Location), lexer::LexicalError>>,
    {
        let __state = *__states.last().unwrap() as usize;
        let __error = __lalrpop_util::ParseError::UnrecognizedToken {
            token: __opt_lookahead.clone(),
            expected: __expected_tokens(__state),
        };
        let mut __dropped_tokens = vec![];
        loop {
            let __state = *__states.last().unwrap() as usize;
            let __action = __ACTION[__state * 9 + 8];
            if __action >= 0 {
                break;
            }
            let __lookahead_start = __opt_lookahead.as_ref().map(|l| &l.0);
            if let Some(r) = __reduce(  __action, __lookahead_start, __states, __symbols, ::std::marker::PhantomData::<()> ) {
                return Ok(Some(r?));
            }
        }
        let __states_len = __states.len();
        let __top0;
        '__find_state: loop {
            for __top in (0..__states_len).rev() {
                let __state = __states[__top];
                let __action = __ACTION[(__state * 9 + 8) as usize];
                if __action <= 0 { continue; }
                let __error_state = __action - 1;
                if __accepts( __error_state, &__states[..__top + 1], *__opt_integer, ::std::marker::PhantomData::<()>,) {
                    __top0 = __top;
                    break '__find_state;
                }
            }
            '__eof: loop {
                match __opt_lookahead.take() {
                    None => {
                        return Err(__error)
                    }
                    Some(mut __lookahead) => {
                        __dropped_tokens.push(__lookahead);
                        __lookahead = match __tokens.next() {
                            Some(Ok(v)) => v,
                            None => break '__eof,
                            Some(Err(e)) => return Err(__lalrpop_util::ParseError::User { error: e }),
                        };
                        *__last_location = __lookahead.2.clone();
                        let __integer;
                        __integer = match __lookahead.1 {
                            lexer::Token::Comma if true => 0,
                            lexer::Token::Equals if true => 1,
                            lexer::Token::LSquareBracket if true => 2,
                            lexer::Token::RSquareBracket if true => 3,
                            lexer::Token::LBrace if true => 4,
                            lexer::Token::RBrace if true => 5,
                            lexer::Token::String if true => 6,
                            lexer::Token::Text if true => 7,
                            _ => {
                                let __state = *__states.last().unwrap() as usize;
                                let __error = __lalrpop_util::ParseError::UnrecognizedToken {
                                    token: Some(__lookahead),
                                    expected: __expected_tokens(__state),
                                };
                                return Err(__error);
                            }
                        };
                        *__opt_lookahead = Some(__lookahead);
                        *__opt_integer = Some(__integer);
                        continue '__find_state;
                    }
                }
            }
            *__opt_lookahead = None;
            *__opt_integer = None;
        };
        let __top = __top0;
        let __start = if let Some(__popped_sym) = __symbols.get(__top) {
            __popped_sym.0.clone()
        } else if let Some(__dropped_token) = __dropped_tokens.first() {
            __dropped_token.0.clone()
        } else if __top > 0 {
            __symbols[__top - 1].2.clone()
        } else {
            Default::default()
        };
        let __end = if let Some(__dropped_token) = __dropped_tokens.last() {
            __dropped_token.2.clone()
        } else if __states_len - 1 > __top {
            __symbols.last().unwrap().2.clone()
        } else if let Some(__lookahead) = __opt_lookahead.as_ref() {
            __lookahead.0.clone()
        } else {
            __start.clone()
        };
        __states.truncate(__top + 1);
        __symbols.truncate(__top);
        let __recover_state = __states[__top];
        let __error_action = __ACTION[(__recover_state * 9 + 8) as usize];
        let __error_state = __error_action - 1;
        __states.push(__error_state);
        let __recovery = __lalrpop_util::ErrorRecovery {
            error: __error,
            dropped_tokens: __dropped_tokens,
        };
        __symbols.push((__start, __Symbol::Termerror(__recovery), __end));
        Ok(None)
    }
    fn __accepts<
    >(
        __error_state: i32,
        __states: & [i32],
        __opt_integer: Option<usize>,
        _: ::std::marker::PhantomData<()>,
    ) -> bool
    {
        let mut __states = __states.to_vec();
        __states.push(__error_state);
        loop {
            let mut __states_len = __states.len();
            let __top = __states[__states_len - 1];
            let __action = match __opt_integer {
                None => __EOF_ACTION[__top as usize],
                Some(__integer) => __ACTION[(__top * 9) as usize + __integer],
            };
            if __action == 0 { return false; }
            if __action > 0 { return true; }
            let (__to_pop, __nt) = match -__action {
                1 => {
                    (4, 0)
                }
                2 => {
                    (4, 0)
                }
                3 => {
                    (2, 0)
                }
                4 => {
                    (0, 1)
                }
                5 => {
                    (1, 1)
                }
                6 => {
                    (4, 2)
                }
                7 => {
                    (4, 2)
                }
                8 => {
                    (2, 2)
                }
                9 => {
                    (5, 2)
                }
                10 => {
                    (5, 2)
                }
                11 => {
                    (3, 2)
                }
                12 => {
                    (2, 3)
                }
                13 => {
                    (0, 4)
                }
                14 => {
                    (1, 4)
                }
                15 => {
                    (2, 5)
                }
                16 => {
                    (3, 5)
                }
                17 => {
                    (0, 6)
                }
                18 => {
                    (0, 7)
                }
                19 => {
                    (2, 8)
                }
                20 => {
                    (3, 8)
                }
                21 => {
                    (4, 8)
                }
                22 => {
                    (1, 9)
                }
                23 => {
                    (1, 9)
                }
                24 => {
                    (2, 10)
                }
                25 => {
                    (5, 10)
                }
                26 => {
                    (6, 10)
                }
                27 => {
                    (5, 10)
                }
                28 => {
                    (6, 10)
                }
                29 => {
                    (3, 10)
                }
                30 => {
                    (4, 10)
                }
                31 => {
                    (3, 11)
                }
                32 => {
                    (3, 11)
                }
                33 => {
                    (1, 11)
                }
                34 => {
                    (1, 12)
                }
                35 => {
                    (1, 13)
                }
                36 => {
                    (1, 13)
                }
                37 => {
                    (1, 14)
                }
                38 => {
                    (1, 14)
                }
                39 => {
                    (2, 14)
                }
                40 => {
                    (2, 14)
                }
                41 => {
                    (2, 15)
                }
                42 => {
                    (3, 15)
                }
                43 => {
                    (4, 15)
                }
                44 => {
                    (2, 15)
                }
                45 => {
                    (5, 15)
                }
                46 => {
                    (6, 15)
                }
                47 => {
                    (5, 15)
                }
                48 => {
                    (6, 15)
                }
                49 => {
                    (3, 15)
                }
                50 => {
                    (4, 15)
                }
                51 => {
                    (1, 15)
                }
                52 => {
                    (1, 15)
                }
                53 => return true,
                _ => panic!("invalid action code {}", __action)
            };
            __states_len -= __to_pop;
            __states.truncate(__states_len);
            let __top = __states[__states_len - 1];
            let __next_state = __GOTO[(__top * 17 + __nt) as usize] - 1;
            __states.push(__next_state);
        }
    }
    pub fn __reduce<
    >(
        __action: i32,
        __lookahead_start: Option<&lexer::Location>,
        __states: &mut ::std::vec::Vec<i32>,
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>,
        _: ::std::marker::PhantomData<()>,
    ) -> Option<Result<Node,__lalrpop_util::ParseError<lexer::Location, lexer::Token, lexer::LexicalError>>>
    {
        let __nonterminal = match -__action {
            1 => {
                // (<MapElement> ",") = String, "=", Value, "," => ActionFn(47);
                let __sym3 = __pop_Term_22_2c_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action47::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29(__nt), __end));
                0
            }
            2 => {
                // (<MapElement> ",") = Text, "=", Value, "," => ActionFn(48);
                let __sym3 = __pop_Term_22_2c_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action48::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29(__nt), __end));
                0
            }
            3 => {
                // (<MapElement> ",") = Value, "," => ActionFn(49);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action49::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29(__nt), __end));
                0
            }
            4 => {
                // (<MapElement> ",")* =  => ActionFn(15);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action15::<>(&__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2a(__nt), __end));
                1
            }
            5 => {
                // (<MapElement> ",")* = (<MapElement> ",")+ => ActionFn(16);
                let __sym0 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action16::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2a(__nt), __end));
                1
            }
            6 => {
                // (<MapElement> ",")+ = String, "=", Value, "," => ActionFn(53);
                let __sym3 = __pop_Term_22_2c_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action53::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            7 => {
                // (<MapElement> ",")+ = Text, "=", Value, "," => ActionFn(54);
                let __sym3 = __pop_Term_22_2c_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action54::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            8 => {
                // (<MapElement> ",")+ = Value, "," => ActionFn(55);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action55::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            9 => {
                // (<MapElement> ",")+ = (<MapElement> ",")+, String, "=", Value, "," => ActionFn(56);
                let __sym4 = __pop_Term_22_2c_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermString(__symbols);
                let __sym0 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action56::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            10 => {
                // (<MapElement> ",")+ = (<MapElement> ",")+, Text, "=", Value, "," => ActionFn(57);
                let __sym4 = __pop_Term_22_2c_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermText(__symbols);
                let __sym0 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action57::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            11 => {
                // (<MapElement> ",")+ = (<MapElement> ",")+, Value, "," => ActionFn(58);
                let __sym2 = __pop_Term_22_2c_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action58::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__nt), __end));
                2
            }
            12 => {
                // (<Value> ",") = Value, "," => ActionFn(20);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action20::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29(__nt), __end));
                3
            }
            13 => {
                // (<Value> ",")* =  => ActionFn(18);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action18::<>(&__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2a(__nt), __end));
                4
            }
            14 => {
                // (<Value> ",")* = (<Value> ",")+ => ActionFn(19);
                let __sym0 = __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action19::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2a(__nt), __end));
                4
            }
            15 => {
                // (<Value> ",")+ = Value, "," => ActionFn(65);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action65::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2b(__nt), __end));
                5
            }
            16 => {
                // (<Value> ",")+ = (<Value> ",")+, Value, "," => ActionFn(66);
                let __sym2 = __pop_Term_22_2c_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action66::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2b(__nt), __end));
                5
            }
            17 => {
                // @L =  => ActionFn(24);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action24::<>(&__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::Nt_40L(__nt), __end));
                6
            }
            18 => {
                // @R =  => ActionFn(23);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action23::<>(&__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::Nt_40R(__nt), __end));
                7
            }
            19 => {
                // Array = "[", "]" => ActionFn(4);
                let __sym1 = __pop_Term_22_5d_22(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action4::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtArray(__nt), __end));
                8
            }
            20 => {
                // Array = "[", Value, "]" => ActionFn(67);
                let __sym2 = __pop_Term_22_5d_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action67::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtArray(__nt), __end));
                8
            }
            21 => {
                // Array = "[", (<Value> ",")+, Value, "]" => ActionFn(68);
                let __sym3 = __pop_Term_22_5d_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action68::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtArray(__nt), __end));
                8
            }
            22 => {
                // Key = String => ActionFn(39);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action39::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtKey(__nt), __end));
                9
            }
            23 => {
                // Key = Text => ActionFn(40);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action40::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtKey(__nt), __end));
                9
            }
            24 => {
                // Map = "{", "}" => ActionFn(9);
                let __sym1 = __pop_Term_22_7d_22(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action9::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            25 => {
                // Map = "{", String, "=", Value, "}" => ActionFn(59);
                let __sym4 = __pop_Term_22_7d_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermString(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action59::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            26 => {
                // Map = "{", (<MapElement> ",")+, String, "=", Value, "}" => ActionFn(60);
                let __sym5 = __pop_Term_22_7d_22(__symbols);
                let __sym4 = __pop_NtValue(__symbols);
                let __sym3 = __pop_Term_22_3d_22(__symbols);
                let __sym2 = __pop_TermString(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action60::<>(__sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            27 => {
                // Map = "{", Text, "=", Value, "}" => ActionFn(61);
                let __sym4 = __pop_Term_22_7d_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermText(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action61::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            28 => {
                // Map = "{", (<MapElement> ",")+, Text, "=", Value, "}" => ActionFn(62);
                let __sym5 = __pop_Term_22_7d_22(__symbols);
                let __sym4 = __pop_NtValue(__symbols);
                let __sym3 = __pop_Term_22_3d_22(__symbols);
                let __sym2 = __pop_TermText(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action62::<>(__sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            29 => {
                // Map = "{", Value, "}" => ActionFn(63);
                let __sym2 = __pop_Term_22_7d_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action63::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            30 => {
                // Map = "{", (<MapElement> ",")+, Value, "}" => ActionFn(64);
                let __sym3 = __pop_Term_22_7d_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action64::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtMap(__nt), __end));
                10
            }
            31 => {
                // MapElement = String, "=", Value => ActionFn(45);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action45::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtMapElement(__nt), __end));
                11
            }
            32 => {
                // MapElement = Text, "=", Value => ActionFn(46);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Term_22_3d_22(__symbols);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action46::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtMapElement(__nt), __end));
                11
            }
            33 => {
                // MapElement = Value => ActionFn(8);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action8::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtMapElement(__nt), __end));
                11
            }
            34 => {
                // Scalar = TextOrString+ => ActionFn(35);
                let __sym0 = __pop_NtTextOrString_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action35::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtScalar(__nt), __end));
                12
            }
            35 => {
                // TextOrString = String => ActionFn(36);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action36::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtTextOrString(__nt), __end));
                13
            }
            36 => {
                // TextOrString = Text => ActionFn(37);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action37::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtTextOrString(__nt), __end));
                13
            }
            37 => {
                // TextOrString+ = String => ActionFn(41);
                let __sym0 = __pop_TermString(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action41::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtTextOrString_2b(__nt), __end));
                14
            }
            38 => {
                // TextOrString+ = Text => ActionFn(42);
                let __sym0 = __pop_TermText(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action42::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtTextOrString_2b(__nt), __end));
                14
            }
            39 => {
                // TextOrString+ = TextOrString+, String => ActionFn(43);
                let __sym1 = __pop_TermString(__symbols);
                let __sym0 = __pop_NtTextOrString_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action43::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtTextOrString_2b(__nt), __end));
                14
            }
            40 => {
                // TextOrString+ = TextOrString+, Text => ActionFn(44);
                let __sym1 = __pop_TermText(__symbols);
                let __sym0 = __pop_NtTextOrString_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action44::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtTextOrString_2b(__nt), __end));
                14
            }
            41 => {
                // Value = "[", "]" => ActionFn(69);
                let __sym1 = __pop_Term_22_5d_22(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action69::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            42 => {
                // Value = "[", Value, "]" => ActionFn(70);
                let __sym2 = __pop_Term_22_5d_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action70::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            43 => {
                // Value = "[", (<Value> ",")+, Value, "]" => ActionFn(71);
                let __sym3 = __pop_Term_22_5d_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_5b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action71::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            44 => {
                // Value = "{", "}" => ActionFn(72);
                let __sym1 = __pop_Term_22_7d_22(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action72::<>(__sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            45 => {
                // Value = "{", String, "=", Value, "}" => ActionFn(73);
                let __sym4 = __pop_Term_22_7d_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermString(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action73::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            46 => {
                // Value = "{", (<MapElement> ",")+, String, "=", Value, "}" => ActionFn(74);
                let __sym5 = __pop_Term_22_7d_22(__symbols);
                let __sym4 = __pop_NtValue(__symbols);
                let __sym3 = __pop_Term_22_3d_22(__symbols);
                let __sym2 = __pop_TermString(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action74::<>(__sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            47 => {
                // Value = "{", Text, "=", Value, "}" => ActionFn(75);
                let __sym4 = __pop_Term_22_7d_22(__symbols);
                let __sym3 = __pop_NtValue(__symbols);
                let __sym2 = __pop_Term_22_3d_22(__symbols);
                let __sym1 = __pop_TermText(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = super::__action75::<>(__sym0, __sym1, __sym2, __sym3, __sym4);
                let __states_len = __states.len();
                __states.truncate(__states_len - 5);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            48 => {
                // Value = "{", (<MapElement> ",")+, Text, "=", Value, "}" => ActionFn(76);
                let __sym5 = __pop_Term_22_7d_22(__symbols);
                let __sym4 = __pop_NtValue(__symbols);
                let __sym3 = __pop_Term_22_3d_22(__symbols);
                let __sym2 = __pop_TermText(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action76::<>(__sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            49 => {
                // Value = "{", Value, "}" => ActionFn(77);
                let __sym2 = __pop_Term_22_7d_22(__symbols);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action77::<>(__sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            50 => {
                // Value = "{", (<MapElement> ",")+, Value, "}" => ActionFn(78);
                let __sym3 = __pop_Term_22_7d_22(__symbols);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__symbols);
                let __sym0 = __pop_Term_22_7b_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action78::<>(__sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            51 => {
                // Value = TextOrString+ => ActionFn(79);
                let __sym0 = __pop_NtTextOrString_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action79::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            52 => {
                // Value = error => ActionFn(38);
                let __sym0 = __pop_Termerror(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action38::<>(__sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                15
            }
            53 => {
                // __Value = Value => ActionFn(0);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(__sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {}", __action)
        };
        let __state = *__states.last().unwrap() as usize;
        let __next_state = __GOTO[__state * 17 + __nonterminal] - 1;
        __states.push(__next_state);
        None
    }
    fn __pop_Term_22_2c_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_2c_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_3d_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_3d_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_5b_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_5b_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_5d_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_5d_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_7b_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_7b_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_7d_22<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_7d_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_TermString<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::TermString(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_TermText<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Token, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::TermText(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termerror<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, __lalrpop_util::ErrorRecovery<lexer::Location, lexer::Token, lexer::LexicalError>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termerror(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, (Span, Node), lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2a<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2a(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cMapElement_3e_20_22_2c_22_29_2b<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cMapElement_3e_20_22_2c_22_29_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cValue_3e_20_22_2c_22_29<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Node, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2a<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, ::std::vec::Vec<Node>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2a(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28_3cValue_3e_20_22_2c_22_29_2b<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, ::std::vec::Vec<Node>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28_3cValue_3e_20_22_2c_22_29_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_40L<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Location, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_40L(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_40R<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, lexer::Location, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_40R(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtArray<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Node, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtArray(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtKey<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Span, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtKey(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtMap<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Node, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtMap(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtMapElement<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, (Span, Node), lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtMapElement(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtScalar<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Span, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtScalar(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtTextOrString<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Span, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtTextOrString(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtTextOrString_2b<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, ::std::vec::Vec<Span>, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtTextOrString_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtValue<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Node, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtValue(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt____Value<
    >(
        __symbols: &mut ::std::vec::Vec<(lexer::Location,__Symbol<>,lexer::Location)>
    ) -> (lexer::Location, Node, lexer::Location)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt____Value(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
}
pub use self::__parse__Value::parse_Value;

fn __action0<
>(
    (_, __0, _): (lexer::Location, Node, lexer::Location),
) -> Node
{
    (__0)
}

fn __action1<
>(
    (_, __0, _): (lexer::Location, lexer::Location, lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Location, lexer::Location),
) -> Span
{
    (__0, __1)
}

fn __action2<
>(
    (_, __0, _): (lexer::Location, lexer::Location, lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Location, lexer::Location),
) -> Span
{
    (__0, __1)
}

fn __action3<
>(
    (_, __0, _): (lexer::Location, lexer::Location, lexer::Location),
    (_, _, _): (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Location, lexer::Location),
) -> Span
{
    (__0, __1)
}

fn __action4<
>(
    (_, __0, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    Node::Array(Vec::new())
}

fn __action5<
>(
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, r, _): (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
    (_, f, _): (lexer::Location, Node, lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    Node::Array(build_vec(r, f))
}

fn __action6<
>(
    (_, __0, _): (lexer::Location, lexer::Location, lexer::Location),
    (_, _, _): (lexer::Location, Span, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Location, lexer::Location),
) -> Span
{
    (__0, __1)
}

fn __action7<
>(
    (_, k, _): (lexer::Location, Span, lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, v, _): (lexer::Location, Node, lexer::Location),
) -> (Span, Node)
{
    (k, v)
}

fn __action8<
>(
    (_, v, _): (lexer::Location, Node, lexer::Location),
) -> (Span, Node)
{
    ((0,0), v)
}

fn __action9<
>(
    (_, __0, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    Node::Map(Vec::new())
}

fn __action10<
>(
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
    (_, r, _): (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    (_, l, _): (lexer::Location, (Span, Node), lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    Node::Map(build_vec(r, l))
}

fn __action11<
>(
    (_, __0, _): (lexer::Location, Node, lexer::Location),
) -> Node
{
    (__0)
}

fn __action12<
>(
    (_, __0, _): (lexer::Location, Node, lexer::Location),
) -> Node
{
    (__0)
}

fn __action13<
>(
    (_, __0, _): (lexer::Location, Span, lexer::Location),
) -> Node
{
    Node::Leaf((__0))
}

fn __action14<
>(
    (_, __0, _): (lexer::Location, lexer::Location, lexer::Location),
    (_, _, _): (lexer::Location, __lalrpop_util::ErrorRecovery<lexer::Location, lexer::Token, lexer::LexicalError>, lexer::Location),
    (_, __1, _): (lexer::Location, lexer::Location, lexer::Location),
) -> Node
{
    Node::Leaf((__0, __1))
}

fn __action15<
>(
    __lookbehind: &lexer::Location,
    __lookahead: &lexer::Location,
) -> ::std::vec::Vec<(Span, Node)>
{
    vec![]
}

fn __action16<
>(
    (_, v, _): (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    v
}

fn __action17<
>(
    (_, __0, _): (lexer::Location, (Span, Node), lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
) -> (Span, Node)
{
    (__0)
}

fn __action18<
>(
    __lookbehind: &lexer::Location,
    __lookahead: &lexer::Location,
) -> ::std::vec::Vec<Node>
{
    vec![]
}

fn __action19<
>(
    (_, v, _): (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
) -> ::std::vec::Vec<Node>
{
    v
}

fn __action20<
>(
    (_, __0, _): (lexer::Location, Node, lexer::Location),
    (_, _, _): (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    (__0)
}

fn __action21<
>(
    (_, __0, _): (lexer::Location, Span, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    vec![__0]
}

fn __action22<
>(
    (_, v, _): (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
    (_, e, _): (lexer::Location, Span, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    { let mut v = v; v.push(e); v }
}

fn __action23<
>(
    __lookbehind: &lexer::Location,
    __lookahead: &lexer::Location,
) -> lexer::Location
{
    __lookbehind.clone()
}

fn __action24<
>(
    __lookbehind: &lexer::Location,
    __lookahead: &lexer::Location,
) -> lexer::Location
{
    __lookahead.clone()
}

fn __action25<
>(
    (_, __0, _): (lexer::Location, Node, lexer::Location),
) -> ::std::vec::Vec<Node>
{
    vec![__0]
}

fn __action26<
>(
    (_, v, _): (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
    (_, e, _): (lexer::Location, Node, lexer::Location),
) -> ::std::vec::Vec<Node>
{
    { let mut v = v; v.push(e); v }
}

fn __action27<
>(
    (_, __0, _): (lexer::Location, (Span, Node), lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    vec![__0]
}

fn __action28<
>(
    (_, v, _): (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    (_, e, _): (lexer::Location, (Span, Node), lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    { let mut v = v; v.push(e); v }
}

fn __action29<
>(
    __0: (lexer::Location, lexer::Location, lexer::Location),
    __1: (lexer::Location, Span, lexer::Location),
) -> Span
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action23(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action6(
        __0,
        __1,
        __temp0,
    )
}

fn __action30<
>(
    __0: (lexer::Location, lexer::Location, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
) -> Span
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action23(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
        __0,
        __1,
        __temp0,
    )
}

fn __action31<
>(
    __0: (lexer::Location, lexer::Location, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action23(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        __0,
        __1,
        __temp0,
    )
}

fn __action32<
>(
    __0: (lexer::Location, lexer::Location, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action23(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
        __0,
        __1,
        __temp0,
    )
}

fn __action33<
>(
    __0: (lexer::Location, lexer::Location, lexer::Location),
    __1: (lexer::Location, __lalrpop_util::ErrorRecovery<lexer::Location, lexer::Token, lexer::LexicalError>, lexer::Location),
) -> Node
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action23(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action14(
        __0,
        __1,
        __temp0,
    )
}

fn __action34<
>(
    __0: (lexer::Location, Span, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action24(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action29(
        __temp0,
        __0,
    )
}

fn __action35<
>(
    __0: (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action24(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action30(
        __temp0,
        __0,
    )
}

fn __action36<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action24(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
        __temp0,
        __0,
    )
}

fn __action37<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action24(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action32(
        __temp0,
        __0,
    )
}

fn __action38<
>(
    __0: (lexer::Location, __lalrpop_util::ErrorRecovery<lexer::Location, lexer::Token, lexer::LexicalError>, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action24(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action33(
        __temp0,
        __0,
    )
}

fn __action39<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action36(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        __temp0,
    )
}

fn __action40<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> Span
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action37(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        __temp0,
    )
}

fn __action41<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action36(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
        __temp0,
    )
}

fn __action42<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action37(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
        __temp0,
    )
}

fn __action43<
>(
    __0: (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action36(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        __0,
        __temp0,
    )
}

fn __action44<
>(
    __0: (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Span>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action37(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        __0,
        __temp0,
    )
}

fn __action45<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
) -> (Span, Node)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action39(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        __temp0,
        __1,
        __2,
    )
}

fn __action46<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
) -> (Span, Node)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action40(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        __temp0,
        __1,
        __2,
    )
}

fn __action47<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> (Span, Node)
{
    let __start0 = __0.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action45(
        __0,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        __temp0,
        __3,
    )
}

fn __action48<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> (Span, Node)
{
    let __start0 = __0.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        __0,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        __temp0,
        __3,
    )
}

fn __action49<
>(
    __0: (lexer::Location, Node, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> (Span, Node)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action8(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        __temp0,
        __1,
    )
}

fn __action50<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __2.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action45(
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
        __0,
        __1,
        __temp0,
        __5,
    )
}

fn __action51<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __2.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action46(
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
        __0,
        __1,
        __temp0,
        __5,
    )
}

fn __action52<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action8(
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
        __0,
        __1,
        __temp0,
        __3,
    )
}

fn __action53<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __0.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action47(
        __0,
        __1,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
        __temp0,
    )
}

fn __action54<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __0.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action48(
        __0,
        __1,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
        __temp0,
    )
}

fn __action55<
>(
    __0: (lexer::Location, Node, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action49(
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
        __temp0,
    )
}

fn __action56<
>(
    __0: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __1.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action47(
        __1,
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
        __0,
        __temp0,
    )
}

fn __action57<
>(
    __0: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __1.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action48(
        __1,
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
        __0,
        __temp0,
    )
}

fn __action58<
>(
    __0: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<(Span, Node)>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
        __0,
        __temp0,
    )
}

fn __action59<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action15(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action50(
        __0,
        __temp0,
        __1,
        __2,
        __3,
        __4,
    )
}

fn __action60<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action16(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action50(
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
    )
}

fn __action61<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action15(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action51(
        __0,
        __temp0,
        __1,
        __2,
        __3,
        __4,
    )
}

fn __action62<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action16(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action51(
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
    )
}

fn __action63<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action15(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action52(
        __0,
        __temp0,
        __1,
        __2,
    )
}

fn __action64<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action16(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action52(
        __0,
        __temp0,
        __2,
        __3,
    )
}

fn __action65<
>(
    __0: (lexer::Location, Node, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Node>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action20(
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action25(
        __temp0,
    )
}

fn __action66<
>(
    __0: (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> ::std::vec::Vec<Node>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action20(
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action26(
        __0,
        __temp0,
    )
}

fn __action67<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action18(
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action5(
        __0,
        __temp0,
        __1,
        __2,
    )
}

fn __action68<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action19(
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action5(
        __0,
        __temp0,
        __2,
        __3,
    )
}

fn __action69<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action4(
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action11(
        __temp0,
    )
}

fn __action70<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action67(
        __0,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action11(
        __temp0,
    )
}

fn __action71<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<Node>, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action68(
        __0,
        __1,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action11(
        __temp0,
    )
}

fn __action72<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action9(
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action73<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action59(
        __0,
        __1,
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action74<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action60(
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action75<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, lexer::Token, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, Node, lexer::Location),
    __4: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action61(
        __0,
        __1,
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action76<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
    __4: (lexer::Location, Node, lexer::Location),
    __5: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action62(
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action77<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, Node, lexer::Location),
    __2: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action63(
        __0,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action78<
>(
    __0: (lexer::Location, lexer::Token, lexer::Location),
    __1: (lexer::Location, ::std::vec::Vec<(Span, Node)>, lexer::Location),
    __2: (lexer::Location, Node, lexer::Location),
    __3: (lexer::Location, lexer::Token, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action64(
        __0,
        __1,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        __temp0,
    )
}

fn __action79<
>(
    __0: (lexer::Location, ::std::vec::Vec<Span>, lexer::Location),
) -> Node
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action35(
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action13(
        __temp0,
    )
}

pub trait __ToTriple<> {
    type Error;
    fn to_triple(value: Self) -> Result<(lexer::Location,lexer::Token,lexer::Location),Self::Error>;
}

impl<> __ToTriple<> for (lexer::Location, lexer::Token, lexer::Location) {
    type Error = lexer::LexicalError;
    fn to_triple(value: Self) -> Result<(lexer::Location,lexer::Token,lexer::Location),lexer::LexicalError> {
        Ok(value)
    }
}
impl<> __ToTriple<> for Result<(lexer::Location, lexer::Token, lexer::Location),lexer::LexicalError> {
    type Error = lexer::LexicalError;
    fn to_triple(value: Self) -> Result<(lexer::Location,lexer::Token,lexer::Location),lexer::LexicalError> {
        value
    }
}
