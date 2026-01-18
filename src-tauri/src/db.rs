use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");

        std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

        let db_path: PathBuf = app_dir.join("pub_inventory.db");
        let conn = Connection::open(db_path)?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            -- Product categories
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            -- Products with inventory
            CREATE TABLE IF NOT EXISTS products (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                category_id INTEGER,
                low_stock_threshold INTEGER DEFAULT 5,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );

            -- Staff members
            CREATE TABLE IF NOT EXISTS staff (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                pin TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Orders
            CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                staff_id INTEGER NOT NULL,
                table_number INTEGER NOT NULL DEFAULT 1,
                total REAL NOT NULL,
                customer_name TEXT,
                notes TEXT,
                status TEXT DEFAULT 'open',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (staff_id) REFERENCES staff(id)
            );

            -- Order items
            CREATE TABLE IF NOT EXISTS order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                product_id INTEGER NOT NULL,
                quantity INTEGER NOT NULL,
                price_at_sale REAL NOT NULL,
                FOREIGN KEY (order_id) REFERENCES orders(id),
                FOREIGN KEY (product_id) REFERENCES products(id)
            );

            -- Day closings
            CREATE TABLE IF NOT EXISTS day_closings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date DATE NOT NULL UNIQUE,
                total_revenue REAL NOT NULL,
                total_orders INTEGER NOT NULL,
                closed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Day sessions (tracks when day is open for business)
            CREATE TABLE IF NOT EXISTS day_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date DATE NOT NULL UNIQUE,
                started_by INTEGER NOT NULL,
                started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active INTEGER DEFAULT 1,
                FOREIGN KEY (started_by) REFERENCES staff(id)
            );
            "
        )?;

        // Run migrations for existing databases (pass connection to avoid deadlock)
        Self::migrate_conn(&conn)?;

        Ok(())
    }

    fn migrate_conn(conn: &Connection) -> Result<()> {
        // Check if customer_name column exists, add if not
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(orders)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        if !columns.contains(&"customer_name".to_string()) {
            conn.execute("ALTER TABLE orders ADD COLUMN customer_name TEXT", [])?;
        }
        if !columns.contains(&"notes".to_string()) {
            conn.execute("ALTER TABLE orders ADD COLUMN notes TEXT", [])?;
        }
        if !columns.contains(&"status".to_string()) {
            conn.execute("ALTER TABLE orders ADD COLUMN status TEXT DEFAULT 'paid'", [])?;
        }
        if !columns.contains(&"table_number".to_string()) {
            conn.execute("ALTER TABLE orders ADD COLUMN table_number INTEGER NOT NULL DEFAULT 1", [])?;
        }

        // Session-based architecture migration
        // Add session_id to orders table
        if !columns.contains(&"session_id".to_string()) {
            conn.execute("ALTER TABLE orders ADD COLUMN session_id INTEGER", [])?;
        }

        // Add closing fields to day_sessions
        let session_columns: Vec<String> = conn
            .prepare("PRAGMA table_info(day_sessions)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        if !session_columns.contains(&"closed_at".to_string()) {
            conn.execute("ALTER TABLE day_sessions ADD COLUMN closed_at DATETIME", [])?;
        }
        if !session_columns.contains(&"total_revenue".to_string()) {
            conn.execute("ALTER TABLE day_sessions ADD COLUMN total_revenue REAL", [])?;
        }
        if !session_columns.contains(&"total_orders".to_string()) {
            conn.execute("ALTER TABLE day_sessions ADD COLUMN total_orders INTEGER", [])?;
        }

        // Backfill session_id for existing orders that don't have one
        conn.execute(
            "UPDATE orders SET session_id = (
                SELECT ds.id FROM day_sessions ds
                WHERE orders.created_at >= ds.started_at
                ORDER BY ds.started_at DESC LIMIT 1
            ) WHERE session_id IS NULL",
            [],
        )?;

        // Create index for session_id lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_orders_session_id ON orders(session_id)",
            [],
        )?;

        // Remove UNIQUE constraint on date in day_sessions by recreating the table
        // Check if we need to migrate (look for UNIQUE constraint)
        let table_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='day_sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        if table_sql.contains("UNIQUE") {
            // Need to recreate table without UNIQUE constraint
            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS day_sessions_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date DATE,
                    started_by INTEGER NOT NULL,
                    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    is_active INTEGER DEFAULT 1,
                    closed_at DATETIME,
                    total_revenue REAL,
                    total_orders INTEGER,
                    FOREIGN KEY (started_by) REFERENCES staff(id)
                );

                INSERT INTO day_sessions_new (id, date, started_by, started_at, is_active, closed_at, total_revenue, total_orders)
                SELECT id, date, started_by, started_at, is_active, closed_at, total_revenue, total_orders FROM day_sessions;

                DROP TABLE day_sessions;

                ALTER TABLE day_sessions_new RENAME TO day_sessions;
                ",
            )?;
        }

        Ok(())
    }
}

use tauri::Manager;

pub trait DatabaseExt {
    fn db(&self) -> &Database;
}

impl DatabaseExt for AppHandle {
    fn db(&self) -> &Database {
        self.state::<Database>().inner()
    }
}
