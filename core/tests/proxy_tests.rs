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
fn proxy_get_trap() {
    let out = run(r#"
        const handler = {
            get: (target, prop) => {
                if (prop === "greeting") {
                    return "hello from proxy";
                }
                return target[prop];
            }
        };
        const target = { name: "world" };
        const p = new Proxy(target, handler);
        console.log(p.greeting);
        console.log(p.name);
    "#);
    assert_eq!(out, vec!["hello from proxy", "world"]);
}

#[test]
fn proxy_set_trap() {
    let out = run(r#"
        const log = [];
        const handler = {
            set: (target, prop, value) => {
                log.push("set " + prop + " = " + value);
                target[prop] = value;
                return true;
            }
        };
        const target = {};
        const p = new Proxy(target, handler);
        p.x = 5;
        p.y = 10;
        console.log(log[0]);
        console.log(log[1]);
        console.log(target.x);
    "#);
    assert_eq!(out, vec!["set x = 5", "set y = 10", "5"]);
}

#[test]
fn proxy_no_trap_passthrough() {
    let out = run(r#"
        const target = { a: 1, b: 2 };
        const p = new Proxy(target, {});
        console.log(p.a);
        console.log(p.b);
        p.c = 3;
        console.log(target.c);
    "#);
    assert_eq!(out, vec!["1", "2", "3"]);
}

#[test]
fn proxy_get_trap_default_value() {
    let out = run(r#"
        const handler = {
            get: (target, prop) => {
                return 42;
            }
        };
        const p = new Proxy({}, handler);
        console.log(p.anything);
        console.log(p.whatever);
    "#);
    assert_eq!(out, vec!["42", "42"]);
}

#[test]
fn in_operator_object() {
    let out = run(r#"
        const obj = { x: 1, y: 2 };
        console.log("x" in obj);
        console.log("z" in obj);
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn in_operator_array() {
    let out = run(r#"
        const arr = [10, 20, 30];
        console.log(0 in arr);
        console.log(2 in arr);
        console.log(3 in arr);
    "#);
    assert_eq!(out, vec!["true", "true", "false"]);
}

#[test]
fn proxy_has_trap() {
    let out = run(r#"
        const handler = {
            has: (target, prop) => {
                if (prop === "secret") {
                    return false;
                }
                return prop in target;
            }
        };
        const target = { secret: 42, visible: 1 };
        const p = new Proxy(target, handler);
        console.log("secret" in p);
        console.log("visible" in p);
    "#);
    assert_eq!(out, vec!["false", "true"]);
}

#[test]
fn proxy_has_trap_no_trap_fallback() {
    let out = run(r#"
        const target = { a: 1 };
        const p = new Proxy(target, {});
        console.log("a" in p);
        console.log("b" in p);
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn proxy_revocable_basic() {
    let out = run(r#"
        const { proxy, revoke } = Proxy.revocable({ x: 10 }, {});
        console.log(proxy.x);
        revoke();
        console.log(typeof proxy);
    "#);
    assert_eq!(out, vec!["10", "object"]);
}

#[test]
fn proxy_revocable_access_after_revoke() {
    let err = run_err(
        r#"
        const { proxy, revoke } = Proxy.revocable({ x: 10 }, {});
        revoke();
        proxy.x;
    "#,
    );
    assert!(
        err.contains("revoked"),
        "expected revoked error, got: {err}"
    );
}

#[test]
fn proxy_revocable_set_after_revoke() {
    let err = run_err(
        r#"
        const { proxy, revoke } = Proxy.revocable({}, {});
        revoke();
        proxy.y = 5;
    "#,
    );
    assert!(
        err.contains("revoked"),
        "expected revoked error, got: {err}"
    );
}

#[test]
fn proxy_revocable_with_traps() {
    let out = run(r#"
        const handler = {
            get: (target, prop) => {
                return "trapped:" + prop;
            }
        };
        const { proxy, revoke } = Proxy.revocable({ a: 1 }, handler);
        console.log(proxy.a);
        console.log(proxy.b);
    "#);
    assert_eq!(out, vec!["trapped:a", "trapped:b"]);
}

// ── Proxy apply trap ──

#[test]
fn proxy_apply_trap_basic() {
    let out = run(r#"
        const target = (a, b) => a + b;
        const handler = {
            apply: (fn_, thisArg, args) => {
                return args[0] * args[1];
            }
        };
        const p = new Proxy(target, handler);
        console.log(p(3, 4));
    "#);
    assert_eq!(out, vec!["12"]);
}

#[test]
fn proxy_apply_trap_logging() {
    let out = run(r#"
        const target = (x) => x * 2;
        const handler = {
            apply: (fn_, thisArg, args) => {
                console.log("called with " + args[0]);
                return fn_(args[0]);
            }
        };
        const p = new Proxy(target, handler);
        console.log(p(5));
    "#);
    assert_eq!(out, vec!["called with 5", "10"]);
}

#[test]
fn proxy_apply_no_trap_passthrough() {
    let out = run(r#"
        const target = (x) => x + 1;
        const p = new Proxy(target, {});
        console.log(p(9));
    "#);
    assert_eq!(out, vec!["10"]);
}

// ── Proxy construct trap ──

#[test]
fn proxy_construct_trap_basic() {
    let out = run(r#"
        class Foo {
            constructor(x) {
                this.x = x;
            }
        }
        const handler = {
            construct: (target, args, newTarget) => {
                console.log("constructing with " + args[0]);
                return Reflect.construct(target, [args[0] * 10]);
            }
        };
        const P = new Proxy(Foo, handler);
        const obj = new P(3);
        console.log(obj.x);
    "#);
    assert_eq!(out, vec!["constructing with 3", "30"]);
}

#[test]
fn proxy_construct_no_trap() {
    let out = run(r#"
        class Bar {
            constructor(v) {
                this.v = v;
            }
        }
        const P = new Proxy(Bar, {});
        const obj = new P(42);
        console.log(obj.v);
    "#);
    assert_eq!(out, vec!["42"]);
}

// ── delete operator ──

#[test]
fn delete_object_property() {
    let out = run(r#"
        const obj = { a: 1, b: 2, c: 3 };
        console.log(delete obj.b);
        console.log("b" in obj);
        console.log(obj.a);
        console.log(obj.c);
    "#);
    assert_eq!(out, vec!["true", "false", "1", "3"]);
}

#[test]
fn delete_computed_property() {
    let out = run(r#"
        const obj = { x: 10, y: 20 };
        const key = "x";
        console.log(delete obj[key]);
        console.log("x" in obj);
        console.log(obj.y);
    "#);
    assert_eq!(out, vec!["true", "false", "20"]);
}

#[test]
fn delete_nonexistent_property() {
    let out = run(r#"
        const obj = { a: 1 };
        console.log(delete obj.z);
    "#);
    assert_eq!(out, vec!["false"]);
}

// ── Proxy deleteProperty trap ──

#[test]
fn proxy_delete_property_trap() {
    let out = run(r#"
        const target = { a: 1, b: 2 };
        const handler = {
            deleteProperty: (target, prop) => {
                console.log("deleting " + prop);
                delete target[prop];
                return true;
            }
        };
        const p = new Proxy(target, handler);
        console.log(delete p.a);
        console.log("a" in target);
    "#);
    assert_eq!(out, vec!["deleting a", "true", "false"]);
}

#[test]
fn proxy_delete_property_trap_deny() {
    let out = run(r#"
        const target = { secret: 42, normal: 1 };
        const handler = {
            deleteProperty: (target, prop) => {
                if (prop === "secret") {
                    return false;
                }
                delete target[prop];
                return true;
            }
        };
        const p = new Proxy(target, handler);
        console.log(delete p.secret);
        console.log(delete p.normal);
        console.log("secret" in target);
        console.log("normal" in target);
    "#);
    assert_eq!(out, vec!["false", "true", "true", "false"]);
}

#[test]
fn proxy_delete_no_trap_passthrough() {
    let out = run(r#"
        const target = { x: 1, y: 2 };
        const p = new Proxy(target, {});
        console.log(delete p.x);
        console.log("x" in target);
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

// ── Proxy ownKeys trap ──

#[test]
fn proxy_own_keys_trap() {
    let out = run(r#"
        const target = { a: 1, b: 2, c: 3 };
        const handler = {
            ownKeys: (target) => {
                return ["a", "c"];
            }
        };
        const p = new Proxy(target, handler);
        const keys = Object.keys(p);
        console.log(keys.length);
        console.log(keys[0]);
        console.log(keys[1]);
    "#);
    assert_eq!(out, vec!["2", "a", "c"]);
}

#[test]
fn proxy_own_keys_no_trap() {
    let out = run(r#"
        const target = { x: 10, y: 20 };
        const p = new Proxy(target, {});
        const keys = Object.keys(p);
        console.log(keys.length);
    "#);
    assert_eq!(out, vec!["2"]);
}

// ── Proxy getPrototypeOf trap ──

#[test]
fn proxy_get_prototype_of_trap() {
    let out = run(r#"
        const fakeProto = { greet: "hello" };
        const handler = {
            getPrototypeOf: (target) => {
                return fakeProto;
            }
        };
        const p = new Proxy({}, handler);
        const proto = Object.getPrototypeOf(p);
        console.log(proto.greet);
    "#);
    assert_eq!(out, vec!["hello"]);
}

#[test]
fn proxy_get_prototype_of_no_trap() {
    let out = run(r#"
        const target = {};
        const p = new Proxy(target, {});
        const proto = Object.getPrototypeOf(p);
        console.log(proto);
    "#);
    assert_eq!(out, vec!["null"]);
}
