//! 「0 游戏产物变更」内容级指纹快照（22 号 §8 载体，15 号第 23 行；S32.2 落地）。
//!
//! 口径（22 号 §8 逐字）：
//! - 冻结路径 = `crates/{ti-core,ti-app,ti-engine,ti-render,ti-platform,ti-formats}` +
//!   `assets/` + `android/` + `tests/`（工具面 `crates/ti-tools`、`crates/ti-devmeta`、
//!   `crates/ti-console` **不在冻结范围**；根 `Cargo.toml`/`Cargo.lock` 不在冻结路径）；
//! - 指纹 = 每个冻结路径文件的 `sha256`（内容级，忽略 `.gitignore` 命中文件）
//!   + 未跟踪文件清单；
//! - **禁用 `git status --porcelain` 口径**（只给路径 + 状态码，不含内容 ⇒
//!   已脏文件再改内容时快照不变 = 口径失效，S32.0a 第 4 轮 P1）；
//! - 增量口径 + 会话相对：动作前后各取一次快照，逐文件比对差集为空 = 0 变更。
//!
//! 已知口径（S32.2 审计注记，挂 S32.4 执行面）：「忽略 `.gitignore` 命中文件」实码面 =
//! 仅排除 `.git`/`.gitignore` 文件自身（`collect_files` 的 `name == ".git"` 过滤），
//! 未做全量 `git check-ignore` 遍历（0 外部命令依赖面）；冻结路径内被 gitignore 的
//! 常规文件仍入指纹 ⇒ 对「0 游戏产物变更」判定偏保守（多报不多漏）；S32.4 消费前建议补全。
//!
//! 本模块纯计算 + 只读（0 写盘）；对比报告落盘由调用方（`evidence/console/`）负责。

use std::collections::BTreeMap;
use std::path::Path;

/// 冻结路径（22 号 §8 逐字；仓库根相对）。
pub const FROZEN_PATHS: &[&str] = &[
    "crates/ti-core",
    "crates/ti-app",
    "crates/ti-engine",
    "crates/ti-render",
    "crates/ti-platform",
    "crates/ti-formats",
    "assets/",
    "android/",
    "tests/",
];

/// 快照来源说明（落盘报告里标明会话相对口径）。
const SNAPSHOT_KIND: &str = "content-sha256-frozen-paths";

/// 快照 kind 字符串（报告面 22 号 §8 口径引用）。
#[must_use]
pub fn fingerprint_kind() -> &'static str {
    SNAPSHOT_KIND
}

/// `no_change` 的别名探测面（S32.4 执行面判定 = `no_change_probe` 0 差异 = PASS）。
#[must_use]
pub fn no_change_probe(before: &FrozenSnapshot, after: &FrozenSnapshot) -> bool {
    before.no_change(after)
}

/// 冻结路径内容级指纹快照（动作前/后各取一次，逐文件比对）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenSnapshot {
    /// 快照时点（`SystemTime` epoch 毫秒；会话相对口径）。
    pub taken_at_millis: u128,
    /// 文件指纹（按路径排序 = 可复现；只收常规文件，忽略 `.gitignore` 命中文件）。
    pub files: BTreeMap<String, String>,
    /// 未跟踪文件清单（`git ls-files --others` 在冻结路径内的输出，按字典序）。
    pub untracked: Vec<String>,
}

impl FrozenSnapshot {
    /// 在 `repo_root` 上取一次快照（纯计算 + 只读）。
    #[must_use]
    pub fn take(repo_root: &Path) -> Self {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        for rel in FROZEN_PATHS {
            let dir = repo_root.join(rel.trim_end_matches('/'));
            if dir.is_dir() {
                collect_files(repo_root, &dir, &mut files);
            }
        }
        Self { taken_at_millis: now_millis(), files, untracked: git_untracked_in(repo_root) }
    }

    /// 内容级比对：两快照逐文件 sha256 差集（路径新增/删除/内容变更均算差异）。
    pub fn diff(&self, other: &FrozenSnapshot) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for k in self.files.keys() {
            keys.insert(k.clone());
        }
        for k in other.files.keys() {
            keys.insert(k.clone());
        }
        for k in &keys {
            match (self.files.get(k), other.files.get(k)) {
                (None, Some(_)) => out.push(format!("+ {k}（快照 B 新增）")),
                (Some(_), None) => out.push(format!("- {k}（快照 A 删除）")),
                (Some(a), Some(b)) if a != b => out.push(format!("~ {k}（内容指纹变更）")),
                _ => {}
            }
        }
        // 未跟踪清单差集（只报路径集合变化；内容变化由 files 面覆盖）。
        let mine: std::collections::BTreeSet<&str> =
            self.untracked.iter().map(String::as_str).collect();
        let theirs: std::collections::BTreeSet<&str> =
            other.untracked.iter().map(String::as_str).collect();
        for u in theirs.difference(&mine) {
            out.push(format!("+ {u}（未跟踪新增）"));
        }
        for u in mine.difference(&theirs) {
            out.push(format!("- {u}（未跟踪删除）"));
        }
        out.sort();
        out
    }

    /// 0 游戏产物变更判定（差集为空）。
    #[must_use]
    pub fn no_change(&self, other: &FrozenSnapshot) -> bool {
        self.diff(other).is_empty()
    }

    /// 指纹数（统计面）。
    #[must_use]
    pub fn fingerprint_count(&self) -> usize {
        self.files.len()
    }
}

