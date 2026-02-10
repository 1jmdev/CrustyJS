use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("execution should succeed");
    interp.output().to_vec()
}

#[test]
fn math_constants() {
    let output = run_and_capture(
        r#"
        console.log(Math.PI);
        console.log(Math.E);
        "#,
    );

    assert_eq!(output.len(), 2);
    assert!(output[0].starts_with("3.14159"));
    assert!(output[1].starts_with("2.71828"));
}

#[test]
fn math_methods_basic() {
    let output = run_and_capture(
        r#"
        console.log(Math.floor(4.7));
        console.log(Math.max(1, 5, 3));
        console.log(Math.abs(-5));
        console.log(Math.sqrt(16));
        console.log(Math.pow(2, 10));
        console.log(Math.round(4.5));
        "#,
    );

    assert_eq!(output, vec!["4", "5", "5", "4", "1024", "5"]);
}

#[test]
fn json_stringify_and_parse() {
    let output = run_and_capture(
        r#"
        const str = JSON.stringify({ a: 1, b: [2, 3] });
        console.log(str);
        const obj = JSON.parse(str);
        console.log(obj.a);
        console.log(obj.b[1]);
        "#,
    );

    assert_eq!(output[0], "{\"a\":1.0,\"b\":[2.0,3.0]}");
    assert_eq!(output[1], "1");
    assert_eq!(output[2], "3");
}

#[test]
fn object_statics_and_date_now() {
    let output = run_and_capture(
        r#"
        const obj = { a: 1, b: 2 };
        console.log(Object.keys(obj).length);
        console.log(Object.values(obj)[0]);
        console.log(Object.entries(obj)[1][0]);
        const merged = Object.assign({}, obj, { c: 3 });
        console.log(merged.c);
        console.log(Date.now() > 0);
        "#,
    );

    assert_eq!(output[0], "2");
    assert!(output[1] == "1" || output[1] == "2");
    assert!(output[2] == "a" || output[2] == "b");
    assert_eq!(output[3], "3");
    assert_eq!(output[4], "true");
}

#[test]
fn array_reduce_and_sort() {
    let output = run_and_capture(
        r#"
        const nums = [3, 1, 4, 1, 5];
        const sum = nums.reduce((acc, n) => acc + n, 0);
        const sorted = [...nums].sort((a, b) => a - b);
        console.log(sum);
        console.log(sorted[0]);
        console.log(sorted[4]);
        "#,
    );

    assert_eq!(output, vec!["14", "1", "5"]);
}
