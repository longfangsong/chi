use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, multispace0, one_of},
    combinator::{map, recognize},
    error::{Error, ErrorKind},
    multi::{fold_many0, many0, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

fn dbg_fmt_list<T: fmt::Debug>(list: &[T], f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
    if list.len() == 0 {
        write!(f, "nil")?;
    } else {
        write!(f, "cons ")?;
        write!(f, "({:?})", list[0])?;
        write!(f, " (")?;
        dbg_fmt_list(&list[1..], f)?;
        write!(f, ")")?;
    }
    Ok(())
}

fn fmt_list<T: fmt::Display>(list: &[T], f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
    if list.len() == 0 {
        write!(f, "nil")?;
    } else {
        write!(f, "cons ")?;
        write!(f, "({})", list[0])?;
        write!(f, " (")?;
        fmt_list(&list[1..], f)?;
        write!(f, ")")?;
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exp {
    Apply(Box<Exp>, Box<Exp>),
    Lambda(String, Box<Exp>),
    Case(Box<Exp>, Vec<Branch>),
    Rec(String, Box<Exp>),
    Var(String),
    Const(String, Vec<Exp>),
}

impl fmt::Debug for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exp::Apply(a, b) => write!(f, "apply ({:?}) ({:?})", a, b),
            Exp::Lambda(var, exp) => write!(f, "lambda {} ({:?})", var, exp),
            Exp::Case(exp, branches) => {
                write!(f, "case ({:?}) (", exp)?;
                dbg_fmt_list(branches, f)?;
                write!(f, ")")
            }
            Exp::Rec(var, exp) => write!(f, "rec {} ({:?})", var, exp),
            Exp::Var(var) => write!(f, "var {}", var),
            Exp::Const(constructor, arguments) => {
                write!(f, "const {} (", constructor)?;
                dbg_fmt_list(arguments, f)?;
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exp::Apply(exp, exp1) => write!(f, "[{} {}]", exp, exp1),
            Exp::Lambda(x, exp) => write!(f, "λ{}.({})", x, exp),
            Exp::Var(x) => write!(f, "{}", x),
            Exp::Case(exp, vec) => write!(
                f,
                "case {} of {{\n{} \n}}",
                exp,
                vec.iter()
                    .map(|b| format!(
                        "  {}({}) -> {}",
                        b.constructor,
                        b.variables.join(", "),
                        b.expression
                    ))
                    .collect::<Vec<String>>()
                    .join(";\n")
            ),
            Exp::Rec(x, exp) => write!(f, "rec {} = {}", x, exp),
            Exp::Const(constructor, vec) => write!(
                f,
                "{}({})",
                constructor,
                vec.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub constructor: String,
    pub variables: Vec<String>,
    pub expression: Box<Exp>,
}

impl fmt::Debug for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "branch {} (", self.constructor)?;
        fmt_list(&self.variables, f)?;
        write!(f, ") ({:?})", self.expression)
    }
}

fn constructor(input: &str) -> IResult<&str, String> {
    let first = one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let rest = recognize(many0(alt((alphanumeric1, recognize(one_of("_-'"))))));

    map(pair(first, rest), |(f, r): (char, &str)| {
        format!("{}{}", f, r)
    })(input)
}

fn variable(input: &str) -> IResult<&str, String> {
    let first = one_of("abcdefghijklmnopqrstuvwxyz_");
    let rest = recognize(many0(alt((alphanumeric1, recognize(one_of("_-'"))))));

    map(pair(first, rest), |(f, r): (char, &str)| {
        format!("{}{}", f, r)
    })(input)
}

fn apply(code: &str) -> IResult<&str, Exp> {
    let (rest, first) = preceded(multispace0, higher_prior_exp)(code)?;
    let (rest, second) = preceded(multispace0, higher_prior_exp)(rest)?;
    fold_many0(
        preceded(multispace0, higher_prior_exp),
        move || Exp::Apply(Box::new(first.clone()), Box::new(second.clone())),
        |lhs, rhs| Exp::Apply(Box::new(lhs), Box::new(rhs)),
    )(rest)
}

fn lambda(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            alt((tag(r#"\"#), tag("λ"), tag("𝜆"))),
            preceded(multispace0, variable),
            preceded(multispace0, char('.')),
            preceded(multispace0, parse_exp),
        )),
        |(_, var, _, exp)| Exp::Lambda(var, Box::new(exp)),
    )(code)
}

fn parse_branch(input: &str) -> IResult<&str, Branch> {
    map(
        tuple((
            preceded(multispace0, constructor),
            delimited(
                preceded(multispace0, char('(')),
                separated_list0(
                    preceded(multispace0, char(',')),
                    preceded(multispace0, variable),
                ),
                preceded(multispace0, char(')')),
            ),
            preceded(multispace0, alt((tag("->"), tag("→")))),
            preceded(multispace0, parse_exp),
        )),
        |(constructor, variables, _, expression)| Branch {
            constructor,
            variables,
            expression: Box::new(expression),
        },
    )(input)
}

pub fn take_until_branch_list(code: &str) -> IResult<&str, &str> {
    let mut chars = code.chars();
    let mut index = 0;
    let mut case_count: usize = 0;

    while let Some(c) = chars.next() {
        match c {
            'c' => {
                if code[index..].starts_with("case") {
                    case_count += 1;
                }
                index += c.len_utf8();
            }
            '{' => {
                if case_count == 0 {
                    return Ok((&code[index..], &code[..index]));
                }
                index += c.len_utf8();
            }
            '}' => {
                case_count -= 1;
                index += c.len_utf8();
            }
            _ => index += c.len_utf8(),
        }
    }
    Err(nom::Err::Error(Error::new(code, ErrorKind::TakeUntil)))
}

fn branch_list(code: &str) -> IResult<&str, Vec<Branch>> {
    delimited(
        preceded(multispace0, char('{')),
        separated_list0(preceded(multispace0, char(';')), parse_branch),
        preceded(multispace0, char('}')),
    )(code)
}

fn case(code: &str) -> IResult<&str, Exp> {
    let (code, _) = tag("case")(code)?;
    // We have to prevent something like `x of ...` being parsed as `Apply x of`.
    let (code, exp_part) = take_until_branch_list(code)?;
    let exp_part = exp_part.trim();
    if !exp_part.ends_with("of") {
        return Err(nom::Err::Error(Error::new(code, ErrorKind::Tag)));
    }
    let exp_part = exp_part.trim_end_matches("of").trim();
    let exp = parse_exp(exp_part)?.1;
    let (code, branches) = branch_list(code)?;
    Ok((code, Exp::Case(Box::new(exp), branches)))
}

fn rec(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("rec"),
            preceded(multispace0, variable),
            preceded(multispace0, char('=')),
            preceded(multispace0, parse_exp),
        )),
        |(_, var, _, exp)| Exp::Rec(var, Box::new(exp)),
    )(code)
}

