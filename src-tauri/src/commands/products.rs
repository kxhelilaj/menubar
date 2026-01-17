use crate::db::DatabaseExt;
use crate::models::{CreateProduct, Product, UpdateProduct};
use tauri::AppHandle;

#[tauri::command]
pub fn get_products(app: AppHandle) -> Result<Vec<Product>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price, p.quantity, p.category_id, c.name, p.low_stock_threshold, p.created_at
             FROM products p
             LEFT JOIN categories c ON p.category_id = c.id
             ORDER BY p.name",
        )
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                price: row.get(2)?,
                quantity: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                low_stock_threshold: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

#[tauri::command]
pub fn create_product(app: AppHandle, product: CreateProduct) -> Result<Product, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let threshold = product.low_stock_threshold.unwrap_or(5);

    conn.execute(
        "INSERT INTO products (name, price, quantity, category_id, low_stock_threshold) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![product.name, product.price, product.quantity, product.category_id, threshold],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price, p.quantity, p.category_id, c.name, p.low_stock_threshold, p.created_at
             FROM products p
             LEFT JOIN categories c ON p.category_id = c.id
             WHERE p.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let product = stmt
        .query_row([id], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                price: row.get(2)?,
                quantity: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                low_stock_threshold: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(product)
}

#[tauri::command]
pub fn update_product(app: AppHandle, product: UpdateProduct) -> Result<Product, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE products SET name = ?1, price = ?2, quantity = ?3, category_id = ?4, low_stock_threshold = ?5 WHERE id = ?6",
        rusqlite::params![product.name, product.price, product.quantity, product.category_id, product.low_stock_threshold, product.id],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price, p.quantity, p.category_id, c.name, p.low_stock_threshold, p.created_at
             FROM products p
             LEFT JOIN categories c ON p.category_id = c.id
             WHERE p.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let product = stmt
        .query_row([product.id], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                price: row.get(2)?,
                quantity: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                low_stock_threshold: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(product)
}

#[tauri::command]
pub fn delete_product(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM products WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_low_stock(app: AppHandle) -> Result<Vec<Product>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price, p.quantity, p.category_id, c.name, p.low_stock_threshold, p.created_at
             FROM products p
             LEFT JOIN categories c ON p.category_id = c.id
             WHERE p.quantity <= p.low_stock_threshold
             ORDER BY p.quantity ASC",
        )
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                price: row.get(2)?,
                quantity: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                low_stock_threshold: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}
