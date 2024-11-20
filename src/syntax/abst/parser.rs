use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::syntax::{constructor, variable, Branch, Exp};

fn parse_list<'a, T, E: nom::error::ParseError<&'a str>, F>(
    parse_item: F,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<T>, E>
where
    F: Fn(&'a str) -> IResult<&'a str, T, E> + Copy,
{
    move |code| {
        if let Ok((rest, _nil)) = preceded(multispace0::<_, ()>, tag("nil"))(code) {
            Ok((rest, vec![]))
        } else {
            map(
                tuple((
                    preceded(multispace0, tag("cons")),
                    alt((
                        delimited(
                            preceded(multispace0, char('(')),
                            parse_item,
                            preceded(multispace0, char(')')),
                        ),
                        preceded(multispace0, parse_item),
                    )),
                    alt((
                        delimited(
                            preceded(multispace0, char('(')),
                            parse_list(parse_item),
                            preceded(multispace0, char(')')),
                        ),
                        parse_list(parse_item),
                    )),
                )),
                |(_, item, rest)| {
                    let mut items = vec![item];
                    items.extend(rest);
                    items
                },
            )(code)
        }
    }
}

fn var(code: &str) -> IResult<&str, Exp> {
    map(
        pair(tag("var"), preceded(multispace0, variable)),
        |(_, v)| Exp::Var(v),
    )(code)
}

fn constr(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("const"),
            preceded(multispace0, constructor),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_list(var),
                    preceded(multispace0, char(')')),
                ),
                parse_list(var),
            )),
        )),
        |(_, name, params)| Exp::Const(name, params),
    )(code)
}

fn apply(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("apply"),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
        )),
        |(_, f, x)| Exp::Apply(Box::new(f), Box::new(x)),
    )(code)
}

fn lambda(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("lambda"),
            preceded(multispace0, variable),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
        )),
        |(_, v, e)| Exp::Lambda(v, Box::new(e)),
    )(code)
}

fn branch(code: &str) -> IResult<&str, Branch> {
    map(
        tuple((
            tag("branch"),
            preceded(multispace0, constructor),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_list(variable),
                    preceded(multispace0, char(')')),
                ),
                parse_list(variable),
            )),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
        )),
        |(_, c, v, e)| Branch {
            constructor: c,
            parameters: v,
            expression: Box::new(e),
        },
    )(code)
}

fn case(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("case"),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_list(branch),
                    preceded(multispace0, char(')')),
                ),
                parse_list(branch),
            )),
        )),
        |(_, e, bs)| Exp::Case(Box::new(e), bs),
    )(code)
}

fn rec(code: &str) -> IResult<&str, Exp> {
    map(
        tuple((
            tag("rec"),
            preceded(multispace0, variable),
            alt((
                delimited(
                    preceded(multispace0, char('(')),
                    parse_exp,
                    preceded(multispace0, char(')')),
                ),
                parse_exp,
            )),
        )),
        |(_, v, e)| Exp::Rec(v, Box::new(e)),
    )(code)
}

fn parse_exp(code: &str) -> IResult<&str, Exp> {
    alt((var, constr, apply, lambda, case, rec))(code)
}

pub fn parse(code: &str) -> Result<Exp, ()> {
    if let Ok((rest, exp)) = parse_exp(code)
        && rest.trim() == ""
    {
        Ok(exp)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var() {
        let code = "var x";
        let result = var(code);
        assert_eq!(result, Ok(("", Exp::Var("x".to_string()))));
    }

    #[test]
    fn test_constr() {
        let code = "const C nil";
        let result = constr(code);
        assert_eq!(result, Ok(("", Exp::Const("C".to_string(), vec![]))));

        let code = "const C (cons (var x) nil)";
        let result = constr(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Const("C".to_string(), vec![Exp::Var("x".to_string())])
            ))
        );
    }

    #[test]
    fn test_apply() {
        let code = "apply (var x) (var y)";
        let result = apply(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Apply(
                    Box::new(Exp::Var("x".to_string())),
                    Box::new(Exp::Var("y".to_string()))
                )
            ))
        );

        let code = "apply (apply (var x) (var y)) (var z)";
        let result = apply(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Apply(
                    Box::new(Exp::Apply(
                        Box::new(Exp::Var("x".to_string())),
                        Box::new(Exp::Var("y".to_string())),
                    )),
                    Box::new(Exp::Var("z".to_string())),
                )
            ),)
        );
    }

    #[test]
    fn test_lambda() {
        let code = "lambda x (var x)";
        let result = lambda(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Lambda("x".to_string(), Box::new(Exp::Var("x".to_string())))
            ))
        );

        let code = "lambda x (apply (var x) (var y))";
        let result = lambda(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Lambda(
                    "x".to_string(),
                    Box::new(Exp::Apply(
                        Box::new(Exp::Var("x".to_string())),
                        Box::new(Exp::Var("y".to_string()))
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_branch() {
        let code = "branch C nil (var x)";
        let result = branch(code);
        assert_eq!(
            result,
            Ok((
                "",
                Branch {
                    constructor: "C".to_string(),
                    parameters: vec![],
                    expression: Box::new(Exp::Var("x".to_string()))
                }
            ))
        );
        let code = "branch Suc (cons n nil) (var n)";
        let result = branch(code);
        assert_eq!(
            result,
            Ok((
                "",
                Branch {
                    constructor: "Suc".to_string(),
                    parameters: vec!["n".to_string()],
                    expression: Box::new(Exp::Var("n".to_string()))
                }
            ))
        );
    }

    #[test]
    fn test_case() {
        let code = "case (var n)
(cons (branch Zero nil (var x))
(cons (branch Suc (cons n nil) (var n))
nil))";
        let result = case(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Case(
                    Box::new(Exp::Var("n".to_string())),
                    vec![
                        Branch {
                            constructor: "Zero".to_string(),
                            parameters: vec![],
                            expression: Box::new(Exp::Var("x".to_string()))
                        },
                        Branch {
                            constructor: "Suc".to_string(),
                            parameters: vec!["n".to_string()],
                            expression: Box::new(Exp::Var("n".to_string()))
                        }
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_rec() {
        let code = "rec x (var x)";
        let result = rec(code);
        assert_eq!(
            result,
            Ok((
                "",
                Exp::Rec("x".to_string(), Box::new(Exp::Var("x".to_string())))
            ))
        );
    }
}
