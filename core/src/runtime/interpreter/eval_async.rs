use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{Expr, Param, Stmt};
use crate::runtime::value::promise::{JsPromise, PromiseState};
use crate::runtime::value::JsValue;
use std::cell::RefCell;
use std::rc::Rc;

impl Interpreter {
    pub(crate) fn execute_function_body(
        &mut self,
        params: &[Param],
        body: &[Stmt],
        closure_env: &[Rc<RefCell<crate::runtime::environment::Scope>>],
        this_binding: Option<JsValue>,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let params = params.to_vec();
        let body = body.to_vec();
        let captured = closure_env.to_vec();
        let saved_scopes = self.env.replace_scopes(captured);

        self.env.push_scope_with_this(this_binding);
        for (idx, param) in params.iter().enumerate() {
            let mut value = args.get(idx).cloned().unwrap_or(JsValue::Undefined);
            if matches!(value, JsValue::Undefined) {
                if let Some(default_expr) = &param.default {
                    value = self.eval_expr(default_expr)?;
                }
            }
            self.eval_pattern_binding(&param.pattern, value)?;
        }

        let mut result = JsValue::Undefined;
        let call_result = (|| -> Result<(), RuntimeError> {
            for stmt in &body {
                match self.eval_stmt(stmt)? {
                    super::ControlFlow::Return(val) => {
                        result = val;
                        break;
                    }
                    super::ControlFlow::Break => {
                        return Err(RuntimeError::TypeError {
                            message: "illegal break statement".to_string(),
                        });
                    }
                    super::ControlFlow::None => {}
                }
            }
            Ok(())
        })();

        self.env.pop_scope();
        self.env.replace_scopes(saved_scopes);
        call_result?;
        Ok(result)
    }

    pub(crate) fn execute_async_function_body(
        &mut self,
        params: &[Param],
        body: &[Stmt],
        closure_env: &[Rc<RefCell<crate::runtime::environment::Scope>>],
        this_binding: Option<JsValue>,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let promise = Rc::new(RefCell::new(JsPromise::pending()));
        self.async_depth += 1;
        let result = self.execute_function_body(params, body, closure_env, this_binding, args);
        self.async_depth = self.async_depth.saturating_sub(1);

        match result {
            Ok(value) => {
                self.settle_promise(&promise, false, value)?;
            }
            Err(err) => {
                let trace = self.call_stack.format_trace();
                let err = self.attach_stack_to_error(err, &trace);
                let rejected = match err {
                    RuntimeError::Thrown { value } => value,
                    other => JsValue::String(other.to_string()),
                };
                self.settle_promise(&promise, true, rejected)?;
            }
        }

        Ok(JsValue::Promise(promise))
    }

    pub(crate) fn eval_await_expr(&mut self, expr: &Expr) -> Result<JsValue, RuntimeError> {
        if self.async_depth == 0 {
            return Err(RuntimeError::TypeError {
                message: "await is only valid inside async functions".to_string(),
            });
        }

        let value = self.eval_expr(expr)?;
        match value {
            JsValue::Promise(promise) => {
                self.run_event_loop_until_promise_settled(&promise)?;
                match &promise.borrow().state {
                    PromiseState::Pending => Err(RuntimeError::TypeError {
                        message: "awaited promise did not settle".to_string(),
                    }),
                    PromiseState::Fulfilled(v) => Ok(v.clone()),
                    PromiseState::Rejected(v) => Err(RuntimeError::Thrown { value: v.clone() }),
                }
            }
            other => Ok(other),
        }
    }
}
