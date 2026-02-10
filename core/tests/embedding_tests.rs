use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crustyjs::{ClassBuilder, Engine, EventTarget, Value};

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

#[test]
fn register_class_method_on_instances() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    let class_def = ClassBuilder::new("Element")
        .constructor(|args| {
            let tag = args
                .get(0)
                .cloned()
                .unwrap_or(Value::String("div".to_string()));
            if let Value::Object(object) = args.this() {
                object.borrow_mut().set("tag".to_string(), tag);
            }
            Ok(Value::Undefined)
        })
        .method("tagName", |args| {
            let this = args.this();
            if let Value::Object(object) = this {
                return Ok(object.borrow().get("tag").unwrap_or(Value::Undefined));
            }
            Ok(Value::Undefined)
        })
        .build();

    ctx.register_class(class_def);
    ctx.eval("let el = Element('article'); let tag = el.tagName();")
        .expect("class method invocation should succeed");

    let tag = ctx.get_global("tag").expect("tag should be set");
    assert_eq!(tag, Value::String("article".to_string()));
}

#[test]
fn context_exposes_event_loop_drivers() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    ctx.eval("let tick = 0; setTimeout(() => { tick = 1; }, 0);")
        .expect("timer script should evaluate");

    ctx.run_microtasks()
        .expect("microtask execution should not fail");
    ctx.run_pending_timers()
        .expect("pending timer execution should not fail");

    let tick = ctx.get_global("tick").expect("tick should exist");
    assert_eq!(tick, Value::Number(1.0));
}

#[test]
fn register_class_getter_setter_and_inheritance() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    let parent = ClassBuilder::new("Base")
        .method("base", |_| Ok(Value::String("base".to_string())))
        .build();
    ctx.register_class(parent);

    let child = ClassBuilder::new("Element")
        .inherit("Base")
        .constructor(|args| {
            if let Value::Object(object) = args.this() {
                object
                    .borrow_mut()
                    .set("_html".to_string(), Value::String("init".to_string()));
            }
            Ok(Value::Undefined)
        })
        .property_getter("innerHTML", |args| {
            if let Value::Object(object) = args.this() {
                return Ok(object.borrow().get("_html").unwrap_or(Value::Undefined));
            }
            Ok(Value::Undefined)
        })
        .property_setter("innerHTML", |args| {
            if let Value::Object(object) = args.this() {
                let val = args.get(0).cloned().unwrap_or(Value::Undefined);
                object.borrow_mut().set("_html".to_string(), val);
            }
            Ok(Value::Undefined)
        })
        .build();
    ctx.register_class(child);

    ctx.eval("let el = Element(); el.innerHTML = 'next'; let a = el.innerHTML; let b = el.base();")
        .expect("native getter, setter and inherited method should work");

    let a = ctx.get_global("a").expect("a should exist");
    let b = ctx.get_global("b").expect("b should exist");
    assert_eq!(a, Value::String("next".to_string()));
    assert_eq!(b, Value::String("base".to_string()));
}

#[test]
fn context_drives_animation_callbacks() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    ctx.eval(
        "let calls = 0; let ts = 0; const id = requestAnimationFrame((t) => { calls = calls + 1; ts = t; });",
    )
    .expect("requestAnimationFrame should schedule callback");

    ctx.run_animation_callbacks(123.0)
        .expect("animation callbacks should run");

    let calls = ctx.get_global("calls").expect("calls should exist");
    let ts = ctx.get_global("ts").expect("ts should exist");
    assert_eq!(calls, Value::Number(1.0));
    assert_eq!(ts, Value::Number(123.0));

    ctx.eval("cancelAnimationFrame(requestAnimationFrame(() => { calls = calls + 1; }));")
        .expect("cancelAnimationFrame call should succeed");
    ctx.run_animation_callbacks(200.0)
        .expect("animation callbacks should run");

    let calls_after = ctx.get_global("calls").expect("calls should exist");
    assert_eq!(calls_after, Value::Number(1.0));
}

#[test]
fn context_dispatches_event_target_listeners() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();

    ctx.eval("let seen = 0; function onEvent(ev) { seen = ev; }")
        .expect("script should evaluate");
    let callback = ctx
        .get_global("onEvent")
        .expect("onEvent should be available");

    let mut target = EventTarget::new();
    target.add_event_listener("click", callback.clone());
    ctx.dispatch_event(&target, "click", Value::Number(7.0))
        .expect("dispatch should succeed");

    let seen = ctx.get_global("seen").expect("seen should exist");
    assert_eq!(seen, Value::Number(7.0));

    target.remove_event_listener("click", &callback);
    ctx.dispatch_event(&target, "click", Value::Number(9.0))
        .expect("dispatch should succeed");
    let seen_after = ctx.get_global("seen").expect("seen should exist");
    assert_eq!(seen_after, Value::Number(7.0));
}
