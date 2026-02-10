use super::error_handling::JsException;
use super::{ControlFlow, Interpreter};
use crate::errors::RuntimeError;
use crate::parser::ast::{Expr, Stmt, SwitchCase};

impl Interpreter {
    pub(crate) fn eval_try_catch(
        &mut self,
        try_block: &[Stmt],
        catch_param: &Option<String>,
        catch_block: &Option<Vec<Stmt>>,
        finally_block: &Option<Vec<Stmt>>,
    ) -> Result<ControlFlow, RuntimeError> {
        let mut flow = ControlFlow::None;
        let mut pending_error = None;

        match self.eval_block(try_block) {
            Ok(v) => flow = v,
            Err(err) => {
                if let RuntimeError::Thrown { value } = err {
                    if let Some(catch_stmts) = catch_block {
                        self.env.push_scope(&mut self.heap);
                        if let Some(name) = catch_param {
                            self.env.define(name.clone(), value);
                        }
                        let mut catch_flow = ControlFlow::None;
                        for stmt in catch_stmts {
                            catch_flow = self.eval_stmt(stmt)?;
                            if matches!(
                                catch_flow,
                                ControlFlow::Return(_)
                                    | ControlFlow::Break(_)
                                    | ControlFlow::Continue(_)
                                    | ControlFlow::Yield(_)
                            ) {
                                break;
                            }
                        }
                        self.env.pop_scope();
                        flow = catch_flow;
                    } else {
                        pending_error = Some(RuntimeError::Thrown { value });
                    }
                } else {
                    pending_error = Some(err);
                }
            }
        }

        if let Some(finally_stmts) = finally_block {
            let finally_flow = self.eval_block(finally_stmts)?;
            if !matches!(finally_flow, ControlFlow::None) {
                return Ok(finally_flow);
            }
        }

        if let Some(err) = pending_error {
            return Err(err);
        }

        Ok(flow)
    }

    pub(crate) fn eval_switch(
        &mut self,
        discriminant: &Expr,
        cases: &[SwitchCase],
    ) -> Result<ControlFlow, RuntimeError> {
        let value = self.eval_expr(discriminant)?;
        let mut selected = None;
        let mut default_idx = None;

        for (idx, case) in cases.iter().enumerate() {
            if case.test.is_none() {
                default_idx = Some(idx);
                continue;
            }
            let case_value = self.eval_expr(case.test.as_ref().expect("checked above"))?;
            if case_value == value {
                selected = Some(idx);
                break;
            }
        }

        let mut idx = selected.or(default_idx);
        while let Some(i) = idx {
            for stmt in &cases[i].body {
                match self.eval_stmt(stmt)? {
                    ControlFlow::None => {}
                    ControlFlow::Break(_) => return Ok(ControlFlow::None),
                    ControlFlow::Continue(label) => return Ok(ControlFlow::Continue(label)),
                    ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    ControlFlow::Yield(v) => return Ok(ControlFlow::Yield(v)),
                }
            }
            idx = if i + 1 < cases.len() {
                Some(i + 1)
            } else {
                None
            };
        }

        Ok(ControlFlow::None)
    }

    pub(crate) fn eval_throw_expr(&mut self, expr: &Expr) -> Result<ControlFlow, RuntimeError> {
        let value = self.eval_expr(expr)?;
        Err(JsException::new(value).into_runtime_error())
    }
}
