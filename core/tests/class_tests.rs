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
fn class_inheritance_and_super_constructor() {
    let src = r#"
        class Animal {
          constructor(name) {
            this.name = name;
          }
          speak() {
            return this.name + " makes a noise";
          }
        }

        class Dog extends Animal {
          constructor(name, breed) {
            super(name);
            this.breed = breed;
          }
          speak() {
            return this.name + " barks";
          }
        }

        const dogs = [new Dog("Rex", "Shepherd"), new Dog("Bella", "Lab")];
        const names = dogs.map(d => d.name);
        console.log(names);
        console.log(dogs[0].speak());
        console.log(typeof dogs[0]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["[Rex, Bella]", "Rex barks", "object"]);
}

#[test]
fn inherited_method_resolves_from_parent_prototype() {
    let src = r#"
        class Animal {
          constructor(name) {
            this.name = name;
          }
          speak() {
            return this.name + " makes a noise";
          }
        }
        class Cat extends Animal {
          constructor(name) {
            super(name);
          }
        }
        const c = new Cat("Milo");
        console.log(c.speak());
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["Milo makes a noise"]);
}

#[test]
fn instanceof_checks_prototype_chain() {
    let src = r#"
        class Animal {
          constructor(name) {
            this.name = name;
          }
        }
        class Dog extends Animal {
          constructor(name) {
            super(name);
          }
        }
        const d = new Dog("Rex");
        console.log(d instanceof Dog);
        console.log(d instanceof Animal);
        console.log(d instanceof Error);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["true", "true", "false"]);
}

#[test]
fn class_getter_and_setter_accessors_work() {
    let src = r#"
        class Counter {
          constructor() {
            this._count = 0;
          }
          get count() {
            return this._count;
          }
          set count(v) {
            this._count = v;
          }
        }
        const c = new Counter();
        console.log(c.count);
        c.count = 5;
        console.log(c.count);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["0", "5"]);
}
