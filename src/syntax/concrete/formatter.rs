use crate::syntax::{Branch, Exp};

fn format_branch(branch: &Branch, nest_level: usize) -> String {
    format!(
        "{}{}({}) -> {}",
        "  ".repeat(nest_level),
        branch.constructor,
        branch.parameters.join(", "),
        format_exp(branch.expression.as_ref(), nest_level)
    )
}

fn format_exp(exp: &Exp, nest_level: usize) -> String {
    match exp {
        Exp::Var(var) => var.clone(),
        Exp::Const(constructor, arguments) => {
            let formatted_arguments = arguments
                .iter()
                .map(|argument| format_exp(argument, nest_level))
                .collect::<Vec<String>>()
                .join(", ");
            format!("{}({})", constructor, formatted_arguments)
        }
        Exp::Apply(lhs, rhs) => {
            let lhs = match lhs.as_ref() {
                Exp::Rec(_, _) | Exp::Lambda(_, _) => format!("({})", format_exp(lhs, nest_level)),
                _ => format_exp(lhs, nest_level).to_string(),
            };
            let rhs = match rhs.as_ref() {
                Exp::Apply(_, _) | Exp::Lambda(_, _) => format!("({})", format_exp(rhs, nest_level)),
                _ => format_exp(rhs, nest_level).to_string(),
            };
            format!("{} {}", lhs, rhs)
        }
        Exp::Lambda(var, exp) => {
            let exp = match exp.as_ref() {
                Exp::Apply(_, _) => format!("({})", format_exp(exp, nest_level)),
                _ => format_exp(exp, nest_level).to_string(),
            };
            format!("λ{}.{}", var, exp)
        }
        Exp::Case(exp, branches) => {
            let formatted_branches = branches
                .iter()
                .map(|branch| format_branch(branch, nest_level + 1))
                .collect::<Vec<String>>()
                .join(";\n");
            format!(
                "case {} of {{\n{}\n{}}}",
                format_exp(exp, nest_level),
                formatted_branches,
                "  ".repeat(nest_level),
            )
        }
        Exp::Rec(var, exp) => format!("rec {} = {}", var, format_exp(exp, nest_level)),
    }
}

pub fn format(exp: &Exp) -> String {
    format_exp(exp, 0)
}

#[cfg(test)]
mod tests {
    use crate::syntax::concrete::parser::parse;

    use super::*;

    #[test]
    fn test_format() {
        let term = parse("λx.x y").unwrap();
        assert_eq!(format(&term), "λx.(x y)");

        let term = parse("(λx.x) y").unwrap();
        assert_eq!(format(&term), "(λx.x) y");

        let term = parse("x y z").unwrap();
        assert_eq!(format(&term), "x y z");

        let term = parse("x (y z)").unwrap();
        assert_eq!(format(&term), "x (y z)");
    }
}
