//! oracle：S3.2 差分基准，直译场景装载/渲染管线，测试专用。
//!
//! 依据 `evidence/S3.2.log` 的 REA 实测资源清单与 `docs/spec/main_loop.md` §4：
//! - 资源容器：`UI.pak`（场景 `.Map` + 控件表）、`glpics.pak`（精灵 PCX）、
//!   `cards.lpk`（卡牌）、`goods.lpk`（商品）、`music.lpk`（音频）；
//! - 场景装载器族（`sub_42574`/`3ECDC`/`402D8`/`40D80`/`2077C`）通过通用装载
//!   `sub_37EDC` 从容器索引取记录；
//! - 渲染管线：WinCE GX API（`GXBeginDraw` → 图层 blit → `GXEndDraw`），
//!   图层顺序 = 装载顺序；
//! - 存档：`gl0.sav`..`gl4.sav` + `gl.rec`（魔数 0x01315A47）+ `settings.dat`。
//!
//! 本 oracle 模拟装载校验（容器存在性、gl.rec 魔数、glmapm 探测），
//! 生成逐位一致的差分基准，供 `rules::render::SceneManager` 对拍。

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
    /// 全部容器（按枚举声明顺序，diff 测试遍历用）。
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

    /// 容器可读名称（错误消息用）。
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

/// 资源引用：容器 + 资源名（`&'static str`，测试代码外禁止 unwrap）。
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

/// 装载后的场景资产（与 `ResourceRef` 一一对应）。
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

/// 装载错误：缺失容器 + 缺失资源名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    /// 缺失的资源引用（容器不存在或资源名不在容器索引中）。
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

/// 渲染计划：图层顺序 = 装载顺序（`GXBeginDraw` → blit → `GXEndDraw`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlan {
    /// 图层（按装载顺序，即 blit 顺序）。
    pub layers: Vec<SceneAsset>,
    /// blit 调用次数（= 图层数，每层一次 GX blit）。
    pub blit_count: usize,
}

/// gl.rec 魔数（`evidence/S3.2.log`：0x01315A47）。
const GL_REC_MAGIC: u32 = 0x0131_5A47;

/// 模拟 gl.rec 魔数校验（装载 `sub_40D80` 时执行）。
fn check_gl_rec_magic(magic: u32) -> bool {
    magic == GL_REC_MAGIC
}

/// 模拟 glmapm 探测（`sub_402D8`：`CreateFileW` 直读 `data/glmapm0x.Map` 判断存在性）。
///
/// 返回 true 表示该地图已解锁（文件存在）。
fn probe_glmapm(index: u32, available: &[u32]) -> bool {
    available.contains(&index)
}

/// oracle：模拟场景装载。
///
/// 校验规则（直译 `sub_40D80`/`sub_402D8`/`sub_42574` 等装载器行为）：
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

        // gl.rec 魔数校验（仅 Glpics 容器需要，sub_40D80 行为）
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
