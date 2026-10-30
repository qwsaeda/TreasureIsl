#!/usr/bin/env bash
# S2.6/S18.8d：Android APK 构建（Linux + NDK 直链，redroid / 真机 / 模拟器装机验证）。
# 全链路：交叉编译 cdylib（.so）→ 组装 APK（manifest+arsc+dex+.so+assets）→ zipalign → v1+v2 签名。
# 产物：dist/TreasureIsl.apk（本机 x86_64 或 aarch64 双架构可选，默认 x86_64）。
# 用法：
#   bash scripts/build_apk.sh                    # x86_64（redroid / x86 模拟器 / x86_64 真机）
#   bash scripts/build_apk.sh aarch64            # aarch64（真机 ARM64）
# 依赖：NDK r25b（/tmp/opencode/android-ndk-r25b）+ aapt2 + zipalign + apksig.jar（本地全备）。
set -euo pipefail
REPO=/disk/workspaces/treasureIsl
NDK=/tmp/opencode/android-ndk-r25b
BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
ARCH=${1:-x86_64}
TARGET="${ARCH}-linux-android"
SRC_SO="/disk/target/${TARGET}/release/libandroid_activity.so"
WORK=/tmp/opencode/apk_build
OUT=$REPO/dist
APK_OUT=$OUT/TreasureIsl.apk

cd "$REPO"
echo "=== S2.6 Android APK 构建（${TARGET}）==="
echo "[1/5] 交叉编译 cdylib（.so）…"
export CC_${ARCH//_/-}_linux_android="$BIN/${ARCH}-linux-android21-clang"
export AR_${ARCH//_/-}_linux_android="$BIN/llvm-ar"
export RUSTFLAGS="-C linker=$BIN/${ARCH}-linux-android21-clang"
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p 'android-activity@0.1.0' --release --target "$TARGET"

echo "[2/5] 组装 APK 目录（manifest + arsc + dex + .so + assets）…"
mkdir -p "$WORK" "$OUT"
# base.apk（aapt2 link 产 manifest+arsc）
if [ ! -f "$WORK/base.apk" ]; then
  JAR=$(find /disk/containerd -name "android.jar" 2>/dev/null | head -1)
  AAPT2=/usr/lib/android-sdk/build-tools/debian/aapt2
  $AAPT2 link -o "$WORK/base.apk" -I "$JAR" --manifest "$REPO/android/AndroidManifest.xml"
fi
# 空 dex（NativeActivity 0 Java 类；dexopt 需合法 dex，用 jq 项目的最小 dex 或手写 64B 头）
DEX_SRC="$WORK/classes.dex"
if [ ! -f "$DEX_SRC" ]; then
  python3 -c "
import struct
magic = b'dex\n035\0'
open('$DEX_SRC','wb').write(magic + b'\x00'*(64-len(magic)))
"
fi

echo "[3/5] 打包 + 签名…"
# 仓内资产 → $WORK/apk_assets/assets/（bevy 资产管线读 APK assets 段 `AAssetManager`；
# bevy 请求路径含 `assets/` 前缀 ⇒ 归档条目 = `assets/ui/...`、`assets/fonts/...`）
rm -rf "$WORK/apk_assets"
mkdir -p "$WORK/apk_assets/assets/ui" "$WORK/apk_assets/assets/fonts" "$WORK/apk_assets/assets/maps"
cp -r "$REPO/assets/ui/." "$WORK/apk_assets/assets/ui/" 2>/dev/null || true
cp -r "$REPO/assets/fonts/." "$WORK/apk_assets/assets/fonts/" 2>/dev/null || true
cp -r "$REPO/assets/maps/." "$WORK/apk_assets/assets/maps/" 2>/dev/null || true
python3 - <<PYEOF
import zipfile, os
base = "$WORK/base.apk"
src_so = "$SRC_SO"
out = "$WORK/unsigned.apk"
zin = zipfile.ZipFile(base, 'r')
with zipfile.ZipFile(out, 'w') as z:
    z.writestr("classes.dex", open("$DEX_SRC","rb").read())
    z.writestr("AndroidManifest.xml", zin.read('AndroidManifest.xml'))
    z.writestr("resources.arsc", zin.read('resources.arsc'))
    zi = zipfile.ZipInfo('lib/${ARCH}/libandroid_activity.so')
    zi.compress_type = zipfile.ZIP_STORED
    z.writestr(zi, open(src_so,'rb').read())
    for root, _, files in os.walk("$WORK/apk_assets/assets"):
        for fn in files:
            full = os.path.join(root, fn)
            arc = os.path.relpath(full, os.path.dirname("$WORK/apk_assets"))
            zi = zipfile.ZipInfo(arc)
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, open(full,'rb').read())
print("APK 组装完成:", out)
PYEOF
ZALIGN=$(find /disk/containerd -name "zipalign" 2>/dev/null | head -1)
$ZALIGN -p -f 4 "$WORK/unsigned.apk" "$WORK/aligned.apk"
cd "$WORK/../apksigner" 2>/dev/null || cd /tmp/opencode/apksigner
java -cp apksig.jar:. SignV2 "$WORK/aligned.apk" "$WORK/keystore/ti.keystore" treasure ti treasure
cp "$WORK/aligned.apk.signed" "$APK_OUT"

echo "[4/5] 部署资产（adb push 到目标设备 /sdcard/TI/data/）…"
echo "  手动执行（装机前）：adb -s <dev> push legacy/data/ /sdcard/TI/data/"
echo "  目标设备：redroid(localhost:5555) / 真机 / Windows 模拟器"

echo "[5/5] 产物"
ls -la "$APK_OUT"
echo "=== S2.6 构建完成：$APK_OUT（${TARGET}）==="
