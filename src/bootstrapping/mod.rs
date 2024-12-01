use bimap::BiMap;
use serde::{Serialize, Deserialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::syntax::{Branch, Constructor, Exp, Variable};

#[wasm_bindgen]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    variable: BiMap<Variable, usize>,
    next_variable_id: usize,
    constructor: BiMap<Constructor, usize>,
    next_constructor_id: usize,
}

impl Context {
    pub fn get_or_create_variable_id(&mut self, variable: &Variable) -> usize {
        if let Some(&id) = self.variable.get_by_left(variable) {
            id
        } else {
            let id = self.next_variable_id;
            self.variable.insert(variable.clone(), id);
            self.next_variable_id += 1;
            id
        }
    }

    pub fn get_or_create_constructor_id(&mut self, constructor: &Constructor) -> usize {
        if let Some(&id) = self.constructor.get_by_left(constructor) {
            id
        } else {
            let id = self.next_constructor_id;
            self.constructor.insert(constructor.clone(), id);
            self.next_constructor_id += 1;
            id
        }
    }
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_variable(&mut self, variable: Variable, id: usize) {
        if id >= self.next_constructor_id {
            self.next_constructor_id = id + 1;
            self.variable.insert(variable.clone(), id);
        } else if let Some(existing) = self.variable.get_by_right(&id).cloned() {
            // !!!this invalidates existing decompilation using this context!!!
            self.variable.insert(variable.clone(), id);
            self.variable
                .insert(existing.clone(), self.next_variable_id);
            self.next_variable_id += 1;
        } else {
            self.variable.insert(variable.clone(), id);
        }
    }

    pub fn set_constructor(&mut self, constructor: Constructor, id: usize) {
        if id >= self.next_constructor_id {
            self.next_constructor_id = id + 1;
            self.constructor.insert(constructor.clone(), id);
        } else if let Some(existing) = self.constructor.get_by_right(&id).cloned() {
            // !!!this invalidates existing decompilation using this context!!!
            self.constructor.insert(constructor.clone(), id);
            self.constructor
                .insert(existing.clone(), self.next_constructor_id);
            self.next_constructor_id += 1;
        } else {
            self.constructor.insert(constructor.clone(), id);
        }
    }

    pub fn variable_assignments(&self) -> JsValue {
        let result: Vec<_> = self.variable.iter().map(|(k,v)| (k.clone(),*v)).collect();
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn constructor_assignments(&self) -> JsValue {
        let result: Vec<_> = self.constructor.iter().map(|(k,v)| (k.clone(),*v)).collect();
        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}

fn number_to_exp(number: usize) -> Exp {
    if number == 0 {
        Exp::Const("Zero".to_string(), vec![])
    } else {
        Exp::Const("Suc".to_string(), vec![number_to_exp(number - 1)])
    }
}

fn decompile_list<T>(
    element_decompiler: impl Fn(&T, &mut Context) -> Exp + Copy,
) -> impl Fn(&[T], &mut Context) -> Exp {
    move |list, ctx| {
        let mut result = Exp::Const("Nil".to_string(), vec![]);
        for item in list.iter().rev() {
            result = Exp::Const(
                "Cons".to_string(),
                vec![element_decompiler(item, ctx), result],
            );
        }
        result
    }
}

fn decompile_var(variable: &Variable, context: &mut Context) -> Exp {
    let id = context.get_or_create_variable_id(variable);
    Exp::Const("Var".to_string(), vec![number_to_exp(id)])
}

fn decompile_branch(branch: &Branch, context: &mut Context) -> Exp {
    let id = context.get_or_create_constructor_id(&branch.constructor);
    let id_result = number_to_exp(id);
    let parameters_result = decompile_list(decompile_var)(&branch.parameters, context);
    let exp_result = decompile(branch.expression.as_ref(), context);
    Exp::Const(
        "Branch".to_string(),
        vec![id_result, parameters_result, exp_result],
    )
}

pub fn decompile(exp: &Exp, context: &mut Context) -> Exp {
    match exp {
        Exp::Var(variable) => decompile_var(variable, context),
        Exp::Const(constructor, exps) => {
            let id = context.get_or_create_constructor_id(constructor);
            let id_result = number_to_exp(id);
            let exps_result = decompile_list(decompile)(exps, context);
            Exp::Const("Const".to_string(), vec![id_result, exps_result])
        }
        Exp::Apply(f, x) => {
            let f_result = decompile(f, context);
            let x_result = decompile(x, context);
            Exp::Const("Apply".to_string(), vec![f_result, x_result])
        }
        Exp::Lambda(var, exp) => {
            let id = context.get_or_create_variable_id(var);
            Exp::Const(
                "Lambda".to_string(),
                vec![number_to_exp(id), decompile(exp, context)],
            )
        }
        Exp::Rec(var, exp) => {
            let id = context.get_or_create_variable_id(var);
            Exp::Const(
                "Rec".to_string(),
                vec![number_to_exp(id), decompile(exp, context)],
            )
        }
        Exp::Case(exp, branches) => {
            let exp_result = decompile(exp, context);
            let branches_result = decompile_list(decompile_branch)(branches, context);
            Exp::Const("Case".to_string(), vec![exp_result, branches_result])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::concrete;

    #[test]
    fn test_decompile() {
        let code = "𝜆 x. Suc(x)";
        let term = concrete::parse(code).unwrap();
        let mut context = Context::default();
        let result = concrete::format(&decompile(&term, &mut context)).to_string();
        assert_eq!(
            result,
            "Lambda(Zero(), Const(Zero(), Cons(Var(Zero()), Nil())))"
        );

        let code = "rec x = x";
        let term = concrete::parse(code).unwrap();
        context = Context::default();
        context.set_variable("x".to_string(), 1);
        let result = concrete::format(&decompile(&term, &mut context)).to_string();
        assert_eq!(result, "Rec(Suc(Zero()), Var(Suc(Zero())))");
    }
}
