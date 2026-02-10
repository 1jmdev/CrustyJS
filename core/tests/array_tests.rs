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
fn array_literal_display() {
    let out = run_and_capture("let arr = [1, 2, 3]; console.log(arr);");
    assert_eq!(out, vec!["[1, 2, 3]"]);
}

#[test]
fn array_index_access() {
    let out = run_and_capture("let arr = [10, 20, 30]; console.log(arr[0]); console.log(arr[2]);");
    assert_eq!(out, vec!["10", "30"]);
}

#[test]
fn array_length() {
    let out = run_and_capture("let arr = [1, 2, 3, 4]; console.log(arr.length);");
    assert_eq!(out, vec!["4"]);
}

#[test]
fn array_out_of_bounds() {
    let out = run_and_capture("let arr = [1, 2]; console.log(arr[5]);");
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn array_index_assignment() {
    let out = run_and_capture("let arr = [1, 2, 3]; arr[1] = 99; console.log(arr[1]);");
    assert_eq!(out, vec!["99"]);
}

#[test]
fn empty_array() {
    let out = run_and_capture("let arr = []; console.log(arr.length); console.log(arr);");
    assert_eq!(out, vec!["0", "[]"]);
}

#[test]
fn array_of_strings() {
    let out = run_and_capture(r#"let arr = ["a", "b", "c"]; console.log(arr);"#);
    assert_eq!(out, vec!["[a, b, c]"]);
}

#[test]
fn array_mixed_types() {
    let out = run_and_capture(r#"let arr = [1, "two", true, null]; console.log(arr);"#);
    assert_eq!(out, vec!["[1, two, true, null]"]);
}

#[test]
fn array_dynamic_index() {
    let src = r#"
        let arr = [10, 20, 30];
        let i = 1;
        console.log(arr[i]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["20"]);
}

#[test]
fn array_grow_via_assignment() {
    let src = r#"
        let arr = [1];
        arr[3] = 99;
        console.log(arr.length);
        console.log(arr[1]);
        console.log(arr[3]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["4", "undefined", "99"]);
}

#[test]
fn array_push() {
    let src = "let arr = [1, 2]; arr.push(3); console.log(arr); console.log(arr.length);";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[1, 2, 3]", "3"]);
}

#[test]
fn array_pop() {
    let src = "let arr = [1, 2, 3]; let last = arr.pop(); console.log(last); console.log(arr);";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["3", "[1, 2]"]);
}

#[test]
fn array_includes() {
    let src = "let arr = [1, 2, 3]; console.log(arr.includes(2)); console.log(arr.includes(5));";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn array_index_of() {
    let src = r#"let arr = ["a", "b", "c"]; console.log(arr.indexOf("b")); console.log(arr.indexOf("z"));"#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["1", "-1"]);
}

#[test]
fn array_join() {
    let src = r#"let arr = [1, 2, 3]; console.log(arr.join("-")); console.log(arr.join());"#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["1-2-3", "1,2,3"]);
}

#[test]
fn array_slice() {
    let src = "let arr = [1, 2, 3, 4, 5]; let s = arr.slice(1, 3); console.log(s);";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[2, 3]"]);
}

#[test]
fn array_concat() {
    let src = "let a = [1, 2]; let b = [3, 4]; let c = a.concat(b); console.log(c);";
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[1, 2, 3, 4]"]);
}

#[test]
fn array_map_with_function() {
    let src = r#"
        function double(x) { return x * 2; }
        let arr = [1, 2, 3];
        let result = arr.map(double);
        console.log(result);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[2, 4, 6]"]);
}

#[test]
fn array_filter_with_function() {
    let src = r#"
        function greaterThanTwo(x) { return x > 2; }
        let arr = [1, 2, 3, 4, 5];
        let result = arr.filter(greaterThanTwo);
        console.log(result);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[3, 4, 5]"]);
}

#[test]
fn array_for_each_with_function() {
    let src = r#"
        function print(x) { console.log(x); }
        let arr = [10, 20, 30];
        arr.forEach(print);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["10", "20", "30"]);
}

#[test]
fn for_loop_basic() {
    let src = r#"
        let sum = 0;
        for (let i = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        console.log(sum);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["10"]);
}

#[test]
fn for_loop_array_iteration() {
    let src = r#"
        let arr = [10, 20, 30];
        let sum = 0;
        for (let i = 0; i < arr.length; i = i + 1) {
            sum = sum + arr[i];
        }
        console.log(sum);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["60"]);
}

#[test]
fn for_of_array() {
    let src = r#"
        let arr = [1, 2, 3];
        let sum = 0;
        for (let item of arr) {
            sum = sum + item;
        }
        console.log(sum);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["6"]);
}

#[test]
fn for_of_with_console_log() {
    let src = r#"
        let arr = ["a", "b", "c"];
        for (let x of arr) {
            console.log(x);
        }
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["a", "b", "c"]);
}
