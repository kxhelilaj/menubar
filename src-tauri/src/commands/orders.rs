use crate::db::DatabaseExt;
use crate::models::{CreateOrder, CreateOrderItem, Order, OrderItem, OrderWithItems};
use tauri::AppHandle;

#[tauri::command]
pub fn create_order(app: AppHandle, order: CreateOrder) -> Result<OrderWithItems, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if there's an active day session and get its ID
    let session_id: i64 = conn
        .query_row(
            "SELECT id FROM day_sessions WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Day is not started. Please start the day first.".to_string())?;

    // Calculate total and validate stock
    let mut total = 0.0;
    let mut item_details: Vec<(i64, i32, f64, String)> = Vec::new();

    for item in &order.items {
        let (price, quantity, name): (f64, i32, String) = conn
            .query_row(
                "SELECT price, quantity, name FROM products WHERE id = ?1",
                [item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Product not found: {}", e))?;

        if quantity < item.quantity {
            return Err(format!(
                "Insufficient stock for {}: requested {}, available {}",
                name, item.quantity, quantity
            ));
        }

        total += price * item.quantity as f64;
        item_details.push((item.product_id, item.quantity, price, name));
    }

    // Create order with status 'open' and link to session
    conn.execute(
        "INSERT INTO orders (staff_id, table_number, total, customer_name, notes, status, session_id) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6)",
        rusqlite::params![order.staff_id, order.table_number, total, order.customer_name, order.notes, session_id],
    )
    .map_err(|e| e.to_string())?;

    let order_id = conn.last_insert_rowid();

    // Create order items and deduct inventory
    let mut items = Vec::new();
    for (product_id, qty, price, name) in item_details {
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![order_id, product_id, qty, price],
        )
        .map_err(|e| e.to_string())?;

        let item_id = conn.last_insert_rowid();

        // Deduct inventory
        conn.execute(
            "UPDATE products SET quantity = quantity - ?1 WHERE id = ?2",
            rusqlite::params![qty, product_id],
        )
        .map_err(|e| e.to_string())?;

        items.push(OrderItem {
            id: item_id,
            order_id,
            product_id,
            product_name: Some(name),
            quantity: qty,
            price_at_sale: price,
        });
    }

    // Get staff name
    let staff_name: Option<String> = conn
        .query_row(
            "SELECT name FROM staff WHERE id = ?1",
            [order.staff_id],
            |row| row.get(0),
        )
        .ok();

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM orders WHERE id = ?1",
            [order_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(OrderWithItems {
        order: Order {
            id: order_id,
            staff_id: order.staff_id,
            staff_name,
            table_number: order.table_number,
            total,
            customer_name: order.customer_name,
            notes: order.notes,
            status: "open".to_string(),
            created_at,
        },
        items,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn add_items_to_order(app: AppHandle, orderId: i64, items: Vec<CreateOrderItem>) -> Result<OrderWithItems, String> {
    let order_id = orderId;
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check order exists and is open
    let status: String = conn
        .query_row(
            "SELECT status FROM orders WHERE id = ?1",
            [order_id],
            |row| row.get(0),
        )
        .map_err(|_| "Order not found".to_string())?;

    if status != "open" {
        return Err("Cannot add items to a paid order".to_string());
    }

    // Calculate additional total and validate stock
    let mut additional_total = 0.0;
    let mut item_details: Vec<(i64, i32, f64, String)> = Vec::new();

    for item in &items {
        let (price, quantity, name): (f64, i32, String) = conn
            .query_row(
                "SELECT price, quantity, name FROM products WHERE id = ?1",
                [item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Product not found: {}", e))?;

        if quantity < item.quantity {
            return Err(format!(
                "Insufficient stock for {}: requested {}, available {}",
                name, item.quantity, quantity
            ));
        }

        additional_total += price * item.quantity as f64;
        item_details.push((item.product_id, item.quantity, price, name));
    }

    // Add items and deduct inventory
    for (product_id, qty, price, _name) in &item_details {
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![order_id, product_id, qty, price],
        )
        .map_err(|e| e.to_string())?;

        // Deduct inventory
        conn.execute(
            "UPDATE products SET quantity = quantity - ?1 WHERE id = ?2",
            rusqlite::params![qty, product_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update order total
    conn.execute(
        "UPDATE orders SET total = total + ?1 WHERE id = ?2",
        rusqlite::params![additional_total, order_id],
    )
    .map_err(|e| e.to_string())?;

    // Drop the lock before calling get_order
    drop(conn);

    // Return updated order
    get_order(app, order_id)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn mark_order_paid(app: AppHandle, orderId: i64) -> Result<OrderWithItems, String> {
    let order_id = orderId;
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE orders SET status = 'paid' WHERE id = ?1 AND status = 'open'",
        [order_id],
    )
    .map_err(|e| e.to_string())?;

    if conn.changes() == 0 {
        return Err("Order not found or already paid".to_string());
    }

    drop(conn);
    get_order(app, order_id)
}

/// Decrease item quantity by 1. If quantity becomes 0, remove the item.
/// If order has no items left, delete the order.
#[tauri::command]
#[allow(non_snake_case)]
pub fn decrease_item_quantity(app: AppHandle, orderItemId: i64) -> Result<Option<OrderWithItems>, String> {
    let item_id = orderItemId;
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get the item details
    let (order_id, product_id, quantity, price_at_sale): (i64, i64, i32, f64) = conn
        .query_row(
            "SELECT order_id, product_id, quantity, price_at_sale FROM order_items WHERE id = ?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| "Order item not found".to_string())?;

    // Check if order is still open
    let status: String = conn
        .query_row(
            "SELECT status FROM orders WHERE id = ?1",
            [order_id],
            |row| row.get(0),
        )
        .map_err(|_| "Order not found".to_string())?;

    if status != "open" {
        return Err("Cannot modify items on a paid order".to_string());
    }

    if quantity <= 1 {
        // Remove the item entirely
        conn.execute("DELETE FROM order_items WHERE id = ?1", [item_id])
            .map_err(|e| e.to_string())?;
    } else {
        // Decrease quantity by 1
        conn.execute(
            "UPDATE order_items SET quantity = quantity - 1 WHERE id = ?1",
            [item_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Restore 1 unit to inventory
    conn.execute(
        "UPDATE products SET quantity = quantity + 1 WHERE id = ?1",
        [product_id],
    )
    .map_err(|e| e.to_string())?;

    // Update order total (subtract price of 1 item)
    conn.execute(
        "UPDATE orders SET total = total - ?1 WHERE id = ?2",
        rusqlite::params![price_at_sale, order_id],
    )
    .map_err(|e| e.to_string())?;

    // Check if order has any items left
    let remaining_items: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM order_items WHERE order_id = ?1",
            [order_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // If no items left, delete the order
    if remaining_items == 0 {
        conn.execute("DELETE FROM orders WHERE id = ?1", [order_id])
            .map_err(|e| e.to_string())?;
        return Ok(None);
    }

    drop(conn);
    Ok(Some(get_order(app, order_id)?))
}

/// Increase item quantity by 1 (if stock is available)
#[tauri::command]
#[allow(non_snake_case)]
pub fn increase_item_quantity(app: AppHandle, orderItemId: i64) -> Result<OrderWithItems, String> {
    let item_id = orderItemId;
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get the item details
    let (order_id, product_id, price_at_sale): (i64, i64, f64) = conn
        .query_row(
            "SELECT order_id, product_id, price_at_sale FROM order_items WHERE id = ?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Order item not found".to_string())?;

    // Check if order is still open
    let status: String = conn
        .query_row(
            "SELECT status FROM orders WHERE id = ?1",
            [order_id],
            |row| row.get(0),
        )
        .map_err(|_| "Order not found".to_string())?;

    if status != "open" {
        return Err("Cannot modify items on a paid order".to_string());
    }

    // Check stock availability
    let stock: i32 = conn
        .query_row(
            "SELECT quantity FROM products WHERE id = ?1",
            [product_id],
            |row| row.get(0),
        )
        .map_err(|_| "Product not found".to_string())?;

    if stock < 1 {
        return Err("Insufficient stock".to_string());
    }

    // Increase quantity by 1
    conn.execute(
        "UPDATE order_items SET quantity = quantity + 1 WHERE id = ?1",
        [item_id],
    )
    .map_err(|e| e.to_string())?;

    // Deduct 1 from inventory
    conn.execute(
        "UPDATE products SET quantity = quantity - 1 WHERE id = ?1",
        [product_id],
    )
    .map_err(|e| e.to_string())?;

    // Update order total (add price of 1 item)
    conn.execute(
        "UPDATE orders SET total = total + ?1 WHERE id = ?2",
        rusqlite::params![price_at_sale, order_id],
    )
    .map_err(|e| e.to_string())?;

    drop(conn);
    get_order(app, order_id)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn update_order_notes(app: AppHandle, orderId: i64, customerName: Option<String>, notes: Option<String>) -> Result<OrderWithItems, String> {
    let order_id = orderId;
    let customer_name = customerName;
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE orders SET customer_name = ?1, notes = ?2 WHERE id = ?3",
        rusqlite::params![customer_name, notes, order_id],
    )
    .map_err(|e| e.to_string())?;

    drop(conn);
    get_order(app, order_id)
}

#[tauri::command]
pub fn get_today_orders(app: AppHandle) -> Result<Vec<OrderWithItems>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE date(o.created_at, 'localtime') = date('now', 'localtime')
             ORDER BY o.status DESC, o.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders: Vec<Order> = stmt
        .query_map([], |row| {
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

#[tauri::command]
pub fn get_open_orders(app: AppHandle) -> Result<Vec<OrderWithItems>, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE o.status = 'open'
             ORDER BY o.table_number ASC",
        )
        .map_err(|e| e.to_string())?;

    let orders: Vec<Order> = stmt
        .query_map([], |row| {
            Ok(Order {
                id: row.get(0)?,
                staff_id: row.get(1)?,
                staff_name: row.get(2)?,
                table_number: row.get::<_, Option<i32>>(3)?.unwrap_or(1),
                total: row.get(4)?,
                customer_name: row.get(5)?,
                notes: row.get(6)?,
                status: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "open".to_string()),
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

#[tauri::command]
pub fn get_order(app: AppHandle, id: i64) -> Result<OrderWithItems, String> {
    let db = app.db();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let order = conn
        .query_row(
            "SELECT o.id, o.staff_id, s.name, o.table_number, o.total, o.customer_name, o.notes, o.status, o.created_at
             FROM orders o
             LEFT JOIN staff s ON o.staff_id = s.id
             WHERE o.id = ?1",
            [id],
            |row| {
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
            },
        )
        .map_err(|e| e.to_string())?;

    let mut item_stmt = conn
        .prepare(
            "SELECT oi.id, oi.order_id, oi.product_id, p.name, oi.quantity, oi.price_at_sale
             FROM order_items oi
             LEFT JOIN products p ON oi.product_id = p.id
             WHERE oi.order_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<OrderItem> = item_stmt
        .query_map([id], |row| {
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

    Ok(OrderWithItems { order, items })
}
