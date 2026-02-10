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

// ── Reflect.get ──

#[test]
fn reflect_get_basic() {
    let out = run(r#"
        const obj = { x: 1, y: 2 };
        console.log(Reflect.get(obj, "x"));
        console.log(Reflect.get(obj, "y"));
    "#);
    assert_eq!(out, vec!["1", "2"]);
}

#[test]
fn reflect_get_missing_key() {
    let out = run(r#"
        const obj = { a: 10 };
        console.log(Reflect.get(obj, "b"));
    "#);
    assert_eq!(out, vec!["undefined"]);
}

// ── Reflect.set ──

#[test]
fn reflect_set_basic() {
    let out = run(r#"
        const obj = {};
        const result = Reflect.set(obj, "x", 42);
        console.log(result);
        console.log(obj.x);
    "#);
    assert_eq!(out, vec!["true", "42"]);
}

#[test]
fn reflect_set_overwrite() {
    let out = run(r#"
        const obj = { a: 1 };
        Reflect.set(obj, "a", 99);
        console.log(obj.a);
    "#);
    assert_eq!(out, vec!["99"]);
}

// ── Reflect.has ──

#[test]
fn reflect_has_basic() {
    let out = run(r#"
        const obj = { x: 1, y: 2 };
        console.log(Reflect.has(obj, "x"));
        console.log(Reflect.has(obj, "z"));
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn reflect_has_array() {
    let out = run(r#"
        const arr = [10, 20, 30];
        console.log(Reflect.has(arr, "1"));
        console.log(Reflect.has(arr, "5"));
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

// ── Reflect.deleteProperty ──

#[test]
fn reflect_delete_property_basic() {
    let out = run(r#"
        const obj = { a: 1, b: 2 };
        const result = Reflect.deleteProperty(obj, "a");
        console.log(result);
        console.log("a" in obj);
        console.log(obj.b);
    "#);
    assert_eq!(out, vec!["true", "false", "2"]);
}

#[test]
fn reflect_delete_property_nonexistent() {
    let out = run(r#"
        const obj = { a: 1 };
        console.log(Reflect.deleteProperty(obj, "z"));
    "#);
    assert_eq!(out, vec!["false"]);
}

// ── Reflect.ownKeys ──

#[test]
fn reflect_own_keys_basic() {
    let out = run(r#"
        const obj = { a: 1, b: 2, c: 3 };
        const keys = Reflect.ownKeys(obj);
        console.log(keys.length);
        console.log(keys.includes("a"));
        console.log(keys.includes("b"));
        console.log(keys.includes("c"));
    "#);
    assert_eq!(out, vec!["3", "true", "true", "true"]);
}

// ── Reflect.apply ──

#[test]
fn reflect_apply_basic() {
    let out = run(r#"
        const add = (a, b) => a + b;
        const result = Reflect.apply(add, undefined, [3, 4]);
        console.log(result);
    "#);
    assert_eq!(out, vec!["7"]);
}

#[test]
fn reflect_apply_with_this() {
    let out = run(r#"
        const obj = { multiplier: 10 };
        const fn_ = (x) => obj.multiplier * x;
        const result = Reflect.apply(fn_, obj, [5]);
        console.log(result);
    "#);
    assert_eq!(out, vec!["50"]);
}

// ── Reflect.construct ──

#[test]
fn reflect_construct_basic() {
    let out = run(r#"
        class Point {
            constructor(x, y) {
                this.x = x;
                this.y = y;
            }
        }
        const p = Reflect.construct(Point, [3, 4]);
        console.log(p.x);
        console.log(p.y);
    "#);
    assert_eq!(out, vec!["3", "4"]);
}

// ── Reflect.getPrototypeOf ──

#[test]
fn reflect_get_prototype_of_basic() {
    let out = run(r#"
        const obj = {};
        const proto = Reflect.getPrototypeOf(obj);
        console.log(proto);
    "#);
    assert_eq!(out, vec!["null"]);
}

#[test]
fn reflect_get_prototype_of_class_instance() {
    let out = run(r#"
        class Animal {
            speak() { return "..."; }
        }
        const a = new Animal();
        const proto = Reflect.getPrototypeOf(a);
        console.log(proto.speak());
    "#);
    assert_eq!(out, vec!["..."]);
}

// ── Reflect with Proxy (integration) ──

#[test]
fn reflect_get_on_proxy() {
    let out = run(r#"
        const target = { x: 100 };
        const handler = {
            get: (t, p) => {
                return t[p] + 1;
            }
        };
        const p = new Proxy(target, handler);
        console.log(Reflect.get(p, "x"));
    "#);
    assert_eq!(out, vec!["101"]);
}

#[test]
fn reflect_set_on_proxy() {
    let out = run(r#"
        const log = [];
        const target = {};
        const handler = {
            set: (t, prop, val) => {
                log.push("trapped:" + prop);
                t[prop] = val;
                return true;
            }
        };
        const p = new Proxy(target, handler);
        Reflect.set(p, "a", 42);
        console.log(log[0]);
        console.log(target.a);
    "#);
    assert_eq!(out, vec!["trapped:a", "42"]);
}

#[test]
fn reflect_has_on_proxy() {
    let out = run(r#"
        const handler = {
            has: (t, p) => {
                return p === "magic";
            }
        };
        const p = new Proxy({}, handler);
        console.log(Reflect.has(p, "magic"));
        console.log(Reflect.has(p, "other"));
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn proxy_trap_using_reflect_for_forwarding() {
    let out = run(r#"
        const target = { a: 1, b: 2 };
        const handler = {
            get: (t, prop) => {
                console.log("accessing:" + prop);
                return Reflect.get(t, prop);
            },
            set: (t, prop, val) => {
                console.log("setting:" + prop);
                return Reflect.set(t, prop, val);
            }
        };
        const p = new Proxy(target, handler);
        console.log(p.a);
        p.b = 20;
        console.log(p.b);
    "#);
    assert_eq!(
        out,
        vec!["accessing:a", "1", "setting:b", "accessing:b", "20"]
    );
}

#[test]
fn reflect_delete_on_proxy() {
    let out = run(r#"
        const target = { x: 1, y: 2 };
        const handler = {
            deleteProperty: (t, prop) => {
                console.log("delete:" + prop);
                delete t[prop];
                return true;
            }
        };
        const p = new Proxy(target, handler);
        console.log(Reflect.deleteProperty(p, "x"));
        console.log("x" in target);
    "#);
    assert_eq!(out, vec!["delete:x", "true", "false"]);
}

#[test]
fn reflect_own_keys_on_proxy() {
    let out = run(r#"
        const handler = {
            ownKeys: (t) => {
                return ["filtered"];
            }
        };
        const p = new Proxy({ a: 1, b: 2 }, handler);
        const keys = Reflect.ownKeys(p);
        console.log(keys.length);
        console.log(keys[0]);
    "#);
    assert_eq!(out, vec!["1", "filtered"]);
}
