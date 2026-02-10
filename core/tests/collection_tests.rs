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
fn map_empty_constructor() {
    let out = run(r#"
        const m = new Map();
        console.log(m.size);
    "#);
    assert_eq!(out, vec!["0"]);
}

#[test]
fn map_from_iterable() {
    let out = run(r#"
        const m = new Map([["a", 1], ["b", 2]]);
        console.log(m.size);
        console.log(m.get("a"));
        console.log(m.get("b"));
    "#);
    assert_eq!(out, vec!["2", "1", "2"]);
}

#[test]
fn map_set_get_has_delete() {
    let out = run(r#"
        const m = new Map();
        m.set("x", 10);
        m.set("y", 20);
        console.log(m.has("x"));
        console.log(m.get("x"));
        console.log(m.delete("x"));
        console.log(m.has("x"));
        console.log(m.size);
    "#);
    assert_eq!(out, vec!["true", "10", "true", "false", "1"]);
}

#[test]
fn map_clear() {
    let out = run(r#"
        const m = new Map([["a", 1], ["b", 2]]);
        m.clear();
        console.log(m.size);
    "#);
    assert_eq!(out, vec!["0"]);
}

#[test]
fn map_overwrite_key() {
    let out = run(r#"
        const m = new Map();
        m.set("a", 1);
        m.set("a", 99);
        console.log(m.get("a"));
        console.log(m.size);
    "#);
    assert_eq!(out, vec!["99", "1"]);
}

#[test]
fn map_nan_key() {
    let out = run(r#"
        const nan = 0 / 0;
        const m = new Map();
        m.set(nan, "found");
        console.log(m.has(nan));
        console.log(m.get(nan));
    "#);
    assert_eq!(out, vec!["true", "found"]);
}

#[test]
fn map_set_returns_map_for_chaining() {
    let out = run(r#"
        const m = new Map();
        m.set("a", 1).set("b", 2).set("c", 3);
        console.log(m.size);
    "#);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn map_get_missing_key_returns_undefined() {
    let out = run(r#"
        const m = new Map();
        console.log(m.get("nope"));
    "#);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn map_foreach() {
    let out = run(r#"
        const m = new Map([["a", 1], ["b", 2]]);
        m.forEach((value, key) => {
            console.log(key + "=" + value);
        });
    "#);
    assert_eq!(out, vec!["a=1", "b=2"]);
}

#[test]
fn map_keys_values_entries() {
    let out = run(r#"
        const m = new Map([["x", 10], ["y", 20]]);
        const keys = [];
        for (const k of m.keys()) { keys.push(k); }
        console.log(keys.join(","));
        const vals = [];
        for (const v of m.values()) { vals.push(v); }
        console.log(vals.join(","));
        const entries = [];
        for (const e of m.entries()) { entries.push(e[0] + ":" + e[1]); }
        console.log(entries.join(","));
    "#);
    assert_eq!(out, vec!["x,y", "10,20", "x:10,y:20"]);
}

#[test]
fn map_for_of_yields_entries() {
    let out = run(r#"
        const m = new Map([["a", 1], ["b", 2]]);
        for (const entry of m) {
            console.log(entry[0] + ":" + entry[1]);
        }
    "#);
    assert_eq!(out, vec!["a:1", "b:2"]);
}

#[test]
fn map_spread_to_array() {
    let out = run(r#"
        const m = new Map([["a", 1], ["b", 2]]);
        const arr = [...m];
        console.log(arr.length);
        console.log(arr[0][0]);
        console.log(arr[1][1]);
    "#);
    assert_eq!(out, vec!["2", "a", "2"]);
}

#[test]
fn set_empty_constructor() {
    let out = run(r#"
        const s = new Set();
        console.log(s.size);
    "#);
    assert_eq!(out, vec!["0"]);
}

#[test]
fn set_from_iterable_deduplicates() {
    let out = run(r#"
        const s = new Set([1, 2, 3, 2, 1]);
        console.log(s.size);
    "#);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn set_add_has_delete() {
    let out = run(r#"
        const s = new Set();
        s.add(10);
        s.add(20);
        console.log(s.has(10));
        console.log(s.delete(10));
        console.log(s.has(10));
        console.log(s.size);
    "#);
    assert_eq!(out, vec!["true", "true", "false", "1"]);
}

#[test]
fn set_clear() {
    let out = run(r#"
        const s = new Set([1, 2, 3]);
        s.clear();
        console.log(s.size);
    "#);
    assert_eq!(out, vec!["0"]);
}

#[test]
fn set_add_returns_set_for_chaining() {
    let out = run(r#"
        const s = new Set();
        s.add(1).add(2).add(3);
        console.log(s.size);
    "#);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn set_foreach() {
    let out = run(r#"
        const s = new Set([10, 20, 30]);
        const result = [];
        s.forEach((value) => {
            result.push(value);
        });
        console.log(result.join(","));
    "#);
    assert_eq!(out, vec!["10,20,30"]);
}

#[test]
fn set_values_iterator() {
    let out = run(r#"
        const s = new Set(["a", "b", "c"]);
        const vals = [];
        for (const v of s.values()) { vals.push(v); }
        console.log(vals.join(","));
    "#);
    assert_eq!(out, vec!["a,b,c"]);
}

#[test]
fn set_entries_yields_pairs() {
    let out = run(r#"
        const s = new Set([1, 2]);
        for (const e of s.entries()) {
            console.log(e[0] + ":" + e[1]);
        }
    "#);
    assert_eq!(out, vec!["1:1", "2:2"]);
}

#[test]
fn set_for_of() {
    let out = run(r#"
        const s = new Set([10, 20, 30]);
        const result = [];
        for (const v of s) {
            result.push(v);
        }
        console.log(result.join(","));
    "#);
    assert_eq!(out, vec!["10,20,30"]);
}

#[test]
fn set_spread_to_array() {
    let out = run(r#"
        const s = new Set([3, 1, 2]);
        const arr = [...s];
        console.log(arr.join(","));
    "#);
    assert_eq!(out, vec!["3,1,2"]);
}

#[test]
fn map_typeof_is_object() {
    let out = run(r#"
        console.log(typeof new Map());
        console.log(typeof new Set());
    "#);
    assert_eq!(out, vec!["object", "object"]);
}
