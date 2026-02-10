use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SYMBOL_ID: AtomicU64 = AtomicU64::new(1);

/// A JS Symbol value — unique, immutable identifier.
#[derive(Debug, Clone)]
pub struct JsSymbol {
    pub id: u64,
    pub description: Option<String>,
}

impl JsSymbol {
    pub fn new(description: Option<String>) -> Self {
        Self {
            id: NEXT_SYMBOL_ID.fetch_add(1, Ordering::Relaxed),
            description,
        }
    }
}

impl PartialEq for JsSymbol {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for JsSymbol {}

impl std::hash::Hash for JsSymbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Display for JsSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.description {
            Some(desc) => write!(f, "Symbol({desc})"),
            None => write!(f, "Symbol()"),
        }
    }
}

/// Well-known symbol IDs — these are reserved and always the same.
/// We use high IDs to avoid collisions with user-created symbols.
pub mod well_known {
    pub const ITERATOR: u64 = u64::MAX;
    pub const TO_PRIMITIVE: u64 = u64::MAX - 1;
    pub const HAS_INSTANCE: u64 = u64::MAX - 2;
    pub const TO_STRING_TAG: u64 = u64::MAX - 3;
}

/// Get the well-known Symbol.iterator.
pub fn symbol_iterator() -> JsSymbol {
    JsSymbol {
        id: well_known::ITERATOR,
        description: Some("Symbol.iterator".to_string()),
    }
}

/// Get the well-known Symbol.toPrimitive.
pub fn symbol_to_primitive() -> JsSymbol {
    JsSymbol {
        id: well_known::TO_PRIMITIVE,
        description: Some("Symbol.toPrimitive".to_string()),
    }
}

/// Get the well-known Symbol.hasInstance.
pub fn symbol_has_instance() -> JsSymbol {
    JsSymbol {
        id: well_known::HAS_INSTANCE,
        description: Some("Symbol.hasInstance".to_string()),
    }
}

/// Get the well-known Symbol.toStringTag.
pub fn symbol_to_string_tag() -> JsSymbol {
    JsSymbol {
        id: well_known::TO_STRING_TAG,
        description: Some("Symbol.toStringTag".to_string()),
    }
}

/// Global symbol registry for Symbol.for() / Symbol.keyFor().
#[derive(Debug, Default)]
pub struct SymbolRegistry {
    by_key: HashMap<String, JsSymbol>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Symbol.for(key) — returns existing or creates new.
    pub fn for_key(&mut self, key: String) -> JsSymbol {
        if let Some(sym) = self.by_key.get(&key) {
            return sym.clone();
        }
        let sym = JsSymbol::new(Some(key.clone()));
        self.by_key.insert(key, sym.clone());
        sym
    }

    /// Symbol.keyFor(symbol) — reverse lookup.
    pub fn key_for(&self, symbol: &JsSymbol) -> Option<String> {
        self.by_key
            .iter()
            .find(|(_, v)| v.id == symbol.id)
            .map(|(k, _)| k.clone())
    }
}
