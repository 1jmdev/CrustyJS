use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("runtime error");
    interp.output().to_vec()
}

#[test]
fn arrow_single_param_expression_body() {
    let src = "const square = x => x * x; console.log(square(5));";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["25"]);
}

#[test]
fn arrow_multi_param_expression_body() {
    let src = "const add = (a, b) => a + b; console.log(add(3, 4));";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["7"]);
}

#[test]
fn arrow_zero_param_block_body() {
    let src = r#"
        const greet = () => { console.log("hi"); };
        greet();
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["hi"]);
}

#[test]
fn arrow_callback_with_map() {
    let src = r#"
        const nums = [1, 2, 3];
        const doubled = nums.map(x => x * 2);
        console.log(doubled);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[2, 4, 6]"]);
}

#[test]
fn arrow_callback_with_filter() {
    let src = r#"
        const nums = [1, 2, 3, 4];
        const out = nums.filter(x => x > 2);
        console.log(out);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[3, 4]"]);
}

#[test]
fn arrow_as_for_each_callback() {
    let src = r#"
        const nums = [10, 20, 30];
        nums.forEach(x => console.log(x));
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["10", "20", "30"]);
}
