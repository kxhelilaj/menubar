use crate::db::DatabaseExt;
use crate::models::{DaySession, DaySummary, Order, OrderItem, OrderWithItems};
use tauri::{AppHandle, Manager};
use std::fs;
use std::io::Write;

/// Get all orders within a date range (for recovery/reporting)
#[tauri::command]
pub fn get_orders_by_date_range(
    app: AppHandle,
    start_date: String,
    end_date: String,
) -> Result<Vec<OrderWithItems>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE date(o.created_at, 'localtime') >= ?1 AND date(o.created_at, 'localtime') <= ?2
             ORDER BY o.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders: Vec<Order> = stmt
        .query_map([&start_date, &end_date], |row| {
            Ok(Order {
                id: row.get(0)?,
                staff_id: row.get(1)?,
                staff_name: row.get(2)?,
                table_number: row.get::<_, Option<i32>>(3)?.unwrap_or(1),
                total: row.get(4)?,
                customer_name: row.get(5)?,
                notes: row.get(6)?,
                status: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "paid".to_string()),
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for order in orders {
        let mut item_stmt = conn
            .prepare(
                "SELECT oi.id, oi.order_id, oi.product_id, p.name, oi.quantity, oi.price_at_sale
                 FROM order_items oi
                 LEFT JOIN products p ON oi.product_id = p.id
                 WHERE oi.order_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let items: Vec<OrderItem> = item_stmt
            .query_map([order.id], |row| {
                Ok(OrderItem {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    product_id: row.get(2)?,
                    product_name: row.get(3)?,
                    quantity: row.get(4)?,
                    price_at_sale: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result.push(OrderWithItems { order, items });
    }

    Ok(result)
}

/// Manually create a day closing for a specific date (for recovery)
/// This creates a closed session for orders from legacy data or missed closings
#[tauri::command]
pub fn create_day_closing_for_date(app: AppHandle, date: String) -> Result<DaySession, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if a closed session already exists for this date
    let existing: Result<i64, _> = conn.query_row(
        "SELECT id FROM day_sessions WHERE date = ?1 AND is_active = 0",
        [&date],
        |row| row.get(0),
    );

    if existing.is_ok() {
        return Err(format!("A closed session already exists for {}", date));
    }

    // Calculate totals for that date (orders without session_id or with matching date)
    let (total_revenue, total_orders): (f64, i32) = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM orders WHERE date(created_at, 'localtime') = ?1",
            [&date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    if total_orders == 0 {
        return Err(format!("No orders found for {}", date));
    }

    // Get first staff member as the "started_by" (for recovery purposes)
    let staff_id: i64 = conn
        .query_row("SELECT id FROM staff LIMIT 1", [], |row| row.get(0))
        .unwrap_or(1);

    // Create a closed session for recovery
    conn.execute(
        "INSERT INTO day_sessions (date, started_by, started_at, is_active, closed_at, total_revenue, total_orders)
         VALUES (?1, ?2, ?1 || ' 00:00:00', 0, CURRENT_TIMESTAMP, ?3, ?4)",
        rusqlite::params![date, staff_id, total_revenue, total_orders],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    // Link orders to this session
    conn.execute(
        "UPDATE orders SET session_id = ?1 WHERE date(created_at, 'localtime') = ?2 AND session_id IS NULL",
        rusqlite::params![id, date],
    )
    .map_err(|e| e.to_string())?;

    let (started_at, closed_at): (String, String) = conn
        .query_row(
            "SELECT started_at, closed_at FROM day_sessions WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let staff_name: Option<String> = conn
        .query_row("SELECT name FROM staff WHERE id = ?1", [staff_id], |row| row.get(0))
        .ok();

    Ok(DaySession {
        id,
        date: Some(date),
        started_by: staff_id,
        started_by_name: staff_name,
        started_at,
        closed_at: Some(closed_at),
        is_active: false,
        total_revenue: Some(total_revenue),
        total_orders: Some(total_orders),
    })
}

#[tauri::command]
pub fn close_day(app: AppHandle) -> Result<DaySession, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| format!("Database lock failed: {}", e))?;

    // Check for active session first - get the start time
    let (session_id, session_date, session_started_at): (i64, Option<String>, String) = conn
        .query_row(
            "SELECT id, date, started_at FROM day_sessions WHERE is_active = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "No active day session to close.".to_string())?;

    println!("[close_day] Closing session {} started at {}", session_id, session_started_at);

    // Calculate totals for ALL orders linked to this session
    let (total_revenue, total_orders): (f64, i32) = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM orders WHERE session_id = ?1",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to calculate totals: {}", e))?;

    println!("[close_day] Found {} orders for session, total revenue: {} ALL", total_orders, total_revenue);

    if total_orders == 0 {
        return Err("No orders found for this session. Cannot close an empty day.".to_string());
    }

    // Check for open orders (tables) - they must be closed first
    let open_orders: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM orders WHERE session_id = ?1 AND status = 'open'",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check for open orders: {}", e))?;

    if open_orders > 0 {
        return Err(format!("Cannot close day: {} tables are still open. Close all tables first.", open_orders));
    }

    // Update the session record with closing data (no longer using day_closings table)
    conn.execute(
        "UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP, total_revenue = ?1, total_orders = ?2 WHERE id = ?3",
        rusqlite::params![total_revenue, total_orders, session_id],
    )
    .map_err(|e| format!("Failed to close day session: {}", e))?;

    // Get the closed_at timestamp
    let closed_at: String = conn
        .query_row(
            "SELECT closed_at FROM day_sessions WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Auto-backup: Save session's orders to a JSON file
    let backup_result = (|| -> Result<String, String> {
        // Get all orders with items for this session
        let mut stmt = conn
            .prepare(
                "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
                 FROM orders o
                 LEFT JOIN staff s ON o.staff_id = s.id
                 WHERE o.session_id = ?1
                 ORDER BY o.created_at ASC",
            )
            .map_err(|e| e.to_string())?;

        let orders: Vec<Order> = stmt
            .query_map([session_id], |row| {
                Ok(Order {
                    id: row.get(0)?,
                    staff_id: row.get(1)?,
                    staff_name: row.get(2)?,
                    table_number: row.get::<_, Option<i32>>(3)?.unwrap_or(1),
                    total: row.get(4)?,
                    customer_name: row.get(5)?,
                    notes: row.get(6)?,
                    status: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "paid".to_string()),
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut orders_with_items = Vec::new();
        for order in orders {
            let mut item_stmt = conn
                .prepare(
                    "SELECT oi.id, oi.order_id, oi.product_id, p.name, oi.quantity, oi.price_at_sale
                     FROM order_items oi
                     LEFT JOIN products p ON oi.product_id = p.id
                     WHERE oi.order_id = ?1",
                )
                .map_err(|e| e.to_string())?;

            let items: Vec<OrderItem> = item_stmt
                .query_map([order.id], |row| {
                    Ok(OrderItem {
                        id: row.get(0)?,
                        order_id: row.get(1)?,
                        product_id: row.get(2)?,
                        product_name: row.get(3)?,
                        quantity: row.get(4)?,
                        price_at_sale: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            orders_with_items.push(OrderWithItems { order, items });
        }

        // Create backup data structure
        let backup_data = serde_json::json!({
            "session_id": session_id,
            "date": session_date,
            "session_started_at": session_started_at,
            "closed_at": closed_at,
            "total_revenue": total_revenue,
            "total_orders": total_orders,
            "orders": orders_with_items,
        });

        // Get app data directory and create backups folder
        let app_data_dir = app.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        let backups_dir = app_data_dir.join("backups");

        fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;

        // Write backup file (include timestamp to handle multiple sessions per day)
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_file = backups_dir.join(format!("day-backup-{}.json", timestamp));
        let mut file = fs::File::create(&backup_file)
            .map_err(|e| format!("Failed to create backup file: {}", e))?;

        let json_content = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| format!("Failed to serialize backup: {}", e))?;

        file.write_all(json_content.as_bytes())
            .map_err(|e| format!("Failed to write backup: {}", e))?;

        Ok(backup_file.to_string_lossy().to_string())
    })();

    match backup_result {
        Ok(path) => println!("[close_day] Auto-backup saved to: {}", path),
        Err(e) => println!("[close_day] Warning: Auto-backup failed: {}", e),
    }

    let date_str = session_date.clone().unwrap_or_else(|| "unknown".to_string());
    println!("[close_day] Successfully closed session for {} with {} orders", date_str, total_orders);

    // Get staff name for the return value
    let started_by: i64 = conn
        .query_row("SELECT started_by FROM day_sessions WHERE id = ?1", [session_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let started_by_name: Option<String> = conn
        .query_row("SELECT name FROM staff WHERE id = ?1", [started_by], |row| row.get(0))
        .ok();

    Ok(DaySession {
        id: session_id,
        date: session_date,
        started_by,
        started_by_name,
        started_at: session_started_at,
        closed_at: Some(closed_at),
        is_active: false,
        total_revenue: Some(total_revenue),
        total_orders: Some(total_orders),
    })
}

#[tauri::command]
pub fn get_sales_history(app: AppHandle, limit: Option<i32>) -> Result<Vec<DaySession>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(30);

    let mut stmt = conn
        .prepare(
            "SELECT ds.id, ds.date, ds.started_by, s.name, ds.started_at, ds.closed_at, ds.is_active, ds.total_revenue, ds.total_orders
             FROM day_sessions ds
             LEFT JOIN staff s ON ds.started_by = s.id
             WHERE ds.is_active = 0 AND ds.closed_at IS NOT NULL
             ORDER BY ds.closed_at DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([limit], |row| {
            Ok(DaySession {
                id: row.get(0)?,
                date: row.get(1)?,
                started_by: row.get(2)?,
                started_by_name: row.get(3)?,
                started_at: row.get(4)?,
                closed_at: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
                total_revenue: row.get(7)?,
                total_orders: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn get_day_summary(app: AppHandle, sessionId: Option<i64>) -> Result<DaySummary, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // If a specific session ID is requested (for historical view), query by session_id
    if let Some(session_id) = sessionId {
        // Get session info for the date field
        let session_date: Option<String> = conn
            .query_row(
                "SELECT date FROM day_sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .ok();

        let date_str = session_date.unwrap_or_else(|| "Unknown".to_string());

        let mut stmt = conn
            .prepare(
                "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
                 FROM orders o
                 LEFT JOIN staff s ON o.staff_id = s.id
                 WHERE o.session_id = ?1
                 ORDER BY o.created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let orders: Vec<Order> = stmt
            .query_map([session_id], |row| {
                Ok(Order {
                    id: row.get(0)?,
                    staff_id: row.get(1)?,
                    staff_name: row.get(2)?,
                    table_number: row.get::<_, Option<i32>>(3)?.unwrap_or(1),
                    total: row.get(4)?,
                    customer_name: row.get(5)?,
                    notes: row.get(6)?,
                    status: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "paid".to_string()),
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut orders_with_items = Vec::new();
        let mut total_revenue = 0.0;

        for order in orders {
            total_revenue += order.total;

            let mut item_stmt = conn
                .prepare(
                    "SELECT oi.id, oi.order_id, oi.product_id, p.name, oi.quantity, oi.price_at_sale
                     FROM order_items oi
                     LEFT JOIN products p ON oi.product_id = p.id
                     WHERE oi.order_id = ?1",
                )
                .map_err(|e| e.to_string())?;

            let items: Vec<OrderItem> = item_stmt
                .query_map([order.id], |row| {
                    Ok(OrderItem {
                        id: row.get(0)?,
                        order_id: row.get(1)?,
                        product_id: row.get(2)?,
                        product_name: row.get(3)?,
                        quantity: row.get(4)?,
                        price_at_sale: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            orders_with_items.push(OrderWithItems { order, items });
        }

        return Ok(DaySummary {
            date: date_str,
            total_revenue,
            total_orders: orders_with_items.len() as i32,
            orders: orders_with_items,
        });
    }

    // For "today" (no session_id param), use the active session
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Check for active session
    let active_session_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM day_sessions WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .ok();

    // If no active session, return empty summary
    let session_id = match active_session_id {
        Some(id) => id,
        None => {
            return Ok(DaySummary {
                date: today,
                total_revenue: 0.0,
                total_orders: 0,
                orders: Vec::new(),
            });
        }
    };

    // Get orders for the active session
    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE o.session_id = ?1
             ORDER BY o.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders: Vec<Order> = stmt
        .query_map([session_id], |row| {
            Ok(Order {
                id: row.get(0)?,
                staff_id: row.get(1)?,
                staff_name: row.get(2)?,
                table_number: row.get::<_, Option<i32>>(3)?.unwrap_or(1),
                total: row.get(4)?,
                customer_name: row.get(5)?,
                notes: row.get(6)?,
                status: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "paid".to_string()),
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut orders_with_items = Vec::new();
    let mut total_revenue = 0.0;

    for order in orders {
        total_revenue += order.total;

        let mut item_stmt = conn
            .prepare(
                "SELECT oi.id, oi.order_id, oi.product_id, p.name, oi.quantity, oi.price_at_sale
                 FROM order_items oi
                 LEFT JOIN products p ON oi.product_id = p.id
                 WHERE oi.order_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let items: Vec<OrderItem> = item_stmt
            .query_map([order.id], |row| {
                Ok(OrderItem {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    product_id: row.get(2)?,
                    product_name: row.get(3)?,
                    quantity: row.get(4)?,
                    price_at_sale: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        orders_with_items.push(OrderWithItems { order, items });
    }

    Ok(DaySummary {
        date: today,
        total_revenue,
        total_orders: orders_with_items.len() as i32,
        orders: orders_with_items,
    })
}

// ============ DAY SESSION MANAGEMENT ============

/// Get the current active day session (if any)
#[tauri::command]
pub fn get_active_session(app: AppHandle) -> Result<Option<DaySession>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get any active session (not date-specific)
    let result = conn.query_row(
        "SELECT ds.id, ds.date, ds.started_by, s.name, ds.started_at, ds.closed_at, ds.is_active, ds.total_revenue, ds.total_orders
         FROM day_sessions ds
         LEFT JOIN staff s ON ds.started_by = s.id
         WHERE ds.is_active = 1
         ORDER BY ds.started_at DESC
         LIMIT 1",
        [],
        |row| {
            Ok(DaySession {
                id: row.get(0)?,
                date: row.get(1)?,
                started_by: row.get(2)?,
                started_by_name: row.get(3)?,
                started_at: row.get(4)?,
                closed_at: row.get(5)?,
                is_active: row.get::<_, i32>(6)? == 1,
                total_revenue: row.get(7)?,
                total_orders: row.get(8)?,
            })
        },
    );

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Start a new day session
#[tauri::command]
pub fn start_day(app: AppHandle, staff_id: i64) -> Result<DaySession, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    println!("[start_day] Starting day for date: {}", today);

    // Check if there's already an active session (any date)
    let existing: Result<i64, _> = conn.query_row(
        "SELECT id FROM day_sessions WHERE is_active = 1",
        [],
        |row| row.get(0),
    );

    if existing.is_ok() {
        return Err("A day session is already active. Close it first.".to_string());
    }

    // Create new session (allow multiple sessions per day - no ON CONFLICT)
    conn.execute(
        "INSERT INTO day_sessions (date, started_by, is_active) VALUES (?1, ?2, 1)",
        rusqlite::params![today, staff_id],
    )
    .map_err(|e| format!("Failed to start day: {}", e))?;

    let id = conn.last_insert_rowid();

    // Get staff name and started_at
    let (staff_name, started_at): (Option<String>, String) = conn
        .query_row(
            "SELECT s.name, ds.started_at
             FROM day_sessions ds
             LEFT JOIN staff s ON ds.started_by = s.id
             WHERE ds.id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    println!("[start_day] Day started successfully by staff_id: {}", staff_id);

    Ok(DaySession {
        id,
        date: Some(today),
        started_by: staff_id,
        started_by_name: staff_name,
        started_at,
        closed_at: None,
        is_active: true,
        total_revenue: None,
        total_orders: None,
    })
}

/// Check if day is active (for order validation)
#[tauri::command]
pub fn is_day_active(app: AppHandle) -> Result<bool, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let result: Result<i32, _> = conn.query_row(
        "SELECT 1 FROM day_sessions WHERE is_active = 1",
        [],
        |row| row.get(0),
    );

    Ok(result.is_ok())
}
