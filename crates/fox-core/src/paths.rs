//! 应用数据目录解析：默认位置 + 用户覆盖。
//!
//! - 默认：`{系统数据目录}/RustFox`（开发构建为 `RustFox-dev`，与正式数据隔离）；
//! - 覆盖优先级：`RUSTFOX_DATA_DIR` 环境变量 > 默认目录下的 `data_dir.txt`（设置页写入）> 默认。
//! - 只有绝对路径生效（相对/空内容视为未配置并警告）；调用方负责建目录与可写校验。
//!
//! 注意：fox-secret 的密钥 historically 固定在 `RustFox`（无 -dev 后缀），
//! 因此覆盖解析与默认后缀分开暴露，各调用方保持自己的历史默认行为。

use std::path::PathBuf;

/// 用户覆盖环境变量（绝对路径）。
pub const DATA_DIR_ENV: &str = "RUSTFOX_DATA_DIR";

/// 存覆盖路径的 bootstrap 文件名（位于**默认**数据目录，位置固定才能自举）。
pub const DATA_DIR_BOOTSTRAP_FILE: &str = "data_dir.txt";

/// 测试钩子：覆盖 bootstrap 文件所在目录（生产代码永不设置）。
pub const BOOTSTRAP_DIR_ENV: &str = "RUSTFOX_BOOTSTRAP_DIR";

/// 开发构建的数据目录后缀（与正式版隔离，避免迁移版本污染）。
pub fn default_data_subdir() -> &'static str {
    if cfg!(debug_assertions) {
        "RustFox-dev"
    } else {
        "RustFox"
    }
}

/// 系统数据目录（`dirs::data_dir`，不可用时回退当前目录）。
pub fn system_data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// 默认数据目录（含 dev 后缀）。
pub fn default_data_dir() -> PathBuf {
    system_data_dir().join(default_data_subdir())
}

/// bootstrap 文件路径（覆盖目录配置本身必须放在固定位置）。
pub fn bootstrap_path() -> PathBuf {
    if let Some(dir) = std::env::var_os(BOOTSTRAP_DIR_ENV) {
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            return dir.join(DATA_DIR_BOOTSTRAP_FILE);
        }
        tracing::warn!("{BOOTSTRAP_DIR_ENV} 非绝对路径，已忽略（测试钩子仅接受绝对路径）");
    }
    default_data_dir().join(DATA_DIR_BOOTSTRAP_FILE)
}

/// 用户覆盖的数据目录（环境变量优先，其次 bootstrap 文件；均无效时 None）。
pub fn data_dir_override() -> Option<PathBuf> {
    if let Some(raw) = std::env::var_os(DATA_DIR_ENV) {
        let p = PathBuf::from(raw);
        if p.is_absolute() {
            return Some(p);
        }
        tracing::warn!("{DATA_DIR_ENV} 非绝对路径，已忽略");
    }
    read_bootstrap_override()
}

/// 读取 bootstrap 文件中的覆盖路径（空文件/相对路径视为未配置）。
fn read_bootstrap_override() -> Option<PathBuf> {
    let raw = std::fs::read_to_string(bootstrap_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let p = PathBuf::from(trimmed);
    if !p.is_absolute() {
        tracing::warn!("bootstrap 数据目录非绝对路径，已忽略");
        return None;
    }
    Some(p)
}

/// 解析生效的数据目录（覆盖 > 默认）。
pub fn resolve_data_dir_with_default(default: PathBuf) -> PathBuf {
    data_dir_override().unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// 环境变量是进程全局的：涉及它的用例串行执行。
    fn env_serial() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    struct EnvGuard {
        key: &'static str,
        prev: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var_os(key);
            // 测试串行持有 env_serial 锁（edition 2021 下 set_var 本就是安全函数）。
            std::env::set_var(key, value);
            Self { key, prev }
        }

        fn remove(key: &'static str) -> Self {
            let prev = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn temp_home(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("rustfox-paths-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("建临时目录");
        dir
    }

    #[test]
    fn env_override_wins_over_bootstrap_and_default() {
        let _serial = env_serial();
        let home = temp_home("env-wins");
        let _env = EnvGuard::set(DATA_DIR_ENV, home.join("custom").to_str().unwrap());
        let _boot = EnvGuard::set(BOOTSTRAP_DIR_ENV, home.join("bootstrap").to_str().unwrap());
        std::fs::create_dir_all(home.join("bootstrap")).expect("建 bootstrap 目录");
        std::fs::write(
            home.join("bootstrap").join(DATA_DIR_BOOTSTRAP_FILE),
            home.join("from-file").to_str().unwrap(),
        )
        .expect("写 bootstrap");
        assert_eq!(
            data_dir_override(),
            Some(home.join("custom")),
            "环境变量优先于 bootstrap 文件"
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn bootstrap_used_when_no_env() {
        let _serial = env_serial();
        let home = temp_home("bootstrap");
        let _env = EnvGuard::remove(DATA_DIR_ENV);
        let _boot = EnvGuard::set(BOOTSTRAP_DIR_ENV, home.join("bootstrap").to_str().unwrap());
        std::fs::create_dir_all(home.join("bootstrap")).expect("建 bootstrap 目录");
        std::fs::write(
            home.join("bootstrap").join(DATA_DIR_BOOTSTRAP_FILE),
            home.join("from-file").to_str().unwrap(),
        )
        .expect("写 bootstrap");
        assert_eq!(
            data_dir_override(),
            Some(home.join("from-file")),
            "无环境变量时采用 bootstrap 文件"
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn relative_and_empty_configs_ignored() {
        let _serial = env_serial();
        let home = temp_home("invalid");
        // 相对路径的环境变量被忽略
        let _env = EnvGuard::set(DATA_DIR_ENV, "relative/path");
        assert_eq!(data_dir_override(), None);
        drop(_env);
        // 空 bootstrap 文件被忽略
        let _env = EnvGuard::remove(DATA_DIR_ENV);
        let _boot = EnvGuard::set(BOOTSTRAP_DIR_ENV, home.join("bootstrap").to_str().unwrap());
        std::fs::create_dir_all(home.join("bootstrap")).expect("建 bootstrap 目录");
        std::fs::write(home.join("bootstrap").join(DATA_DIR_BOOTSTRAP_FILE), "  \n")
            .expect("写空 bootstrap");
        assert_eq!(data_dir_override(), None);
        // 相对路径的 bootstrap 被忽略
        std::fs::write(
            home.join("bootstrap").join(DATA_DIR_BOOTSTRAP_FILE),
            "relative/path",
        )
        .expect("写相对 bootstrap");
        assert_eq!(data_dir_override(), None);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn resolve_falls_back_to_default() {
        let _serial = env_serial();
        let home = temp_home("fallback");
        let _env = EnvGuard::remove(DATA_DIR_ENV);
        let _boot = EnvGuard::set(BOOTSTRAP_DIR_ENV, home.join("missing").to_str().unwrap());
        let def = home.join("default");
        assert_eq!(resolve_data_dir_with_default(def.clone()), def);
        let _ = std::fs::remove_dir_all(&home);
    }
}
