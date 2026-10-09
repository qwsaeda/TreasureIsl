//! S3.2 资源/渲染核心差分测试。
//!
//! 遍历全部容器 × 代表性资源名 × 装载组合，oracle 与 impl 的输出逐位一致
//! （`Result` 的 Ok/Err + assets + `blit_count` 全等）。
//! 另含 2 个语义测试。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::render::{
    RenderError as OError, RenderPlan as OPlan, ResourceContainer as OContainer,
    ResourceRef as ORef, SceneAsset as OAsset, oracle_load_scene, oracle_render_frame,
};
use ti_core::rules::render::{
    RenderError as RError, RenderPlan as RPlan, ResourceContainer as RContainer,
    ResourceRef as RRef, SceneAsset as RAsset, SceneManager,
};

/// 将 oracle 容器转为 impl 容器。
fn c(c: OContainer) -> RContainer {
    match c {
        OContainer::Ui => RContainer::Ui,
        OContainer::Glpics => RContainer::Glpics,
        OContainer::Cards => RContainer::Cards,
        OContainer::Goods => RContainer::Goods,
        OContainer::Music => RContainer::Music,
    }
}

/// 将 oracle 资源引用转为 impl 资源引用。
fn cr(refs: &[ORef]) -> Vec<RRef> {
    refs.iter().map(|r| RRef::new(c(r.container), r.name)).collect()
}

/// 将 oracle 容器列表转为 impl 容器列表。
fn cc(containers: &[OContainer]) -> Vec<RContainer> {
    containers.iter().copied().map(c).collect()
}

/// 将 oracle 资产转为 impl 资产。
fn ca(asset: &OAsset) -> RAsset {
    match asset {
        OAsset::Map(s) => RAsset::Map(s.clone()),
        OAsset::Pcx(s) => RAsset::Pcx(s.clone()),
        OAsset::Mod(s) => RAsset::Mod(s.clone()),
        OAsset::Sav(s) => RAsset::Sav(s.clone()),
    }
}

/// 将 oracle 资产列表转为 impl 资产列表。
fn ca_vec(assets: &[OAsset]) -> Vec<RAsset> {
    assets.iter().map(ca).collect()
}

/// 比较 oracle 与 impl 的装载结果是否逐位一致。
fn assert_load_equal(oracle: &Result<Vec<OAsset>, OError>, impl_out: &Result<Vec<RAsset>, RError>) {
    match (oracle, impl_out) {
        (Ok(oa), Ok(ia)) => {
            assert_eq!(oa.len(), ia.len(), "资产数不一致");
            for (o, i) in oa.iter().zip(ia.iter()) {
                assert_eq!(format!("{o:?}"), format!("{i:?}"), "资产内容不一致");
            }
        }
        (Err(oe), Err(ie)) => {
            assert_eq!(oe.missing.len(), ie.missing.len(), "missing 数不一致");
            for (o, i) in oe.missing.iter().zip(ie.missing.iter()) {
                assert_eq!(c(o.container), i.container, "missing 容器不一致");
                assert_eq!(o.name, i.name, "missing 资源名不一致");
            }
        }
        (o, i) => {
            unreachable!("Ok/Err 状态不一致: oracle={o:?} impl={i:?}");
        }
    }
}

/// 比较 oracle 与 impl 的渲染计划是否逐位一致。
fn assert_plan_equal(plan_o: &OPlan, plan_i: &RPlan) {
    assert_eq!(plan_o.blit_count, plan_i.blit_count, "blit_count 不一致");
    assert_eq!(plan_o.layers.len(), plan_i.layers.len(), "layers 数不一致");
    for (o, i) in plan_o.layers.iter().zip(plan_i.layers.iter()) {
        assert_eq!(format!("{o:?}"), format!("{i:?}"), "图层内容不一致");
    }
}

/// 代表性资源名（每容器 ≥ 5 名，依据 `evidence/S3.2.log` 实测清单）。
const UI_NAMES: &[&str] =
    &["intro.Map", "selectrole.Map", "selectmap.Map", "index.Map", "main.Map"];
const GLPICS_NAMES: &[&str] =
    &["face_panda.pcx", "face_tichy.pcx", "troop_all.pcx", "ship_all.pcx", "Landmark_city_1.pcx"];
const CARDS_NAMES: &[&str] = &["cards.ini", "event_001", "event_002", "event_003", "event_004"];
const GOODS_NAMES: &[&str] = &["goods.mdu", "goods_01", "goods_02", "goods_03", "goods_04"];
const MUSIC_NAMES: &[&str] =
    &["victory.wav", "levelup.wav", "attack.wav", "invalid.wav", "glmusic.mod"];

