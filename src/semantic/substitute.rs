use crate::syntax::{Branch, Exp};

fn substitute_branch(branch: &Branch, from_variable: &str, to_exp: &Exp) -> Branch {
    if branch.parameters.contains(&from_variable.to_string()) {
        branch.clone()
    } else {
        Branch {
            constructor: branch.constructor.clone(),
            parameters: branch.parameters.clone(),
            expression: Box::new(substitute(
                branch.expression.as_ref(),
                from_variable,
                to_exp,
            )),
        }
    }
}

pub fn substitute(exp: &Exp, from_variable: &str, to_exp: &Exp) -> Exp {
    match exp {
        Exp::Apply(f, x) => Exp::Apply(
            Box::new(substitute(f, from_variable, to_exp)),
            Box::new(substitute(x, from_variable, to_exp)),
        ),
        Exp::Lambda(x, exp) if x == from_variable => Exp::Lambda(x.clone(), exp.clone()),
        Exp::Lambda(x, exp) => {
            Exp::Lambda(x.clone(), Box::new(substitute(exp, from_variable, to_exp)))
        }
        Exp::Case(exp, branches) => {
            let exp_result = substitute(exp, from_variable, to_exp);
            let branches_result = branches
                .iter()
                .map(|branch| substitute_branch(branch, from_variable, to_exp))
                .collect();
            Exp::Case(Box::new(exp_result), branches_result)
        }
        Exp::Rec(x, exp) if x == from_variable => Exp::Rec(x.clone(), exp.clone()),
        Exp::Rec(x, exp) => Exp::Rec(x.clone(), Box::new(substitute(exp, from_variable, to_exp))),
        Exp::Var(name) if name == from_variable => to_exp.clone(),
        Exp::Var(name) => Exp::Var(name.clone()),
        Exp::Const(constructor, exps) => Exp::Const(
            constructor.clone(),
            exps.iter()
                .map(|e| substitute(e, from_variable, to_exp))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::concrete;

    #[test]
    fn test_substitute() {
        let code = r#"rec x = x"#;
        let term = concrete::parse(code).unwrap();
        let sub_term = concrete::parse("Z()").unwrap();
        let substitued = substitute(&term, "x", &sub_term);
        assert_eq!(concrete::format(&substitued), "rec x = x");

        let code = r#"rec y = case x of {C() → x; D(x) → x}"#;
        let term = concrete::parse(code).unwrap();
        let substitued = substitute(
            &term,
            "x",
            &Exp::Lambda("z".to_string(), Box::new(Exp::Var("z".to_string()))),
        );
        assert_eq!(
            concrete::format(&substitued),
            "rec y = case λz.z of {\n  C() -> λz.z;\n  D(x) -> x\n}"
        );

        let code = r#"case z of { C(z) → z }"#;
        let term = concrete::parse(code).unwrap();
        let sub_term = concrete::parse("C(λz.z)").unwrap();
        let substitued = substitute(&term, "z", &sub_term);
        assert_eq!(
            concrete::format(&substitued),
            "case C(λz.z) of {\n  C(z) -> z\n}"
        );
    }
}
