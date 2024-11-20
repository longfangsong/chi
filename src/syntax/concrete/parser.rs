use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    error::{Error, ErrorKind},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::syntax::{constructor, variable, Branch, Exp};

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

// we need to prevent parsing things like `case x of ...` as `case (x of)`
fn case_exp(code: &str) -> IResult<&str, Exp> {
    let chars = code.chars();
    let mut index = 0;
    let mut case_count: usize = 0;

    for c in chars {
        match c {
            'c' => {
                if code[index..].starts_with("case") {
                    case_count += 1;
                }
                index += c.len_utf8();
            }
            '{' => {
                if case_count == 0 {
                    let exp_part = &code[..index].trim().trim_end_matches("of").trim();
                    let (rest_code, exp) = parse_exp(exp_part)?;
                    if rest_code.is_empty() {
                        return Ok((&code[index..], exp));
                    } else {
                        return Err(nom::Err::Error(Error::new(code, ErrorKind::TakeUntil)));
                    }
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

fn branch(input: &str) -> IResult<&str, Branch> {
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
        |(constructor, parameters, _, expression)| Branch {
            constructor,
            parameters,
            expression: Box::new(expression),
        },
    )(input)
}

fn branch_list(code: &str) -> IResult<&str, Vec<Branch>> {
    delimited(
        preceded(multispace0, char('{')),
        separated_list0(preceded(multispace0, char(';')), branch),
        tuple((multispace0, opt(char(';')), multispace0, char('}'))),
    )(code)
}

fn case(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((tag("case"), case_exp, branch_list)),
        |(_, exp, branches)| Exp::Case(Box::new(exp), branches),
    )(code)
}

fn higher_than_apply(code: &str) -> IResult<&str, Exp> {
    alt((
        map(variable, Exp::Var),
        parse_const,
        delimited(
            preceded(multispace0, char('(')),
            parse_exp,
            preceded(multispace0, char(')')),
        ),
        case,
    ))(code)
}

fn apply(code: &str) -> IResult<&str, Exp> {
    let (rest, first) = preceded(multispace0, higher_than_apply)(code)?;
    let (rest, second) = preceded(multispace0, higher_than_apply)(rest)?;
    fold_many0(
        preceded(multispace0, higher_than_apply),
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

pub fn remove_comment(code: &str) -> String {
    let mut in_multiline_comment = false;
    let mut result = String::new();
    let mut chars = code.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '-' if !in_multiline_comment && chars.peek() == Some(&'-') => {
                // Skip rest of line for inline comment
                for c in chars.by_ref() {
                    if c == '\n' {
                        result.push(c);
                        break;
                    }
                }
            }
            '{' if chars.peek() == Some(&'-') => {
                chars.next();
                in_multiline_comment = true;
            }
            '-' if in_multiline_comment && chars.peek() == Some(&'}') => {
                chars.next();
                in_multiline_comment = false;
            }
            c if !in_multiline_comment => {
                result.push(c);
            }
            _ => {}
        }
    }
    result
}

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    alt((lambda, rec, case, apply, higher_than_apply))(input)
}

pub fn parse(input: &str) -> Result<Exp, ()> {
    let code = remove_comment(input);
    if let Ok((rest, exp)) = parse_exp(&code)
        && rest.trim() == ""
    {
        Ok(exp)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_const() {
        let code = "C()";
        let (_, constr) = parse_const(code).unwrap();
        assert_eq!(constr, Exp::Const("C".to_string(), vec![]));
        let code = "C(x)";
        let (_, constr) = parse_const(code).unwrap();
        assert_eq!(
            constr,
            Exp::Const("C".to_string(), vec![Exp::Var("x".to_string())])
        );
        let code = "C(x,y)";
        let (_, constr) = parse_const(code).unwrap();
        assert_eq!(
            constr,
            Exp::Const(
                "C".to_string(),
                vec![Exp::Var("x".to_string()), Exp::Var("y".to_string())]
            )
        );
        let code = "C(λx.x y)";
        let (_, constr) = parse_const(code).unwrap();
        assert_eq!(
            constr,
            Exp::Const(
                "C".to_string(),
                vec![Exp::Lambda(
                    "x".to_string(),
                    Box::new(Exp::Apply(
                        Box::new(Exp::Var("x".to_string())),
                        Box::new(Exp::Var("y".to_string()))
                    ))
                )]
            )
        );
    }

    #[test]
    fn test_case() {
        let code = r#"case x of {}"#;
        let (_, case_stmt) = case(code).unwrap();
        assert_eq!(
            case_stmt,
            Exp::Case(Box::new(Exp::Var("x".to_string())), vec![])
        );
        let code = r#"case x of {C(x)->x}"#;
        let (_, case_stmt) = case(code).unwrap();
        assert_eq!(
            case_stmt,
            Exp::Case(
                Box::new(Exp::Var("x".to_string())),
                vec![Branch {
                    constructor: "C".to_string(),
                    parameters: vec!["x".to_string()],
                    expression: Box::new(Exp::Var("x".to_string()))
                }]
            )
        );
        let code = r#"case ((\x.x) y) of {C(x,y)->x; D(x,y)->y}"#;
        let (_, case_stmt) = case(code).unwrap();
        assert_eq!(
            case_stmt,
            Exp::Case(
                Box::new(Exp::Apply(
                    Box::new(Exp::Lambda(
                        "x".to_string(),
                        Box::new(Exp::Var("x".to_string()))
                    )),
                    Box::new(Exp::Var("y".to_string()))
                )),
                vec![
                    Branch {
                        constructor: "C".to_string(),
                        parameters: vec!["x".to_string(), "y".to_string()],
                        expression: Box::new(Exp::Var("x".to_string()))
                    },
                    Branch {
                        constructor: "D".to_string(),
                        parameters: vec!["x".to_string(), "y".to_string()],
                        expression: Box::new(Exp::Var("y".to_string()))
                    }
                ]
            )
        );
    }

    #[test]
    fn test_apply() {
        let code = r#"(\x.x) y"#;
        let (_, exp) = apply(code).unwrap();
        assert_eq!(
            exp,
            Exp::Apply(
                Box::new(Exp::Lambda(
                    "x".to_string(),
                    Box::new(Exp::Var("x".to_string()))
                )),
                Box::new(Exp::Var("y".to_string()))
            )
        );
        let code = r#"x y z"#;
        let (_, exp) = apply(code).unwrap();
        assert_eq!(
            exp,
            Exp::Apply(
                Box::new(Exp::Apply(
                    Box::new(Exp::Var("x".to_string())),
                    Box::new(Exp::Var("y".to_string()))
                )),
                Box::new(Exp::Var("z".to_string()))
            )
        );
    }

    #[test]
    fn test_lambda() {
        let code = r#"\x.x"#;
        let (_, exp) = lambda(code).unwrap();
        assert_eq!(
            exp,
            Exp::Lambda("x".to_string(), Box::new(Exp::Var("x".to_string())))
        );
        let code = r#"\x.y x"#;
        let (_, exp) = lambda(code).unwrap();
        assert_eq!(
            exp,
            Exp::Lambda(
                "x".to_string(),
                Box::new(Exp::Apply(
                    Box::new(Exp::Var("y".to_string())),
                    Box::new(Exp::Var("x".to_string()))
                ))
            )
        );
    }

    #[test]
    fn test_rec() {
        let code = r#"rec x = x"#;
        let (_, exp) = rec(code).unwrap();
        assert_eq!(
            exp,
            Exp::Rec("x".to_string(), Box::new(Exp::Var("x".to_string())))
        );
        let code = r#"rec x = y x"#;
        let (_, exp) = rec(code).unwrap();
        assert_eq!(
            exp,
            Exp::Rec(
                "x".to_string(),
                Box::new(Exp::Apply(
                    Box::new(Exp::Var("y".to_string())),
                    Box::new(Exp::Var("x".to_string()))
                ))
            )
        );
    }

    #[test]
    fn test_remove_comment() {
        let code = " x --abc";
        assert_eq!(remove_comment(code), " x ");
        let code = r#"x --abc
            def
            {- xxxx
            yyyy
            -}
            ghi"#;
        assert_eq!(
            remove_comment(code),
            "x \n            def\n            \n            ghi"
        );
    }

    #[test]
    fn test_parse() {
        let code = r#"(rec foo = λm. λn. case m of
            { Zero() → case n of
              { Zero() → True()
              ; Suc(n) → False()
              }
            ; Suc(m) → case n of
              { Zero() → False()
              ; Suc(n) → foo m n
              }
}) Suc(Suc(Zero())) Suc(Zero())"#;
        let parsed = parse(code).unwrap();
        assert_matches!(
            parsed,
            Exp::Apply(
                box Exp::Apply(box Exp::Rec(_, _), box Exp::Const(_, _)),
                box Exp::Const(_, _),
            )
        );
    }
}
