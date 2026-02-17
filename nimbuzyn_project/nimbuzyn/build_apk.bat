@echo off
:: =============================================================================
:: Nimbuzyn — Build Script para Windows
:: =============================================================================
setlocal EnableDelayedExpansion

echo.
echo ============================================================
echo  Nimbuzyn APK Builder
echo ============================================================
echo.

:: Verificar Rust
where rustup >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust no encontrado.
    echo Instala Rust desde: https://rustup.rs/
    pause
    exit /b 1
)
echo [OK] Rust encontrado

:: Agregar target Android
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi

:: Verificar cargo-apk
where cargo-apk >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Instalando cargo-apk...
    cargo install cargo-apk --locked
)
echo [OK] cargo-apk listo

:: Verificar ANDROID_HOME
if "%ANDROID_HOME%"=="" (
    set "ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk"
    if not exist "!ANDROID_HOME!" (
        echo [ERROR] ANDROID_HOME no definido.
        echo Define la variable de entorno ANDROID_HOME=C:\Users\TU_USUARIO\AppData\Local\Android\Sdk
        pause
        exit /b 1
    )
)
echo [OK] ANDROID_HOME: %ANDROID_HOME%

:: Detectar NDK
if "%ANDROID_NDK_HOME%"=="" (
    for /f "delims=" %%i in ('dir /b /ad "%ANDROID_HOME%\ndk" 2^>nul') do (
        set "ANDROID_NDK_HOME=%ANDROID_HOME%\ndk\%%i"
    )
)
if "%ANDROID_NDK_HOME%"=="" (
    echo [ERROR] NDK no encontrado. Instala desde Android Studio: SDK Manager ^> SDK Tools ^> NDK
    pause
    exit /b 1
)
echo [OK] NDK: %ANDROID_NDK_HOME%

:: Generar iconos
python generate_icons.py . 2>nul || echo [WARN] Python/Pillow no disponible, usando iconos existentes

:: Compilar
echo.
echo Compilando...
echo.

set "NDK_BIN=%ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin"
set "CC_aarch64_linux_android=%NDK_BIN%\aarch64-linux-android26-clang.cmd"
set "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=%CC_aarch64_linux_android%"

cargo apk build --lib --release

if %errorlevel% neq 0 (
    echo [ERROR] Error de compilacion. Revisa los mensajes anteriores.
    pause
    exit /b 1
)

:: Buscar APK
for /r target %%f in (*.apk) do (
    set "APK_FILE=%%f"
)

if "!APK_FILE!"=="" (
    echo [ERROR] APK no encontrado
    pause
    exit /b 1
)

echo.
echo ============================================================
echo  APK generado: !APK_FILE!
echo ============================================================
echo.
echo Para instalar en un dispositivo Android conectado:
echo   adb install -r "!APK_FILE!"
echo.
pause
