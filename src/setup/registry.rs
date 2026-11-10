//! 每用户的卸载登记项。控制面板的「程序和功能」按它显示与卸载，写在 HKCU 下。

pub use platform::{
    install_location, installed_version, legacy_installations, register, relocate_environment,
    remove, remove_legacy,
};

/// 登记项的子键。
const KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Pudding";
const LEGACY_KEYS: [&str; 2] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sleepy Doll",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sleepy-Doll",
];

#[cfg(windows)]
mod platform {
    use std::{
        path::{Path, PathBuf},
        ptr,
    };

    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE,
            REG_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegOpenKeyExW, RegQueryValueExW,
            RegSetValueExW,
        },
    };

    use crate::setup::{EXECUTABLE, Error, NAME, UNINSTALLER, version};

    use super::{KEY, LEGACY_KEYS};

    /// 写入登记项。`directory` 是安装目录。
    pub fn register(directory: &Path) -> Result<(), Error> {
        let executable = directory.join(EXECUTABLE);
        let uninstaller = directory.join(UNINSTALLER);
        let key = Key::create()?;
        key.set_text("DisplayName", NAME)?;
        key.set_text("DisplayVersion", version())?;
        key.set_text("Publisher", NAME)?;
        // 图标取主程序资源里的第一个。
        key.set_text("DisplayIcon", &format!("\"{}\",0", executable.display()))?;
        // 结尾保留分隔符：控制面板把它当目录用。
        let location = format!(
            "{}\\",
            crate::setup::normalize(&directory.to_string_lossy())
        );
        key.set_text("InstallLocation", &location)?;
        // 路径可能带空格，必须带引号。
        key.set_text("UninstallString", &format!("\"{}\"", uninstaller.display()))?;
        key.set_number("NoModify", 1)?;
        key.set_number("NoRepair", 1)?;
        Ok(())
    }

    /// 删除整个登记项，子键一并删掉。
    pub fn remove() -> Result<(), Error> {
        remove_key(KEY)
    }

    pub fn remove_legacy(directory: &Path) -> Result<(), Error> {
        for key in LEGACY_KEYS {
            if read_key_text(key, "InstallLocation")
                .is_some_and(|location| crate::setup::same_path(Path::new(&location), directory))
            {
                remove_key(key)?;
            }
        }
        Ok(())
    }

    pub fn relocate_environment(mappings: &[(PathBuf, PathBuf)]) -> Result<(), Error> {
        let mut changed = false;
        for variable in ["PUDDING_CONFIG", "SLEEPY_DOLL_CONFIG"] {
            let Some(value) = read_key_text(r"Environment", variable) else {
                continue;
            };
            let original = Path::new(&value);
            if let Some((_, destination)) = mappings
                .iter()
                .filter_map(|(from, to)| {
                    crate::data_directory::relative_path(from, original).map(|suffix| {
                        (
                            from.components().count(),
                            crate::setup::append_suffix(to, &suffix),
                        )
                    })
                })
                .max_by_key(|(depth, _)| *depth)
                && !crate::setup::same_path(original, &destination)
            {
                Key::create_at(r"Environment")?
                    .set_text(variable, &destination.to_string_lossy())?;
                changed = true;
            }
        }
        if changed {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
            };
            let environment = wide("Environment");
            unsafe {
                SendMessageTimeoutW(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    environment.as_ptr() as isize,
                    SMTO_ABORTIFHUNG,
                    1000,
                    ptr::null_mut(),
                );
            }
        }
        Ok(())
    }

    fn remove_key(name: &str) -> Result<(), Error> {
        let subkey = wide(name);
        let result = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, subkey.as_ptr()) };
        if result != ERROR_SUCCESS && result != ERROR_FILE_NOT_FOUND {
            return Err(Error::message(format!(
                "无法删除卸载登记项（错误码 {result}）"
            )));
        }
        Ok(())
    }

    /// 登记项里的安装位置。
    pub fn install_location() -> Option<PathBuf> {
        read_text("InstallLocation")
            .filter(|text| !text.is_empty())
            .map(|text| PathBuf::from(crate::setup::normalize(&text)))
    }

    pub fn installed_version() -> Option<String> {
        read_text("DisplayVersion")
    }

    pub fn legacy_installations() -> Vec<crate::setup::Installed> {
        LEGACY_KEYS
            .into_iter()
            .filter_map(|key| {
                let directory = PathBuf::from(crate::setup::normalize(&read_key_text(
                    key,
                    "InstallLocation",
                )?));
                Some(crate::setup::Installed {
                    directory,
                    version: read_key_text(key, "DisplayVersion"),
                })
            })
            .collect()
    }

    /// 已打开的登记项，离开作用域时关闭。
    struct Key(HKEY);

    impl Key {
        fn create() -> Result<Self, Error> {
            Self::create_at(KEY)
        }

        fn create_at(name: &str) -> Result<Self, Error> {
            let subkey = wide(name);
            let mut key: HKEY = ptr::null_mut();
            let result = unsafe {
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    subkey.as_ptr(),
                    0,
                    ptr::null(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_WRITE,
                    ptr::null(),
                    &mut key,
                    ptr::null_mut(),
                )
            };
            if result != ERROR_SUCCESS {
                return Err(Error::message(format!(
                    "无法写入卸载登记项（错误码 {result}）"
                )));
            }
            Ok(Self(key))
        }

        fn set_text(&self, name: &str, value: &str) -> Result<(), Error> {
            let utf16 = wide(value);
            // REG_SZ 的字节数是字符数的两倍，含结尾的 NUL。
            let bytes =
                unsafe { std::slice::from_raw_parts(utf16.as_ptr().cast(), utf16.len() * 2) };
            self.set(name, REG_SZ, bytes)
        }

        fn set_number(&self, name: &str, value: u32) -> Result<(), Error> {
            self.set(name, REG_DWORD, &value.to_le_bytes())
        }

        fn set(&self, name: &str, kind: u32, bytes: &[u8]) -> Result<(), Error> {
            let name = wide(name);
            let result = unsafe {
                RegSetValueExW(
                    self.0,
                    name.as_ptr(),
                    0,
                    kind,
                    bytes.as_ptr(),
                    bytes.len() as u32,
                )
            };
            if result != ERROR_SUCCESS {
                return Err(Error::message(format!(
                    "无法写入登记项 {name:?}（错误码 {result}）"
                )));
            }
            Ok(())
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn read_text(name: &str) -> Option<String> {
        read_key_text(KEY, name)
    }

    fn read_key_text(key_name: &str, name: &str) -> Option<String> {
        let subkey = wide(key_name);
        let mut key: HKEY = ptr::null_mut();
        let result =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut key) };
        if result != ERROR_SUCCESS {
            return None;
        }
        let key = Key(key);
        let name = wide(name);
        let mut kind: u32 = 0;
        let mut size = 0;
        let result = unsafe {
            RegQueryValueExW(
                key.0,
                name.as_ptr(),
                ptr::null(),
                &mut kind,
                ptr::null_mut(),
                &mut size,
            )
        };
        if result != ERROR_SUCCESS || kind != REG_SZ {
            return None;
        }
        // 第一次查询给出字节数；按字符数开缓冲，多留一个给结尾的 NUL。
        let mut units = vec![0u16; size as usize / 2 + 1];
        let mut size = (units.len() * 2) as u32;
        let result = unsafe {
            RegQueryValueExW(
                key.0,
                name.as_ptr(),
                ptr::null(),
                &mut kind,
                units.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if result != ERROR_SUCCESS {
            return None;
        }
        let length = units
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(units.len());
        Some(String::from_utf16_lossy(&units[..length]))
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod platform {
    use std::path::{Path, PathBuf};

    use crate::setup::Error;

    pub fn register(_directory: &Path) -> Result<(), Error> {
        Ok(())
    }

    pub fn remove() -> Result<(), Error> {
        Ok(())
    }

    pub fn install_location() -> Option<PathBuf> {
        None
    }

    pub fn installed_version() -> Option<String> {
        None
    }

    pub fn legacy_installations() -> Vec<crate::setup::Installed> {
        Vec::new()
    }
    pub fn remove_legacy(_: &Path) -> Result<(), Error> {
        Ok(())
    }

    pub fn relocate_environment(_: &[(PathBuf, PathBuf)]) -> Result<(), Error> {
        Ok(())
    }
}
