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
/// A stable per-user folder, independent of the executable and working directory.
#[cfg(windows)]
pub fn data_path() -> Result<PathBuf, String> {
    if let Some(directory) = std::env::var_os("LIGHTDO_DATA_DIR") {
        let directory = PathBuf::from(directory);
        if !directory.is_absolute() {
            return Err("开发数据目录必须为绝对路径，未读取或覆盖任何任务文件".into());
        }
        return Ok(directory.join("state.json"));
    }
    unsafe {
        use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::*};
        let folder = SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None)
            .map_err(|e| format!("无法确定本机任务目录：{e}；未创建空清单"))?;
        let directory = folder.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(folder.0.cast()));
        Ok(PathBuf::from(directory?).join("LightDo").join("state.json"))
    }
}
fn read_data(path: &Path) -> std::io::Result<Vec<u8>> {
    for attempt in 0..=5 {
        match fs::read(path) {
            // Windows may briefly share-lock a file during indexing/scanning.
            // Never treat this as a new empty list. Retry only transient locks.
            Err(error) if attempt < 5 && matches!(error.raw_os_error(), Some(32 | 33)) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            result => return result,
        }
    }
    unreachable!()
}
impl Store {
    pub fn open(path: PathBuf) -> (Self, Result<Model, String>) {
        let result = match read_data(&path) {
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
    #[cfg(windows)]
    #[test]
    fn a_brief_file_lock_does_not_replace_existing_tasks_with_an_empty_list() {
        use std::os::windows::fs::OpenOptionsExt;
        let d = dir();
        let p = d.join("locked.json");
        let mut model = Model::default();
        model.add("重启后保留的未完成任务 🙂");
        fs::write(&p, serde_json::to_vec(&model).unwrap()).unwrap();
        let original = fs::read(&p).unwrap();
        let locked = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&p)
            .unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(180));
            drop(locked);
        });
        let (store, loaded) = Store::open(p.clone());
        release.join().unwrap();
        assert!(store.writable);
        assert_eq!(loaded.unwrap().todos, model.todos);
        assert_eq!(fs::read(&p).unwrap(), original);
        fs::remove_dir_all(d).unwrap();
    }
}
