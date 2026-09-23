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
}

impl FactContext {
    /// Create a new empty FactContext
    pub fn new() -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
            computed_attributes: HashMap::new(),
        }
    }

    /// Create a FactContext from any serializable data structure
    pub fn from_serializable<T: Serialize>(data: &T) -> Result<Self> {
        let value = serde_json::to_value(data)
            .map_err(|e| EngineError::SerializationError(e.to_string()))?;
        Ok(Self {
            data: value,
            computed_attributes: HashMap::new(),
        })
    }

    /// Create from raw serde_json::Value
    pub fn from_value(value: Value) -> Self {
        Self {
            data: value,
            computed_attributes: HashMap::new(),
        }
    }

    /// Retrieve the root data value
    pub fn root_value(&self) -> &Value {
        &self.data
    }

    /// Set a computed attribute on the context (useful for chaining rules)
    pub fn set_computed_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.computed_attributes.insert(key.into(), value);
    }

    /// Get a computed attribute
    pub fn get_computed_attribute(&self, key: &str) -> Option<&Value> {
        self.computed_attributes.get(key)
    }

    /// Get a value from the context using a dotted path (e.g. "applicant.age" or "computed.my_attr")
    pub fn get_path(&self, path: &str) -> Result<&Value> {
        // Check computed attributes first if prefixed
        if let Some(rest) = path.strip_prefix("computed.") {
            return self.computed_attributes.get(rest).ok_or_else(|| {
                EngineError::FactNotFound(format!("Computed attribute not found: {path}"))
            });
        }

        let segments: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;

        for segment in segments {
            // Check if segment is an array index
            if let Ok(idx) = segment.parse::<usize>() {
                match current {
                    Value::Array(arr) => {
                        current = arr.get(idx).ok_or_else(|| {
                            EngineError::FactNotFound(format!("Array index out of bounds at {path}"))
                        })?;
                    }
                    _ => {
                        return Err(EngineError::TypeMismatch {
                            path: path.to_string(),
                            expected: "Array".to_string(),
                            actual: format!("{:?}", current),
                        })
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
                        })
                    }
                }
            }
        }

        Ok(current)
    }

    /// Helper to get an integer from a path
    pub fn get_i64(&self, path: &str) -> Result<i64> {
        let val = self.get_path(path)?;
        val.as_i64().ok_or_else(|| EngineError::TypeMismatch {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_path_query() {
        let data = json!({
            "applicant": {
                "name": "Alex",
                "age": 29,
                "education": {
                    "degree": "master",
                    "points": 126
                },
                "skills": ["rust", "cloud"]
            }
        });

        let ctx = FactContext::from_value(data);
        assert_eq!(ctx.get_str("applicant.name").unwrap(), "Alex");
        assert_eq!(ctx.get_i64("applicant.age").unwrap(), 29);
        assert_eq!(ctx.get_str("applicant.education.degree").unwrap(), "master");
        assert_eq!(ctx.get_str("applicant.skills.0").unwrap(), "rust");
    }
}