fn parse_const(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            preceded(multispace0, constructor),
            delimited(
                preceded(multispace0, char('(')),
                separated_list0(
                    preceded(multispace0, char(',')),
                    preceded(multispace0, parse_exp),
                ),
                preceded(multispace0, char(')')),
            ),
        )),
        |(constructor, arguments)| Exp::Const(constructor, arguments),
    )(code)
}

fn higher_prior_exp(code: &str) -> IResult<&str, Exp> {
    alt((
        map(variable, |var| Exp::Var(var)),
        parse_const,
        delimited(
            preceded(multispace0, char('(')),
            parse_exp,
            preceded(multispace0, char(')')),
        ),
    ))(code)
}

fn higher_or_mid_prior_exp(code: &str) -> IResult<&str, Exp> {
    alt((case, apply, higher_prior_exp))(code)
}

pub fn parse_exp(input: &str) -> IResult<&str, Exp> {
    alt((lambda, rec, higher_or_mid_prior_exp))(input)
}

#[wasm_bindgen]
pub fn parse(input: &str) -> JsValue {
    let expr = parse_exp(input).unwrap().1;
    serde_wasm_bindgen::to_value(&expr).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constructor() {
        let code = r#"C"#;
        assert_eq!(constructor(code), Ok(("", "C".to_string())));
        let code = r#"True"#;
        assert_eq!(constructor(code), Ok(("", "True".to_string())));
        let code = r#"false"#;
        assert!(constructor(code).is_err());
    }

    #[test]
    fn test_parse_variable() {
        let code = r#"x"#;
        assert_eq!(variable(code), Ok(("", "x".to_string())));
        let code = r#"x1"#;
        assert_eq!(variable(code), Ok(("", "x1".to_string())));
        let code = r#"X"#;
        assert!(variable(code).is_err());
    }

    #[test]
    fn test_parse_apply() {
        let code = r#"f x"#;
        assert_eq!(
            apply(code),
            Ok((
                "",
                Exp::Apply(
                    Box::new(Exp::Var("f".to_string())),
                    Box::new(Exp::Var("x".to_string()))
                )
            ))
        );
    }

    #[test]
    fn test_parse_lambda() {
        let code = r#"λx.x"#;
        assert_eq!(
            lambda(code),
            Ok((
                "",
                Exp::Lambda("x".to_string(), Box::new(Exp::Var("x".to_string())))
            ))
        );
        let code = r#"λx.λy.x"#;
        assert_eq!(
            lambda(code),
            Ok((
                "",
                Exp::Lambda(
                    "x".to_string(),
                    Box::new(Exp::Lambda(
                        "y".to_string(),
                        Box::new(Exp::Var("x".to_string()))
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_parse_const() {
        let code = r#"C()"#;
        assert_eq!(
            parse_const(code),
            Ok(("", Exp::Const("C".to_string(), vec![])))
        );
        let code = r#"C(x)"#;
        assert_eq!(
            parse_const(code),
            Ok((
                "",
                Exp::Const("C".to_string(), vec![Exp::Var("x".to_string())])
            ))
        );
        let code = r#"C(x, y)"#;
        assert_eq!(
            parse_const(code),
            Ok((
                "",
                Exp::Const(
                    "C".to_string(),
                    vec![Exp::Var("x".to_string()), Exp::Var("y".to_string())]
                )
            ))
        );
        let code = r#"C(λx.x, y)"#;
        assert_eq!(
            parse_const(code),
            Ok((
                "",
                Exp::Const(
                    "C".to_string(),
                    vec![
                        Exp::Lambda("x".to_string(), Box::new(Exp::Var("x".to_string()))),
                        Exp::Var("y".to_string())
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_rec() {
        let code = r#"rec x = λy.y"#;
        assert_eq!(
            rec(code),
            Ok((
                "",
                Exp::Rec(
                    "x".to_string(),
                    Box::new(Exp::Lambda(
                        "y".to_string(),
                        Box::new(Exp::Var("y".to_string()))
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_parse_branch() {
        let code = r#"C(x) -> x"#;
        assert_eq!(
            parse_branch(code),
            Ok((
                "",
                Branch {
                    constructor: "C".to_string(),
                    variables: vec!["x".to_string()],
                    expression: Box::new(Exp::Var("x".to_string()))
                }
            ))
        );
        let code = r#"C(x, y) -> x"#;
        assert_eq!(
            parse_branch(code),
            Ok((
                "",
                Branch {
                    constructor: "C".to_string(),
                    variables: vec!["x".to_string(), "y".to_string()],
                    expression: Box::new(Exp::Var("x".to_string()))
                }
            ))
        );
    }

    #[test]
    fn test_branch_list() {
        let code = r#"{ C(x) -> x; C(y) -> y }"#;
        assert_eq!(
            branch_list(code),
            Ok((
                "",
                vec![
                    Branch {
                        constructor: "C".to_string(),
                        variables: vec!["x".to_string()],
                        expression: Box::new(Exp::Var("x".to_string()))
                    },
                    Branch {
                        constructor: "C".to_string(),
                        variables: vec!["y".to_string()],
                        expression: Box::new(Exp::Var("y".to_string()))
                    }
                ]
            ))
        );
    }

    #[test]
    fn test_parse_case() {
        let code = r#"case x of { C(x) -> x; C(y) -> y }"#;
        assert_eq!(
            case(code),
            Ok((
                "",
                Exp::Case(
                    Box::new(Exp::Var("x".to_string())),
                    vec![
                        Branch {
                            constructor: "C".to_string(),
                            variables: vec!["x".to_string()],
                            expression: Box::new(Exp::Var("x".to_string()))
                        },
                        Branch {
                            constructor: "C".to_string(),
                            variables: vec!["y".to_string()],
                            expression: Box::new(Exp::Var("y".to_string()))
                        }
                    ]
                )
            ))
        );
        let code = r#"case case y of { C(x) -> x; C(y) -> y } of { C(z) -> z; C(w) -> w } D(q)"#;
        let result = case(code);
        assert!(matches!(
            result,
            Ok((" D(q)", Exp::Case(box Exp::Case(..), ..)))
        ));
    }

    #[test]
    fn test_parse() {
        let code = r#"rec foo = λm. λn. case m of
            { Zero() → case n of
              { Zero() → True()
                ; Suc(n) → False()
              }
            ; Suc(m) → case n of
              { Zero() → False()
              ; Suc(n) → foo m n
              }
            }"#;
        let result = parse_exp(code);
        assert!(matches!(
            result,
            Ok(("", Exp::Rec(_, box Exp::Lambda(_, box Exp::Lambda(_, box Exp::Case(..))))))
        ));
    }

    #[test]
    fn test_t() {
        let code = r#"(rec x = x y)"#;
        let result = parse_exp(code);
        println!("{}", result.unwrap().1);
    }
}
