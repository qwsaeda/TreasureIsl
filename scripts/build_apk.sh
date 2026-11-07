#!/usr/bin/env bash
# S2.6/S18.8d：Android APK 构建（Linux + NDK 直链，redroid / 真机 / 模拟器装机验证）。
# 2026-11-01 真机/MuMu 闪退修复（v2，EXP-146）；编号与 `docs/process/14_tooling_catalog.md`
# build_apk 行「已知坑 ①-⑦」对应（catalog ①members/②CC/⑤ABI/⑥extract/⑦icon，本处按体引用排序）：
#   ①lib 目录 `lib/aarch64`→`lib/arm64-v8a`（Android 标准 ABI 目录；`aarch64` 不被安装器认
#     ⇒ NO_MATCHING_ABIS / 真机「32 位系统」提示 / 闪退；= catalog ⑤）。
#   ②manifest `extractNativeLibs=false`→`true`（`false` 要求 .so 未压缩+16KB 对齐（zipalign -p 4 不满足）
#     且 arm 转译（MuMu/redroid x86_64 跑 arm64）需解压加载 ⇒ 装不上/闪退；改 `android/AndroidManifest.xml`；
#     = catalog ⑥）。
#   ③`android-activity` 不在根 workspace members（S18.8d 移出，`ndk-sys` native 拒编译）
#     ⇒ 需要重编 .so 时临时加回 members 构建（trap 恢复 Cargo.toml/lock），常规（.so 已新）跳过不动；
#     = catalog ①（编译 >15min 须 timeout ≥3600s，catalog ③）。
#   ④x86_64 `CC_${ARCH//_/-}` 连字符变量名 bash `export` 非法 ⇒ 用 `env` 命令（可设含 `-` 变量名）；
#     = catalog ②。
#   ⑤aapt2 link 每次重跑（manifest 变更生效，如 extractNativeLibs）；base.apk 不再缓存。
#   ⑥launcher 图标/应用名 + res/ 两阶段（`aapt2 compile --dir android/res -o res.zip` → link；
#     = catalog ⑦，EXP-148）：`android:icon="@mipmap/ic_launcher"` + `label="@string/app_name"`。
#   ⑦默认 arm64（2026-11-01 用户拍板「主要安卓玩」；x86_64 传参）+ 产物按架构命名
#     （S25 版本规范后 = `TreasureIsl-<x.y.z>-<arch>.apk`，arch = arm64/x86_64；版本单源 = Cargo.toml）
#     + `.so` 已存在且比壳源新 ⇒ 复用（固定产物复用；`--force` 强制重编）。
# S35.0（24 号 APK 体积标准，2026-11-01）：⑧`.so` 改 ZIP_DEFLATED（原 Stored 原样 53MB ⇒ 压缩 20MB）
#     + assets 只带「引擎引用集」（`scripts/apk_audit.py --refset`，未引用 UI 图 0 进包）
#     + [4/5] 体积审计判据（.so 压缩率 + 引用集漂移，FAIL 阻断发布）。
# 用法：
#   bash scripts/build_apk.sh                    # aarch64（真机 ARM64，默认）
#   bash scripts/build_apk.sh x86_64             # x86_64（MuMu/redroid x86 模拟器）
#   bash scripts/build_apk.sh aarch64 --force    # 强制重编 .so（默认复用）
# 依赖：NDK r25b（/tmp/opencode/android-ndk-r25b）+ aapt2 + zipalign + apksigner（系统 build-tools）
#       + java/keytool（签名）。**S46.3 自举修复**：原 `[3/5]` 签名用一次性助手
#       `/tmp/opencode/apksigner/{apksig.jar,SignV2.class,keystore/ti.keystore}`（仓库外临时资产，
#       /tmp 被清后即 ClassNotFoundException: SignV2 ⇒ 构建死在签名步，EXP-173）⇒ 改官方
#       `apksigner` CLI + keystore 缺失自动生成 + `[0/5]` 依赖前置（缺一即 FAIL 早退，不再半路死）。
set -euo pipefail
REPO=/disk/workspaces/treasureIsl
NDK=/tmp/opencode/android-ndk-r25b
BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
ARCH=${1:-aarch64}
FORCE=${2:-}
# S46.3：AAudio（bevy_audio→cpal→ndk-sys `-laaudio`）需 API≥26；21 的 sysroot 无 libaaudio ⇒ 链接失败（EXP-173）
API=${3:-26}
ABI=$([ "$ARCH" = "aarch64" ] && echo arm64-v8a || echo x86_64)
TARGET="${ARCH}-linux-android"
SRC_SO="/disk/target/${TARGET}/release/libandroid_activity.so"
WORK=/tmp/opencode/apk_build
OUT=$REPO/dist
AAPT2=/usr/lib/android-sdk/build-tools/debian/aapt2
APKSIGNER=/usr/lib/android-sdk/build-tools/debian/apksigner

