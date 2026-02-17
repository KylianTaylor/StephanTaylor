#!/usr/bin/env bash
# =============================================================================
# Nimbuzyn — Script de construcción automática del APK
# =============================================================================
# Uso:
#   chmod +x build_apk.sh
#   ./build_apk.sh [--release|--debug] [--target aarch64-linux-android]
#
# Requisitos:
#   - Rust + rustup
#   - Android SDK (API 26+) con NDK r25+
#   - cargo-apk  (se instala automáticamente si falta)
#   - Java JDK 11+ (para firma del APK)
# =============================================================================

set -euo pipefail

# ── Color output ──────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; BOLD='\033[1m'; NC='\033[0m'

log()  { echo -e "${BLUE}${BOLD}[Nimbuzyn]${NC} $*"; }
ok()   { echo -e "${GREEN}✓${NC} $*"; }
warn() { echo -e "${YELLOW}⚠${NC}  $*"; }
err()  { echo -e "${RED}✗${NC} $*"; exit 1; }

# ── Parse arguments ───────────────────────────────────────────────────────────
BUILD_TYPE="release"
TARGET="aarch64-linux-android"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)   BUILD_TYPE="debug" ;;
        --release) BUILD_TYPE="release" ;;
        --target)  TARGET="$2"; shift ;;
        *) warn "Argumento desconocido: $1" ;;
    esac
    shift
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

log "Construyendo Nimbuzyn APK (${BUILD_TYPE}) para ${TARGET}"
echo "────────────────────────────────────────────────────────────"

# ── 1. Verificar Rust ─────────────────────────────────────────────────────────
log "Verificando Rust..."
if ! command -v rustup &>/dev/null; then
    warn "Rust no encontrado. Instalando..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi
RUST_VER=$(rustc --version)
ok "Rust: $RUST_VER"

# ── 2. Agregar target Android ─────────────────────────────────────────────────
log "Configurando target Android: $TARGET"
rustup target add "$TARGET" 2>/dev/null || true
# También añadimos otros targets comunes
rustup target add armv7-linux-androideabi 2>/dev/null || true
rustup target add x86_64-linux-android   2>/dev/null || true
ok "Targets configurados"

# ── 3. Verificar/instalar cargo-apk ──────────────────────────────────────────
log "Verificando cargo-apk..."
if ! command -v cargo-apk &>/dev/null; then
    warn "cargo-apk no encontrado. Instalando (puede tardar unos minutos)..."
    cargo install cargo-apk --locked
fi
ok "cargo-apk: $(cargo apk --version 2>/dev/null || echo 'instalado')"

# ── 4. Verificar Android SDK / NDK ───────────────────────────────────────────
log "Verificando Android SDK/NDK..."

# Intentar detectar ANDROID_HOME automáticamente
if [[ -z "${ANDROID_HOME:-}" ]]; then
    for candidate in \
        "$HOME/Android/Sdk" \
        "$HOME/Library/Android/sdk" \
        "/opt/android-sdk" \
        "/usr/local/lib/android/sdk"
    do
        if [[ -d "$candidate" ]]; then
            export ANDROID_HOME="$candidate"
            ok "ANDROID_HOME detectado: $ANDROID_HOME"
            break
        fi
    done
fi

if [[ -z "${ANDROID_HOME:-}" ]]; then
    err "ANDROID_HOME no definido. Instala Android Studio o define la variable:
    export ANDROID_HOME=/ruta/al/android/sdk
    export ANDROID_NDK_HOME=\$ANDROID_HOME/ndk/<version>"
fi

# Detectar NDK
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    NDK_DIR="$ANDROID_HOME/ndk"
    if [[ -d "$NDK_DIR" ]]; then
        # Tomar la versión más reciente disponible
        LATEST_NDK=$(ls "$NDK_DIR" | sort -V | tail -1)
        if [[ -n "$LATEST_NDK" ]]; then
            export ANDROID_NDK_HOME="$NDK_DIR/$LATEST_NDK"
            ok "NDK detectado: $ANDROID_NDK_HOME"
        fi
    fi
fi

if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    err "ANDROID_NDK_HOME no definido. Instala el NDK desde Android Studio:
    SDK Manager → SDK Tools → NDK (Side by side)
    Luego: export ANDROID_NDK_HOME=\$ANDROID_HOME/ndk/<version>"
fi

ok "SDK: $ANDROID_HOME"
ok "NDK: $ANDROID_NDK_HOME"

# ── 5. Generar iconos (Python) ────────────────────────────────────────────────
log "Generando iconos 3D..."
if command -v python3 &>/dev/null && python3 -c "import PIL" 2>/dev/null; then
    python3 generate_icons.py "$(pwd)"
    ok "Iconos generados"
else
    warn "Python3 + Pillow no disponibles. Usando iconos existentes."
fi

# ── 6. Compilar ───────────────────────────────────────────────────────────────
log "Compilando Nimbuzyn (${BUILD_TYPE})..."
echo ""

BUILD_FLAGS=""
if [[ "$BUILD_TYPE" == "release" ]]; then
    BUILD_FLAGS="--release"
fi

# Set up NDK toolchain paths
NDK_TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt"
if [[ "$(uname -s)" == "Darwin" ]]; then
    NDK_HOST="darwin-x86_64"
elif [[ "$(uname -s)" == "Linux" ]]; then
    NDK_HOST="linux-x86_64"
else
    NDK_HOST="windows-x86_64"
fi

