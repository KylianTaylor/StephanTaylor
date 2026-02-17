# Nimbuzyn 🔵

**Aplicación Android escrita 100% en Rust** — Mensajería · Inventario · Todo en uno.

---

## Características

### 🔐 Autenticación
- Registro e inicio de sesión seguros
- **Contraseñas cifradas con Argon2id** (el estándar de la industria)
- Base de datos SQLite local con soporte WAL
- Cada usuario recibe un **ID único público** (ej: `NIM-4F2A3B`)

### 💬 Chat
- Agregar contactos por ID único
- Mensajes de texto hasta **1000 caracteres**
- Envío de archivos hasta **100 MB**: imágenes, videos, documentos, `.rar`
- Lista de **Amigos** y **Conocidos** separada
- ⭐ Sistema de favoritos — los contactos con estrella aparecen arriba, el resto A–Z

### 📦 Inventario
- Plantilla de productos: código, nombre, cantidad, valor neto, valor venta, ganancias
- Cálculo automático de ganancia unitaria
- **Panel de alerta roja fijo** para productos sin stock (cantidad < 1)
- Resumen en tiempo real: total de productos, valor neto total, ganancias totales
- Buscador de productos

### ⚙️ Configuración
- Editar nombre de usuario
- Cambiar contraseña (verificación de contraseña actual)
- Tema **oscuro / claro**
- Cerrar sesión (con confirmación)

---

## Arquitectura

```
nimbuzyn/
├── Cargo.toml                    # Dependencias y metadata Android
├── AndroidManifest.xml           # Manifiesto Android
├── .cargo/config.toml            # Targets de compilación cruzada
├── res/
│   ├── values/strings.xml        # Recursos de strings
│   └── values/styles.xml         # Tema Android
└── src/
    ├── lib.rs                    # Punto de entrada Android (android_main)
    ├── main.rs                   # Runner de escritorio (para pruebas)
    ├── app.rs                    # Estado global y enrutamiento de pantallas
    ├── theme.rs                  # Sistema de colores y tema egui
    ├── db/
    │   └── mod.rs                # SQLite: auth, contacts, chat, inventory
    ├── models/
    │   └── mod.rs                # Structs: User, Contact, Message, Product
    └── screens/
        ├── mod.rs
        ├── login.rs              # Pantalla de autenticación
        ├── chat.rs               # Lista de contactos + ventana de chat
        ├── inventory.rs          # CRUD de productos con alerta de stock
        └── settings.rs           # Configuración de cuenta y tema
```

### Stack tecnológico

| Componente | Crate |
|---|---|
| UI Framework | `egui` + `eframe` (`android-native-activity`) |
| Base de datos | `rusqlite` (SQLite bundled) |
| Hash de contraseñas | `argon2` (Argon2id) |
| UUID | `uuid` v4 |
| Fechas/horas | `chrono` |
| Serialización | `serde` + `serde_json` |
| Logging Android | `android_logger` |

---

## Compilación

### Prerrequisitos

```bash
# 1. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Agregar target Android
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi  # opcional, 32-bit

# 3. Instalar Android SDK + NDK (API 26+)
#    Descargar Android Studio o usar sdkmanager

# 4. Instalar cargo-apk
cargo install cargo-apk
```

### Variables de entorno

```bash
export ANDROID_HOME=/path/to/android/sdk
export ANDROID_NDK_HOME=/path/to/android/ndk
```

### Compilar y ejecutar en Android

```bash
# Conecta tu dispositivo o inicia un emulador (API >= 26)
adb devices

# Build + instalar + lanzar en una sola línea
cargo apk run --lib --target aarch64-linux-android --release
```

### Probar en escritorio (PC/Mac/Linux)

```bash
cargo run --bin nimbuzyn
# Abre una ventana 390x844 simulando un teléfono
```

---

## Seguridad

- Las contraseñas **nunca** se almacenan en texto plano
- **Argon2id** con salt aleatorio por usuario (OWASP recomendado)
- La base de datos reside en el directorio privado de la app Android
- Para mayor seguridad en producción, se puede integrar **SQLCipher** (cifrado de toda la DB)

---

## Permisos Android requeridos

| Permiso | Uso |
|---|---|
| `INTERNET` | Mensajería y sincronización |
| `READ_MEDIA_IMAGES` | Compartir imágenes |
| `READ_MEDIA_VIDEO` | Compartir videos |
| `READ_EXTERNAL_STORAGE` | (API ≤ 32) Acceso a archivos |

---

## Roadmap / Próximas funcionalidades

- [ ] Sincronización en la nube (WebSocket server en Rust con `tokio` + `axum`)
- [ ] Notificaciones push (Firebase via JNI)
- [ ] Cifrado de mensajes E2E (Curve25519 + AES-GCM)
- [ ] Exportar inventario a CSV/PDF
- [ ] Búsqueda de mensajes
- [ ] Avatares personalizados
- [ ] Soporte para grupos en el chat

---

## Licencia

MIT © Nimbuzyn
