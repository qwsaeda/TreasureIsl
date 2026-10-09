//! S3.2 资源/渲染核心正式实现。
//!
//! 与 `oracle::render` 行为逐位一致（同一输入产生逐位相同的 `Result<Vec<SceneAsset>, RenderError>`
//! 与 `RenderPlan`），但结构不同：用 `SceneManager` 结构体 + 方法组织，
//! 而非自由函数。
//!
//! 依据 `docs/spec/main_loop.md` §4 与 `evidence/S3.2.log`：
//! - 场景装载器族：`sub_42574`/`3ECDC`/`402D8`/`40D80`/`2077C`；
//! - 通用装载 `sub_37EDC`：从容器索引取 PCX/控件表；
//! - 渲染：WinCE GX API，图层顺序 = 装载顺序；
//! - 存档：`gl0.sav`..`gl4.sav` + `gl.rec`（魔数 0x01315A47）+ `settings.dat`。

/// 资源容器类型（`evidence/S3.2.log` 关键发现 §3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceContainer {
    /// UI.pak：场景 .Map + 控件表。
    Ui,
    /// glpics.pak：精灵 PCX（face_*/troop/ship/flag/landmark 等）。
    Glpics,
    /// cards.lpk：卡牌数据。
    Cards,
    /// goods.lpk：商品数据。
    Goods,
    /// music.lpk：音频（victory.wav / glmusic.mod 等）。
    Music,
}

impl ResourceContainer {
    /// 全部容器（按枚举声明顺序）。
    #[must_use]
    pub const fn all() -> [ResourceContainer; 5] {
        [
            ResourceContainer::Ui,
            ResourceContainer::Glpics,
            ResourceContainer::Cards,
            ResourceContainer::Goods,
            ResourceContainer::Music,
        ]
    }

    /// 容器可读名称。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ui => "UI.pak",
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
    /// 资源名（`.Map`/`.pcx`/`.sav`/`.mod` 等小写扩展名形式）。
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
    /// 地图场景（`.Map` 文件）。
    Map(String),
    /// 精灵图（`.pcx` 文件）。
    Pcx(String),
    /// 模型/控件表（`.mod` 文件）。
    Mod(String),
    /// 存档（`.sav` 文件）。
    Sav(String),
}

/// 装载错误。
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
    /// 图层（按装载顺序，即 blit 顺序）。
    pub layers: Vec<SceneAsset>,
    /// blit 调用次数（= 图层数）。
    pub blit_count: usize,
}

/// gl.rec 魔数（0x01315A47）。
const GL_REC_MAGIC: u32 = 0x0131_5A47;

/// 场景管理器：持有可装载资源清单，执行装载与渲染规划。
#[derive(Debug, Clone)]
pub struct SceneManager {
    /// 已装载的场景资产列表（按装载顺序，即渲染图层顺序）。
    scenes: Vec<SceneAsset>,
}

impl SceneManager {
    /// 新建空的场景管理器。
    #[must_use]
    pub fn new() -> Self {
        Self { scenes: Vec::new() }
    }

    /// 当前已装载的资产（只读引用）。
    #[must_use]
    pub fn assets(&self) -> &[SceneAsset] {
        &self.scenes
    }

    /// 装载场景资源。
    ///
    /// 行为与 `oracle::render::oracle_load_scene` 逐位一致：
    /// 1. 每个 `ResourceRef` 的容器必须在 `available_containers` 中存在；
    /// 2. `Glpics` 容器要求 gl.rec 魔数有效（`magic == 0x01315A47`）；
    /// 3. `Ui` 容器中以 `glmapm` 开头的 `.Map` 资源需通过探测
    ///    （编号必须在 `glmapm_indices` 中已解锁）；
    /// 4. 校验通过的资源按扩展名映射为 `SceneAsset` 变体。
    ///
    /// 成功后更新内部 `scenes`；失败时 `scenes` 保持不变。
    ///
    /// # Errors
    ///
    /// 任一资源校验失败时返回 `Err(RenderError { missing })`，列出全部缺失项。
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

    /// 生成渲染计划。
    ///
    /// 图层顺序 = 装载顺序（`assets` 参数即当前已装载资产或指定资产列表），
    /// `blit_count` = 图层数。
    ///
    /// 行为与 `oracle::render::oracle_render_frame` 逐位一致。
    #[must_use]
    pub fn render_frame(assets: &[SceneAsset]) -> RenderPlan {
        RenderPlan { layers: assets.to_vec(), blit_count: assets.len() }
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}