# [0/5] 依赖前置（S46.3：缺一即 FAIL 早退 + 行动信息；不再靠「本地全备」假设，/tmp 清理即死）
echo "[0/5] 依赖前置…"
MISSING=""
for d in "$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin/${ARCH}-linux-android${API}-clang"; do
    [ -x "$d" ] || MISSING="$MISSING $d"
done
[ -x "$AAPT2" ] || MISSING="$MISSING $AAPT2"
ZALIGN=$(find /disk/containerd -name "zipalign" 2>/dev/null | head -1)
[ -n "$ZALIGN" ] && [ -x "$ZALIGN" ] || MISSING="$MISSING zipalign(containerd 内)"
[ -x "$APKSIGNER" ] || MISSING="$MISSING $APKSIGNER"
command -v java >/dev/null || MISSING="$MISSING java"
command -v keytool >/dev/null || MISSING="$MISSING keytool"
JAR=$(find /disk/containerd -name "android.jar" 2>/dev/null | head -1)
[ -n "$JAR" ] || MISSING="$MISSING android.jar(containerd 内)"
if [ -n "$MISSING" ]; then
    echo "  FAIL  依赖缺失：$MISSING" >&2
    echo "  安装指引：NDK=/tmp/opencode/android-ndk-r25b；aapt2/zipalign/apksigner = android build-tools（debian 包 android-sdk-build-tools）；java/keytool = openjdk-17" >&2
    exit 1
fi
echo "  OK  NDK/aapt2/zipalign/apksigner/java/keytool 在位（签名走官方 apksigner CLI）"

mkdir -p "$WORK" "$OUT"
# S25 版本规范（25_versioning.md §2）：产物名带 x.y.z，版本单源 = `Cargo.toml` `[workspace.package] version`
# （禁散写版本字面量；产物名与版本不一致 = 打包漂移，validate_data check_version_consistency 兜底）。
VERSION=$(sed -n '/^\[workspace.package\]/,/^\[/p' "$REPO/Cargo.toml" | grep -m1 '^version' | cut -d'"' -f2)
if [ -z "$VERSION" ]; then
    echo "  FAIL  读不到 Cargo.toml [workspace.package] version（25_versioning.md §1 单源）" >&2
    exit 1
fi
APK_OUT="$OUT/TreasureIsl-${VERSION}-${ARCH}.apk"

cd "$REPO"
echo "=== S2.6 Android APK 构建（${TARGET} / ABI ${ABI}）==="

# [1/5] 交叉编译 cdylib（.so）——已存在且比壳源新且非 --force ⇒ 复用（2026-11-01 S35 用户拍板：
# 手动改过 .rs 的 mtime 会误触发 15min 重编；判据只看「.rs 是否真比 .so 新」，平台侧构建前
# 由 `scripts/build_artifacts.py pre` 锚定 mtime，构建后 `post` 复原）
SHELL_SRC="$REPO/crates/ti-engine/src/bin/treasure_bevy.rs"
REFSET=$(python3 "$REPO/scripts/apk_audit.py" --refset)
if [ "$FORCE" != "--force" ] && [ -f "$SRC_SO" ] && [ "$SRC_SO" -nt "$SHELL_SRC" ]; then
    echo "[1/5] 复用现有 .so（${SRC_SO}；--force 强制重编）"
