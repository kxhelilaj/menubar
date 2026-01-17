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
  DayClosing,
  DaySummary,
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
export const updateOrderNotes = (orderId: number, customerName: string | null, notes: string | null) =>
  invoke<OrderWithItems>("update_order_notes", { orderId, customerName, notes });

// Reports
export const closeDay = () => invoke<DayClosing>("close_day");
export const getSalesHistory = (limit?: number) =>
  invoke<DayClosing[]>("get_sales_history", { limit });
export const getDaySummary = (date?: string) =>
  invoke<DaySummary>("get_day_summary", { date });
