use std::path::PathBuf;

use serde_json::Value;

/// One JSON file per key in a folder outside any repo.
pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Store { dir }
    }

    fn file_for(&self, id: &str) -> PathBuf {
        let name: String = id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        self.dir.join(format!("{name}.json"))
    }

    pub fn load(&self, id: &str) -> Option<Value> {
        let text = std::fs::read_to_string(self.file_for(id)).ok()?;

        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, id: &str, state: &Value) {
        // state is a convenience: a session goes on without it
        let _ = std::fs::create_dir_all(&self.dir);
        let _ = std::fs::write(self.file_for(id), state.to_string());
    }

    pub fn has(&self, id: &str) -> bool {
        self.file_for(id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_returns_none_for_unknown_ids() {
        let dir = std::env::temp_dir().join(format!("heeded-store-{}", std::process::id()));
        let store = Store::new(dir.clone());

        assert!(store.load("nope").is_none());
        store.save("../a b", &serde_json::json!({ "blocks": 2 }));
        assert_eq!(
            store.load("../a b"),
            Some(serde_json::json!({ "blocks": 2 }))
        );
        assert!(store.has("../a b"));
        assert!(dir.join("___a_b.json").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