else
    echo "[1/5] 交叉编译 cdylib（.so）…"
    # ③ android-activity 不在根 members ⇒ 临时加回构建，trap 恢复 Cargo.toml/lock
    cp "$REPO/Cargo.toml" "$WORK/Cargo.toml.bak"
    cp "$REPO/Cargo.lock" "$WORK/Cargo.lock.bak" 2>/dev/null || true
    restore_cargo() {
        cp "$WORK/Cargo.toml.bak" "$REPO/Cargo.toml"
        [ -f "$WORK/Cargo.lock.bak" ] && cp "$WORK/Cargo.lock.bak" "$REPO/Cargo.lock"
    }
    trap restore_cargo EXIT
    python3 - "$REPO" <<'PYEOF'
import sys
p = sys.argv[1] + "/Cargo.toml"
s = open(p).read()
open(p, "w").write(s.replace('    # "android/android-activity",', '    "android/android-activity",'))
PYEOF
    export PATH="$HOME/.cargo/bin:$PATH"
    # ④ CC/AR 连字符变量名（x86_64→x86-64）bash export 非法 ⇒ env 命令（可设含 `-` 变量名）
    env "CC_${TARGET}=$BIN/${ARCH}-linux-android${API}-clang" \
        "AR_${TARGET}=$BIN/llvm-ar" \
        RUSTFLAGS="-C linker=$BIN/${ARCH}-linux-android${API}-clang" \
        cargo build -p android-activity@0.1.0 --release --target "$TARGET"
fi

# [2/5] aapt2 link（每次重跑：manifest 变更生效，如 extractNativeLibs）
echo "[2/5] aapt2 link（manifest 每次重跑）…"
mkdir -p "$WORK" "$OUT"
rm -f "$WORK/base.apk" "$WORK/res.zip"
# ⑥ launcher 图标 + @string/app_name：res/ 目录两阶段编译（aapt2 compile --dir → link 引用）
if [ -d "$REPO/android/res" ]; then
    $AAPT2 compile --dir "$REPO/android/res" -o "$WORK/res.zip"
fi
$AAPT2 link -o "$WORK/base.apk" -I "$JAR" --manifest "$REPO/android/AndroidManifest.xml" "$WORK/res.zip"
# 空 dex（NativeActivity 0 Java 类）
DEX_SRC="$WORK/classes.dex"
if [ ! -f "$DEX_SRC" ]; then
  python3 -c "open('$DEX_SRC','wb').write(b'dex\n035\0'+b'\x00'*56)"
fi

# [3/5] 打包（S35.0 体积标准：.so 压缩 + 资产引用集）+ 签名
echo "[3/5] 打包 + 签名…"
rm -rf "$WORK/apk_assets"
mkdir -p "$WORK/apk_assets/assets/maps" "$WORK/apk_assets/assets/fonts"
cp -r "$REPO/assets/maps/." "$WORK/apk_assets/assets/maps/" 2>/dev/null || true
# S35.0（24 号 APK 体积标准）：ui/fonts/audio 只带「引擎引用集」——`scripts/apk_audit.py --refset`
# 单一事实源（代码字面量 + format! 动态前缀 + 地图岛 kind + registry audio）；未引用图 0 进包。
mkdir -p "$WORK/apk_assets/assets/audio"
python3 - "$REFSET" "$REPO" "$WORK/apk_assets" <<'PYEOF'
import json, sys, pathlib
refset_json, repo, work = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
refs = json.loads(refset_json)
copied = 0
for rel in [x for g in ("ui", "fonts", "audio") for x in refs.get(g, [])] + refs.get("legacy", []):
    src = pathlib.Path(repo) / rel
    if not src.is_file():
        continue
    dst = work / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(src.read_bytes())
    copied += 1
