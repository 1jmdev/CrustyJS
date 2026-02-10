use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;
use crustyjs::{Engine, Value};
use std::path::PathBuf;

#[test]
fn kitchen_sink_example_runs() {
    let path = PathBuf::from("examples/kitchen_sink.js");
    let source = std::fs::read_to_string(&path).expect("read kitchen sink");
    let tokens = lex(&source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new_with_realtime_timers(false);
    interp
        .run_with_path(&program, path)
        .expect("execution should succeed");

    let out = interp.output();
    assert_eq!(out.len(), 4);
    assert_eq!(out[0], "Hello Ada, age 36");
    assert!(out[1].contains("\"sum\":14"));
    assert!(out[1].contains("\"sorted\""));
    assert!(out[1].contains("1.0") || out[1].contains("1"));
    assert!(out[1].contains("\"timestamp\":"));
    assert!(out[2].contains("id"));
    assert!(out[2].contains("role"));
    assert!(out[2].contains("active"));
    assert_eq!(out[3], "1");
}

#[test]
fn engine_context_animation_frame_flow() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();
    ctx.eval("let frame = 0; requestAnimationFrame((t) => { frame = t; });")
        .expect("script should evaluate");
    ctx.run_animation_callbacks(16.0)
        .expect("frame callbacks should run");
    let frame = ctx.get_global("frame").expect("frame should exist");
    assert_eq!(frame, Value::Number(16.0));
}

#[test]
fn engine_context_optional_chaining_flow() {
    let engine = Engine::new();
    let mut ctx = engine.new_context();
    ctx.eval("let data = null; let out = data?.nested?.value ?? 'none';")
        .expect("script should evaluate");
    let out = ctx.get_global("out").expect("out should exist");
    assert_eq!(out, Value::String("none".to_string()));
}
