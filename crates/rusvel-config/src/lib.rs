//! TOML-backed configuration adapter implementing [`ConfigPort`].
//!
//! Global config lives at `~/.rusvel/config.toml`. Per-session overrides
//! are kept in memory and overlay the global values.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::ConfigPort;
use serde_json::Value;

/// TOML-file-backed configuration with optional per-session overlays.
pub struct TomlConfig {
    path: PathBuf,
    global: RwLock<HashMap<String, Value>>,
    sessions: RwLock<HashMap<String, HashMap<String, Value>>>,
}

impl TomlConfig {
    /// Load (or create) global config from `~/.rusvel/config.toml`.
    pub fn load_default() -> Result<Self> {
        let dir = dirs_or_home().join(".rusvel");
        Self::load(dir.join("config.toml"))
    }

    /// Load (or create) config from a specific path.
    pub fn load(path: PathBuf) -> Result<Self> {
        let global = if path.exists() {
            read_toml(&path)?
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| RusvelError::Config(e.to_string()))?;
            }
            let defaults = default_config();
            write_toml(&path, &defaults)?;
            defaults
        };

        Ok(Self {
            path,
            global: RwLock::new(global),
            sessions: RwLock::new(HashMap::new()),
        })
    }

    /// Set a per-session override. Session overrides shadow global values.
    pub fn set_session_value(&self, session_id: &str, key: &str, value: Value) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(lock_err)?;
        sessions
            .entry(session_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
        Ok(())
    }

    /// Get a value with session overlay: session value wins over global.
    pub fn get_for_session(&self, session_id: &str, key: &str) -> Result<Option<Value>> {
        let sessions = self.sessions.read().map_err(lock_err)?;
        if let Some(session_map) = sessions.get(session_id)
            && let Some(v) = session_map.get(key)
        {
            return Ok(Some(v.clone()));
        }
        drop(sessions);
        self.get_value(key)
    }

    /// Clear all overrides for a session.
    pub fn clear_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(lock_err)?;
        sessions.remove(session_id);
        Ok(())
    }
}

impl ConfigPort for TomlConfig {
    fn get_value(&self, key: &str) -> Result<Option<Value>> {
        let global = self.global.read().map_err(lock_err)?;
        Ok(global.get(key).cloned())
    }

    fn set_value(&self, key: &str, value: Value) -> Result<()> {
        let mut global = self.global.write().map_err(lock_err)?;
        global.insert(key.to_string(), value);
        write_toml(&self.path, &global)?;
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME").map_or_else(|_| PathBuf::from("."), PathBuf::from)
}

fn lock_err<T>(_: T) -> RusvelError {
    RusvelError::Config("config lock poisoned".into())
}

fn default_config() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("llm.default_model".into(), Value::String("gpt-4o".into()));
    m.insert("llm.temperature".into(), serde_json::json!(0.7));
    m.insert("log.level".into(), Value::String("info".into()));
    m
}

fn read_toml(path: &PathBuf) -> Result<HashMap<String, Value>> {
    let content = fs::read_to_string(path).map_err(|e| RusvelError::Config(e.to_string()))?;
    let table: toml::Table =
        toml::from_str(&content).map_err(|e| RusvelError::Config(e.to_string()))?;
    let mut map = HashMap::new();
    flatten_toml("", &toml::Value::Table(table), &mut map);
    Ok(map)
}

fn write_toml(path: &PathBuf, map: &HashMap<String, Value>) -> Result<()> {
    // Build a nested TOML table from dotted keys.
    let mut root = toml::Table::new();
    for (key, value) in map {
        insert_dotted(&mut root, key, json_to_toml(value));
    }
    let content = toml::to_string_pretty(&root).map_err(|e| RusvelError::Config(e.to_string()))?;
    fs::write(path, content).map_err(|e| RusvelError::Config(e.to_string()))
}

