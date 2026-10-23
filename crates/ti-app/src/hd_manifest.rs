//! N7（EXP-048/060）：HD PNG 注入 manifest 类型 + 加载器。
//!
//! wasm/android 宿主用本模块的 [`HdF1Manifest`] 替代原来 bin 里硬编码的
//! `[u32; 116]` 静态 f1 数组与 `[14, 31, 32, 37]` 多帧数组：
//! f1 清单来自 `assets_hd_pack/hd_f1_manifest.json`（与 native `load_hd_pack_from_dir`
//! 目录枚举同源，注入数 = 目录全量，EXP-048/060）。

/// `assets_hd_pack/hd_f1_manifest.json` 的 schema。
///
/// - `static_f1`：`ui_*.png` 静态 f1（`HdPack::load_single` 语义）
/// - `multiframe_f1`：`*_fr*.png` 多帧族首帧（`HdPack::load_animated` 语义）
#[derive(Debug, Clone, Default, serde::Deserialize, PartialEq)]
pub struct HdF1Manifest {
    /// 静态精灵 f1（`NN_hd.png`，1 帧）。
    pub static_f1: Vec<u32>,
    /// 多帧精灵 f1（`NN_hd_f0..f4.png`，5 帧）。
    pub multiframe_f1: Vec<u32>,
}

/// 全量条目数（static + multiframe；EXP-048/060 断言注入数须等于此值）。
#[must_use]
pub fn manifest_expected_count(m: &HdF1Manifest) -> usize {
    m.static_f1.len() + m.multiframe_f1.len()
}

/// N7（EXP-048/060）：按 manifest 清单 fetch 所有 HD PNG，解码后组成
/// `f1 → 帧列表` 映射。与 native [`GameApp::load_hd_pack_from_dir`] 同口径：
/// - 静态 f1：fetch `assets_hd_pack/1x/ui/{f1}_hd.png`，帧号 0；
/// - 多帧 f1：fetch `assets_hd_pack/1x/ui/{f1}_hd_f{0..=4}.png`，帧号 0..=4；
/// - 解码失败 / fetch 失败 / 全透明帧 → 跳过（PCX 回退，不 panic）；
/// - 帧按帧号排序，同 f1 合并成一个列表。
///
/// `fetch` 为同步闭包（wasm 侧可在 async 上下文里 spawn 线程或用同步
/// `macroquad::file` 轮询实现；此处不引入 async 依赖）。
///
/// 返回 `f1 → Vec<PNG 原始字节>`（未解码，供 [`GameApp::load_hd_pngs`] 消费）。
#[must_use]
pub fn collect_hd_pngs_sync(
    manifest: &HdF1Manifest,
    fetch: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
) -> Vec<(u32, Vec<Vec<u8>>)> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<u32, Vec<(u32, Vec<u8>)>> = BTreeMap::new();
    for f1 in manifest.static_f1.iter().copied() {
        let name = format!("assets_hd_pack/1x/ui/{f1}_hd.png");
        let Some(bytes) = fetch(&name) else { continue };
        if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
            if img.rgba.iter().any(|&b| b > 0) {
                map.entry(f1).or_default().push((0, bytes));
            }
        }
    }
    for f1 in manifest.multiframe_f1.iter().copied() {
        // 飞轮 R1 ①-10：帧数 = `f0..fN` 探测（首个缺帧即止；原 `0..5` 写死假设）。
        let mut frame = 0u32;
        loop {
            let name = format!("assets_hd_pack/1x/ui/{f1}_hd_f{frame}.png");
            match fetch(&name) {
                Some(bytes) => {
                    if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                        if img.rgba.iter().any(|&b| b > 0) {
                            map.entry(f1).or_default().push((frame, bytes));
                        }
                    }
                    frame += 1;
                }
                None => break,
            }
        }
    }
    map.into_iter()
        .map(|(f1, mut frames)| {
            frames.sort_by_key(|(n, _)| *n);
            (f1, frames.into_iter().map(|(_, b)| b).collect())
        })
        .collect()
}

/// N7（EXP-048/060）：manifest 驱动 HD PNG 注入的 **async 版**（android 壳
/// `read_asset` 闭包）；语义与 [`collect_hd_pngs_sync`] 完全一致（静态 1 帧 +
/// 多帧 `f0..fN` 探测，首个缺帧即止；解码失败/全透明帧跳过，不 panic）。
pub async fn load_hd_pngs_from_manifest<F, Fut>(
    manifest: &HdF1Manifest,
    fetch: &mut F,
) -> Vec<(u32, Vec<Vec<u8>>)>
where
    F: FnMut(&str) -> Fut,
    Fut: std::future::Future<Output = Option<Vec<u8>>>,
{
    use std::collections::BTreeMap;
    let mut map: BTreeMap<u32, Vec<(u32, Vec<u8>)>> = BTreeMap::new();
    for f1 in manifest.static_f1.iter().copied() {
        let name = format!("assets_hd_pack/1x/ui/{f1}_hd.png");
        let Some(bytes) = fetch(&name).await else { continue };
        if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
            if img.rgba.iter().any(|&b| b > 0) {
                map.entry(f1).or_default().push((0, bytes));
            }
        }
    }
    for f1 in manifest.multiframe_f1.iter().copied() {
        let mut frame = 0u32;
        loop {
            let name = format!("assets_hd_pack/1x/ui/{f1}_hd_f{frame}.png");
            match fetch(&name).await {
                Some(bytes) => {
                    if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                        if img.rgba.iter().any(|&b| b > 0) {
                            map.entry(f1).or_default().push((frame, bytes));
                        }
                    }
                    frame += 1;
                }
                None => break,
            }
        }
    }
    map.into_iter()
        .map(|(f1, mut frames)| {
            frames.sort_by_key(|(n, _)| *n);
            (f1, frames.into_iter().map(|(_, b)| b).collect())
        })
        .collect()
}
