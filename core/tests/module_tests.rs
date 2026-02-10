use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;
use std::fs;

fn run_file(path: &std::path::Path) -> Vec<String> {
    let source = fs::read_to_string(path).expect("read source");
    let tokens = lex(&source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp
        .run_with_path(&program, path.to_path_buf())
        .expect("execution should succeed");
    interp.output().to_vec()
}

fn run_file_result(path: &std::path::Path) -> Result<Vec<String>, crustyjs::errors::RuntimeError> {
    let source = fs::read_to_string(path).expect("read source");
    let tokens = lex(&source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp.run_with_path(&program, path.to_path_buf())?;
    Ok(interp.output().to_vec())
}

#[test]
fn import_named_function_from_module() {
    let dir = std::env::temp_dir().join(format!("crustyjs_mod_{}_a", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");

    let users = dir.join("users.js");
    let main = dir.join("main.js");

    fs::write(
        &users,
        r#"export function fetchUser(id) { return { name: "Alice Doe", age: 30, id: id }; }"#,
    )
    .expect("write users");
    fs::write(
        &main,
        r#"
import { fetchUser } from "./users.js";
const u = fetchUser(1);
console.log(u.name);
"#,
    )
    .expect("write main");

    let out = run_file(&main);
    assert_eq!(out, vec!["Alice Doe"]);
}

#[test]
fn import_default_export() {
    let dir = std::env::temp_dir().join(format!("crustyjs_mod_{}_b", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");

    let util = dir.join("util.js");
    let main = dir.join("main.js");

    fs::write(&util, r#"export default 7;"#).expect("write util");
    fs::write(
        &main,
        r#"
import value from "./util.js";
console.log(value);
"#,
    )
    .expect("write main");

    let out = run_file(&main);
    assert_eq!(out, vec!["7"]);
}

#[test]
fn import_default_exported_function() {
    let dir = std::env::temp_dir().join(format!("crustyjs_mod_{}_d", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");

    let util = dir.join("util.js");
    let main = dir.join("main.js");

    fs::write(&util, r#"export default function answer() { return 42; }"#).expect("write util");
    fs::write(
        &main,
        r#"
import answer from "./util.js";
console.log(answer());
"#,
    )
    .expect("write main");

    let out = run_file(&main);
    assert_eq!(out, vec!["42"]);
}

#[test]
fn circular_import_is_reported() {
    let dir = std::env::temp_dir().join(format!("crustyjs_mod_{}_c", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");

    let a = dir.join("a.js");
    let b = dir.join("b.js");
    let main = dir.join("main.js");

    fs::write(&a, "import { b } from './b.js'; export const a = 1;").expect("write a");
    fs::write(&b, "import { a } from './a.js'; export const b = 2;").expect("write b");
    fs::write(&main, "import { a } from './a.js'; console.log(a);").expect("write main");

    let err = run_file_result(&main).expect_err("expected circular import error");
    assert!(err.to_string().contains("circular import detected"));
}

#[test]
fn module_parse_error_reports_file_line_and_column() {
    let dir = std::env::temp_dir().join(format!("crustyjs_mod_{}_e", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");

    let bad = dir.join("bad.js");
    let main = dir.join("main.js");

    fs::write(&bad, "export function (").expect("write bad module");
    fs::write(&main, "import { x } from './bad.js';").expect("write main");

    let err = run_file_result(&main).expect_err("expected parse error");
    let msg = err.to_string();
    assert!(msg.contains("failed to parse module"));
    assert!(msg.contains(":1:"));
}
