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
fn object_literal_dot_access() {
    let out = run_and_capture(r#"let obj = { x: 1, y: 2 }; console.log(obj.x + obj.y);"#);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn object_bracket_access() {
    let out = run_and_capture(r#"let obj = { name: "Rex" }; console.log(obj["name"]);"#);
    assert_eq!(out, vec!["Rex"]);
}

#[test]
fn object_dot_assignment() {
    let out = run_and_capture(r#"let obj = { x: 1 }; obj.x = 42; console.log(obj.x);"#);
    assert_eq!(out, vec!["42"]);
}

#[test]
fn object_bracket_assignment() {
    let out = run_and_capture(r#"let obj = { x: 1 }; obj["x"] = 99; console.log(obj["x"]);"#);
    assert_eq!(out, vec!["99"]);
}

#[test]
fn object_add_new_property() {
    let out = run_and_capture(r#"let obj = {}; obj.name = "Bella"; console.log(obj.name);"#);
    assert_eq!(out, vec!["Bella"]);
}

#[test]
fn object_missing_property_is_undefined() {
    let out = run_and_capture(r#"let obj = { x: 1 }; console.log(obj.y);"#);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn object_multiple_properties() {
    let src = r#"
        let person = { name: "Alice", age: 30 };
        console.log(person.name);
        console.log(person.age);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["Alice", "30"]);
}

#[test]
fn object_nested_access() {
    let src = r#"
        let a = { val: 10 };
        let b = { val: 20 };
        console.log(a.val + b.val);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["30"]);
}

#[test]
fn object_dynamic_bracket_key() {
    let src = r#"
        let obj = { x: 1, y: 2 };
        let key = "y";
        console.log(obj[key]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["2"]);
}

#[test]
fn object_bracket_assign_new_key() {
    let src = r#"
        let obj = {};
        let key = "color";
        obj[key] = "blue";
        console.log(obj.color);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["blue"]);
}

#[test]
fn prototype_chain_lookup_via_object_create() {
    let src = r#"
        let proto = { greet: "hello" };
        let child = Object.create(proto);
        console.log(child.greet);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["hello"]);
}

#[test]
fn prototype_shadowing_prefers_own_property() {
    let src = r#"
        let proto = { name: "proto" };
        let child = Object.create(proto);
        child.name = "own";
        console.log(child.name);
        console.log(proto.name);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["own", "proto"]);
}

#[test]
fn method_call_binds_this_to_receiver() {
    let src = r#"
        let obj = {
            name: "Rex",
            speak: () => this.name + " barks"
        };
        console.log(obj.speak());
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["Rex barks"]);
}

#[test]
fn plain_function_has_undefined_this() {
    let src = r#"
        function showThisType() { return typeof this; }
        console.log(showThisType());
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn detached_method_loses_receiver_this() {
    let src = r#"
        let obj = {
            name: "Rex",
            speak: () => typeof this
        };
        let fnc = obj.speak;
        console.log(fnc());
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn object_method_shorthand_binds_this() {
    let src = r#"
        let obj = {
            name: "Rex",
            speak() {
                return this.name + " barks";
            }
        };
        console.log(obj.speak());
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["Rex barks"]);
}
