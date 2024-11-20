use crate::syntax::{Branch, Exp};

fn format_name_list(list: &[String]) -> String {
    if list.is_empty() {
        "nil".to_string()
    } else if list.len() == 1 {
        format!("cons {} {}", &list[0], format_name_list(&list[1..]))
    } else {
        format!("cons {} ({})", &list[0], format_name_list(&list[1..]))
    }
}

fn format_branch(branch: &Branch, nest_level: usize) -> String {
    let constructor = branch.constructor.to_string();
    let parameters = format_name_list(&branch.parameters);
    let expression = format_exp(branch.expression.as_ref(), nest_level);
    if branch.parameters.is_empty() {
        format!("branch {} {} ({})", constructor, parameters, expression)
    } else {
        format!("branch {} ({}) ({})", constructor, parameters, expression)
    }
}

fn format_branch_list(list: &[Branch], nest_level: usize) -> String {
    if list.is_empty() {
        "nil".to_string()
    } else if list.len() == 1 {
        format!(
            "cons ({}) \n{}{}",
            format_branch(&list[0], nest_level),
            "  ".repeat(nest_level),
            format_branch_list(&list[1..], nest_level)
        )
    } else {
        format!(
            "cons ({}) (\n{}{})",
            format_branch(&list[0], nest_level),
            "  ".repeat(nest_level),
            format_branch_list(&list[1..], nest_level)
        )
    }
}

fn format_exp_list(list: &[Exp], nest_level: usize) -> String {
    if list.is_empty() {
        "nil".to_string()
    } else {
        let first_result = format_exp(&list[0], nest_level);
        let rest_result = format_exp_list(&list[1..], nest_level);
        if list.len() == 1 {
            format!("cons ({}) {}", first_result, rest_result)
        } else {
            format!("cons ({}) ({})", first_result, rest_result)
        }
    }
}

fn format_exp(exp: &Exp, nest_level: usize) -> String {
    match exp {
        Exp::Var(x) => format!("var {}", x),
        Exp::Lambda(x, exp) => format!("lambda {} ({})", x, format_exp(exp, nest_level)),
        Exp::Apply(f, x) => {
            let f_fmt = format_exp(f, nest_level);
            let x_fmt = format_exp(x, nest_level);
            let f_in_new_line = f_fmt.len() > 30;
            let x_in_new_line = f_in_new_line || x_fmt.len() > 30;
            let f = if f_in_new_line {
                format!(
                    "\n{}({})",
                    "  ".repeat(nest_level),
                    format_exp(f, nest_level + 1)
                )
            } else {
                format!("({})", f_fmt)
            };
            let x = if x_in_new_line {
                format!(
                    "\n{}({})",
                    "  ".repeat(nest_level),
                    format_exp(x, nest_level + 1)
                )
            } else {
                format!("({})", x_fmt)
            };
            format!("apply {} {}", f, x)
        }
        Exp::Case(exp, branches) => {
            let exp_fmt = format_exp(exp, nest_level);
            let exp = if exp_fmt.len() > 30 {
                format!(
                    "\n{}({})",
                    "  ".repeat(nest_level),
                    format_exp(exp, nest_level + 1)
                )
            } else {
                format!("({})", exp_fmt)
            };
            let branches = format_branch_list(branches, nest_level + 1);
            format!(
                "case {} (\n{}{})",
                exp,
                "  ".repeat(nest_level + 1),
                branches
            )
        }
        Exp::Rec(x, exp) => format!("rec {} ({})", x, format_exp(exp, nest_level)),
        Exp::Const(c, exps) => format!("const {} ({})", c, format_exp_list(exps, nest_level)),
    }
}

pub fn format(exp: &Exp) -> String {
    format_exp(exp, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::concrete;

    #[test]
    fn test_format() {
        let code = r#"rec y = case x of {C() → x; D(x) → x}"#;
        let exp = concrete::parse(code).unwrap();
        assert_eq!("rec y (case (var x) (\n  cons (branch C nil (var x)) (\n  cons (branch D (cons x nil) (var x)) \n  nil)))", format(&exp));

        let code = "𝜆 x. Suc(x)";
        let exp = concrete::parse(code).unwrap();
        assert_eq!("lambda x (const Suc (cons (var x) nil))", format(&exp));
    }
}
