#![no_std]

extern crate alloc;

mod lex;
mod ops;
mod opt;
pub mod parse;

pub use lex::Token;
pub use ops::{MathOp, OrdOp};
pub use opt::Opt;
pub use parse::{defs, main};

use alloc::{string::ToString, string::String, vec::Vec};
use chumsky::prelude::*;

type Span = core::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

pub type Error = Simple<String>;

fn lex() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
    let comment = just("#").then(take_until(just('\n'))).padded();

    lex::token()
        .padded_by(comment.repeated())
        .map_with_span(|tok, span| (tok, span))
        .padded()
        .repeated()
}

pub fn parse<T, P>(src: &str, parser: P) -> Result<T, Vec<Error>>
where
    P: Parser<Token, T, Error = Simple<Token>> + Clone,
{
    let (tokens, lex_errs) = lex()
        .then_ignore(end())
        .recover_with(skip_then_retry_until([]))
        .parse_recovery(src);

    let (parsed, parse_errs) = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let stream = chumsky::Stream::from_iter(len..len + 1, tokens.into_iter());
        parser.then_ignore(end()).parse_recovery(stream)
    } else {
        (None, Vec::new())
    };

    let lex_errs = lex_errs.into_iter().map(|e| e.map(|c| c.to_string()));
    let parse_errs = parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string()));
    let errs: Vec<_> = lex_errs.chain(parse_errs).collect();

    if errs.is_empty() {
        Ok(parsed.unwrap())
    } else {
        Err(errs)
    }
}
