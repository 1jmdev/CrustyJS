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
