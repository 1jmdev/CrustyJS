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
    format!("{}", interp.run(&program).unwrap_err())
}

#[test]
fn symbol_creation_and_typeof() {
    let out = run(r#"
        const s = Symbol("hello");
        console.log(typeof s);
    "#);
    assert_eq!(out, vec!["symbol"]);
}

#[test]
fn symbol_description_property() {
    let out = run(r#"
        const s = Symbol("desc");
        console.log(s.description);
    "#);
    assert_eq!(out, vec!["desc"]);
}

#[test]
fn symbol_without_description() {
    let out = run(r#"
        const s = Symbol();
        console.log(typeof s);
        console.log(s.description);
    "#);
    assert_eq!(out, vec!["symbol", "undefined"]);
}

#[test]
fn symbol_to_string() {
    let out = run(r#"
        const s = Symbol("test");
        console.log(s.toString());
    "#);
    assert_eq!(out, vec!["Symbol(test)"]);
}

#[test]
fn symbols_are_unique() {
    let out = run(r#"
        const a = Symbol("x");
        const b = Symbol("x");
        console.log(a === b);
    "#);
    assert_eq!(out, vec!["false"]);
}

#[test]
fn symbol_as_property_key() {
    let out = run(r#"
        const id = Symbol("id");
        const obj = {};
        obj[id] = 42;
        console.log(obj[id]);
    "#);
    assert_eq!(out, vec!["42"]);
}

#[test]
fn symbol_property_not_visible_as_string() {
    let out = run(r#"
        const sym = Symbol("hidden");
        const obj = { name: "test" };
        obj[sym] = "secret";
        console.log(obj.name);
        console.log(obj[sym]);
    "#);
    assert_eq!(out, vec!["test", "secret"]);
}

#[test]
fn symbol_for_returns_same_symbol() {
    let out = run(r#"
        const a = Symbol.for("shared");
        const b = Symbol.for("shared");
        console.log(a === b);
    "#);
    assert_eq!(out, vec!["true"]);
}

#[test]
fn symbol_for_vs_symbol_are_different() {
    let out = run(r#"
        const a = Symbol.for("key");
        const b = Symbol("key");
        console.log(a === b);
    "#);
    assert_eq!(out, vec!["false"]);
}

#[test]
fn symbol_key_for_reverse_lookup() {
    let out = run(r#"
        const s = Symbol.for("myKey");
        console.log(Symbol.keyFor(s));
    "#);
    assert_eq!(out, vec!["myKey"]);
}

#[test]
fn symbol_key_for_returns_undefined_for_non_global() {
    let out = run(r#"
        const s = Symbol("local");
        console.log(Symbol.keyFor(s));
    "#);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn new_symbol_throws() {
    let err = run_err("new Symbol('test');");
    assert!(err.contains("not a constructor"), "got: {err}");
}

#[test]
fn symbol_iterator_well_known() {
    let out = run(r#"
        const iter = Symbol.iterator;
        console.log(typeof iter);
        console.log(iter.toString());
    "#);
    assert_eq!(out, vec!["symbol", "Symbol(Symbol.iterator)"]);
}

#[test]
fn symbol_is_truthy() {
    let out = run(r#"
        const s = Symbol();
        console.log(!!s);
    "#);
    assert_eq!(out, vec!["true"]);
}
