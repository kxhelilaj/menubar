use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub quantity: i32,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub low_stock_threshold: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub price: f64,
    pub quantity: i32,
    pub category_id: Option<i64>,
    pub low_stock_threshold: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProduct {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub quantity: i32,
    pub category_id: Option<i64>,
    pub low_stock_threshold: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Staff {
    pub id: i64,
    pub name: String,
    pub pin: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStaff {
    pub name: String,
    pub pin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub staff_id: i64,
    pub staff_name: Option<String>,
    pub table_number: i32,
    pub total: f64,
    pub customer_name: Option<String>,
    pub notes: Option<String>,
    pub status: String, // "open" or "paid"
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub quantity: i32,
    pub price_at_sale: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: i64,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrder {
    pub staff_id: i64,
    pub table_number: i32,
    pub customer_name: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateOrderItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaySummary {
    pub date: String,
    pub total_revenue: f64,
    pub total_orders: i32,
    pub orders: Vec<OrderWithItems>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaySession {
    pub id: i64,
    pub date: Option<String>,           // Now optional (sessions can span dates)
    pub started_by: i64,
    pub started_by_name: Option<String>,
    pub started_at: String,
    pub closed_at: Option<String>,      // When session was closed
    pub is_active: bool,
    pub total_revenue: Option<f64>,     // Stored at close time
    pub total_orders: Option<i32>,      // Stored at close time
}
