import { invoke } from "@tauri-apps/api/core";
import {
  Category,
  Product,
  CreateProduct,
  UpdateProduct,
  Staff,
  CreateStaff,
  CreateOrder,
  OrderWithItems,
  DaySummary,
  DaySession,
} from "../types";

// Categories
export const getCategories = () => invoke<Category[]>("get_categories");
export const createCategory = (name: string) =>
  invoke<Category>("create_category", { name });
export const deleteCategory = (id: number) =>
  invoke<void>("delete_category", { id });

// Products
export const getProducts = () => invoke<Product[]>("get_products");
export const createProduct = (product: CreateProduct) =>
  invoke<Product>("create_product", { product });
export const updateProduct = (product: UpdateProduct) =>
  invoke<Product>("update_product", { product });
export const deleteProduct = (id: number) =>
  invoke<void>("delete_product", { id });
export const getLowStock = () => invoke<Product[]>("get_low_stock");

// Staff
export const getStaff = () => invoke<Staff[]>("get_staff");
export const createStaff = (staff: CreateStaff) =>
  invoke<Staff>("create_staff", { staff });
export const deleteStaff = (id: number) => invoke<void>("delete_staff", { id });
export const verifyStaffPin = (id: number, pin: string) =>
  invoke<boolean>("verify_staff_pin", { id, pin });

// Orders
export const createOrder = (order: CreateOrder) =>
  invoke<OrderWithItems>("create_order", { order });
export const getTodayOrders = () => invoke<OrderWithItems[]>("get_today_orders");
export const getOpenOrders = () => invoke<OrderWithItems[]>("get_open_orders");
export const getOrder = (id: number) =>
  invoke<OrderWithItems>("get_order", { id });
export const addItemsToOrder = (orderId: number, items: { product_id: number; quantity: number }[]) =>
  invoke<OrderWithItems>("add_items_to_order", { orderId, items });
export const markOrderPaid = (orderId: number) =>
  invoke<OrderWithItems>("mark_order_paid", { orderId });
export const decreaseItemQuantity = (orderItemId: number) =>
  invoke<OrderWithItems | null>("decrease_item_quantity", { orderItemId });
export const increaseItemQuantity = (orderItemId: number) =>
  invoke<OrderWithItems>("increase_item_quantity", { orderItemId });
export const updateOrderNotes = (orderId: number, customerName: string | null, notes: string | null) =>
  invoke<OrderWithItems>("update_order_notes", { orderId, customerName, notes });

// Reports
export const closeDay = () => invoke<DaySession>("close_day");
export const getSalesHistory = (limit?: number) =>
  invoke<DaySession[]>("get_sales_history", { limit });
export const getDaySummary = (sessionId?: number) =>
  invoke<DaySummary>("get_day_summary", { sessionId });
export const getOrdersByDateRange = (startDate: string, endDate: string) =>
  invoke<OrderWithItems[]>("get_orders_by_date_range", { startDate, endDate });
export const createDayClosingForDate = (date: string) =>
  invoke<DaySession>("create_day_closing_for_date", { date });

// Day Sessions
export const getActiveSession = () =>
  invoke<DaySession | null>("get_active_session");
export const startDay = (staffId: number) =>
  invoke<DaySession>("start_day", { staffId });
export const isDayActive = () =>
  invoke<boolean>("is_day_active");
