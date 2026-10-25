//! System material policy. Downgrade is explicit; plain transparency is
//! never reported as successful refraction.
use windows::{
    Win32::{
        System::Registry::*,
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
    core::w,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Material {
    Glass,
    HighContrast,
    TransparencyOff,
    Remote,
    Software,
}
impl Material {
    pub fn detect(software: bool) -> Self {
        unsafe {
            if high_contrast() {
                return Self::HighContrast;
            }
            let mut value = 1u32;
            let mut size = 4;
            let _ = RegGetValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
                w!("EnableTransparency"),
                RRF_RT_REG_DWORD,
                None,
                Some(&mut value as *mut _ as *mut _),
                Some(&mut size),
            );
            if value == 0 {
                return Self::TransparencyOff;
            }
            if GetSystemMetrics(SM_REMOTESESSION) != 0 {
                return Self::Remote;
            }
            if software {
                return Self::Software;
            }
            Self::Glass
        }
    }
    pub fn glass(self) -> bool {
        self == Self::Glass
    }
    pub fn reason(self) -> &'static str {
        match self {
            Self::Glass => "实时玻璃",
            Self::HighContrast => "玻璃降级：系统高对比度",
            Self::TransparencyOff => "玻璃降级：系统已关闭透明效果",
            Self::Remote => "玻璃降级：远程桌面",
            Self::Software => "玻璃降级：图形硬件不可用",
        }
    }
}
pub fn high_contrast() -> bool {
    unsafe {
        let mut info = HIGHCONTRASTW {
            cbSize: size_of::<HIGHCONTRASTW>() as u32,
            ..Default::default()
        };
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            info.cbSize,
            Some(&mut info as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
            && info.dwFlags.0 & HCF_HIGHCONTRASTON.0 != 0
    }
}
