use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub done: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Edge,
    Floating,
    Desktop,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Left,
    #[default]
    Right,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub side: Side,
    pub theme: Theme,
    pub opacity: u32,
    pub auto_hide: bool,
    pub hide_ms: u64,
    pub dwell_ms: u64,
    pub pinned: bool,
    pub locked: bool,
    pub topmost: bool,
    pub titlebar: bool,
    pub fullscreen_pause: bool,
    pub hotkey_mod: u32,
    pub hotkey_vk: u32,
    pub autostart: bool,
    pub introduced: bool,
    pub tray_tip_shown: bool,
    pub import_decided: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Edge,
            side: Side::Right,
            theme: Theme::System,
            opacity: 48,
            auto_hide: true,
            hide_ms: 400,
            dwell_ms: 250,
            pinned: false,
            locked: false,
            topmost: false,
            titlebar: true,
            fullscreen_pause: true,
            hotkey_mod: 3,
            hotkey_vk: 0x4c,
            autostart: false,
            introduced: false,
            tray_tip_shown: false,
            import_decided: false,
            x: 160,
            y: 120,
            width: 400,
            height: 600,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub version: u32,
    pub todos: Vec<Todo>,
    pub settings: Settings,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            version: 1,
            todos: Vec::new(),
            settings: Settings::default(),
        }
    }
}
impl Model {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("不支持的数据版本；原文件已保留".into());
        }
        let mut ids = HashSet::new();
        for t in &self.todos {
            if t.id.is_empty() || !ids.insert(&t.id) || t.text.trim().is_empty() {
                return Err("任务数据无效；原文件已保留".into());
            }
        }
        if !(10..=90).contains(&self.settings.opacity)
            || !(80..=3000).contains(&self.settings.dwell_ms)
            || !(100..=5000).contains(&self.settings.hide_ms)
        {
            return Err("设置值超出范围；原文件已保留".into());
        }
        if !(320..=1000).contains(&self.settings.width)
            || !(360..=1600).contains(&self.settings.height)
        {
            return Err("窗口尺寸数据超出安全范围；原文件已保留".into());
        }
        Ok(())
    }
    pub fn add(&mut self, text: &str) -> Option<String> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = format!(
            "{:x}-{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        self.todos.insert(
            0,
            Todo {
                id: id.clone(),
                text: text.into(),
                done: false,
            },
        );
        Some(id)
    }
    pub fn edit(&mut self, id: &str, text: &str) {
        if !text.trim().is_empty() {
            if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
                t.text = text.trim().into()
            }
        }
    }
    pub fn toggle(&mut self, id: &str) {
        if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
            t.done = !t.done
        }
    }
    pub fn delete(&mut self, id: &str) {
        self.todos.retain(|t| t.id != id)
    }
    pub fn import(&mut self, text: &str) -> Result<usize, String> {
        let todos: Vec<Todo> = serde_json::from_str(text).map_err(|_| "QuietDo 数据格式无效")?;
        let mut check = self.clone();
        check.todos = todos.clone();
        check.validate()?;
        let old: HashSet<String> = self.todos.iter().map(|t| t.id.clone()).collect();
        let added: Vec<_> = todos.into_iter().filter(|t| !old.contains(&t.id)).collect();
        let count = added.len();
        self.todos.extend(added);
        self.settings.import_decided = true;
        Ok(count)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn todo_semantics_and_unicode() {
        let mut m = Model::default();
        assert!(m.add("  \t ").is_none());
        let a = m.add(" 中文🙂 ").unwrap();
        let b = m.add("second").unwrap();
        m.toggle(&a);
        assert_eq!(m.todos[1].id, a);
        assert!(m.todos[1].done);
        m.toggle(&a);
        m.edit(&a, " 编辑 ");
        assert_eq!(m.todos[1].text, "编辑");
        m.edit(&a, "  ");
        assert_eq!(m.todos[1].text, "编辑");
        m.delete(&b);
        assert_eq!(m.todos.len(), 1);
        let json = serde_json::to_string(&m).unwrap();
        let again: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(m.todos, again.todos);
    }
    #[test]
    fn import_is_explicit_and_deduplicated() {
        let mut m = Model::default();
        let json = r#"[{"id":"quiet-1","text":"原有任务","done":true}]"#;
        assert_eq!(m.import(json).unwrap(), 1);
        assert_eq!(m.import(json).unwrap(), 0);
        assert!(m.import("bad").is_err());
        assert_eq!(m.todos.len(), 1);
    }
    #[test]
    fn future_or_duplicate_data_rejected() {
        let mut m = Model::default();
        m.version = 2;
        assert!(m.validate().is_err());
        m.version = 1;
        m.todos = vec![
            Todo {
                id: "a".into(),
                text: "a".into(),
                done: false
            };
            2
        ];
        assert!(m.validate().is_err());
    }
    #[test]
    fn unsafe_geometry_cannot_allocate_unbounded_surfaces() {
        let mut m = Model::default();
        m.settings.width = i32::MAX;
        assert!(m.validate().is_err());
        m.settings.width = 400;
        m.settings.height = -1;
        assert!(m.validate().is_err());
    }
}
