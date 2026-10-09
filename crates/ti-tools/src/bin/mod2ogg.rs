//! `mod2ogg` —— INHL.MOD → OGG/Opus 转码（S1.7b）。
//!
//! 原版 `music.lpk` 中的 `804615c2.bin` 是自定义 `INHL.MOD` 格式
//! （`data/music.lpk` 实测，F-208）：
//! - 0x00-0x07: `INHL.MOD` 魔数
//! - 0x14 起: NUL 结尾标题（"I Need Her Love"）
//! - 0x4844 起: 8bit 无符号单声道 PCM 采样数据（至文件尾）
//!
//! 用法：
//! ```text
//! cargo run -p ti-tools --bin mod2ogg -- <input.mod|music.lpk> <output.ogg>
//! cargo run -p ti-tools --bin mod2ogg -- --rate 8000 <input.mod> <output.ogg>
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use ti_platform::mod_pcm::ModAudio;
use ti_platform::ogg_encode::encode_ogg;

/// MOD 条目在 `music.lpk` 容器中的 key（F-208 实测）。
const MOD_KEY: u32 = 0x8046_15C2;

fn parse_args() -> (PathBuf, Option<u32>, PathBuf) {
    let mut rate: Option<u32> = None;
    let mut positional: Vec<String> = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--rate" => rate = it.next().and_then(|s| s.parse().ok()),
            s if s.starts_with("--") => {}
            s => positional.push(s.to_string()),
        }
    }
    let input = positional.first().map_or_else(|| PathBuf::from("data/music.lpk"), PathBuf::from);
    let output = positional.get(1).map_or_else(|| PathBuf::from("out/glmusic.ogg"), PathBuf::from);
    (input, rate, output)
}

fn read_mod_bytes(input: &PathBuf) -> Result<Vec<u8>, ExitCode> {
    if input.extension().is_some_and(|e| e == "lpk" || e == "pak") {
        let buf = fs::read(input).map_err(|e| {
            eprintln!("[mod2ogg] 读取 {} 失败: {e}", input.display());
            ExitCode::FAILURE
        })?;
        let name = input.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let pkg = ti_formats::Package::parse(name, &buf).map_err(|e| {
            eprintln!("[mod2ogg] 解析容器失败: {e}");
            ExitCode::FAILURE
        })?;
        let idx = pkg.find_key(MOD_KEY).ok_or_else(|| {
            eprintln!("[mod2ogg] 容器中未找到 MOD 条目 (key={MOD_KEY:08X})");
            ExitCode::FAILURE
        })?;
        pkg.read_entry(&buf, idx).map_err(|e| {
            eprintln!("[mod2ogg] 读取 MOD 条目失败: {e}");
            ExitCode::FAILURE
        })
    } else {
        fs::read(input).map_err(|e| {
            eprintln!("[mod2ogg] 读取 {} 失败: {e}", input.display());
            ExitCode::FAILURE
        })
    }
}

fn main() -> ExitCode {
    let (input, rate_override, output) = parse_args();

    let mod_bytes = match read_mod_bytes(&input) {
        Ok(b) => b,
        Err(code) => return code,
    };

    let mut audio = match ModAudio::parse(&mod_bytes) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[mod2ogg] INHL.MOD 解析失败: {e}");
            return ExitCode::FAILURE;
        }
    };

    let rate = rate_override.unwrap_or(audio.sample_rate);
    audio.sample_rate = rate;
    let pcm_i16 = audio.to_i16();
    #[allow(clippy::cast_precision_loss)]
    let duration = f64::from(u32::try_from(pcm_i16.len()).unwrap_or(u32::MAX)) / f64::from(rate);
    eprintln!(
        "[mod2ogg] 标题 \"{}\"，PCM {} 采样 @ {rate} Hz = {:.2}s",
        audio.title,
        pcm_i16.len(),
        duration
    );

    match encode_ogg(&pcm_i16, rate) {
        Ok(ogg) => {
            if let Some(parent) = output.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(&output, &ogg) {
                eprintln!("[mod2ogg] 写 {} 失败: {e}", output.display());
                return ExitCode::FAILURE;
            }
            eprintln!(
                "[mod2ogg] 写出 {}（{} 字节 OGG/Opus，目标 64 kb/s）",
                output.display(),
                ogg.len()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[mod2ogg] 编码失败: {e}");
            ExitCode::FAILURE
        }
    }
}
