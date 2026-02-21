use std::any::Any;

pub fn format_panic(payload: Box<dyn Any + Send>) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return format!("panic while running test: {msg}");
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return format!("panic while running test: {msg}");
    }
    "panic while running test".to_string()
}