const GL_REC_MAGIC_OK: u32 = 0x0131_5A47;
const GL_REC_MAGIC_BAD: u32 = 0xDEAD_BEEF;

const ALL_CONTAINERS: &[OContainer] =
    &[OContainer::Ui, OContainer::Glpics, OContainer::Cards, OContainer::Goods, OContainer::Music];

const GLMAPM_INDICES: &[u32] = &[1, 2];

fn o_refs(names: &[&'static str], container: OContainer) -> Vec<ORef> {
    names.iter().map(|n| ORef::new(container, n)).collect()
}

/// 单容器装载：oracle vs impl 逐位一致。
fn check_single_container(container: OContainer, names: &[&'static str]) {
    let orefs = o_refs(names, container);
    let irefs = cr(&orefs);
    let icontainers = cc(ALL_CONTAINERS);

    let oracle = oracle_load_scene(&orefs, ALL_CONTAINERS, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    if let Ok(assets) = &oracle {
        let plan_o = oracle_render_frame(assets);
        let plan_i: RPlan = SceneManager::render_frame(&ca_vec(assets));
        assert_plan_equal(&plan_o, &plan_i);
    }
}

#[test]
fn diff_all_single_containers() {
    check_single_container(OContainer::Ui, UI_NAMES);
    check_single_container(OContainer::Glpics, GLPICS_NAMES);
    check_single_container(OContainer::Cards, CARDS_NAMES);
    check_single_container(OContainer::Goods, GOODS_NAMES);
    check_single_container(OContainer::Music, MUSIC_NAMES);
}

/// 多容器组合装载（Ui + Glpics + Cards 混合）。
#[test]
fn diff_mixed_container_combination() {
    let orefs: Vec<ORef> = [
        o_refs(UI_NAMES, OContainer::Ui),
        o_refs(GLPICS_NAMES, OContainer::Glpics),
        o_refs(CARDS_NAMES, OContainer::Cards),
    ]
    .concat();
    let irefs = cr(&orefs);
    let icontainers = cc(ALL_CONTAINERS);

    let oracle = oracle_load_scene(&orefs, ALL_CONTAINERS, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    if let Ok(assets) = &oracle {
        let plan_o = oracle_render_frame(assets);
        let plan_i: RPlan = SceneManager::render_frame(&ca_vec(assets));
        assert_plan_equal(&plan_o, &plan_i);
    }
}

/// gl.rec 魔数错误 → Glpics 容器全部资源缺失。
#[test]
fn diff_bad_gl_rec_magic() {
    let orefs: Vec<ORef> =
        [o_refs(UI_NAMES, OContainer::Ui), o_refs(GLPICS_NAMES, OContainer::Glpics)].concat();
    let irefs = cr(&orefs);
    let icontainers = cc(ALL_CONTAINERS);

    let oracle = oracle_load_scene(&orefs, ALL_CONTAINERS, GL_REC_MAGIC_BAD, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &icontainers, GL_REC_MAGIC_BAD, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    // 魔数错误时 Ui(5) 成功，Glpics(5) 全部 missing → 整体 Err
    match &oracle {
        Err(err) => {
            assert_eq!(err.missing.len(), 5, "魔数错误时 5 个 Glpics 资源均应 missing");
            for r in &err.missing {
                assert_eq!(r.container, OContainer::Glpics);
            }
        }
        Ok(_) => {
            unreachable!("魔数错误时 Glpics 资源应全部缺失，结果应为 Err");
        }
    }
}

/// glmapm 探测：01/02 已解锁，03 未解锁。
#[test]
fn diff_glmapm_probe() {
    let orefs = vec![
        ORef::new(OContainer::Ui, "glmapm01.Map"),
        ORef::new(OContainer::Ui, "glmapm02.Map"),
        ORef::new(OContainer::Ui, "glmapm03.Map"),
    ];
    let irefs = cr(&orefs);
    let icontainers = cc(ALL_CONTAINERS);

    let oracle = oracle_load_scene(&orefs, ALL_CONTAINERS, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    // glmapm03 索引 3 不在 GLMAPM_INDICES → Err(1 missing)
    match &oracle {
        Err(err) => {
            assert_eq!(err.missing.len(), 1, "应只有 glmapm03 缺失");
            assert_eq!(err.missing[0], ORef::new(OContainer::Ui, "glmapm03.Map"));
        }
        Ok(assets) => {
            assert_eq!(assets.len(), 2, "glmapm01 + glmapm02 成功");
        }
    }
}

/// 容器缺失 → 该容器所有资源报告为 missing。
#[test]
fn diff_missing_container() {
    let orefs = o_refs(UI_NAMES, OContainer::Ui);
    let irefs = cr(&orefs);
    let limited: &[OContainer] =
        &[OContainer::Glpics, OContainer::Cards, OContainer::Goods, OContainer::Music];
    let limited_i = cc(limited);

    let oracle = oracle_load_scene(&orefs, limited, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &limited_i, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    // 全部 5 个 Ui 资源都应 missing
    match &oracle {
        Err(err) => {
            assert_eq!(err.missing.len(), 5, "5 个 Ui 资源均应 missing");
            for r in &err.missing {
                assert_eq!(r.container, OContainer::Ui);
            }
        }
        Ok(_) => {
            unreachable!("Ui 容器缺失时不应成功");
        }
    }
}

/// 空引用列表 → Ok(空列表)。
#[test]
fn diff_empty_refs() {
    let oracle = oracle_load_scene(&[], ALL_CONTAINERS, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let icontainers = cc(ALL_CONTAINERS);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&[], &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    match &oracle {
        Ok(assets) => {
            assert!(assets.is_empty());
            let plan_o = oracle_render_frame(assets);
            let plan_i: RPlan = SceneManager::render_frame(&ca_vec(assets));
            assert_plan_equal(&plan_o, &plan_i);
            assert_eq!(plan_o.blit_count, 0);
            assert_eq!(plan_i.blit_count, 0);
        }
        Err(e) => {
            unreachable!("空引用不应出错: {e}");
        }
    }
}

/// 语义测试 (b)：全装载成功 → `blit_count` == 资产数。
#[test]
fn semantic_full_load_blit_count() {
    let orefs: Vec<ORef> = [
        o_refs(UI_NAMES, OContainer::Ui),
        o_refs(GLPICS_NAMES, OContainer::Glpics),
        o_refs(MUSIC_NAMES, OContainer::Music),
    ]
    .concat();
    let irefs = cr(&orefs);
    let icontainers = cc(ALL_CONTAINERS);

    let oracle = oracle_load_scene(&orefs, ALL_CONTAINERS, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    let mut sm = SceneManager::new();
    let impl_out = sm.load_scene(&irefs, &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert_load_equal(&oracle, &impl_out);

    match &oracle {
        Ok(assets) => {
            assert_eq!(assets.len(), 15, "Ui(5) + Glpics(5) + Music(5) = 15");
            let plan_o = oracle_render_frame(assets);
            let plan_i: RPlan = SceneManager::render_frame(&ca_vec(assets));
            assert_plan_equal(&plan_o, &plan_i);
            assert_eq!(plan_o.blit_count, 15, "blit_count == 资产数");
            assert_eq!(plan_o.layers.len(), 15);
            assert_eq!(plan_i.blit_count, 15);
            assert_eq!(plan_i.layers.len(), 15);
        }
        Err(e) => {
            unreachable!("全可用容器装载不应失败: {e}");
        }
    }
}

/// 语义测试补充：缺容器时 impl `SceneManager.scenes` 保持不变。
#[test]
fn impl_scenes_unchanged_on_error() {
    let orefs_ui = o_refs(UI_NAMES, OContainer::Ui);
    let irefs_ui = cr(&orefs_ui);
    let limited: &[OContainer] =
        &[OContainer::Glpics, OContainer::Cards, OContainer::Goods, OContainer::Music];
    let limited_i = cc(limited);
    let icontainers = cc(ALL_CONTAINERS);

    let orefs_music = o_refs(MUSIC_NAMES, OContainer::Music);
    let irefs_music = cr(&orefs_music);

    let mut sm = SceneManager::new();
    sm.load_scene(&irefs_music, &icontainers, GL_REC_MAGIC_OK, GLMAPM_INDICES)
        .expect("Music 全可用应成功");
    assert_eq!(sm.assets().len(), 5, "初始装载 Music(5) 成功");

    let result = sm.load_scene(&irefs_ui, &limited_i, GL_REC_MAGIC_OK, GLMAPM_INDICES);
    assert!(result.is_err(), "缺 Ui 容器应失败");
    assert_eq!(sm.assets().len(), 5, "失败后 scenes 保持原 5 个 Music 资产");
}
