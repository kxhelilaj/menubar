use crate::db::DatabaseExt;
use crate::models::Category;
use tauri::AppHandle;

#[tauri::command]
pub fn get_categories(app: AppHandle) -> Result<Vec<Category>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM categories ORDER BY name")
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(categories)
}

#[tauri::command]
pub fn create_category(app: AppHandle, name: String) -> Result<Category, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("INSERT INTO categories (name) VALUES (?1)", [&name])
        .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(Category { id, name })
}

#[tauri::command]
pub fn delete_category(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Set category_id to NULL for products in this category
    conn.execute(
        "UPDATE products SET category_id = NULL WHERE category_id = ?1",
        [id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
