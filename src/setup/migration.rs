//! 安装时接管旧名安装。源目录在新程序、数据和外壳登记完成前保持不动。
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Error, Installed, NAME, failed, registry, shell};

const NAMES: [&str; 2] = ["Sleepy Doll", "Sleepy-Doll"];
const OLD_PROGRAMS: [&str; 2] = ["sleepy-doll.exe", "sleepy-doll-agent-host.exe"];

pub(super) fn is_legacy_name(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| NAMES.iter().any(|old| name.eq_ignore_ascii_case(old)))
}

fn is_installation(path: &Path) -> bool {
    !path.join(".git").exists()
        && !path.join("Cargo.toml").exists()
        && path.join(OLD_PROGRAMS[0]).is_file()
}

pub(super) fn replacement_directory(previous: &Path) -> PathBuf {
    if is_legacy_name(previous) {
        previous.with_file_name(NAME)
    } else {
        previous.to_path_buf()
    }
}

pub(super) fn previous_installation() -> Option<Installed> {
    candidates(None)
        .into_iter()
        .find(|previous| is_installation(&previous.directory))
}

fn candidates(destination: Option<&Path>) -> Vec<Installed> {
    let mut installations = registry::legacy_installations();
    let mut parents = vec![PathBuf::from(r"D:\")];
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        parents.push(PathBuf::from(local).join("Programs"));
    }
    if let Some(parent) = destination.and_then(Path::parent) {
        parents.insert(0, parent.to_path_buf());
    }
    for parent in parents {
        for name in NAMES {
            let directory = parent.join(name);
            if !installations
                .iter()
                .any(|item| super::same_path(&item.directory, &directory))
            {
                installations.push(Installed {
                    directory,
                    version: None,
                });
            }
        }
    }
    installations
}

fn previous_for(destination: &Path) -> Option<PathBuf> {
    candidates(Some(destination)).into_iter().find_map(|item| {
        (is_installation(&item.directory)
            && super::same_path(&replacement_directory(&item.directory), destination))
        .then_some(item.directory)
    })
}

pub(super) fn required(destination: &Path) -> bool {
    previous_for(destination).is_some()
}

/// 文件复制、目录重命名和 JSON 修改都有逆操作；不在失败的安装上安排源目录删除。
#[derive(Default)]
pub(super) struct Migration {
    source: Option<PathBuf>,
    destination: PathBuf,
    copied: Vec<PathBuf>,
    created_directories: Vec<PathBuf>,
    renamed: Vec<(PathBuf, PathBuf)>,
    documents: Vec<(PathBuf, Vec<u8>)>,
    roots: Vec<(PathBuf, PathBuf)>,
    configurations: Vec<(PathBuf, PathBuf)>,
    retired_user_directories: Vec<PathBuf>,
    guards: Vec<fs::File>,
    locked_paths: Vec<PathBuf>,
    committed: bool,
}

impl Migration {
    pub(super) fn prepare(destination: &Path) -> Result<Self, Error> {
        let mut migration = Self::default();
        migration.destination = destination.to_path_buf();
        migration.source = previous_for(destination);
        if let Some(source) = migration.source.clone() {
            super::validate_directory(&source.to_string_lossy())?;
            // 仅请求本产品旧进程正常退出，不通过进程名结束其他安装或宿主。
            let programs: Vec<_> = OLD_PROGRAMS.iter().map(|name| source.join(name)).collect();
            super::uninstall::close_program_files(&programs, "迁移")?;
            migration.lock_user(&source.join("user"))?;
            migration.lock_user(&source.join("bridge/user"))?;
            migration
                .roots
                .push((source.clone(), destination.to_path_buf()));
            migration.roots.push((
                source.join(OLD_PROGRAMS[0]),
                destination.join(super::EXECUTABLE),
            ));
            if !super::same_path(&source, destination) {
                migration.copy(&source, destination)?;
            }
        }
        if let Some(roaming) = std::env::var_os("APPDATA").map(PathBuf::from) {
            for name in NAMES {
                let source = roaming.join(name);
                if !source.join("user/config.json").is_file() {
                    continue;
                }
                let target = if !destination.join("user/config.json").is_file() {
                    destination.to_path_buf()
                } else {
                    roaming.join(NAME)
                };
                migration.lock_user(&source.join("user"))?;
                migration.roots.push((source.clone(), target.clone()));
                migration.copy(&source, &target)?;
            }
        }
        if migration.source.is_none()
            && migration.roots.is_empty()
            && !destination.join(".sleepy-doll").exists()
            && !destination.join("user/.sleepy-doll").exists()
            && !destination.join("user/data/sleepy-doll.db").exists()
            && !destination.join("bridge/user/config.json").is_file()
        {
            return Ok(migration);
        }
        if migration.source.is_none() {
            migration.lock_user(&destination.join("user"))?;
        }

        // 先规划全部目录映射，再修正引用，避免父目录替换遮住更具体的旧数据路径。
        let mut roots = vec![destination.to_path_buf()];
        roots.extend(migration.roots.iter().map(|(_, to)| to.clone()));
        roots.sort();
        roots.dedup();
        for root in &roots {
            migration.normalize_tree(root)?;
            migration.normalize_plugin_manifests(root)?;
        }
        for root in roots {
            migration.rewrite_documents(&root)?;
        }
        Ok(migration)
    }

    fn lock_user(&mut self, user: &Path) -> Result<(), Error> {
        let mut locks = vec![];
        for name in ["data", ".sleepy-doll"] {
            for file in ["sleepy-doll.runtime.lock", "pudding.runtime.lock"] {
                locks.push(user.join(name).join(file));
            }
        }
        let config = user.join("config.json");
        if config.is_file() {
            let mut value: Value =
                serde_json::from_slice(&fs::read(&config).map_err(failed("无法读取旧配置"))?)
                    .map_err(|error| Error::message(format!("旧配置无法解析：{error}")))?;
            crate::config::expand_env(&mut value)
                .map_err(|error| Error::message(error.to_string()))?;
            if let Some(database) = value.pointer("/storage/database").and_then(Value::as_str) {
                locks.push(user.join(database).with_extension("runtime.lock"));
            }
        }
        let mut locks = locks
            .into_iter()
            .filter(|path| path.is_file())
            .map(|path| fs::canonicalize(path).map_err(failed("无法解析旧数据锁路径")))
            .collect::<Result<Vec<_>, _>>()?;
        locks.sort();
        locks.dedup();
        for path in locks {
            if self
                .locked_paths
                .iter()
                .any(|locked| super::same_path(locked, &path))
            {
                continue;
            }
            check_path(&path)?;
            let file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(failed("无法锁定旧数据"))?;
            file.try_lock().map_err(|_| {
                Error::message("旧版仍在写入用户数据，迁移已停止；旧配置和会话未删除")
            })?;
            self.guards.push(file);
            self.locked_paths.push(path);
        }
        Ok(())
    }

    fn copy(&mut self, source: &Path, target: &Path) -> Result<(), Error> {
        // Windows 的独占文件锁也阻止本进程用另一句柄读取。运行锁不承载用户数据，
        // 由新程序自行创建；迁移期间继续持有源锁，但不复制或校验锁文件本身。
        if self
            .locked_paths
            .iter()
            .any(|locked| super::same_path(locked, source))
        {
            return Ok(());
        }
        check_path(source)?;
        super::cleanup::validate(&[super::Removal::File(target.to_path_buf())])?;
        if source.is_dir() {
            if target.exists() {
                check_path(target)?;
            } else {
                fs::create_dir_all(target).map_err(failed("无法创建迁移目录"))?;
                self.created_directories.push(target.to_path_buf());
            }
            for entry in fs::read_dir(source).map_err(failed("无法读取旧版目录"))? {
                let entry = entry.map_err(failed("无法读取旧版文件"))?;
                // 仅排除本轮安装缓存；不能按 .cache 名称丢弃用户自定义数据。
                if super::same_path(&entry.path(), &super::scratch_root()) {
                    continue;
                }
                self.copy(&entry.path(), &target.join(entry.file_name()))?;
            }
        } else if target.exists() {
            check_path(target)?;
            if digest(source)? != digest(target)? {
                return Err(Error::message(format!(
                    "新旧目录中存在不同内容，未覆盖也未删除旧版：{}",
                    target.display()
                )));
            }
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(failed("无法创建迁移目录"))?;
            }
            // 先登记，复制中途失败也能撤回不完整文件。
            self.copied.push(target.to_path_buf());
            fs::copy(source, target).map_err(failed(format!("无法迁移 {}", source.display())))?;
            if digest(source)? != digest(target)? {
                return Err(Error::message("旧数据复制校验失败，未删除旧安装"));
            }
        }
        Ok(())
    }

    fn rename(&mut self, source: &Path, target: &Path) -> Result<(), Error> {
        if !source.exists() {
            return Ok(());
        }
        check_path(source)?;
        if target.exists() {
            return Err(Error::message(format!(
                "迁移目标已存在，旧数据未删除：{}",
                target.display()
            )));
        }
        fs::rename(source, target).map_err(failed(format!("无法迁移 {}", source.display())))?;
        // 除当前目标中的路径，还要记录复制前的源路径。
        for (previous, current) in self.roots.clone() {
            if let Some(relative) = crate::data_directory::relative_path(&current, source) {
                self.roots
                    .push((previous.join(relative), target.to_path_buf()));
            }
        }
        self.roots
            .push((source.to_path_buf(), target.to_path_buf()));
        self.renamed
            .push((source.to_path_buf(), target.to_path_buf()));
        Ok(())
    }

    fn normalize_tree(&mut self, root: &Path) -> Result<(), Error> {
        // 曾有桥恢复工具在 bridge/user 写入配置，主用户目录为空时整体归位。
        let bridge_user = root.join("bridge/user");
        if bridge_user.join("config.json").is_file() {
            let target = root.join("user");
            self.copy(&bridge_user, &target)?;
            for (previous, current) in self.roots.clone() {
                if let Some(suffix) = crate::data_directory::relative_path(&current, &bridge_user) {
                    let source = previous.join(suffix);
                    if source.join("config.json").is_file() {
                        self.configurations
                            .push((source.join("config.json"), target.join("config.json")));
                    }
                    self.roots.push((source, target.clone()));
                }
            }
            self.roots.push((bridge_user.clone(), target));
            self.retired_user_directories.push(bridge_user);
        }
        let user = root.join("user");
        self.rename(&root.join(".sleepy-doll"), &root.join("data"))?;
        self.rename(&user.join(".sleepy-doll"), &user.join("data"))?;
        for data in [root.join("data"), user.join("data")] {
            for (old, new) in [
                ("sleepy-doll.db", "pudding.db"),
                ("sleepy-doll.db-wal", "pudding.db-wal"),
                ("sleepy-doll.db-shm", "pudding.db-shm"),
                ("sleepy-doll.db-journal", "pudding.db-journal"),
                ("sleepy-doll.runtime.lock", "pudding.runtime.lock"),
                ("sleepy-doll.artifacts", "pudding.artifacts"),
            ] {
                self.rename(&data.join(old), &data.join(new))?;
            }
        }
        Ok(())
    }

    fn normalize_plugin_manifests(&mut self, root: &Path) -> Result<(), Error> {
        if self
            .retired_user_directories
            .iter()
            .any(|retired| super::same_path(root, retired))
        {
            return Ok(());
        }
        if !root.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(root).map_err(failed("无法读取扩展目录"))? {
            let entry = entry.map_err(failed("无法读取扩展文件"))?;
            let mut path = entry.path();
            check_path(&path)?;
            if !path.is_dir()
                || matches!(
                    entry.file_name().to_str(),
                    Some("webview2" | "tmp" | "updates" | "bridge-cache")
                )
            {
                continue;
            }
            if entry.file_name() == ".sleepy-doll-plugin" {
                let target = path.with_file_name(".pudding-plugin");
                self.rename(&path, &target)?;
                path = target;
            }
            self.normalize_plugin_manifests(&path)?;
        }
        Ok(())
    }

    fn rewrite_documents(&mut self, directory: &Path) -> Result<(), Error> {
        if self
            .retired_user_directories
            .iter()
            .any(|retired| super::same_path(directory, retired))
        {
            return Ok(());
        }
        if !directory.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(directory).map_err(failed("无法读取配置目录"))? {
            let entry = entry.map_err(failed("无法读取配置文件"))?;
            let path = entry.path();
            check_path(&path)?;
            if path.is_dir() {
                if !matches!(
                    entry.file_name().to_str(),
                    Some("webview2" | "tmp" | "updates" | "bridge-cache")
                ) {
                    self.rewrite_documents(&path)?;
                }
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                let original = fs::read(&path).map_err(failed("无法读取迁移配置"))?;
                let Ok(mut document) = serde_json::from_slice::<Value>(&original) else {
                    continue;
                };
                if rewrite(&mut document, path.parent().unwrap(), &self.roots) {
                    self.documents.push((path.clone(), original));
                    crate::config::atomic_write(&path, &document)
                        .map_err(|error| Error::message(error.to_string()))?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn finish(&mut self) -> Result<(), Error> {
        if self.roots.is_empty() {
            self.committed = true;
            return Ok(());
        }
        let mut removals = Vec::new();
        removals.extend(
            self.retired_user_directories
                .iter()
                .cloned()
                .map(super::Removal::Tree),
        );
        if let Some(source) = &self.source {
            if !super::same_path(source, &self.destination) {
                removals.push(super::Removal::Tree(source.clone()));
            }
            removals.push(super::Removal::File(self.destination.join(OLD_PROGRAMS[0])));
            // 可选 Agent Host 并非每个历史安装都有；有新入口时才移除旧入口。
            if self.destination.join("pudding-agent-host.exe").is_file() {
                removals.push(super::Removal::File(self.destination.join(OLD_PROGRAMS[1])));
            }
            // 旧版有桌面入口才恢复桌面入口；开始菜单始终保留。
            for folder in [shell::programs_directory(), shell::desktop_directory()]
                .into_iter()
                .flatten()
            {
                let mut had_shortcut = false;
                for name in NAMES {
                    let link = folder.join(format!("{name}.lnk"));
                    if link.is_file()
                        && shell::shortcut_target(&link)?.is_some_and(|target| {
                            super::same_path(&target, &source.join(OLD_PROGRAMS[0]))
                        })
                    {
                        had_shortcut = true;
                        removals.push(super::Removal::File(link));
                    }
                }
                if had_shortcut
                    || shell::programs_directory()
                        .is_some_and(|programs| super::same_path(&folder, &programs))
                {
                    shell::create_shortcut(
                        &folder.join(super::install::shortcut_name()),
                        &self.destination.join(super::EXECUTABLE),
                        "",
                    )?;
                }
            }
        }
        for (source, target) in &self.roots {
            if is_legacy_name(source)
                && target.file_name().is_some_and(|name| name == NAME)
                && self
                    .source
                    .as_ref()
                    .is_none_or(|installation| !super::same_path(installation, source))
            {
                removals.push(super::Removal::Tree(source.clone()));
            }
        }
        // 程序和配置已集成。后续外壳迁移失败时保留新数据，避免已切换的任务指向空目录。
        self.committed = true;
        let marker = self.destination.join("user/installation-migration.json");
        fs::create_dir_all(marker.parent().unwrap()).map_err(failed("无法创建配置迁移记录目录"))?;
        crate::config::atomic_write(&marker, &serde_json::json!(self.roots))
            .map_err(|error| Error::message(format!("无法记录旧配置路径映射：{error}")))?;
        // 系统任务的 XML 摘要由定时任务模块校验，不能只改 EXE 路径而保留旧摘要。
        let mut configurations = Vec::new();
        for (source, config) in self
            .configurations
            .iter()
            .cloned()
            .chain(self.roots.iter().map(|(previous, current)| {
                (
                    previous.join("user/config.json"),
                    current.join("user/config.json"),
                )
            }))
        {
            if !source.is_file() || !config.is_file() || configurations.contains(&config) {
                continue;
            }
            let mut raw: Value =
                serde_json::from_slice(&fs::read(&config).map_err(failed("无法读取迁移后的配置"))?)
                    .map_err(|error| Error::message(error.to_string()))?;
            crate::config::expand_env(&mut raw)
                .map_err(|error| Error::message(error.to_string()))?;
            if let Some(database) = raw.pointer("/storage/database").and_then(Value::as_str) {
                let database = config.parent().unwrap().join(database);
                if database.is_file() {
                    crate::scheduler::system::migrate_installation(
                        &source,
                        &config,
                        &self.destination.join(super::EXECUTABLE),
                        &database,
                    )
                    .map_err(|error| Error::message(error.to_string()))?;
                }
            }
            configurations.push(config);
        }
        registry::relocate_environment(&self.roots)?;
        if let Some(source) = &self.source {
            registry::remove_legacy(source)?;
        }
        self.guards.clear();
        super::cleanup_after_exit(&removals)?;
        Ok(())
    }
}

impl Drop for Migration {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for (path, bytes) in self.documents.iter().rev() {
            let _ = fs::write(path, bytes);
        }
        for (source, target) in self.renamed.iter().rev() {
            let _ = fs::rename(target, source);
        }
        for path in self.copied.iter().rev() {
            let _ = fs::remove_file(path);
        }
        for path in self.created_directories.iter().rev() {
            let _ = fs::remove_dir(path);
        }
    }
}

/// 只替换完整路径段；不替换聊天文字、模型密钥或用户自己的外部工具路径。
fn rewrite(value: &mut Value, base: &Path, mappings: &[(PathBuf, PathBuf)]) -> bool {
    match value {
        Value::Array(values) => {
            let mut changed = false;
            for value in values {
                changed |= rewrite(value, base, mappings);
            }
            changed
        }
        Value::Object(values) => {
            let mut changed = false;
            for (key, value) in values {
                if matches!(
                    key.as_str(),
                    "token"
                        | "apiKey"
                        | "api_key"
                        | "password"
                        | "secret"
                        | "authorization"
                        | "systemPrompt"
                        | "prompt"
                        | "content"
                        | "text"
                        | "model"
                        | "activeModel"
                ) {
                    continue;
                }
                changed |= rewrite(value, base, mappings);
            }
            changed
        }
        Value::String(text) if !text.contains("${ENV:") && !text.contains("://") => {
            let path = Path::new(text);
            if !path.is_absolute()
                && !text.contains(['/', '\\'])
                && !matches!(
                    text.as_str(),
                    "sleepy-doll.db"
                        | "sleepy-doll.artifacts"
                        | ".sleepy-doll"
                        | ".sleepy-doll-plugin"
                )
            {
                return false;
            }
            let absolute = {
                let mut resolved = PathBuf::new();
                for component in base.join(path).components() {
                    match component {
                        std::path::Component::CurDir => {}
                        std::path::Component::ParentDir => {
                            resolved.pop();
                        }
                        _ => resolved.push(component.as_os_str()),
                    }
                }
                resolved
            };
            let matched = mappings
                .iter()
                .filter_map(|(from, to)| {
                    crate::data_directory::relative_path(from, &absolute).map(|suffix| {
                        (from.components().count(), super::append_suffix(to, &suffix))
                    })
                })
                .max_by_key(|(depth, _)| *depth);
            if let Some((_, relocated)) = matched {
                if super::same_path(&absolute, &relocated) {
                    return false;
                }
                *text = relocated.to_string_lossy().into_owned();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn digest(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file =
        fs::File::open(path).map_err(failed(format!("无法读取待迁移文件 {}", path.display())))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(failed(format!("无法校验待迁移文件 {}", path.display())))?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().to_vec())
}

fn check_path(path: &Path) -> Result<(), Error> {
    super::cleanup::validate(&[super::Removal::File(path.to_path_buf())])?;
    let metadata = fs::symlink_metadata(path).map_err(failed("无法检查迁移路径"))?;
    #[cfg(windows)]
    let link = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let link = metadata.file_type().is_symlink();
    if link || !(metadata.is_file() || metadata.is_dir()) {
        return Err(Error::message(format!(
            "迁移路径不是普通文件或目录：{}",
            path.display()
        )));
    }
    Ok(())
}
