use crate::db::DatabaseExt;
use crate::models::{DayClosing, DaySummary, Order, OrderItem, OrderWithItems};
use tauri::AppHandle;

#[tauri::command]
pub fn close_day(app: AppHandle) -> Result<DayClosing, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Check if day is already closed
    let existing: Result<i64, _> = conn.query_row(
        "SELECT id FROM day_closings WHERE date = ?1",
        [&today],
        |row| row.get(0),
    );

    if existing.is_ok() {
        return Err("Day has already been closed".to_string());
    }

    // Calculate today's totals
    let (total_revenue, total_orders): (f64, i32) = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM orders WHERE date(created_at, 'localtime') = date('now', 'localtime')",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Insert day closing
    conn.execute(
        "INSERT INTO day_closings (date, total_revenue, total_orders) VALUES (?1, ?2, ?3)",
        rusqlite::params![today, total_revenue, total_orders],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let closed_at: String = conn
        .query_row(
            "SELECT closed_at FROM day_closings WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(DayClosing {
        id,
        date: today,
        total_revenue,
        total_orders,
        closed_at,
    })
}

#[tauri::command]
pub fn get_sales_history(app: AppHandle, limit: Option<i32>) -> Result<Vec<DayClosing>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(30);

    let mut stmt = conn
        .prepare(
            "SELECT id, date, total_revenue, total_orders, closed_at
             FROM day_closings
             ORDER BY date DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let closings = stmt
        .query_map([limit], |row| {
            Ok(DayClosing {
                id: row.get(0)?,
                date: row.get(1)?,
                total_revenue: row.get(2)?,
                total_orders: row.get(3)?,
                closed_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(closings)
}

#[tauri::command]
pub fn get_day_summary(app: AppHandle, date: Option<String>) -> Result<DaySummary, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    // Get orders for the date (convert stored UTC timestamp to localtime for comparison)
    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE date(o.created_at, 'localtime') = ?1
             ORDER BY o.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders: Vec<Order> = stmt
        .query_map([&date], |row| {
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
        date,
        total_revenue,
        total_orders: orders_with_items.len() as i32,
        orders: orders_with_items,
    })
}
