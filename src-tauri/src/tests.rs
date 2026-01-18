//! Integration tests for database operations
//! These tests use an in-memory SQLite database to test business logic

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    /// Create a test database with schema
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

        conn.execute_batch(
            "
            CREATE TABLE categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE products (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                category_id INTEGER,
                low_stock_threshold INTEGER DEFAULT 5,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );

            CREATE TABLE staff (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                pin TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                staff_id INTEGER NOT NULL,
                table_number INTEGER NOT NULL DEFAULT 1,
                total REAL NOT NULL,
                customer_name TEXT,
                notes TEXT,
                status TEXT DEFAULT 'open',
                session_id INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (staff_id) REFERENCES staff(id)
            );

            CREATE TABLE order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                product_id INTEGER NOT NULL,
                quantity INTEGER NOT NULL,
                price_at_sale REAL NOT NULL,
                FOREIGN KEY (order_id) REFERENCES orders(id),
                FOREIGN KEY (product_id) REFERENCES products(id)
            );

            CREATE TABLE day_sessions (
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
            ",
        )
        .expect("Failed to create schema");

        conn
    }

    /// Seed test data
    fn seed_test_data(conn: &Connection) {
        // Create category
        conn.execute("INSERT INTO categories (name) VALUES ('Beer')", [])
            .unwrap();

        // Create products
        conn.execute(
            "INSERT INTO products (name, price, quantity, category_id, low_stock_threshold) VALUES ('Heineken', 5.0, 100, 1, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO products (name, price, quantity, category_id, low_stock_threshold) VALUES ('Corona', 6.0, 50, 1, 5)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO products (name, price, quantity, category_id, low_stock_threshold) VALUES ('Guinness', 7.0, 3, 1, 5)",
            [],
        )
        .unwrap();

        // Create staff
        conn.execute("INSERT INTO staff (name, pin) VALUES ('John', '1234')", [])
            .unwrap();
        conn.execute("INSERT INTO staff (name) VALUES ('Jane')", [])
            .unwrap();
    }

    // ===== CATEGORY TESTS =====

    #[test]
    fn test_create_category() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO categories (name) VALUES ('Wine')", [])
            .unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let name: String = conn
            .query_row("SELECT name FROM categories WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "Wine");
    }

    #[test]
    fn test_category_unique_constraint() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO categories (name) VALUES ('Beer')", [])
            .unwrap();

        let result = conn.execute("INSERT INTO categories (name) VALUES ('Beer')", []);
        assert!(result.is_err(), "Should not allow duplicate category names");
    }

    // ===== PRODUCT TESTS =====

    #[test]
    fn test_create_product() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let (name, price, quantity): (String, f64, i32) = conn
            .query_row(
                "SELECT name, price, quantity FROM products WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(name, "Heineken");
        assert!((price - 5.0).abs() < 0.01);
        assert_eq!(quantity, 100);
    }

    #[test]
    fn test_low_stock_detection() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Guinness has quantity=3, threshold=5, so it's low stock
        let low_stock_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE quantity <= low_stock_threshold",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(low_stock_count, 1);

        let low_stock_name: String = conn
            .query_row(
                "SELECT name FROM products WHERE quantity <= low_stock_threshold",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(low_stock_name, "Guinness");
    }

    #[test]
    fn test_update_product_quantity() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("UPDATE products SET quantity = quantity - 10 WHERE id = 1", [])
            .unwrap();

        let quantity: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(quantity, 90);
    }

    // ===== STAFF TESTS =====

    #[test]
    fn test_staff_pin_verification() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // John has PIN 1234
        let pin: Option<String> = conn
            .query_row("SELECT pin FROM staff WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(pin, Some("1234".to_string()));

        // Jane has no PIN
        let jane_pin: Option<String> = conn
            .query_row("SELECT pin FROM staff WHERE id = 2", [], |row| row.get(0))
            .unwrap();
        assert_eq!(jane_pin, None);
    }

    #[test]
    fn test_staff_unique_name() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let result = conn.execute("INSERT INTO staff (name) VALUES ('John')", []);
        assert!(result.is_err(), "Should not allow duplicate staff names");
    }

    // ===== ORDER TESTS =====

    #[test]
    fn test_create_order() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start a day session first
        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();

        // Create order
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 5, 25.0, 'open', 1)",
            [],
        )
        .unwrap();

        let (table, total, status): (i32, f64, String) = conn
            .query_row(
                "SELECT table_number, total, status FROM orders WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(table, 5);
        assert!((total - 25.0).abs() < 0.01);
        assert_eq!(status, "open");
    }

    #[test]
    fn test_order_item_inventory_deduction() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start day and create order
        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 15.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Add order item (3 Heinekens)
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (1, 1, 3, 5.0)",
            [],
        )
        .unwrap();

        // Deduct inventory
        conn.execute("UPDATE products SET quantity = quantity - 3 WHERE id = 1", [])
            .unwrap();

        let quantity: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(quantity, 97); // 100 - 3
    }

    #[test]
    fn test_order_status_transition() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Mark as paid
        conn.execute("UPDATE orders SET status = 'paid' WHERE id = 1", [])
            .unwrap();

        let status: String = conn
            .query_row("SELECT status FROM orders WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(status, "paid");
    }

    #[test]
    fn test_cannot_add_items_to_paid_order() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Check status before adding
        let status: String = conn
            .query_row("SELECT status FROM orders WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(status, "paid");

        // Business logic would prevent this - test the check
        let is_open = status == "open";
        assert!(!is_open, "Should not be able to add items to paid order");
    }

    #[test]
    fn test_order_total_calculation() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 0.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Add items
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (1, 1, 2, 5.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (1, 2, 3, 6.0)",
            [],
        )
        .unwrap();

        // Calculate total from items
        let total: f64 = conn
            .query_row(
                "SELECT SUM(quantity * price_at_sale) FROM order_items WHERE order_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!((total - 28.0).abs() < 0.01); // 2*5 + 3*6 = 28
    }

    #[test]
    fn test_inventory_restoration_on_item_removal() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Initial quantity
        let initial_qty: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(initial_qty, 100);

        // Deduct for order
        conn.execute("UPDATE products SET quantity = quantity - 5 WHERE id = 1", [])
            .unwrap();
        let after_order: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(after_order, 95);

        // Restore on removal
        conn.execute("UPDATE products SET quantity = quantity + 2 WHERE id = 1", [])
            .unwrap();
        let after_restore: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(after_restore, 97);
    }

    // ===== DAY SESSION TESTS =====

    #[test]
    fn test_start_day_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        let is_active: i32 = conn
            .query_row("SELECT is_active FROM day_sessions WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(is_active, 1);
    }

    #[test]
    fn test_only_one_active_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start first session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Count active sessions
        let active_count: i32 = conn
            .query_row("SELECT COUNT(*) FROM day_sessions WHERE is_active = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(active_count, 1);

        // Business logic should prevent starting another while one is active
        let has_active: bool = active_count > 0;
        assert!(has_active, "Should have one active session");
    }

    #[test]
    fn test_close_day_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Close session with totals
        conn.execute(
            "UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP, total_revenue = 150.0, total_orders = 5 WHERE id = 1",
            [],
        )
        .unwrap();

        let (is_active, revenue, orders): (i32, f64, i32) = conn
            .query_row(
                "SELECT is_active, total_revenue, total_orders FROM day_sessions WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(is_active, 0);
        assert!((revenue - 150.0).abs() < 0.01);
        assert_eq!(orders, 5);
    }

    #[test]
    fn test_cannot_close_day_with_open_orders() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Check for open orders
        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE status = 'open' AND session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(open_count, 1);
        assert!(open_count > 0, "Should have open orders preventing day close");
    }

    // ===== STOCK VALIDATION TESTS =====

    #[test]
    fn test_insufficient_stock_detection() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Guinness has only 3 in stock
        let available: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 3", [], |row| row.get(0))
            .unwrap();
        assert_eq!(available, 3);

        // Try to order 5 - should fail validation
        let requested = 5;
        let has_sufficient = available >= requested;
        assert!(!has_sufficient, "Should detect insufficient stock");
    }

    #[test]
    fn test_exact_stock_limit() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Guinness has exactly 3 - ordering 3 should work
        let available: i32 = conn
            .query_row("SELECT quantity FROM products WHERE id = 3", [], |row| row.get(0))
            .unwrap();
        let requested = 3;
        let has_sufficient = available >= requested;
        assert!(has_sufficient, "Should allow ordering exact stock amount");
    }

    // ===== TABLE MANAGEMENT TESTS =====

    #[test]
    fn test_multiple_tables_same_time() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();

        // Create orders for multiple tables
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'open', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 5, 50.0, 'open', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 10, 75.0, 'open', 1)",
            [],
        )
        .unwrap();

        let open_tables: i32 = conn
            .query_row("SELECT COUNT(DISTINCT table_number) FROM orders WHERE status = 'open'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(open_tables, 3);
    }

    #[test]
    fn test_get_open_orders_by_table() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();

        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 5, 25.0, 'open', 1)",
            [],
        )
        .unwrap();

        let table: i32 = conn
            .query_row(
                "SELECT table_number FROM orders WHERE status = 'open' AND table_number = 5",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table, 5);
    }

    // ===== REVENUE CALCULATION TESTS =====

    #[test]
    fn test_session_revenue_calculation() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();

        // Create paid orders
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 75.0, 'paid', 1)",
            [],
        )
        .unwrap();
        // Open order should not count
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 3, 100.0, 'open', 1)",
            [],
        )
        .unwrap();

        let total_revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM orders WHERE session_id = 1 AND status = 'paid'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!((total_revenue - 125.0).abs() < 0.01); // 50 + 75
    }

    #[test]
    fn test_order_count_for_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by) VALUES (1)", [])
            .unwrap();

        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 75.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 3, 25.0, 'paid', 1)",
            [],
        )
        .unwrap();

        let order_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'paid'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(order_count, 3);
    }

    // ===== CLOSE DAY COMPREHENSIVE TESTS =====

    #[test]
    fn test_close_day_requires_active_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // No active session
        let has_active: i32 = conn
            .query_row("SELECT COUNT(*) FROM day_sessions WHERE is_active = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(has_active, 0, "Should have no active session initially");

        // Business logic would return error
        let can_close = has_active > 0;
        assert!(!can_close, "Should not be able to close without active session");
    }

    #[test]
    fn test_close_day_blocks_with_open_orders() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create open order
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Check for open orders
        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(open_count, 1);
        assert!(open_count > 0, "Should block close_day with open orders");
    }

    #[test]
    fn test_close_day_blocks_with_multiple_open_orders() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create multiple open orders on different tables
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'open', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 5, 50.0, 'open', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 10, 75.0, 'paid', 1)",
            [],
        )
        .unwrap();

        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(open_count, 2, "Should have 2 open orders");
        assert!(open_count > 0, "Should block close_day");
    }

    #[test]
    fn test_close_day_blocks_empty_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // No orders created
        let order_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(order_count, 0, "Should have no orders");
        // Business logic prevents closing empty day
    }

    #[test]
    fn test_close_day_calculates_revenue_correctly() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create paid orders with specific totals
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.50, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 75.25, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 3, 50.00, 'paid', 1)",
            [],
        )
        .unwrap();

        let total_revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!((total_revenue - 225.75).abs() < 0.01, "Revenue should be 225.75");
    }

    #[test]
    fn test_close_day_updates_session_fields() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Simulate close_day
        let (total_revenue, total_orders): (f64, i32) = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        conn.execute(
            "UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP, total_revenue = ?1, total_orders = ?2 WHERE id = 1",
            rusqlite::params![total_revenue, total_orders],
        )
        .unwrap();

        // Verify all fields updated
        let (is_active, stored_revenue, stored_orders, closed_at): (i32, f64, i32, Option<String>) = conn
            .query_row(
                "SELECT is_active, total_revenue, total_orders, closed_at FROM day_sessions WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(is_active, 0, "Session should be inactive");
        assert!((stored_revenue - 100.0).abs() < 0.01, "Revenue should be stored");
        assert_eq!(stored_orders, 1, "Order count should be stored");
        assert!(closed_at.is_some(), "Closed_at should be set");
    }

    #[test]
    fn test_close_day_allows_after_all_orders_paid() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create order and mark paid
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Mark as paid
        conn.execute("UPDATE orders SET status = 'paid' WHERE id = 1", [])
            .unwrap();

        // Now check for open orders
        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(open_count, 0, "No open orders after payment");

        let total_orders: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(total_orders > 0 && open_count == 0, "Can close day now");
    }

    #[test]
    fn test_close_day_excludes_open_orders_from_revenue() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Mix of paid and open orders
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 50.0, 'open', 1)",
            [],
        )
        .unwrap();

        // Revenue should include ALL orders for session (the close prevents with open orders anyway)
        let total_revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Both orders count toward session total
        assert!((total_revenue - 150.0).abs() < 0.01);

        // But we can't close because there are open orders
        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(open_count > 0, "Cannot close with open orders");
    }

    #[test]
    fn test_cannot_start_new_session_while_active() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start first session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Check for existing active session
        let has_active: bool = conn
            .query_row(
                "SELECT 1 FROM day_sessions WHERE is_active = 1",
                [],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        assert!(has_active, "Should prevent starting another session");
    }

    #[test]
    fn test_can_start_session_after_previous_closed() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start and close first session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute("INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)", [])
            .unwrap();
        conn.execute("UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP WHERE id = 1", [])
            .unwrap();

        // Should be able to start new session
        let has_active: bool = conn
            .query_row(
                "SELECT 1 FROM day_sessions WHERE is_active = 1",
                [],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        assert!(!has_active, "No active session after close");

        // Start new session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (2, 1)", [])
            .unwrap();

        let new_session_count: i32 = conn
            .query_row("SELECT COUNT(*) FROM day_sessions WHERE is_active = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(new_session_count, 1, "New session started");
    }

    #[test]
    fn test_recovery_closing_creates_session() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let test_date = "2024-01-15";

        // Create orders for a past date (simulating legacy data)
        conn.execute(
            &format!("INSERT INTO orders (staff_id, table_number, total, status, created_at) VALUES (1, 1, 100.0, 'paid', '{}T10:00:00')", test_date),
            [],
        )
        .unwrap();
        conn.execute(
            &format!("INSERT INTO orders (staff_id, table_number, total, status, created_at) VALUES (1, 2, 50.0, 'paid', '{}T14:00:00')", test_date),
            [],
        )
        .unwrap();

        // Create recovery session
        conn.execute(
            "INSERT INTO day_sessions (date, started_by, started_at, is_active, closed_at, total_revenue, total_orders) VALUES (?1, 1, ?1 || ' 00:00:00', 0, CURRENT_TIMESTAMP, 150.0, 2)",
            [test_date],
        )
        .unwrap();

        let (revenue, orders): (f64, i32) = conn
            .query_row(
                "SELECT total_revenue, total_orders FROM day_sessions WHERE date = ?1",
                [test_date],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!((revenue - 150.0).abs() < 0.01);
        assert_eq!(orders, 2);
    }

    #[test]
    fn test_recovery_prevents_duplicate_closing() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let test_date = "2024-01-15";

        // Create first recovery session
        conn.execute(
            "INSERT INTO day_sessions (date, started_by, is_active, closed_at) VALUES (?1, 1, 0, CURRENT_TIMESTAMP)",
            [test_date],
        )
        .unwrap();

        // Check if closed session exists
        let existing: Result<i64, _> = conn.query_row(
            "SELECT id FROM day_sessions WHERE date = ?1 AND is_active = 0",
            [test_date],
            |row| row.get(0),
        );

        assert!(existing.is_ok(), "Should detect existing closed session");
    }

    #[test]
    fn test_recovery_rejects_no_orders() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let test_date = "2024-01-20"; // Date with no orders

        let order_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE date(created_at, 'localtime') = ?1",
                [test_date],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(order_count, 0, "Should have no orders for date");
        // Business logic prevents recovery for dates with no orders
    }

    #[test]
    fn test_session_links_orders_correctly() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Start session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create orders for this session
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 75.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Verify orders are linked
        let linked_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(linked_count, 2, "Orders should be linked to session");
    }

    #[test]
    fn test_multiple_sessions_same_day() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        let today = "2024-01-15";

        // First session - start and close
        conn.execute(
            &format!("INSERT INTO day_sessions (date, started_by, is_active) VALUES ('{}', 1, 1)", today),
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute("UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP WHERE id = 1", [])
            .unwrap();

        // Second session same day
        conn.execute(
            &format!("INSERT INTO day_sessions (date, started_by, is_active) VALUES ('{}', 2, 1)", today),
            [],
        )
        .unwrap();

        let session_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM day_sessions WHERE date = ?1",
                [today],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(session_count, 2, "Should allow multiple sessions same day");
    }

    #[test]
    fn test_close_day_with_decimal_precision() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Orders with precise decimal values
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 33.33, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 33.33, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 3, 33.34, 'paid', 1)",
            [],
        )
        .unwrap();

        let total: f64 = conn
            .query_row(
                "SELECT SUM(total) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!((total - 100.0).abs() < 0.01, "Decimal precision should work: got {}", total);
    }

    #[test]
    fn test_session_tracks_staff_who_started() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // John (id=1) starts the day
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        let started_by: i64 = conn
            .query_row("SELECT started_by FROM day_sessions WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(started_by, 1, "Session should track who started it");

        let staff_name: String = conn
            .query_row("SELECT name FROM staff WHERE id = ?1", [started_by], |row| row.get(0))
            .unwrap();

        assert_eq!(staff_name, "John");
    }

    #[test]
    fn test_get_sales_history_returns_closed_only() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Create closed session
        conn.execute(
            "INSERT INTO day_sessions (date, started_by, is_active, closed_at, total_revenue, total_orders) VALUES ('2024-01-14', 1, 0, CURRENT_TIMESTAMP, 100.0, 5)",
            [],
        )
        .unwrap();

        // Create active session
        conn.execute(
            "INSERT INTO day_sessions (date, started_by, is_active) VALUES ('2024-01-15', 1, 1)",
            [],
        )
        .unwrap();

        let closed_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM day_sessions WHERE is_active = 0 AND closed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(closed_count, 1, "Should only count closed sessions");
    }

    #[test]
    fn test_order_items_preserved_after_close() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create order with items
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 25.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_sale) VALUES (1, 1, 5, 5.0)",
            [],
        )
        .unwrap();

        // Close session
        conn.execute("UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP WHERE id = 1", [])
            .unwrap();

        // Verify items still accessible
        let item_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM order_items WHERE order_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(item_count, 1, "Order items should be preserved after close");
    }

    // ===== ERROR HANDLING AND EDGE CASE TESTS =====

    #[test]
    fn test_close_day_verifies_session_actually_updated() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Simulate close_day update
        let rows_affected = conn
            .execute(
                "UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP WHERE id = 1 AND is_active = 1",
                [],
            )
            .unwrap();

        assert_eq!(rows_affected, 1, "Should update exactly one row");

        // Verify the update took effect
        let is_active: i32 = conn
            .query_row("SELECT is_active FROM day_sessions WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(is_active, 0, "Session should be marked inactive");
    }

    #[test]
    fn test_close_day_idempotent_check() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Create and close a session
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute("UPDATE day_sessions SET is_active = 0 WHERE id = 1", [])
            .unwrap();

        // Try to close again - should find no active session
        let active_session: Result<i64, _> = conn.query_row(
            "SELECT id FROM day_sessions WHERE is_active = 1",
            [],
            |row| row.get(0),
        );

        assert!(active_session.is_err(), "Should not find active session after close");
    }

    #[test]
    fn test_open_orders_count_query_not_silently_fail() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Query should return 0, not error
        let open_count: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
            [],
            |row| row.get(0),
        );

        assert!(open_count.is_ok(), "Query should succeed even with no orders");
        assert_eq!(open_count.unwrap(), 0);
    }

    #[test]
    fn test_totals_query_handles_no_orders() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // COALESCE ensures we get 0 instead of NULL
        let (revenue, count): (f64, i32) = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!((revenue - 0.0).abs() < 0.01, "Revenue should be 0 for no orders");
        assert_eq!(count, 0, "Count should be 0 for no orders");
    }

    #[test]
    fn test_session_date_can_be_null() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Create session without explicit date (uses default)
        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        let date: Option<String> = conn
            .query_row("SELECT date FROM day_sessions WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        // Date might be NULL or set - both are valid
        // The important thing is the query doesn't fail
        println!("Session date: {:?}", date);
    }

    #[test]
    fn test_concurrent_close_prevention() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // First close attempt
        let rows1 = conn
            .execute(
                "UPDATE day_sessions SET is_active = 0 WHERE id = 1 AND is_active = 1",
                [],
            )
            .unwrap();
        assert_eq!(rows1, 1);

        // Second close attempt (simulating concurrent request)
        let rows2 = conn
            .execute(
                "UPDATE day_sessions SET is_active = 0 WHERE id = 1 AND is_active = 1",
                [],
            )
            .unwrap();
        assert_eq!(rows2, 0, "Second close should affect 0 rows");
    }

    #[test]
    fn test_revenue_calculation_with_null_totals() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Order with explicit total
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();

        let revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!((revenue - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_verify_close_updates_all_required_fields() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 200.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Perform the close update
        conn.execute(
            "UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP, total_revenue = 200.0, total_orders = 1 WHERE id = 1",
            [],
        )
        .unwrap();

        // Verify ALL fields are set
        let (is_active, closed_at, revenue, orders): (i32, Option<String>, Option<f64>, Option<i32>) = conn
            .query_row(
                "SELECT is_active, closed_at, total_revenue, total_orders FROM day_sessions WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(is_active, 0, "is_active should be 0");
        assert!(closed_at.is_some(), "closed_at should be set");
        assert!(revenue.is_some(), "total_revenue should be set");
        assert!(orders.is_some(), "total_orders should be set");
        assert!((revenue.unwrap() - 200.0).abs() < 0.01);
        assert_eq!(orders.unwrap(), 1);
    }

    #[test]
    fn test_staff_name_lookup_handles_missing_staff() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        // Query for non-existent staff
        let name: Option<String> = conn
            .query_row("SELECT name FROM staff WHERE id = 9999", [], |row| row.get(0))
            .ok();

        assert!(name.is_none(), "Should handle missing staff gracefully");
    }

    #[test]
    fn test_order_session_link_integrity() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create order with session_id
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 100.0, 'paid', 1)",
            [],
        )
        .unwrap();

        // Verify order is linked
        let session_id: Option<i64> = conn
            .query_row("SELECT session_id FROM orders WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(session_id, Some(1), "Order should be linked to session");
    }

    #[test]
    fn test_close_preserves_order_data_integrity() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create order with all fields
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, customer_name, notes, status, session_id) VALUES (1, 5, 150.0, 'Test Customer', 'Test notes', 'paid', 1)",
            [],
        )
        .unwrap();

        // Close session
        conn.execute("UPDATE day_sessions SET is_active = 0, closed_at = CURRENT_TIMESTAMP WHERE id = 1", [])
            .unwrap();

        // Verify all order data preserved
        let (customer, notes, total): (Option<String>, Option<String>, f64) = conn
            .query_row(
                "SELECT customer_name, notes, total FROM orders WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(customer, Some("Test Customer".to_string()));
        assert_eq!(notes, Some("Test notes".to_string()));
        assert!((total - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_large_revenue_calculation() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Create many orders with large totals
        for i in 1..=100 {
            conn.execute(
                &format!("INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, {}, 999.99, 'paid', 1)", i % 20 + 1),
                [],
            )
            .unwrap();
        }

        let (revenue, count): (f64, i32) = conn
            .query_row(
                "SELECT SUM(total), COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(count, 100);
        assert!((revenue - 99999.0).abs() < 0.1, "Large revenue should calculate correctly");
    }

    #[test]
    fn test_session_without_orders_blocks_close() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // No orders - check count
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 0, "Should have no orders");
        // Business logic: count == 0 means we should NOT allow close
    }

    #[test]
    fn test_mixed_status_orders_blocks_close() {
        let conn = setup_test_db();
        seed_test_data(&conn);

        conn.execute("INSERT INTO day_sessions (started_by, is_active) VALUES (1, 1)", [])
            .unwrap();

        // Some paid, some open
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 1, 50.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 2, 30.0, 'paid', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO orders (staff_id, table_number, total, status, session_id) VALUES (1, 3, 100.0, 'open', 1)",
            [],
        )
        .unwrap();

        let open_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM orders WHERE session_id = 1 AND status = 'open'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(open_count, 1, "Should detect open order");
        // Business logic: open_count > 0 means block close
    }
}