/// Flatten a nested TOML table into dotted keys.
fn flatten_toml(prefix: &str, val: &toml::Value, out: &mut HashMap<String, Value>) {
    match val {
        toml::Value::Table(table) => {
            for (k, v) in table {
                let full = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_toml(&full, v, out);
            }
        }
        other => {
            out.insert(prefix.to_string(), toml_to_json(other));
        }
    }
}

fn toml_to_json(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::json!(i),
        toml::Value::Float(f) => serde_json::json!(f),
        toml::Value::Boolean(b) => Value::Bool(*b),
        toml::Value::Array(a) => Value::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => {
            let obj: serde_json::Map<String, Value> = t
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect();
            Value::Object(obj)
        }
        toml::Value::Datetime(d) => Value::String(d.to_string()),
    }
}

fn json_to_toml(v: &Value) -> toml::Value {
    match v {
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else {
                toml::Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Array(a) => toml::Value::Array(a.iter().map(json_to_toml).collect()),
        Value::Object(o) => {
            let t: toml::Table = o
                .iter()
                .map(|(k, v)| (k.clone(), json_to_toml(v)))
                .collect();
            toml::Value::Table(t)
        }
        Value::Null => toml::Value::String(String::new()),
    }
}

/// Insert a value at a dotted key path into a nested TOML table.
fn insert_dotted(table: &mut toml::Table, key: &str, value: toml::Value) {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    if parts.len() == 1 {
        table.insert(parts[0].to_string(), value);
    } else {
        let entry = table
            .entry(parts[0])
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if let toml::Value::Table(sub) = entry {
            insert_dotted(sub, parts[1], value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config() -> (TempDir, TomlConfig) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = TomlConfig::load(path).unwrap();
        (dir, cfg)
    }

    #[test]
    fn creates_default_config_file() {
        let (dir, _cfg) = test_config();
        assert!(dir.path().join("config.toml").exists());
    }

    #[test]
    fn get_set_roundtrip() {
        let (_dir, cfg) = test_config();
        cfg.set_value("app.name", Value::String("test".into()))
            .unwrap();
        let v = cfg.get_value("app.name").unwrap();
        assert_eq!(v, Some(Value::String("test".into())));
    }

    #[test]
    fn persists_to_disk() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        {
            let cfg = TomlConfig::load(path.clone()).unwrap();
            cfg.set_value("db.url", Value::String("sqlite://test.db".into()))
                .unwrap();
        }

        // Reload from disk
        let cfg2 = TomlConfig::load(path).unwrap();
        let v = cfg2.get_value("db.url").unwrap();
        assert_eq!(v, Some(Value::String("sqlite://test.db".into())));
    }

    #[test]
    fn session_overrides_global() {
        let (_dir, cfg) = test_config();
        cfg.set_value("llm.default_model", Value::String("gpt-4o".into()))
            .unwrap();
        cfg.set_session_value("s1", "llm.default_model", Value::String("claude".into()))
            .unwrap();

        // Session value wins
        let v = cfg.get_for_session("s1", "llm.default_model").unwrap();
        assert_eq!(v, Some(Value::String("claude".into())));

        // Global still returns original
        let v = cfg.get_value("llm.default_model").unwrap();
        assert_eq!(v, Some(Value::String("gpt-4o".into())));

        // Different session falls through to global
        let v = cfg.get_for_session("s2", "llm.default_model").unwrap();
        assert_eq!(v, Some(Value::String("gpt-4o".into())));
    }

    #[test]
    fn clear_session_removes_overrides() {
        let (_dir, cfg) = test_config();
        cfg.set_session_value("s1", "key", Value::Bool(true))
            .unwrap();
        cfg.clear_session("s1").unwrap();
        let v = cfg.get_for_session("s1", "key").unwrap();
        assert_eq!(v, None);
    }

    #[test]
    fn get_typed_works() {
        let (_dir, cfg) = test_config();
        cfg.set_value("count", serde_json::json!(42)).unwrap();
        let v: Option<i64> = cfg.get_typed("count").unwrap();
        assert_eq!(v, Some(42));
    }
}
