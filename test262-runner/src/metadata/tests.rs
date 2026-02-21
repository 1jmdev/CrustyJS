use super::*;

#[test]
fn test_basic_frontmatter() {
    let source = r#"// Copyright
/*---
includes: [propertyHelper.js]
flags: [onlyStrict]
---*/
var x = 1;
"#;
    let meta = parse_frontmatter(source).unwrap();
    assert_eq!(meta.includes, vec!["propertyHelper.js"]);
    assert!(meta.is_only_strict());
    assert!(meta.negative.is_none());
}

#[test]
fn test_negative_frontmatter() {
    let source = r#"/*---
negative:
  phase: parse
  type: SyntaxError
flags:
  - module
---*/
export var a, a;
"#;
    let meta = parse_frontmatter(source).unwrap();
    let neg = meta.negative.as_ref().unwrap();
    assert_eq!(neg.phase, "parse");
    assert_eq!(neg.error_type, "SyntaxError");
    assert!(meta.is_module());
}

#[test]
fn test_no_frontmatter() {
    let source = "var x = 1;";
    assert!(parse_frontmatter(source).is_none());
}
