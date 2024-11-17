#![feature(box_patterns)]

mod grammar;
use std::collections::HashSet;

use grammar::{Branch, Exp};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

fn free_variables_branch(branch: &Branch) -> HashSet<String> {
    let mut result = free_variables(branch.expression.as_ref());
    for var in &branch.variables {
        result.remove(var);
    }
    result
}

pub fn free_variables(exp: &Exp) -> HashSet<String> {
    match exp {
        Exp::Var(x) => HashSet::from([x.clone()]),
        Exp::Apply(exp1, exp2) => HashSet::union(&free_variables(exp1), &free_variables(exp2))
            .cloned()
            .collect(),
        Exp::Lambda(x, exp) | Exp::Rec(x, exp) => {
            let mut result = free_variables(exp);
            result.remove(x);
            result
        }
        Exp::Const(_, exps) => exps.iter().fold(HashSet::new(), |acc, e| {
            acc.union(&free_variables(e)).cloned().collect()
        }),
        Exp::Case(exp, branches) => {
            let mut result = free_variables(exp);
            for branch in branches {
                result.extend(free_variables_branch(branch));
            }
            result
        }
    }
}

fn substitute_branch(branch: &Branch, from_variable: &str, to_exp: &Exp) -> Branch {
    if branch.variables.contains(&from_variable.to_string()) {
        branch.clone()
    } else {
        Branch {
            constructor: branch.constructor.clone(),
            variables: branch.variables.clone(),
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

pub fn eval_branch(exp: &Exp, branch: &Branch) -> Exp {
    if let Exp::Const(constructor, exps) = exp {
        if constructor == &branch.constructor && branch.variables.len() == exps.len() {
            let bindings = Iterator::zip(branch.variables.iter(), exps.iter());
            let mut result = *(branch.expression.clone());
            for (var, exp) in bindings {
                result = substitute(&result, var, exp);
            }
            return eval(&result);
        }
    }
    unreachable!()
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
                if let Some(branch) = branches.iter().find(|branch| {
                    branch.constructor == constructor && branch.variables.len() == exps.len()
                }) {
                    return eval_branch(&Exp::Const(constructor, exps), branch);
                }
            }
            return Exp::Case(e.clone(), branches.clone());
        }
        Exp::Lambda(x, exp) => Exp::Lambda(x.clone(), exp.clone()),
        Exp::Const(constructor, exps) => {
            Exp::Const(constructor.clone(), exps.iter().map(eval).collect())
        }
        Exp::Rec(x, exp) => eval(&substitute(&exp, &x, &Exp::Rec(x.clone(), exp.clone()))),
    }
}

#[wasm_bindgen]
pub fn eval_chi(exp: JsValue) -> JsValue {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    let result = eval(&exp);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn fmt(exp: JsValue) -> String {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    format!("{}", exp)
}

#[wasm_bindgen]
pub fn fmt_abstract(exp: JsValue) -> String {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    format!("{:?}", exp)
}

mod tests {
    use super::*;
    use grammar::parse_exp;
    #[test]
    fn test_eval() {
        let code_add = r#"(rec foo = 𝜆 m. 𝜆 n. case n of {
    Zero() → m;
    Suc(n) → case m of {
    Zero() → Zero();
    Suc(m) → foo m n}})
    Suc(Suc(Zero())) Suc(Zero())"#;
        let term_add = parse_exp(code_add).unwrap().1;
        let t = eval(&term_add);
        assert_eq!(format!("{}", t), "Suc(Zero())");

        let code = r#"
            (rec add = λm.λn.case n of {
                Zero() -> m;
                Suc(n) -> Suc(add m n)
                }
            ) Suc(Suc(Zero())) Suc(Zero())"#;
        let term = parse_exp(code).unwrap().1;
        assert_eq!(format!("{}", term), "Suc(Suc(Suc(Zero())))");
    }
}
