//! oracle 场景装载/渲染管线差分基准（S11 降级为纯基准函数；类型在 domain::render）。
//!
//! 差分基准 = 从 `legacy/docs/spec/*.md` 伪代码直译的纯函数，供 `tests/diff_*.rs` 对账
//! 正向实现（`rules::`）；本模块**不参与运行时**，S11 起物理归档 `legacy/` 前保留为对照基准。
#![allow(clippy::doc_markdown)]

pub use crate::domain::render::{
    RenderError, RenderPlan, ResourceContainer, ResourceRef, SceneAsset,
};

/// gl.rec 魔数（`evidence/S3.2.log`：0x01315A47）。
const GL_REC_MAGIC: u32 = 0x0131_5A47;

/// 模拟 gl.rec 魔数校验（装载 `index` 时执行）。
fn check_gl_rec_magic(magic: u32) -> bool {
    magic == GL_REC_MAGIC
}

/// 模拟 glmapm 探测（`selectmap`：`CreateFileW` 直读 `data/glmapm0x.Map` 判断存在性）。
///
/// 返回 true 表示该地图已解锁（文件存在）。
fn probe_glmapm(index: u32, available: &[u32]) -> bool {
    available.contains(&index)
}

/// oracle：模拟场景装载。
///
/// 校验规则（直译 `index`/`selectmap`/`intro` 等装载器行为）：
/// 1. 每个 `ResourceRef` 的容器必须在 `available_containers` 中存在；
/// 2. `Glpics` 容器要求 gl.rec 魔数有效（`magic == 0x01315A47`）；
/// 3. `Ui` 容器中的 `.Map` 场景若名字以 `glmapm` 开头，需通过探测
///    （`glmapm_indices` 中的编号必须已解锁）；
/// 4. 校验通过的资源按 `ResourceRef` 扩展名映射为 `SceneAsset` 变体。
///
/// # Errors
///
/// 任一资源校验失败时返回 `Err(RenderError { missing })`，列出全部缺失项。
pub fn oracle_load_scene(
    refs: &[ResourceRef],
    available_containers: &[ResourceContainer],
    gl_rec_magic: u32,
    glmapm_indices: &[u32],
) -> Result<Vec<SceneAsset>, RenderError> {
    let mut assets: Vec<SceneAsset> = Vec::new();
    let mut missing: Vec<ResourceRef> = Vec::new();

    for ref_ in refs {
        let container_ok = available_containers.contains(&ref_.container);
        if !container_ok {
            missing.push(*ref_);
            continue;
        }

        // gl.rec 魔数校验（仅 Glpics 容器需要，index 行为）
        let magic_ok = match ref_.container {
            ResourceContainer::Glpics => check_gl_rec_magic(gl_rec_magic),
            _ => true,
        };
        if !magic_ok {
            missing.push(*ref_);
            continue;
        }

        // glmapm 探测（Ui 容器中以 "glmapm" 开头的 .Map 资源）
        let glmapm_ok = if ref_.container == ResourceContainer::Ui
            && ref_.name.starts_with("glmapm")
            && ref_.name.ends_with(".Map")
        {
            let name_bytes = ref_.name.as_bytes();
            let digits = &name_bytes[6..ref_.name.len() - 4];
            let idx: u32 = digits.iter().fold(0u32, |acc, &b| acc * 10 + u32::from(b - b'0'));
            probe_glmapm(idx, glmapm_indices)
        } else {
            true
        };
        if !glmapm_ok {
            missing.push(*ref_);
            continue;
        }

        // 扩展名 → SceneAsset 变体
        let asset = match ref_.name.rsplit('.').next() {
            Some("pcx") => SceneAsset::Pcx(ref_.name.to_string()),
            Some("mod") => SceneAsset::Mod(ref_.name.to_string()),
            Some("sav") => SceneAsset::Sav(ref_.name.to_string()),
            _ => SceneAsset::Map(ref_.name.to_string()),
        };
        assets.push(asset);
    }

    if missing.is_empty() { Ok(assets) } else { Err(RenderError { missing }) }
}

/// oracle：生成渲染计划。
///
/// 图层顺序 = 装载顺序（`assets` 参数即装载结果），
/// `blit_count` = 图层数（每层一次 GX blit 调用）。
#[must_use]
pub fn oracle_render_frame(assets: &[SceneAsset]) -> RenderPlan {
    RenderPlan { layers: assets.to_vec(), blit_count: assets.len() }
}
