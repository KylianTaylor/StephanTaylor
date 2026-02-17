use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ──────────────────────────────────────────────
// USER MODEL
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub uid: String,          // unique public ID (e.g. "NIM-4F2A3B")
    pub username: String,
    pub display_name: String,
    pub avatar_color: u32,    // packed RGBA for avatar placeholder
    pub created_at: String,
}

impl User {
    pub fn new(username: String, display_name: String) -> Self {
        let uid = format!(
            "NIM-{}",
            &Uuid::new_v4().to_string().to_uppercase()[..6]
        );
        User {
            id: 0,
            uid,
            username,
            display_name,
            avatar_color: 0xFF_4A_90_E2,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

// ──────────────────────────────────────────────
// CONTACT / FRIEND MODEL
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactType {
    Friend,
    Acquaintance,
}

impl std::fmt::Display for ContactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContactType::Friend => write!(f, "Amigo"),
            ContactType::Acquaintance => write!(f, "Conocido"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: i64,
    pub owner_uid: String,      // who owns this contact
    pub contact_uid: String,    // the contact's UID
    pub display_name: String,
    pub avatar_color: u32,
    pub contact_type: ContactType,
    pub starred: bool,          // starred contacts appear at top
    pub added_at: String,
}

// ──────────────────────────────────────────────
// MESSAGE MODEL
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Text,
    Image,
    Video,
    Document,
    Archive,    // .rar files
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Image => write!(f, "image"),
            MessageType::Video => write!(f, "video"),
            MessageType::Document => write!(f, "document"),
            MessageType::Archive => write!(f, "archive"),
        }
    }
}

impl MessageType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "image"    => MessageType::Image,
            "video"    => MessageType::Video,
            "document" => MessageType::Document,
            "archive"  => MessageType::Archive,
            _          => MessageType::Text,
        }
    }

    /// Returns the emoji icon for display
    pub fn icon(&self) -> &str {
        match self {
            MessageType::Text     => "💬",
            MessageType::Image    => "🖼",
            MessageType::Video    => "🎬",
            MessageType::Document => "📄",
            MessageType::Archive  => "📦",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_uid: String,
    pub content: String,          // text content or file path
    pub msg_type: MessageType,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,   // bytes
    pub sent_at: String,
    pub is_read: bool,
}

impl Message {
    pub const MAX_TEXT_LEN: usize = 1000;
    pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

    pub fn is_valid_file_size(size: u64) -> bool {
        size <= Self::MAX_FILE_SIZE
    }

    pub fn is_valid_file_type(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"  // images
            | "mp4" | "mkv" | "avi" | "mov" | "webm"          // videos
            | "pdf" | "doc" | "docx" | "xls" | "xlsx"
            | "ppt" | "pptx" | "txt" | "csv"                   // documents
            | "rar" | "zip" | "7z"                             // archives
        )
    }
}

// ──────────────────────────────────────────────
// CHAT SESSION MODEL
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub participant_a: String,
    pub participant_b: String,
    pub created_at: String,
    pub last_message: Option<String>,
    pub last_message_at: Option<String>,
    pub unread_count: u32,
}

// ──────────────────────────────────────────────
// PRODUCT / INVENTORY MODEL
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub owner_uid: String,
    pub code: String,           // product code
    pub name: String,
    pub quantity: f64,          // can be 0 (triggers red alert)
    pub net_value: f64,         // costo / valor neto
    pub sale_value: f64,        // precio de venta
    pub profit_value: f64,      // ganancias (calculado)
    pub created_at: String,
    pub updated_at: String,
}

impl Product {
    pub fn calculate_profit(&mut self) {
        self.profit_value = self.sale_value - self.net_value;
    }

    pub fn total_net(&self) -> f64 {
        self.quantity * self.net_value
    }

    pub fn total_profit(&self) -> f64 {
        self.quantity * self.profit_value
    }

    pub fn is_out_of_stock(&self) -> bool {
        self.quantity < 1.0
    }
}

// ──────────────────────────────────────────────
// APP-WIDE STATE MODELS
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppTheme {
    Light,
    Dark,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: AppTheme,
    pub notifications_enabled: bool,
    pub font_size: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: AppTheme::Dark,
            notifications_enabled: true,
            font_size: 14.0,
        }
    }
}
