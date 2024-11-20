use nom::{
    branch::alt,
    character::complete::{alphanumeric1, one_of},
    combinator::{map, recognize},
    multi::many0,
    sequence::pair,
    IResult,
};
use serde::{Deserialize, Serialize};

pub mod abst;
pub mod concrete;

pub type Variable = String;
pub type Constructor = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Branch {
    pub constructor: Constructor,
    pub parameters: Vec<Variable>,
    pub expression: Box<Exp>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exp {
    Apply(Box<Exp>, Box<Exp>),
    Lambda(String, Box<Exp>),
    Case(Box<Exp>, Vec<Branch>),
    Rec(Variable, Box<Exp>),
    Var(Variable),
    Const(String, Vec<Exp>),
}

fn constructor(input: &str) -> IResult<&str, Constructor> {
    let first = one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let rest = recognize(many0(alt((alphanumeric1, recognize(one_of("_-'"))))));

    map(pair(first, rest), |(f, r): (char, &str)| {
        format!("{}{}", f, r)
    })(input)
}

fn variable(input: &str) -> IResult<&str, Variable> {
    let first = one_of("abcdefghijklmnopqrstuvwxyz_");
    let rest = recognize(many0(alt((alphanumeric1, recognize(one_of("_-'"))))));

    map(pair(first, rest), |(f, r): (char, &str)| {
        format!("{}{}", f, r)
    })(input)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_parse_constructor() {
        let code = "Foo";
        let (_, constr) = constructor(code).unwrap();
        assert_eq!(constr, "Foo");

        let code = "Foo_bar";
        let (_, constr) = constructor(code).unwrap();
        assert_eq!(constr, "Foo_bar");

        let code = "Foo'bar";
        let (_, constr) = constructor(code).unwrap();
        assert_eq!(constr, "Foo'bar");

        let code = "foo";
        assert_matches!(constructor(code), Err(_));
    }

    #[test]
    fn test_parse_variable() {
        let code = "foo";
        let (_, var) = variable(code).unwrap();
        assert_eq!(var, "foo");

        let code = "foo_bar";
        let (_, var) = variable(code).unwrap();
        assert_eq!(var, "foo_bar");

        let code = "foo'bar";
        let (_, var) = variable(code).unwrap();
        assert_eq!(var, "foo'bar");

        let code = "Foo";
        assert_matches!(variable(code), Err(_));
    }
}