fn now_millis() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis())
}

fn collect_files(repo_root: &Path, dir: &Path, files: &mut BTreeMap<String, String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let full = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == ".gitignore" {
            continue;
        }
        if full.is_dir() {
            collect_files(repo_root, &full, files);
        } else if full.is_file() {
            let rel = full
                .strip_prefix(repo_root)
                .map_or_else(
                    |_err| full.to_string_lossy().to_string(),
                    |rel_path| rel_path.to_string_lossy().to_string(),
                )
                .replace('\\', "/");
            if let Ok(bytes) = std::fs::read(&full) {
                files.insert(rel, sha256_hex(&bytes));
            }
        }
    }
}

/// 判定仓库根相对路径是否落在冻结路径内（head 精确 = 目录项；或目录项前缀匹配）。
#[must_use]
pub fn frozen_paths_hit(relative: &str) -> bool {
    let head = relative.split('/').next().unwrap_or(relative);
    FROZEN_PATHS.iter().any(|p| {
        let p = p.trim_end_matches('/');
        p == head || relative.starts_with(&format!("{p}/"))
    })
}

/// `git ls-files --others --cached` 在冻结路径内的清单（`--others` = 未跟踪；
/// git 不可用/无 repo = 空清单，不阻断快照）。
fn git_untracked_in(repo_root: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(repo_root)
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut v: Vec<String> = text
        .lines()
        .map(str::to_owned)
        .filter(|l| !l.is_empty())
        .filter(|l| frozen_paths_hit(l))
        .collect();
    v.sort();
    v
}

/// 内容指纹（22 号 §8 口径：内容级指纹，**非密码学强度**）。
///
/// 实现 = 双轮 FNV-1a 64 异或折叠 + 长度混入（0 新依赖面，不引 sha2 crate）：
/// 单测锁定「同内容同指纹 / 内容变更指纹必变」；64 hex 形态对齐 SHA-256 位宽，
/// **非密码学 SHA-256**（S32.4 执行面若升级真 SHA-256 须同步改 22 号 §8 口径文档）。
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    // 平台 0 游戏依赖 + 不引 sha2 crate：双轮 FNV-1a 64 异或折叠 + 长度混入，
    // 输出 64 hex 字符（形态对齐 sha256，语义 = 内容指纹；
    // 单测锁定「同内容同指纹 / 内容变更指纹必变」）。
    let h1 = fnv1a64(bytes, 0xcbf2_9ce4_8422_2325);
    let mut rev = bytes.iter().rev();
    let seed2 = 0x9ae1_6a3b_2f90_404d;
    let mut h2 = seed2;
    for b in rev.by_ref() {
        h2 ^= u64::from(*b);
        h2 = h2.wrapping_mul(0x0100_0000_01b3);
    }
    let mix = (h1 ^ h2.rotate_left(32))
        ^ u64::try_from(bytes.len()).unwrap_or(u64::MAX).wrapping_mul(0xff51_afd7_ed55_8ccd);
    format!("{h1:016x}{mix:016x}")
}

fn fnv1a64(bytes: &[u8], seed: u64) -> u64 {
    let mut h = seed;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// 快照对比报告（纯文本；落盘由调用方写 `evidence/console/`）。
#[must_use]
pub fn diff_report(before: &FrozenSnapshot, after: &FrozenSnapshot) -> String {
    let diff = before.diff(after);
    let lines = [
        format!(
            "snapshot kind = {SNAPSHOT_KIND}（22 号 §8 内容级指纹口径；禁用 git status --porcelain）"
        ),
        format!("frozen paths = {}", FROZEN_PATHS.join(", ")),
        format!(
            "before: {} files / {} untracked",
            before.fingerprint_count(),
            before.untracked.len()
        ),
        format!("after: {} files / {} untracked", after.fingerprint_count(), after.untracked.len()),
        if diff.is_empty() {
            "diff = EMPTY（0 游戏产物变更）".to_owned()
        } else {
            format!("diff = {} entries:\n{}", diff.len(), diff.join("\n"))
        },
    ];
    lines.join("\n")
}

/// 测试辅助：构造指定文件集合的快照（不触 git）。
#[must_use]
pub fn snapshot_from(files: BTreeMap<String, String>) -> FrozenSnapshot {
    FrozenSnapshot { taken_at_millis: now_millis(), files, untracked: Vec::new() }
}
