extern crate nom;

use nom::branch::alt;
use nom::bytes::streaming::{is_not, take_while_m_n};
use nom::character::streaming::{char, multispace1};
use nom::combinator::{map, map_opt, map_res, value, verify};
use nom::error::ParseError;
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded, precededc};
use nom::IResult;


fn parse_unicode<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    let parse_hex = take_while_m_n(1, 6, |c: char|c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'),delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

fn parse_escaped_char<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        ))
    )(input)
}

fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('\\'), multispace1)(input)
}

fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

fn parse_fragment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E> {
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace)
        ))(input)
}

fn parse_string<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    let build_string = fold_many0(
        parse_fragment,
        String::new(),
        |mut string,fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        }
    );

    delimited(char('"'), build_string, char('"'))(input)
}

fn main() {
    let data = "\"abc\"";
    println!("EXAMPLE 1:\nParsing a simple input string: {}", data);
    let result = parse_string::<()>(data);
    assert_eq!(result, Ok(("", String::from("abc"))));
    println!("Result: {}\n\n", result.unwrap().1);

    let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
    println!(
        "EXAMPLE 2:\nParsing a string with escape sequences, newline literal, and escaped whitespace:\n\n{}\n",
        data
    );
    let result = parse_string::<()>(data);
    println!("Result: \n\n{}", result.unwrap().1);
}