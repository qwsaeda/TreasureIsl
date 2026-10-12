//! 渲染资源域类型（业务文档：`docs/business/loop_core.md` §资源管线）。
//!
//! 资源容器分类 + 资源引用 + 场景资产 + 渲染计划（图层顺序）。
//! **容器与文件名的对应关系是实测事实**（`docs/reverse/formats/lucencypackage.md`），
//! 但**渲染管线本身是现代工程实现**（GPU 逐精灵 batch，见 `docs/modernization.md` §1）。

/// 资源容器类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceContainer {
    /// 场景 `.Map` + 控件表。
    Ui,
    /// 精灵 PCX（角色立绘/棋子/地标）。
    Glpics,
    /// 卡牌数据。
    Cards,
    /// 商品数据。
    Goods,
    /// 音频（BGM + 音效）。
    Music,
}

impl ResourceContainer {
    /// 全部容器（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [ResourceContainer; 5] {
        [Self::Ui, Self::Glpics, Self::Cards, Self::Goods, Self::Music]
    }

    /// 容器可读名称（错误消息用）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ui => "ui.pak",
            Self::Glpics => "glpics.pak",
            Self::Cards => "cards.lpk",
            Self::Goods => "goods.lpk",
            Self::Music => "music.lpk",
        }
    }
}

/// 资源引用：容器 + 资源名。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRef {
    /// 所属容器。
    pub container: ResourceContainer,
    /// 资源名（`.Map`/`.pcx`/`.sav` 等）。
    pub name: &'static str,
}

impl ResourceRef {
    /// 构造资源引用。
    #[must_use]
    pub const fn new(container: ResourceContainer, name: &'static str) -> Self {
        Self { container, name }
    }
}

/// 装载后的场景资产。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneAsset {
    /// 地图场景（`.Map`）。
    Map(String),
    /// 精灵图（`.pcx`）。
    Pcx(String),
    /// 模型/控件表（`.mod`）。
    Mod(String),
    /// 存档（`.sav`）。
    Sav(String),
}

/// 装载错误：缺失容器或资源名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    /// 缺失的资源引用。
    pub missing: Vec<ResourceRef>,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "render: {} missing resource(s)", self.missing.len())?;
        for ref_ in &self.missing {
            write!(f, "\n  - {} / {}", ref_.container.label(), ref_.name)?;
        }
        Ok(())
    }
}

/// 渲染计划：图层顺序 = 装载顺序。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlan {
    /// 图层（按装载顺序 = blit 顺序）。
    pub layers: Vec<SceneAsset>,
    /// blit 调用次数（= 图层数）。
    pub blit_count: usize,
}
