use std::num::ParseIntError;
use std::str::FromStr;

use from_pest::pest::iterators::Pairs;
use from_pest::pest::Span;
use from_pest::{ConversionError, FromPest, Void};
use pest_ast::FromPest;

use super::Rule;

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::main))]
pub struct MainProgram {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::stmt))]
pub struct Stmt {}

#[derive(Debug, Clone)]
pub struct Expr<'p> {
    pub lhs: Term<'p>,
    pub rhs: Option<(Operator, Term<'p>)>,
}

impl<'p> FromPest<'p> for Expr<'p> {
    type Rule = Rule;
    type FatalError = Void;

    fn from_pest(
        pairs: &mut Pairs<'p, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        pairs
            .peek()
            .filter(|pair| pair.as_rule() == Rule::expr)
            .ok_or(::from_pest::ConversionError::NoMatch)?;
        let mut pairs = pairs.next().unwrap().into_inner();

        let lhs = Term::from_pest(&mut pairs)?;
        let rhs = match Operator::from_pest(&mut pairs) {
            Ok(op) => Some((op, Term::from_pest(&mut pairs)?)),
            Err(ConversionError::NoMatch) => None,
            Err(err) => return Err(err),
        };

        Ok(Expr { lhs, rhs })
    }
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::expr_inner))]
pub struct ExprInner<'p> {
    pub val: Value<'p>,
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::term))]
pub enum Term<'p> {
    Value(Value<'p>),
    Expr(Box<Expr<'p>>),
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::value))]
pub enum Value<'p> {
    Primitive(PrimitiveValue),
    String(StringLiteral<'p>),
    Array(Array<'p>),
    Tuple(Tuple<'p>),
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::primitive_value))]
pub enum PrimitiveValue {
    Char(Char),
    Integer(Integer),
    Float(Float),
}

#[derive(Debug, Clone)]
pub struct Params<'p> {
    pub params: Vec<Expr<'p>>,
}

impl<'p> FromPest<'p> for Params<'p> {
    type Rule = Rule;
    type FatalError = Void;

    fn from_pest(
        pairs: &mut Pairs<'p, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let mut params = Vec::with_capacity(3);
        loop {
            match Expr::from_pest(pairs) {
                Ok(expr) => {
                    params.push(expr);
                },
                Err(ConversionError::NoMatch) => break,
                Err(err) => return Err(err),
            }
        }
        Ok(Params {
            params
        })
    }
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::tuple))]
pub struct Tuple<'p> {
    pub params: Params<'p>,
}

#[derive(Debug, FromPest, Clone)]
#[pest_ast(rule(Rule::array))]
pub struct Array<'p> {
    pub params: Params<'p>,
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::string))]
pub struct StringLiteral<'p> {
    #[pest_ast(outer(with(span_into_str), with(str_to_str_lit)))]
    pub val: &'p str,
}

fn str_to_str_lit(s: &str) -> &str {
    s.trim_matches('\"')
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::char))]
pub struct Char {
    #[pest_ast(outer(with(span_into_str), with(str_to_char)))]
    pub val: char,
}

fn str_to_char(s: &str) -> char {
    s.trim_matches('\'').chars().next().unwrap()
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::operator))]
pub enum Operator {
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    Pow(Pow),
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::op_add))]
pub struct Add;

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::op_sub))]
pub struct Sub;

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::op_mul))]
pub struct Mul;

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::op_div))]
pub struct Div;

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::op_pow))]
pub struct Pow;

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::float))]
pub struct Float {
    #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
    pub val: f64,
}

fn span_into_float(span: Span) -> f64 {
    let (integer, decimal) = span.as_str().split_once(".").unwrap();
    let mut float_str = match &integer[..2] {
        "0b" | "0B" => bin_str_to_int(&integer[2..]).unwrap().to_string(),
        "0o" | "0O" => oct_str_to_int(&integer[2..]).unwrap().to_string(),
        "0x" | "0X" => hex_str_to_int(&integer[2..]).unwrap().to_string(),
        other => other.to_owned(),
    };
    float_str.push('.');
    float_str.push_str(decimal);
    f64::from_str(&float_str).unwrap()
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::integer))]
pub enum Integer {
    Dec(IntegerDec),
    Bin(IntegerBin),
    Oct(IntegerOct),
    Hex(IntegerHex),
}

impl Integer {
    pub fn as_i64(&self) -> i64 {
        match self {
            Integer::Dec(i) => i.val,
            Integer::Bin(i) => i.val,
            Integer::Oct(i) => i.val,
            Integer::Hex(i) => i.val,
        }
    }
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::integer_dec))]
pub struct IntegerDec {
    #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
    pub val: i64,
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::integer_bin))]
pub struct IntegerBin {
    #[pest_ast(outer(with(span_into_str), with(bin_str_to_int), with(Result::unwrap)))]
    pub val: i64,
}

fn bin_str_to_int(s: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&s[2..], 2)
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::integer_oct))]
pub struct IntegerOct {
    #[pest_ast(outer(with(span_into_str), with(oct_str_to_int), with(Result::unwrap)))]
    pub val: i64,
}

fn oct_str_to_int(s: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&s[2..], 8)
}

#[derive(Debug, FromPest, Copy, Clone)]
#[pest_ast(rule(Rule::integer_hex))]
pub struct IntegerHex {
    #[pest_ast(outer(with(span_into_str), with(hex_str_to_int), with(Result::unwrap)))]
    pub val: i64,
}

fn hex_str_to_int(s: &str) -> Result<i64, ParseIntError> {
    i64::from_str_radix(&s[2..], 16)
}

fn span_into_str(span: Span) -> &str {
    span.as_str()
}