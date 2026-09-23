use crate::core::error::{EngineError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// FactContext wraps arbitrary JSON-compatible fact data and provides path-based queries and mutations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactContext {
    data: Value,
    #[serde(default)]
    computed_attributes: HashMap<String, Value>,
    #[serde(default)]
    tags: Vec<String>,
}

impl FactContext {
    /// Create a new empty FactContext
    pub fn new() -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
            computed_attributes: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Create a FactContext from any serializable data structure
    pub fn from_serializable<T: Serialize>(data: &T) -> Result<Self> {
        let value = serde_json::to_value(data)
            .map_err(|e| EngineError::SerializationError(e.to_string()))?;
        Ok(Self {
            data: value,
            computed_attributes: HashMap::new(),
            tags: Vec::new(),
        })
    }

    /// Create from raw serde_json::Value
    pub fn from_value(value: Value) -> Self {
        Self {
            data: value,
            computed_attributes: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Retrieve the root data value
    pub fn root_value(&self) -> &Value {
        &self.data
    }

    /// Add a diagnostic tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let t = tag.into();
        if !self.tags.contains(&t) {
            self.tags.push(t);
        }
    }

    /// Check if context contains a tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Get all tags
    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }

    /// Set a computed attribute on the context (useful for chaining rules / forward chaining)
    pub fn set_computed_attribute(&mut self, key: impl Into<String>, value: Value) {
        let mut k = key.into();
        if let Some(stripped) = k.strip_prefix("computed.") {
            k = stripped.to_string();
        }
        self.computed_attributes.insert(k, value);
    }

    /// Remove a computed attribute
    pub fn remove_computed_attribute(&mut self, key: &str) {
        let mut k = key;
        if let Some(stripped) = k.strip_prefix("computed.") {
            k = stripped;
        }
        self.computed_attributes.remove(k);
    }

    /// Get a computed attribute
    pub fn get_computed_attribute(&self, key: &str) -> Option<&Value> {
        let mut k = key;
        if let Some(stripped) = k.strip_prefix("computed.") {
            k = stripped;
        }
        self.computed_attributes.get(k)
    }

    /// Check if a path exists in data or computed attributes and is non-null
    pub fn path_exists(&self, path: &str) -> bool {
        match self.get_path(path) {
            Ok(v) => !v.is_null(),
            Err(_) => false,
        }
    }

    /// Get a value from the context using a dotted path (e.g. "applicant.age" or "computed.has_clb_9_plus")
    pub fn get_path(&self, path: &str) -> Result<&Value> {
        // Special check for tags
        if path == "applicant.tags" || path == "tags" {
            // Can check tags
        }

        // Check computed attributes first if prefixed with "computed." or "derived."
        if let Some(rest) = path
            .strip_prefix("computed.")
            .or_else(|| path.strip_prefix("derived."))
        {
            return self.computed_attributes.get(rest).ok_or_else(|| {
                EngineError::FactNotFound(format!("Computed attribute not found: {path}"))
            });
        }

        let segments: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;

        for segment in segments {
            if current.is_null() {
                return Err(EngineError::FactNotFound(format!(
                    "Null encountered in path '{path}' at segment '{segment}'"
                )));
            }

            // Check if segment is an array index
            if let Ok(idx) = segment.parse::<usize>() {
                match current {
                    Value::Array(arr) => {
                        current = arr.get(idx).ok_or_else(|| {
                            EngineError::FactNotFound(format!(
                                "Array index out of bounds at '{path}'"
                            ))
                        })?;
                    }
                    _ => {
                        return Err(EngineError::TypeMismatch {
                            path: path.to_string(),
                            expected: "Array".to_string(),
                            actual: format!("{:?}", current),
                        });
                    }
                }
            } else {
                match current {
                    Value::Object(map) => {
                        current = map.get(segment).ok_or_else(|| {
                            EngineError::FactNotFound(format!("Field '{segment}' in path '{path}'"))
                        })?;
                    }
                    _ => {
                        return Err(EngineError::TypeMismatch {
                            path: path.to_string(),
                            expected: "Object".to_string(),
                            actual: format!("{:?}", current),
                        });
                    }
                }
            }
        }

        Ok(current)
    }

    /// Helper to get an integer from a path
    pub fn get_i64(&self, path: &str) -> Result<i64> {
        let val = self.get_path(path)?;
        val.as_i64()
            .or_else(|| val.as_str().and_then(|s| s.parse::<i64>().ok()))
            .ok_or_else(|| EngineError::TypeMismatch {
                path: path.to_string(),
                expected: "i64".to_string(),
                actual: format!("{:?}", val),
            })
    }

    /// Helper to get a float from a path
    pub fn get_f64(&self, path: &str) -> Result<f64> {
        let val = self.get_path(path)?;
        val.as_f64()
            .or_else(|| val.as_i64().map(|i| i as f64))
            .or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok()))
            .ok_or_else(|| EngineError::TypeMismatch {
                path: path.to_string(),
                expected: "f64".to_string(),
                actual: format!("{:?}", val),
            })
    }

    /// Helper to get a string from a path
    pub fn get_str(&self, path: &str) -> Result<&str> {
        let val = self.get_path(path)?;
        val.as_str().ok_or_else(|| EngineError::TypeMismatch {
            path: path.to_string(),
            expected: "string".to_string(),
            actual: format!("{:?}", val),
        })
    }

    /// Helper to get a boolean from a path
    pub fn get_bool(&self, path: &str) -> Result<bool> {
        let val = self.get_path(path)?;
        val.as_bool().ok_or_else(|| EngineError::TypeMismatch {
            path: path.to_string(),
            expected: "bool".to_string(),
            actual: format!("{:?}", val),
        })
    }
}
