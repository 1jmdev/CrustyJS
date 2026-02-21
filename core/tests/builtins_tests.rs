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
fn object_extended_statics() {
    let output = run_and_capture(
        r#"
        const fromObj = Object.fromEntries([["x", 1], ["y", 2]]);
        console.log(fromObj.x + fromObj.y);

        const obj = { a: 1 };
        Object.defineProperty(obj, "b", { value: 3 });
        console.log(Object.hasOwn(obj, "b"));
        console.log(Object.getOwnPropertyNames(obj).length);
        console.log(Object.getOwnPropertyDescriptor(obj, "b").value);

        const proto = { p: 9 };
        const child = {};
        Object.setPrototypeOf(child, proto);
        console.log(Object.getPrototypeOf(child) === proto);

        console.log(Object.is(NaN, NaN));
        console.log(Object.is(0, -0));
        "#,
    );

    assert_eq!(output, vec!["3", "true", "2", "3", "true", "true", "false"]);
}

#[test]
fn object_prototype_methods_work() {
    let output = run_and_capture(
        r#"
        const base = { a: 1 };
        const child = Object.create(base);
        child.b = 2;

        console.log(base.isPrototypeOf(child));
        console.log(child.hasOwnProperty("b"));
        console.log(child.hasOwnProperty("a"));

        Object.defineProperty(child, "hidden", { value: 9, enumerable: false });
        console.log(child.propertyIsEnumerable("b"));
        console.log(child.propertyIsEnumerable("hidden"));

        console.log(child.toLocaleString());
        console.log(child.valueOf() === child);
        console.log(child.toString());
        "#,
    );

    assert_eq!(
        output,
        vec![
            "true",
            "true",
            "false",
            "true",
            "false",
            "[object Object]",
            "true",
            "[object Object]"
        ]
    );
}

#[test]
fn object_integrity_apis_work() {
    let output = run_and_capture(
        r#"
        const o1 = { x: 1 };
        console.log(Object.isExtensible(o1));
        Object.preventExtensions(o1);
        o1.y = 2;
        console.log(Object.isExtensible(o1));
        console.log(o1.y);

        const o2 = { a: 1 };
        Object.seal(o2);
        console.log(Object.isSealed(o2));
        delete o2.a;
        console.log(o2.a);

        const o3 = { a: 1 };
        Object.freeze(o3);
        o3.a = 9;
        console.log(Object.isFrozen(o3));
        console.log(o3.a);
        "#,
    );

    assert_eq!(
        output,
        vec!["true", "false", "undefined", "true", "1", "true", "1"]
    );
}

#[test]
fn object_descriptor_apis_work() {
    let output = run_and_capture(
        r#"
        const obj = {};
        Object.defineProperties(obj, {
            a: { value: 1, enumerable: true },
            b: { value: 2, enumerable: false }
        });

        console.log(Object.keys(obj).length);
        console.log(Object.getOwnPropertyNames(obj).length);
        console.log(Object.getOwnPropertyDescriptor(obj, "b").enumerable);
        console.log(Object.getOwnPropertyDescriptors(obj).a.value);

        const arr = [10, 20];
        console.log(Object.getOwnPropertyNames(arr).length >= 3);
        console.log(Object.getOwnPropertyDescriptor(arr, "length").writable);
        "#,
    );

    assert_eq!(output, vec!["1", "2", "false", "1", "true", "true"]);
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

#[test]
fn queue_microtask_builtin_works() {
    let output = run_and_capture(
        r#"
        let x = 0;
        queueMicrotask(() => { x = 1; console.log(x); });
        "#,
    );

    assert_eq!(output, vec!["1"]);
}

#[test]
fn console_alias_methods_and_performance_now() {
    let output = run_and_capture(
        r#"
        console.info("i");
        console.warn("w");
        console.error("e");
        console.debug("d");
        const t1 = performance.now();
        const t2 = performance.now();
        console.log(t2 >= t1);
        "#,
    );

    assert_eq!(output[0], "i");
    assert_eq!(output[1], "w");
    assert_eq!(output[2], "e");
    assert_eq!(output[3], "d");
    assert_eq!(output[4], "true");
}
