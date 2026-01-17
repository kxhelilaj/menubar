use crate::db::DatabaseExt;
use crate::models::{CreateStaff, Staff};
use tauri::AppHandle;

#[tauri::command]
pub fn get_staff(app: AppHandle) -> Result<Vec<Staff>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, pin, created_at FROM staff ORDER BY name")
        .map_err(|e| e.to_string())?;

    let staff = stmt
        .query_map([], |row| {
            Ok(Staff {
                id: row.get(0)?,
                name: row.get(1)?,
                pin: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(staff)
}

#[tauri::command]
pub fn create_staff(app: AppHandle, staff: CreateStaff) -> Result<Staff, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO staff (name, pin) VALUES (?1, ?2)",
        rusqlite::params![staff.name, staff.pin],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let mut stmt = conn
        .prepare("SELECT id, name, pin, created_at FROM staff WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let staff = stmt
        .query_row([id], |row| {
            Ok(Staff {
                id: row.get(0)?,
                name: row.get(1)?,
                pin: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(staff)
}

#[tauri::command]
pub fn delete_staff(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if staff has orders
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM orders WHERE staff_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err("Cannot delete staff member with existing orders".to_string());
    }

    conn.execute("DELETE FROM staff WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn verify_staff_pin(app: AppHandle, id: i64, pin: String) -> Result<bool, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let stored_pin: Option<String> = conn
        .query_row("SELECT pin FROM staff WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    match stored_pin {
        Some(p) => Ok(p == pin),
        None => Ok(true), // No PIN set, allow access
    }
}
