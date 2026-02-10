use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Negative {
    pub phase: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestMetadata {
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub negative: Option<Negative>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl TestMetadata {
    pub fn is_module(&self) -> bool {
        self.flags.iter().any(|f| f == "module")
    }

    pub fn is_only_strict(&self) -> bool {
        self.flags.iter().any(|f| f == "onlyStrict")
    }

    pub fn is_no_strict(&self) -> bool {
        self.flags.iter().any(|f| f == "noStrict")
    }

    pub fn is_raw(&self) -> bool {
        self.flags.iter().any(|f| f == "raw")
    }

    pub fn is_async(&self) -> bool {
        self.flags.iter().any(|f| f == "async")
    }
}

pub fn parse_frontmatter(source: &str) -> Option<TestMetadata> {
    let start_marker = "/*---";
    let end_marker = "---*/";

    let start = source.find(start_marker)?;
    let yaml_start = start + start_marker.len();
    let end = source[yaml_start..].find(end_marker)?;
    let yaml_str = &source[yaml_start..yaml_start + end];

    serde_yaml::from_str(yaml_str).ok()
}

#[cfg(test)]
mod tests {
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
}
