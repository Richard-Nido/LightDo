#[cfg(windows)]
pub mod app;
pub mod model;
#[cfg(windows)]
pub mod platform;
#[cfg(windows)]
pub mod settings_ui;
pub mod state;
pub mod storage;
#[cfg(windows)]
pub mod ui;

/// Background acquisition diagnostics never include pixels or task text.
#[cfg(windows)]
pub mod diagnostics {
    pub fn note_sparingly(_kind: &str, _message: impl FnOnce() -> String) {}
    pub fn mark(_message: impl FnOnce() -> String) {}
    pub struct Timed;
    impl Timed {
        pub fn start(_kind: &str, _message: impl FnOnce() -> String) -> Self {
            Self
        }
    }
}
