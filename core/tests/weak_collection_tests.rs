use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn run(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lex");
    let program = parse(tokens).expect("parse");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("run");
    interp.output().to_vec()
}

fn run_err(source: &str) -> String {
    let tokens = lex(source).expect("lex");
    let program = parse(tokens).expect("parse");
    let mut interp = Interpreter::new();
    interp.run(&program).unwrap_err().to_string()
}

#[test]
fn weak_map_empty_constructor() {
    let out = run(r#"
        const wm = new WeakMap();
        console.log(typeof wm);
    "#);
    assert_eq!(out, vec!["object"]);
}

#[test]
fn weak_map_set_get_has_delete() {
    let out = run(r#"
        const wm = new WeakMap();
        const key = {};
        wm.set(key, 42);
        console.log(wm.has(key));
        console.log(wm.get(key));
        console.log(wm.delete(key));
        console.log(wm.has(key));
    "#);
    assert_eq!(out, vec!["true", "42", "true", "false"]);
}

#[test]
fn weak_map_requires_object_key() {
    let err = run_err(
        r#"
        const wm = new WeakMap();
        wm.set("string_key", 1);
    "#,
    );
    assert!(err.contains("Invalid value used as weak map key"));
}

#[test]
fn weak_map_number_key_rejected() {
    let err = run_err(
        r#"
        const wm = new WeakMap();
        wm.set(123, "val");
    "#,
    );
    assert!(err.contains("Invalid value used as weak map key"));
}

#[test]
fn weak_map_from_iterable() {
    let out = run(r#"
        const k1 = {};
        const k2 = {};
        const wm = new WeakMap([[k1, "a"], [k2, "b"]]);
        console.log(wm.get(k1));
        console.log(wm.get(k2));
    "#);
    assert_eq!(out, vec!["a", "b"]);
}

#[test]
fn weak_map_iterable_rejects_non_object_key() {
    let err = run_err(
        r#"
        new WeakMap([["bad", 1]]);
    "#,
    );
    assert!(err.contains("Invalid value used as weak map key"));
}

#[test]
fn weak_map_set_returns_weak_map_for_chaining() {
    let out = run(r#"
        const wm = new WeakMap();
        const a = {};
        const b = {};
        wm.set(a, 1).set(b, 2);
        console.log(wm.has(a));
        console.log(wm.has(b));
    "#);
    assert_eq!(out, vec!["true", "true"]);
}

#[test]
fn weak_map_overwrite_value() {
    let out = run(r#"
        const wm = new WeakMap();
        const key = {};
        wm.set(key, "old");
        wm.set(key, "new");
        console.log(wm.get(key));
    "#);
    assert_eq!(out, vec!["new"]);
}

#[test]
fn weak_map_get_missing_returns_undefined() {
    let out = run(r#"
        const wm = new WeakMap();
        console.log(wm.get({}));
    "#);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn weak_map_array_as_key() {
    let out = run(r#"
        const wm = new WeakMap();
        const arr = [1, 2, 3];
        wm.set(arr, "found");
        console.log(wm.get(arr));
        console.log(wm.has(arr));
    "#);
    assert_eq!(out, vec!["found", "true"]);
}

#[test]
fn weak_set_empty_constructor() {
    let out = run(r#"
        const ws = new WeakSet();
        console.log(typeof ws);
    "#);
    assert_eq!(out, vec!["object"]);
}

#[test]
fn weak_set_add_has_delete() {
    let out = run(r#"
        const ws = new WeakSet();
        const obj = {};
        ws.add(obj);
        console.log(ws.has(obj));
        console.log(ws.delete(obj));
        console.log(ws.has(obj));
    "#);
    assert_eq!(out, vec!["true", "true", "false"]);
}

#[test]
fn weak_set_requires_object_value() {
    let err = run_err(
        r#"
        const ws = new WeakSet();
        ws.add("string");
    "#,
    );
    assert!(err.contains("Invalid value used as weak set value"));
}

#[test]
fn weak_set_from_iterable() {
    let out = run(r#"
        const a = {};
        const b = {};
        const ws = new WeakSet([a, b]);
        console.log(ws.has(a));
        console.log(ws.has(b));
    "#);
    assert_eq!(out, vec!["true", "true"]);
}

#[test]
fn weak_set_iterable_rejects_non_object() {
    let err = run_err(
        r#"
        new WeakSet([1, 2, 3]);
    "#,
    );
    assert!(err.contains("Invalid value used as weak set value"));
}

#[test]
fn weak_set_add_returns_weak_set_for_chaining() {
    let out = run(r#"
        const ws = new WeakSet();
        const a = {};
        const b = {};
        ws.add(a).add(b);
        console.log(ws.has(a));
        console.log(ws.has(b));
    "#);
    assert_eq!(out, vec!["true", "true"]);
}

#[test]
fn weak_set_no_duplicates() {
    let out = run(r#"
        const ws = new WeakSet();
        const obj = {};
        ws.add(obj);
        ws.add(obj);
        console.log(ws.has(obj));
        console.log(ws.delete(obj));
        console.log(ws.has(obj));
    "#);
    assert_eq!(out, vec!["true", "true", "false"]);
}