NDK_BIN="$NDK_TOOLCHAIN/$NDK_HOST/bin"

# Export linker variables
export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android26-clang"
export CXX_aarch64_linux_android="$NDK_BIN/aarch64-linux-android26-clang++"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"

export CC_armv7_linux_androideabi="$NDK_BIN/armv7a-linux-androideabi26-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$CC_armv7_linux_androideabi"

export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_BIN/x86_64-linux-android26-clang"

# Run the build
cargo apk build --lib $BUILD_FLAGS

echo ""

# ── 7. Localizar el APK ───────────────────────────────────────────────────────
log "Localizando APK generado..."

APK_SEARCH_PATH="target/release/apk"
if [[ "$BUILD_TYPE" == "debug" ]]; then
    APK_SEARCH_PATH="target/debug/apk"
fi

APK_FILE=$(find target -name "*.apk" -newer Cargo.toml 2>/dev/null | head -1)

if [[ -z "$APK_FILE" ]]; then
    # Fallback paths
    for path in \
        "target/${TARGET}/${BUILD_TYPE}/apk/nimbuzyn.apk" \
        "target/${BUILD_TYPE}/apk/nimbuzyn.apk" \
        "target/apk/nimbuzyn.apk"
    do
        if [[ -f "$path" ]]; then
            APK_FILE="$path"
            break
        fi
    done
fi

if [[ -z "$APK_FILE" ]]; then
    warn "APK no encontrado en la ruta estándar. Buscando..."
    find target -name "*.apk" 2>/dev/null | head -5 || true
    err "No se encontró el APK. Revisa los errores de compilación arriba."
fi

APK_SIZE=$(du -sh "$APK_FILE" | cut -f1)
ok "APK generado: $APK_FILE ($APK_SIZE)"

# ── 8. Firmar el APK (debug keystore) ────────────────────────────────────────
log "Verificando firma del APK..."

KEYSTORE="$HOME/.android/debug.keystore"
SIGNED_APK="nimbuzyn-${BUILD_TYPE}-signed.apk"

if command -v apksigner &>/dev/null; then
    # Check if already signed
    if apksigner verify "$APK_FILE" &>/dev/null; then
        ok "APK ya está firmado correctamente"
        cp "$APK_FILE" "$SIGNED_APK"
    else
        warn "Firmando APK con debug keystore..."
        if [[ ! -f "$KEYSTORE" ]]; then
            mkdir -p "$(dirname "$KEYSTORE")"
            keytool -genkey -v \
                -keystore "$KEYSTORE" \
                -storepass android \
                -alias androiddebugkey \
                -keypass android \
                -keyalg RSA -keysize 2048 \
                -validity 10000 \
                -dname "CN=Android Debug,O=Android,C=US" 2>/dev/null
        fi
        "$ANDROID_HOME/build-tools/$(ls "$ANDROID_HOME/build-tools" | sort -V | tail -1)/apksigner" sign \
            --ks "$KEYSTORE" \
            --ks-pass pass:android \
            --key-pass pass:android \
            --out "$SIGNED_APK" \
            "$APK_FILE"
        ok "APK firmado: $SIGNED_APK"
        APK_FILE="$SIGNED_APK"
    fi
elif command -v jarsigner &>/dev/null; then
    warn "apksigner no encontrado, usando jarsigner..."
    if [[ ! -f "$KEYSTORE" ]]; then
        mkdir -p "$(dirname "$KEYSTORE")"
        keytool -genkey -v \
            -keystore "$KEYSTORE" \
            -storepass android \
            -alias androiddebugkey \
            -keypass android \
            -keyalg RSA -keysize 2048 \
            -validity 10000 \
            -dname "CN=Android Debug,O=Android,C=US" 2>/dev/null
    fi
    jarsigner -verbose \
        -keystore "$KEYSTORE" \
        -storepass android \
        -keypass android \
        -signedjar "$SIGNED_APK" \
        "$APK_FILE" \
        androiddebugkey
    ok "APK firmado con jarsigner: $SIGNED_APK"
    APK_FILE="$SIGNED_APK"
else
    warn "Ni apksigner ni jarsigner disponibles. El APK puede no instalarse en producción."
fi

# ── 9. Instalar en dispositivo (opcional) ────────────────────────────────────
echo ""
echo "────────────────────────────────────────────────────────────"
ok "${BOLD}APK final: $APK_FILE${NC}"
echo ""

if command -v adb &>/dev/null; then
    DEVICES=$(adb devices 2>/dev/null | grep -v "List of devices" | grep "device$" | wc -l)
    if [[ "$DEVICES" -gt 0 ]]; then
        log "Dispositivo Android detectado. ¿Instalar? (s/n)"
        read -r -t 10 INSTALL_CONFIRM || INSTALL_CONFIRM="n"
        if [[ "$INSTALL_CONFIRM" =~ ^[sS]$ ]]; then
            adb install -r "$APK_FILE"
            ok "Nimbuzyn instalado en el dispositivo"
            log "Iniciando app..."
            adb shell am start -n "com.nimbuzyn.app/android.app.NativeActivity"
        fi
    else
        log "Para instalar, conecta un dispositivo Android y ejecuta:"
        echo "    adb install -r $APK_FILE"
    fi
else
    log "Para instalar en un dispositivo Android:"
    echo "    adb install -r $APK_FILE"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo -e "  ${GREEN}${BOLD}✅ Nimbuzyn APK listo para distribuir!${NC}"
echo "════════════════════════════════════════════════════════════"
echo ""
