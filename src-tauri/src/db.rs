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