print(f"assets 引用集打包：{copied} 文件（ui {len(refs.get('ui', []))} / fonts {len(refs.get('fonts', []))} / audio {len(refs.get('audio', []))}；未引用不进包）")
PYEOF
python3 - "$WORK" "$ABI" "$SRC_SO" "$DEX_SRC" <<'PYEOF'
import zipfile, os, sys
WORK, ABI, src_so, dex = sys.argv[1:5]
zin = zipfile.ZipFile(f"{WORK}/base.apk", 'r')
with zipfile.ZipFile(f"{WORK}/unsigned.apk", 'w') as z:
    z.writestr("classes.dex", open(dex, 'rb').read())
    z.writestr("AndroidManifest.xml", zin.read('AndroidManifest.xml'))
    z.writestr("resources.arsc", zin.read('resources.arsc'))
    # ⑥ 复制 base.apk 的 res/ 条目（mipmap launcher 图标 + values 资源；aapt2 link 编译进 base）
    for info in zin.infolist():
        if info.filename.startswith("res/") and not info.is_dir():
            z.writestr(info, zin.read(info.filename))
    zi = zipfile.ZipInfo(f'lib/{ABI}/libandroid_activity.so')  # ① ABI 目录（arm64-v8a/x86_64）
    # S35.0（24 号标准①）：.so 必须 ZIP_DEFLATED（实测 deflate-9 = 原始 38%；Stored 原样 = APK 膨胀 30MB+）。
    zi.compress_type = zipfile.ZIP_DEFLATED
    z.writestr(zi, open(src_so, 'rb').read())
    for root, _, files in os.walk(f"{WORK}/apk_assets/assets"):
        for fn in files:
            full = os.path.join(root, fn)
            arc = os.path.relpath(full, f"{WORK}/apk_assets")
            zi = zipfile.ZipInfo(arc)
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, open(full, 'rb').read())
print("APK 组装完成")
PYEOF
# S35.0（24 号标准①）：.so 已压缩（ZIP_DEFLATED）⇒ zipalign 用常规 4 字节对齐（非 -p 16KB unaligned 模式）。
# 历史坑 ②：`-p`（unaligned + extractNativeLibs）曾为转译加载服务；现 .so 压缩后安装器统一解压，
# 对齐口径回到标准 4 字节（aapt2 已对齐 res/ 资源；.so 压缩块无需 16KB 页对齐）。
$ZALIGN -f 4 "$WORK/unsigned.apk" "$WORK/aligned.apk"
# S46.3 签名（原自造 SignV2 助手依赖 /tmp/opencode/apksigner 已随 /tmp 清理丢失，EXP-173）：
# 改官方 `apksigner` CLI（v2+v3 自动，minSdk 26 无需 v1；原 SignV2 助手为 v1+v2）；keystore 缺失自动生成（原私钥已丢失，
# 新 key 为新签名——装机需卸载旧包；如可恢复旧 ti.keystore 请放回 $WORK/keystore/ 以保持签名一致）。
mkdir -p "$WORK/keystore"
KS="$WORK/keystore/ti.keystore"
if [ ! -f "$KS" ]; then
    echo "  (签名自举) 生成新 keystore $KS（原私钥已随 /tmp 清理丢失；新 key = 新签名）"
    rm -f "$KS"   # 清残留半成品（此前 keytool 失败可能留空文件）
    keytool -genkeypair -v -keystore "$KS" -storetype JKS -alias treasure \
        -keyalg RSA -keysize 2048 -validity 10000 -storepass treasure -keypass treasure \
        -dname "CN=TreasureIsl, OU=Dev, O=TreasureIsl, L=CN, C=CN"
fi
$APKSIGNER sign --ks "$KS" --ks-key-alias treasure --ks-pass pass:treasure --key-pass pass:treasure \
    --v1-signing-enabled true --v2-signing-enabled true \
    --out "$WORK/aligned.apk.signed" "$WORK/aligned.apk"
echo "  v2+v3 signed → $WORK/aligned.apk.signed"
cp "$WORK/aligned.apk.signed" "$APK_OUT"

# [4/5] 体积审计（S35.0 判据①：.so 压缩率 + 引用集漂移；FAIL = 非 0 退出阻断发布）
echo "[4/5] 体积审计（scripts/apk_audit.py --apk）…"
python3 "$REPO/scripts/apk_audit.py" --apk "$APK_OUT"

echo "[5/5] 产物"
ls -la "$APK_OUT"
echo "=== S2.6 构建完成：$APK_OUT（${TARGET} / ABI ${ABI}）==="
