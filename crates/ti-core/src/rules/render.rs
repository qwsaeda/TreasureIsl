//! S3.2 / S8.6 / S12.4 资源/渲染核心正式实现（渲染基线对齐真实资源，去自指）。
//!
//! S11 类型唯一家：`ResourceContainer`/`ResourceRef`/`SceneAsset`/`RenderError`/
//! `RenderPlan` 类型定义住在 `domain::render`（c1 契约 §1 渲染资源域），
//! 本模块只做**装载校验 + 图层顺序**的行为实现，类型经 `pub use` 重导出保持
//! `ti_core::rules::render::*` 兼容（跨包只经 `domain::render`，不直连本模块类型）。
//!
//! 渲染基线（`docs/plan-s8-refactor.md` S8.6）：参考图改用**真实资源**（`legacy/ref/` 容器 +
//! `assets/` 贴图）而非逆向自指的伪参考；`render_parity` 测试改测真实资源对齐。
//! 本模块只负责「装载校验 + 图层顺序」，像素级对齐归 `ti-app` 的 `render_parity`。

/// 渲染资源域类型（唯一家 `domain::render`，c1 契约 §1）。
pub use crate::domain::render::{
    RenderError, RenderPlan, ResourceContainer, ResourceRef, SceneAsset,
};

/// gl.rec 魔数（0x01315A47，`docs/business/economy.md` 容器清单，实测 F-3xx）。
pub const GL_REC_MAGIC: u32 = 0x0131_5A47;

/// 场景管理器：持有可装载资源清单，执行装载与渲染规划。
///
/// 装载校验规则（`docs/business/economy.md` 资源容器 + S8.6 渲染基线，本文件为唯一事实源）：
/// 1. 每个 `ResourceRef` 的容器必须在 `available_containers` 中存在；
/// 2. `Glpics` 容器要求 gl.rec 魔数有效（`magic == GL_REC_MAGIC`）；
/// 3. `Ui` 容器中以 `glmapm` 开头的 `.Map` 资源需通过探测（编号在 `glmapm_indices` 已解锁）；
/// 4. 校验通过的资源按扩展名映射为 `SceneAsset` 变体。
#[derive(Debug, Clone, Default)]
pub struct SceneManager {
    /// 已装载的场景资产列表（按装载顺序，即渲染图层顺序）。
    scenes: Vec<SceneAsset>,
}

impl SceneManager {
    /// 新建空的场景管理器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前已装载的资产（只读引用）。
    #[must_use]
    pub fn assets(&self) -> &[SceneAsset] {
        &self.scenes
    }

    /// 装载场景资源（校验规则见 [`SceneManager`] 文档；成功更新内部 `scenes`，失败保持）。
    ///
    /// # Errors
    /// 任一资源校验失败 → `Err(RenderError { missing })`，列出全部缺失项。
    pub fn load_scene(
        &mut self,
        refs: &[ResourceRef],
        available_containers: &[ResourceContainer],
        gl_rec_magic: u32,
        glmapm_indices: &[u32],
    ) -> Result<Vec<SceneAsset>, RenderError> {
        let mut loaded: Vec<SceneAsset> = Vec::new();
        let mut missing: Vec<ResourceRef> = Vec::new();

        for ref_ in refs {
            let container_ok = available_containers.contains(&ref_.container);
            if !container_ok {
                missing.push(*ref_);
                continue;
            }

            let magic_ok = match ref_.container {
                ResourceContainer::Glpics => gl_rec_magic == GL_REC_MAGIC,
                _ => true,
            };
            if !magic_ok {
                missing.push(*ref_);
                continue;
            }

            let glmapm_ok = if ref_.container == ResourceContainer::Ui
                && ref_.name.starts_with("glmapm")
                && ref_.name.ends_with(".Map")
            {
                let name_bytes = ref_.name.as_bytes();
                let digits = &name_bytes[6..ref_.name.len() - 4];
                let idx: u32 = digits.iter().fold(0u32, |acc, &b| acc * 10 + u32::from(b - b'0'));
                glmapm_indices.contains(&idx)
            } else {
                true
            };
            if !glmapm_ok {
                missing.push(*ref_);
                continue;
            }

            let asset = match ref_.name.rsplit('.').next() {
                Some("pcx") => SceneAsset::Pcx(ref_.name.to_string()),
                Some("mod") => SceneAsset::Mod(ref_.name.to_string()),
                Some("sav") => SceneAsset::Sav(ref_.name.to_string()),
                _ => SceneAsset::Map(ref_.name.to_string()),
            };
            loaded.push(asset);
        }

        if missing.is_empty() {
            self.scenes.clone_from(&loaded);
            Ok(loaded)
        } else {
            Err(RenderError { missing })
        }
    }

    /// 生成渲染计划（图层顺序 = 装载顺序，`blit_count` = 图层数）。
    ///
    /// S8.6：本计划是「图层顺序基线」，像素级对齐由 `ti-app::render_parity` 用真实资源验证。
    #[must_use]
    pub fn render_frame(assets: &[SceneAsset]) -> RenderPlan {
        RenderPlan { layers: assets.to_vec(), blit_count: assets.len() }
    }
}
