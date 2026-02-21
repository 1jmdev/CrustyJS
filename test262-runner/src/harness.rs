use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static HARNESS_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

const HOST_HARNESS: &str = r#"(function (global) {
  if (typeof global.$262 !== "object" || global.$262 === null) {
    global.$262 = {};
  }

  var host262 = global.$262;
  host262.global = global;

  if (typeof host262.evalScript !== "function") {
    host262.evalScript = function (source) {
      return global.eval(source);
    };
  }

  if (typeof host262.createRealm !== "function") {
    host262.createRealm = function () {
      return {
        global: global,
        evalScript: host262.evalScript,
      };
    };
  }

  if (typeof host262.detachArrayBuffer !== "function") {
    host262.detachArrayBuffer = function () {};
  }

  if (typeof host262.gc !== "function") {
    host262.gc = function () {};
  }
})(globalThis);

var $262 = globalThis.$262;"#;

fn harness_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test262")
        .join("harness")
}

fn load_harness_files() -> HashMap<String, String> {
    let dir = harness_dir();
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "js") {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        map.insert(name, content);
                    }
                }
            }
        }
    }
    map
}

pub fn get_harness_cache() -> &'static HashMap<String, String> {
    HARNESS_CACHE.get_or_init(load_harness_files)
}

pub fn host_harness_source() -> &'static str {
    HOST_HARNESS
}

pub fn compose_source(includes: &[String], test_source: &str) -> String {
    let cache = get_harness_cache();
    let mut parts: Vec<&str> = Vec::new();

    parts.push(host_harness_source());

    if let Some(sta) = cache.get("sta.js") {
        parts.push(sta);
    }
    if let Some(assert_js) = cache.get("assert.js") {
        parts.push(assert_js);
    }

    for include in includes {
        if include == "sta.js" || include == "assert.js" {
            continue;
        }
        if let Some(content) = cache.get(include.as_str()) {
            parts.push(content);
        }
    }

    parts.push(test_source);
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_cache_loads() {
        let cache = get_harness_cache();
        assert!(cache.contains_key("assert.js"));
        assert!(cache.contains_key("sta.js"));
    }

    #[test]
    fn test_compose_source_includes_harness() {
        let source = compose_source(&[], "var x = 1;");
        assert!(source.contains("Test262Error"));
        assert!(source.contains("assert"));
        assert!(source.contains("$262"));
        assert!(source.contains("var x = 1;"));
    }

    #[test]
    fn test_compose_source_with_extra_include() {
        let source = compose_source(
            &["propertyHelper.js".to_string()],
            "verifyProperty({}, 'x', {});",
        );
        assert!(source.contains("verifyProperty"));
    }
}
