use crate::model::Model;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
pub struct Store {
    pub path: PathBuf,
    pub writable: bool,
}
impl Store {
    pub fn open(path: PathBuf) -> (Self, Result<Model, String>) {
        let result = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice::<Model>(&bytes)
                .map_err(|_| "读取失败：数据文件损坏，已禁止保存以保护原文件".into())
                .and_then(|m| {
                    m.validate()?;
                    Ok(m)
                }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Model::default()),
            Err(e) => Err(format!("读取失败：{e}；原数据保持不变")),
        };
        (
            Self {
                path,
                writable: result.is_ok(),
            },
            result,
        )
    }
    pub fn save(&self, model: &Model) -> Result<(), String> {
        if !self.writable {
            return Err("数据未成功读取，禁止覆盖保存。请修复数据文件后重启。".into());
        }
        model.validate()?;
        let data = serde_json::to_vec_pretty(model).map_err(|e| e.to_string())?;
        atomic_write(&self.path, &data)
            .map_err(|e| format!("保存失败：{e}。更改尚未保存，请保留窗口并重试。"))
    }
}
fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("无数据目录"))?;
    fs::create_dir_all(parent)?;
    let temp = path.with_extension("json.tmp");
    let result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp)?;
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        #[cfg(windows)]
        {
            use windows::{
                Win32::Storage::FileSystem::{
                    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
                },
                core::HSTRING,
            };
            unsafe {
                MoveFileExW(
                    &HSTRING::from(temp.as_os_str()),
                    &HSTRING::from(path.as_os_str()),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            }
            .map_err(std::io::Error::other)?;
        }
        #[cfg(not(windows))]
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "lightdo-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }
    #[test]
    fn corrupt_load_never_overwrites() {
        let d = dir();
        let p = d.join("corrupt.json");
        fs::write(&p, b"broken").unwrap();
        let (s, m) = Store::open(p.clone());
        assert!(m.is_err());
        assert!(s.save(&Model::default()).is_err());
        assert_eq!(fs::read(&p).unwrap(), b"broken");
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn atomic_roundtrip_and_failure() {
        let d = dir();
        let p = d.join("round.json");
        let (s, m) = Store::open(p.clone());
        let mut m = m.unwrap();
        for i in 0..50 {
            m.add(&format!("任务 {i}"));
            s.save(&m).unwrap();
        }
        let (_, again) = Store::open(p.clone());
        assert_eq!(again.unwrap().todos, m.todos);
        assert!(!p.with_extension("json.tmp").exists());
        let bad = Store {
            path: p.join("impossible.json"),
            writable: true,
        };
        assert!(bad.save(&m).is_err());
        let (_, again) = Store::open(p);
        assert_eq!(again.unwrap().todos.len(), 50);
        fs::remove_dir_all(d).unwrap();
    }
}
