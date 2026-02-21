use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn builtin_console_log(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let line = args
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        println!("{line}");
        self.output.push(line);
        Ok(JsValue::Undefined)
    }
}
