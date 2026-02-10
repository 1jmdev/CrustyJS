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
