use crate::syntax::{Branch, Exp};

mod substitute;
pub use substitute::substitute;

fn eval_branch(arguments: &[Exp], branch: &Branch) -> Exp {
    let bindings = Iterator::zip(branch.parameters.iter(), arguments.iter());
    let mut result = *(branch.expression.clone());
    for (var, exp) in bindings.rev() {
        result = substitute(&result, var, exp);
    }
    eval(&result)
}

pub fn eval(exp: &Exp) -> Exp {
    match exp {
        Exp::Var(x) => Exp::Var(x.clone()),
        Exp::Apply(f, param) => {
            if let Exp::Lambda(x, exp) = eval(f) {
                let param = eval(param);
                eval(&substitute(&exp, &x, &param))
            } else {
                Exp::Apply(f.clone(), param.clone())
            }
        }
        Exp::Case(e, branches) => {
            let exp = eval(e);
            if let Exp::Const(constructor, exps) = exp {
                if let Some(branch) = branches
                    .iter()
                    .find(|branch| branch.constructor == constructor)
                {
                    if branch.parameters.len() != exps.len() {
                        panic!("Number of parameters does not match");
                    }
                    return eval_branch(&exps, branch);
                }
            }
            Exp::Case(e.clone(), branches.clone())
        }
        Exp::Lambda(x, exp) => Exp::Lambda(x.clone(), exp.clone()),
        Exp::Const(constructor, exps) => {
            Exp::Const(constructor.clone(), exps.iter().map(eval).collect())
        }
        Exp::Rec(x, exp) => eval(&substitute(exp, x, &Exp::Rec(x.clone(), exp.clone()))),
    }
}

#[cfg(test)]
mod tests {
    use crate::{semantic::eval, syntax::concrete};

    #[test]
    fn test_eval() {
        let code_sub = r#"(rec foo = 𝜆 m. 𝜆 n. case n of {
        Zero() → m;
        Suc(n) → case m of {
        Zero() → Zero();
        Suc(m) → foo m n}})
        Suc(Suc(Zero())) Suc(Zero())"#;
        let term_sub = concrete::parse(code_sub).unwrap();
        let result = eval(&term_sub);
        assert_eq!(concrete::format(&result), "Suc(Zero())");

        let code = r#"case C(D(),E()) of { C(x, x) → x }"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "E()");

        let code = r#"case C(λx.x, Zero()) of { C(f, x) → f x }"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "Zero()");

        let code = r#"case (λx.x) C() of { C() → C() }"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "C()");

        let code = r#"((λx.x)(λx.x))(λx.x)"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "λx.x");

        let code = r#"((λx.x)(λx.x))(λx.x)"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "λx.x");
    }

    #[test]
    #[should_panic]
    fn test_eval_too_much_arg() {
        let code = r#"case C(C()) of { C() → C(); C(x) → x }"#;
        let term = concrete::parse(code).unwrap();
        let result = eval(&term);
        assert_eq!(concrete::format(&result), "λx.x");
    }
}
