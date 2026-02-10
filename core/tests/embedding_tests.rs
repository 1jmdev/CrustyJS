use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crustyjs::{ClassBuilder, Engine, Value};

#[test]
fn engine_context_eval_and_globals() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    ctx.set_global("seed", Value::Number(41.0));
    ctx.eval("let answer = seed + 1;")
        .expect("eval should succeed");

    let value = ctx
        .get_global("answer")
        .expect("answer global should be present");
    assert_eq!(value, Value::Number(42.0));
}

#[test]
fn eval_module_from_file_path() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("crustyjs-embed-{unique}.js"));

    fs::write(&path, "let moduleValue = 7 * 6;").expect("temporary test module should be written");
    ctx.eval_module(&path)
        .expect("module evaluation should succeed");

    let value = ctx
        .get_global("moduleValue")
        .expect("moduleValue should be defined");
    assert_eq!(value, Value::Number(42.0));

    fs::remove_file(path).expect("temporary test module should be cleaned up");
}

#[test]
fn register_and_call_native_function() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    ctx.set_global_function("double", |args| {
        let value = args.get(0).cloned().unwrap_or(Value::Undefined).to_number();
        Ok(Value::Number(value * 2.0))
    });

    ctx.eval("let result = double(21);")
        .expect("native function call should succeed");
    let result = ctx
        .get_global("result")
        .expect("result should be available");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn register_class_constructor() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    let class_def = ClassBuilder::new("Element")
        .constructor(|args| {
            let tag = args
                .get(0)
                .cloned()
                .unwrap_or(Value::String("div".to_string()));
            Ok(tag)
        })
        .build();

    ctx.register_class(class_def);
    ctx.eval("let created = Element('section');")
        .expect("class constructor call should succeed");
    let created = ctx
        .get_global("created")
        .expect("created should be available");
    assert_eq!(created, Value::String("section".to_string()));
}
