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
fn regex_literal_test() {
    let out = run(r#"
        const re = /hello/;
        console.log(re.test("hello world"));
        console.log(re.test("goodbye"));
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn regex_literal_case_insensitive() {
    let out = run(r#"
        const re = /hello/i;
        console.log(re.test("HELLO"));
        console.log(re.test("Hello World"));
    "#);
    assert_eq!(out, vec!["true", "true"]);
}

#[test]
fn regex_exec_returns_array() {
    let out = run(r#"
        const re = /(\d+)-(\d+)/;
        const result = re.exec("date: 2024-01");
        console.log(result[0]);
        console.log(result[1]);
        console.log(result[2]);
    "#);
    assert_eq!(out, vec!["2024-01", "2024", "01"]);
}

#[test]
fn regex_exec_no_match_returns_null() {
    let out = run(r#"
        const re = /xyz/;
        const result = re.exec("hello");
        console.log(result);
    "#);
    assert_eq!(out, vec!["null"]);
}

#[test]
fn regex_source_and_flags_properties() {
    let out = run(r#"
        const re = /abc/gi;
        console.log(re.source);
        console.log(re.flags);
        console.log(re.global);
        console.log(re.ignoreCase);
        console.log(re.multiline);
    "#);
    assert_eq!(out, vec!["abc", "gi", "true", "true", "false"]);
}

#[test]
fn regex_to_string() {
    let out = run(r#"
        const re = /test/gi;
        console.log(re.toString());
    "#);
    assert_eq!(out, vec!["/test/gi"]);
}

#[test]
fn regex_constructor_from_string() {
    let out = run(r#"
        const re = new RegExp("hello", "i");
        console.log(re.test("HELLO"));
        console.log(re.source);
        console.log(re.flags);
    "#);
    assert_eq!(out, vec!["true", "hello", "i"]);
}

#[test]
fn regex_constructor_from_regexp() {
    let out = run(r#"
        const re1 = /hello/g;
        const re2 = new RegExp(re1);
        console.log(re2.source);
        console.log(re2.flags);
    "#);
    assert_eq!(out, vec!["hello", "g"]);
}

#[test]
fn regex_constructor_no_flags() {
    let out = run(r#"
        const re = new RegExp("\\d+");
        console.log(re.test("123"));
        console.log(re.test("abc"));
    "#);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn regex_global_exec_advances_lastindex() {
    let out = run(r#"
        const re = /\d+/g;
        const s = "a1b22c333";
        const m1 = re.exec(s);
        console.log(m1[0]);
        console.log(re.lastIndex);
        const m2 = re.exec(s);
        console.log(m2[0]);
        const m3 = re.exec(s);
        console.log(m3[0]);
        const m4 = re.exec(s);
        console.log(m4);
    "#);
    assert_eq!(out, vec!["1", "2", "22", "333", "null"]);
}

#[test]
fn string_match_with_regex() {
    let out = run(r#"
        const result = "hello world".match(/o/g);
        console.log(result.length);
        console.log(result[0]);
        console.log(result[1]);
    "#);
    assert_eq!(out, vec!["2", "o", "o"]);
}

#[test]
fn string_search_with_regex() {
    let out = run(r#"
        console.log("hello world".search(/world/));
        console.log("hello world".search(/xyz/));
    "#);
    assert_eq!(out, vec!["6", "-1"]);
}

#[test]
fn string_replace_with_regex() {
    let out = run(r#"
        console.log("hello world".replace(/world/, "rust"));
    "#);
    assert_eq!(out, vec!["hello rust"]);
}

#[test]
fn string_replace_with_regex_global() {
    let out = run(r#"
        console.log("aabbcc".replace(/b/g, "X"));
    "#);
    assert_eq!(out, vec!["aaXXcc"]);
}

#[test]
fn string_split_with_regex() {
    let out = run(r#"
        const parts = "one1two2three".split(/\d/);
        console.log(parts.length);
        console.log(parts[0]);
        console.log(parts[1]);
        console.log(parts[2]);
    "#);
    assert_eq!(out, vec!["3", "one", "two", "three"]);
}

#[test]
fn regex_typeof() {
    let out = run(r#"
        console.log(typeof /abc/);
    "#);
    assert_eq!(out, vec!["object"]);
}

#[test]
fn regex_multiline_flag() {
    let out = run(r#"
        const re = /^hello/m;
        console.log(re.test("world\nhello"));
    "#);
    assert_eq!(out, vec!["true"]);
}

#[test]
fn regex_dotall_flag() {
    let out = run(r#"
        const re = /hello.world/s;
        console.log(re.test("hello\nworld"));
    "#);
    assert_eq!(out, vec!["true"]);
}

#[test]
fn regex_division_disambiguation() {
    let out = run(r#"
        const a = 10;
        const b = 2;
        const c = a / b;
        console.log(c);
        const re = /test/;
        console.log(re.test("test"));
    "#);
    assert_eq!(out, vec!["5", "true"]);
}

#[test]
fn regex_invalid_flags_error() {
    let err = run_err(
        r#"
        const re = new RegExp("abc", "z");
    "#,
    );
    assert!(err.contains("invalid regex flag"));
}
