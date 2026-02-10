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
fn for_of_over_array() {
    let out = run(r#"
        const arr = [10, 20, 30];
        for (const x of arr) {
            console.log(x);
        }
    "#);
    assert_eq!(out, vec!["10", "20", "30"]);
}

#[test]
fn for_of_over_string() {
    let out = run(r#"
        for (const ch of "abc") {
            console.log(ch);
        }
    "#);
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn spread_array_uses_iterable() {
    let out = run(r#"
        const a = [1, 2];
        const b = [...a, 3];
        console.log(b.length);
        console.log(b[2]);
    "#);
    assert_eq!(out, vec!["3", "3"]);
}

#[test]
fn spread_string_uses_iterable() {
    let out = run(r#"
        const chars = [..."hello"];
        console.log(chars.length);
        console.log(chars[0]);
    "#);
    assert_eq!(out, vec!["5", "h"]);
}

#[test]
fn custom_iterable_with_symbol_iterator() {
    let out = run(r#"
        const range = { start: 1, end: 4 };
        const iterSym = Symbol.iterator;
        range[iterSym] = () => {
            let current = range.start;
            const end = range.end;
            return {
                next: () => {
                    if (current <= end) {
                        const val = current;
                        current = current + 1;
                        return { value: val, done: false };
                    }
                    return { value: undefined, done: true };
                }
            };
        };
        for (const n of range) {
            console.log(n);
        }
    "#);
    assert_eq!(out, vec!["1", "2", "3", "4"]);
}

#[test]
fn spread_custom_iterable() {
    let out = run(r#"
        const obj = {};
        obj[Symbol.iterator] = () => {
            let i = 0;
            return {
                next: () => {
                    i = i + 1;
                    if (i <= 3) {
                        return { value: i, done: false };
                    }
                    return { done: true };
                }
            };
        };
        const arr = [...obj];
        console.log(arr.length);
        console.log(arr[0]);
        console.log(arr[2]);
    "#);
    assert_eq!(out, vec!["3", "1", "3"]);
}
