use anyhow::{anyhow, Result};
use rusqlite::{Connection, params};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use crate::models::*;

// ──────────────────────────────────────────────
// DATABASE MANAGER
// ──────────────────────────────────────────────

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the SQLite database at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    // ──────────────────────────────────────────
    // MIGRATIONS / SCHEMA
    // ──────────────────────────────────────────

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS users (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uid         TEXT    NOT NULL UNIQUE,
                username    TEXT    NOT NULL UNIQUE,
                display_name TEXT   NOT NULL,
                password_hash TEXT  NOT NULL,
                avatar_color INTEGER NOT NULL DEFAULT 0,
                theme       TEXT    NOT NULL DEFAULT 'dark',
                notifications INTEGER NOT NULL DEFAULT 1,
                font_size   REAL    NOT NULL DEFAULT 14.0,
                created_at  TEXT    NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contacts (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                owner_uid    TEXT    NOT NULL,
                contact_uid  TEXT    NOT NULL,
                display_name TEXT    NOT NULL,
                avatar_color INTEGER NOT NULL DEFAULT 0,
                contact_type TEXT    NOT NULL DEFAULT 'acquaintance',
                starred      INTEGER NOT NULL DEFAULT 0,
                added_at     TEXT    NOT NULL,
                UNIQUE(owner_uid, contact_uid)
            );

            CREATE TABLE IF NOT EXISTS chats (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                participant_a TEXT NOT NULL,
                participant_b TEXT NOT NULL,
                created_at    TEXT NOT NULL,
                last_message  TEXT,
                last_msg_at   TEXT,
                unread_count  INTEGER NOT NULL DEFAULT 0,
                UNIQUE(participant_a, participant_b)
            );

            CREATE TABLE IF NOT EXISTS messages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id     INTEGER NOT NULL REFERENCES chats(id),
                sender_uid  TEXT    NOT NULL,
                content     TEXT    NOT NULL,
                msg_type    TEXT    NOT NULL DEFAULT 'text',
                file_name   TEXT,
                file_size   INTEGER,
                sent_at     TEXT    NOT NULL,
                is_read     INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS products (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                owner_uid   TEXT NOT NULL,
                code        TEXT NOT NULL,
                name        TEXT NOT NULL,
                quantity    REAL NOT NULL DEFAULT 0.0,
                net_value   REAL NOT NULL DEFAULT 0.0,
                sale_value  REAL NOT NULL DEFAULT 0.0,
                profit_value REAL NOT NULL DEFAULT 0.0,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                UNIQUE(owner_uid, code)
            );

            CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
            CREATE INDEX IF NOT EXISTS idx_messages_sent_at ON messages(sent_at);
            CREATE INDEX IF NOT EXISTS idx_products_owner  ON products(owner_uid);
            CREATE INDEX IF NOT EXISTS idx_contacts_owner  ON contacts(owner_uid);
        ")?;
        Ok(())
    }

    // ──────────────────────────────────────────
    // AUTH
    // ──────────────────────────────────────────

    /// Register a new user; password is hashed with Argon2id.
    pub fn register_user(&self, username: &str, display_name: &str, password: &str) -> Result<User> {
        // Check uniqueness
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM users WHERE username = ?1",
            params![username],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if exists {
            return Err(anyhow!("El nombre de usuario ya existe"));
        }

        // Hash password with Argon2id
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Error al cifrar contraseña: {}", e))?
            .to_string();

        let uid = format!(
            "NIM-{}",
            &uuid::Uuid::new_v4().to_string().to_uppercase()[..6]
        );
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO users (uid, username, display_name, password_hash, avatar_color, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![uid, username, display_name, hash, 0xFF_4A_90_E2u32, now],
        )?;

        let id = self.conn.last_insert_rowid();
        Ok(User {
            id,
            uid,
            username: username.to_string(),
            display_name: display_name.to_string(),
            avatar_color: 0xFF_4A_90_E2,
            created_at: now,
        })
    }

    /// Verify credentials and return the User if valid.
    pub fn login(&self, username: &str, password: &str) -> Result<User> {
        let result = self.conn.query_row(
            "SELECT id, uid, username, display_name, password_hash, avatar_color, created_at
             FROM users WHERE username = ?1",
            params![username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        );

        match result {
            Ok((id, uid, uname, display_name, hash_str, avatar_color, created_at)) => {
                let parsed_hash = PasswordHash::new(&hash_str)
                    .map_err(|e| anyhow!("Hash inválido: {}", e))?;
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .map_err(|_| anyhow!("Contraseña incorrecta"))?;
                Ok(User { id, uid, username: uname, display_name, avatar_color, created_at })
            }
            Err(_) => Err(anyhow!("Usuario no encontrado")),
        }
    }

    /// Update display name for a user.
    pub fn update_display_name(&self, uid: &str, display_name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE users SET display_name = ?1 WHERE uid = ?2",
            params![display_name, uid],
        )?;
        Ok(())
    }

    /// Update user password.
    pub fn update_password(&self, uid: &str, new_password: &str) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| anyhow!("{}", e))?
            .to_string();
        self.conn.execute(
            "UPDATE users SET password_hash = ?1 WHERE uid = ?2",
            params![hash, uid],
        )?;
        Ok(())
    }

    /// Save theme preference.
    pub fn update_theme(&self, uid: &str, theme: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE users SET theme = ?1 WHERE uid = ?2",
            params![theme, uid],
        )?;
        Ok(())
    }

    /// Get user settings.
    pub fn get_settings(&self, uid: &str) -> Result<AppSettings> {
        let (theme_str, notifications, font_size): (String, i64, f64) = self.conn.query_row(
            "SELECT theme, notifications, font_size FROM users WHERE uid = ?1",
            params![uid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        Ok(AppSettings {
            theme: if theme_str == "dark" { AppTheme::Dark } else { AppTheme::Light },
            notifications_enabled: notifications != 0,
            font_size: font_size as f32,
        })
    }

    // ──────────────────────────────────────────
    // CONTACTS
    // ──────────────────────────────────────────

    /// Find a user by their unique UID.
    pub fn find_user_by_uid(&self, uid: &str) -> Result<User> {
        self.conn.query_row(
            "SELECT id, uid, username, display_name, avatar_color, created_at FROM users WHERE uid = ?1",
            params![uid],
            |row| Ok(User {
                id: row.get(0)?,
                uid: row.get(1)?,
                username: row.get(2)?,
                display_name: row.get(3)?,
                avatar_color: row.get(4)?,
                created_at: row.get(5)?,
            }),
        ).map_err(|_| anyhow!("ID '{}' no encontrado", uid))
    }

    /// Add a contact (friend or acquaintance).
    pub fn add_contact(
        &self,
        owner_uid: &str,
        contact_uid: &str,
        display_name: &str,
        avatar_color: u32,
        contact_type: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO contacts
             (owner_uid, contact_uid, display_name, avatar_color, contact_type, starred, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            params![owner_uid, contact_uid, display_name, avatar_color, contact_type, now],
        )?;
        Ok(())
    }

    /// Toggle the starred state of a contact.
    pub fn toggle_star(&self, owner_uid: &str, contact_uid: &str) -> Result<bool> {
        let current: i64 = self.conn.query_row(
            "SELECT starred FROM contacts WHERE owner_uid = ?1 AND contact_uid = ?2",
            params![owner_uid, contact_uid],
            |r| r.get(0),
        )?;
        let new_val = if current == 0 { 1 } else { 0 };
        self.conn.execute(
            "UPDATE contacts SET starred = ?1 WHERE owner_uid = ?2 AND contact_uid = ?3",
            params![new_val, owner_uid, contact_uid],
        )?;
        Ok(new_val == 1)
    }

    /// Get all contacts of a user, sorted: starred first then A-Z.
    pub fn get_contacts(&self, owner_uid: &str, contact_type: &str) -> Result<Vec<Contact>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, owner_uid, contact_uid, display_name, avatar_color, contact_type, starred, added_at
             FROM contacts
             WHERE owner_uid = ?1 AND contact_type = ?2
             ORDER BY starred DESC, display_name ASC",
        )?;
        let rows = stmt.query_map(params![owner_uid, contact_type], |row| {
            Ok(Contact {
                id: row.get(0)?,
                owner_uid: row.get(1)?,
                contact_uid: row.get(2)?,
                display_name: row.get(3)?,
                avatar_color: row.get(4)?,
                contact_type: {
                    let t: String = row.get(5)?;
                    if t == "friend" { ContactType::Friend } else { ContactType::Acquaintance }
                },
                starred: row.get::<_, i64>(6)? != 0,
                added_at: row.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("{}", e))
    }

    /// Remove a contact.
    pub fn remove_contact(&self, owner_uid: &str, contact_uid: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM contacts WHERE owner_uid = ?1 AND contact_uid = ?2",
            params![owner_uid, contact_uid],
        )?;
        Ok(())
    }

    // ──────────────────────────────────────────
    // CHAT & MESSAGES
    // ──────────────────────────────────────────

    /// Get or create a chat between two users.
    pub fn get_or_create_chat(&self, uid_a: &str, uid_b: &str) -> Result<Chat> {
        let (a, b) = if uid_a < uid_b { (uid_a, uid_b) } else { (uid_b, uid_a) };

        // Try to find existing
        let existing = self.conn.query_row(
            "SELECT id, participant_a, participant_b, created_at, last_message, last_msg_at, unread_count
             FROM chats WHERE participant_a = ?1 AND participant_b = ?2",
            params![a, b],
            |row| Ok(Chat {
                id: row.get(0)?,
                participant_a: row.get(1)?,
                participant_b: row.get(2)?,
                created_at: row.get(3)?,
                last_message: row.get(4)?,
                last_message_at: row.get(5)?,
                unread_count: row.get::<_, u32>(6)?,
            }),
        );

        if let Ok(chat) = existing {
            return Ok(chat);
        }

        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO chats (participant_a, participant_b, created_at) VALUES (?1, ?2, ?3)",
            params![a, b, now],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Chat {
            id,
            participant_a: a.to_string(),
            participant_b: b.to_string(),
            created_at: now.clone(),
            last_message: None,
            last_message_at: None,
            unread_count: 0,
        })
    }

    /// Send a text message.
    pub fn send_message(
        &self,
        chat_id: i64,
        sender_uid: &str,
        content: &str,
        msg_type: &str,
        file_name: Option<&str>,
        file_size: Option<u64>,
    ) -> Result<Message> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO messages (chat_id, sender_uid, content, msg_type, file_name, file_size, sent_at, is_read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
            params![chat_id, sender_uid, content, msg_type, file_name, file_size.map(|s| s as i64), now],
        )?;
        let id = self.conn.last_insert_rowid();

        // Update last message on chat
        let preview = if msg_type == "text" {
            content.chars().take(50).collect::<String>()
        } else {
            format!("[{}]", msg_type)
        };
        self.conn.execute(
            "UPDATE chats SET last_message = ?1, last_msg_at = ?2 WHERE id = ?3",
            params![preview, now, chat_id],
        )?;

        Ok(Message {
            id,
            chat_id,
            sender_uid: sender_uid.to_string(),
            content: content.to_string(),
            msg_type: MessageType::from_str(msg_type),
            file_name: file_name.map(str::to_string),
            file_size,
            sent_at: now,
            is_read: false,
        })
    }

    /// Load messages for a chat (paginated, newest last).
    pub fn get_messages(&self, chat_id: i64, limit: usize, offset: usize) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chat_id, sender_uid, content, msg_type, file_name, file_size, sent_at, is_read
             FROM messages WHERE chat_id = ?1
             ORDER BY sent_at ASC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(params![chat_id, limit as i64, offset as i64], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_uid: row.get(2)?,
                content: row.get(3)?,
                msg_type: {
                    let t: String = row.get(4)?;
                    MessageType::from_str(&t)
                },
                file_name: row.get(5)?,
                file_size: row.get::<_, Option<i64>>(6)?.map(|s| s as u64),
                sent_at: row.get(7)?,
                is_read: row.get::<_, i64>(8)? != 0,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("{}", e))
    }

    // ──────────────────────────────────────────
    // PRODUCTS / INVENTORY
    // ──────────────────────────────────────────

    /// Insert or replace a product.
    pub fn upsert_product(&self, p: &Product) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        if p.id == 0 {
            self.conn.execute(
                "INSERT INTO products
                 (owner_uid, code, name, quantity, net_value, sale_value, profit_value, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    p.owner_uid, p.code, p.name, p.quantity,
                    p.net_value, p.sale_value, p.profit_value, now
                ],
            )?;
            Ok(self.conn.last_insert_rowid())
        } else {
            self.conn.execute(
                "UPDATE products SET code=?1, name=?2, quantity=?3, net_value=?4,
                 sale_value=?5, profit_value=?6, updated_at=?7
                 WHERE id=?8",
                params![
                    p.code, p.name, p.quantity, p.net_value,
                    p.sale_value, p.profit_value, now, p.id
                ],
            )?;
            Ok(p.id)
        }
    }

    /// Get all products for a user.
    pub fn get_products(&self, owner_uid: &str) -> Result<Vec<Product>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, owner_uid, code, name, quantity, net_value, sale_value, profit_value, created_at, updated_at
             FROM products WHERE owner_uid = ?1
             ORDER BY name ASC",
        )?;
        let rows = stmt.query_map(params![owner_uid], |row| {
            Ok(Product {
                id: row.get(0)?,
                owner_uid: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
                quantity: row.get(4)?,
                net_value: row.get(5)?,
                sale_value: row.get(6)?,
                profit_value: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("{}", e))
    }

    /// Delete a product by ID.
    pub fn delete_product(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM products WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Compute summary stats for the inventory dashboard.
    pub fn inventory_summary(&self, owner_uid: &str) -> Result<InventorySummary> {
        let (total_products, total_net, total_profit): (i64, f64, f64) = self.conn.query_row(
            "SELECT COUNT(*), SUM(quantity * net_value), SUM(quantity * profit_value)
             FROM products WHERE owner_uid = ?1",
            params![owner_uid],
            |r| Ok((r.get(0)?, r.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                     r.get::<_, Option<f64>>(2)?.unwrap_or(0.0))),
        )?;
        let out_of_stock: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM products WHERE owner_uid = ?1 AND quantity < 1",
            params![owner_uid],
            |r| r.get(0),
        )?;
        Ok(InventorySummary {
            total_products: total_products as u64,
            total_net_value: total_net,
            total_profit_value: total_profit,
            out_of_stock_count: out_of_stock as u64,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct InventorySummary {
    pub total_products: u64,
    pub total_net_value: f64,
    pub total_profit_value: f64,
    pub out_of_stock_count: u64,
}
