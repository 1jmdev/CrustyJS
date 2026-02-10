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

#[test]
fn basic_generator_with_next() {
    let out = run(r#"
        function* nums() {
            yield 1;
            yield 2;
            yield 3;
        }
        const g = nums();
        console.log(g.next().value);
        console.log(g.next().value);
        console.log(g.next().value);
        console.log(g.next().done);
    "#);
    assert_eq!(out, vec!["1", "2", "3", "true"]);
}

#[test]
fn generator_done_flag() {
    let out = run(r#"
        function* one() {
            yield 42;
        }
        const g = one();
        const first = g.next();
        console.log(first.value);
        console.log(first.done);
        const second = g.next();
        console.log(second.done);
    "#);
    assert_eq!(out, vec!["42", "false", "true"]);
}

#[test]
fn generator_return_value() {
    let out = run(r#"
        function* withReturn() {
            yield 1;
            return 99;
        }
        const g = withReturn();
        console.log(g.next().value);
        const last = g.next();
        console.log(last.value);
        console.log(last.done);
    "#);
    assert_eq!(out, vec!["1", "99", "true"]);
}

#[test]
fn generator_no_yields() {
    let out = run(r#"
        function* empty() {
            return 5;
        }
        const g = empty();
        const r = g.next();
        console.log(r.value);
        console.log(r.done);
    "#);
    assert_eq!(out, vec!["5", "true"]);
}

#[test]
fn generator_for_of() {
    let out = run(r#"
        function* range() {
            yield 10;
            yield 20;
            yield 30;
        }
        for (const x of range()) {
            console.log(x);
        }
    "#);
    assert_eq!(out, vec!["10", "20", "30"]);
}

#[test]
fn generator_spread() {
    let out = run(r#"
        function* abc() {
            yield "a";
            yield "b";
            yield "c";
        }
        const arr = [...abc()];
        console.log(arr.length);
        console.log(arr[0]);
        console.log(arr[2]);
    "#);
    assert_eq!(out, vec!["3", "a", "c"]);
}

#[test]
fn generator_return_method() {
    let out = run(r#"
        function* nums() {
            yield 1;
            yield 2;
            yield 3;
        }
        const g = nums();
        console.log(g.next().value);
        const r = g.return(42);
        console.log(r.value);
        console.log(r.done);
        console.log(g.next().done);
    "#);
    assert_eq!(out, vec!["1", "42", "true", "true"]);
}

#[test]
fn generator_with_loop() {
    let out = run(r#"
        function* count() {
            let i = 0;
            while (i < 3) {
                yield i;
                i = i + 1;
            }
        }
        const arr = [...count()];
        console.log(arr.length);
        console.log(arr[0]);
        console.log(arr[1]);
        console.log(arr[2]);
    "#);
    assert_eq!(out, vec!["3", "0", "1", "2"]);
}

#[test]
fn generator_with_conditional() {
    let out = run(r#"
        function* evens(n) {
            let i = 0;
            while (i < n) {
                if (i % 2 === 0) {
                    yield i;
                }
                i = i + 1;
            }
        }
        const arr = [...evens(6)];
        console.log(arr.length);
        console.log(arr[0]);
        console.log(arr[1]);
        console.log(arr[2]);
    "#);
    assert_eq!(out, vec!["3", "0", "2", "4"]);
}

#[test]
fn multiple_generator_instances_are_independent() {
    let out = run(r#"
        function* counter() {
            yield 1;
            yield 2;
        }
        const a = counter();
        const b = counter();
        console.log(a.next().value);
        console.log(b.next().value);
        console.log(a.next().value);
        console.log(b.next().value);
    "#);
    assert_eq!(out, vec!["1", "1", "2", "2"]);
}

#[test]
fn generator_for_of_with_break() {
    let out = run(r#"
        function* five() {
            yield 10;
            yield 20;
            yield 30;
            yield 40;
            yield 50;
        }
        for (const x of five()) {
            if (x === 30) {
                break;
            }
            console.log(x);
        }
    "#);
    assert_eq!(out, vec!["10", "20"]);
}
