export interface Category {
  id: number;
  name: string;
}

export interface Product {
  id: number;
  name: string;
  price: number;
  quantity: number;
  category_id: number | null;
  category_name: string | null;
  low_stock_threshold: number;
  created_at: string;
}

export interface CreateProduct {
  name: string;
  price: number;
  quantity: number;
  category_id: number | null;
  low_stock_threshold?: number;
}

export interface UpdateProduct {
  id: number;
  name: string;
  price: number;
  quantity: number;
  category_id: number | null;
  low_stock_threshold: number;
}

export interface Staff {
  id: number;
  name: string;
  pin: string | null;
  created_at: string;
}

export interface CreateStaff {
  name: string;
  pin: string | null;
}

export interface Order {
  id: number;
  staff_id: number;
  staff_name: string | null;
  table_number: number;
  total: number;
  customer_name: string | null;
  notes: string | null;
  status: "open" | "paid";
  created_at: string;
}

export interface OrderItem {
  id: number;
  order_id: number;
  product_id: number;
  product_name: string | null;
  quantity: number;
  price_at_sale: number;
}

export interface CreateOrderItem {
  product_id: number;
  quantity: number;
}

export interface CreateOrder {
  staff_id: number;
  table_number: number;
  customer_name?: string | null;
  notes?: string | null;
  items: CreateOrderItem[];
}

export interface OrderWithItems {
  order: Order;
  items: OrderItem[];
}

export interface DaySummary {
  date: string;
  total_revenue: number;
  total_orders: number;
  orders: OrderWithItems[];
}

export interface CartItem {
  product: Product;
  quantity: number;
}

export interface DaySession {
  id: number;
  date: string | null;           // Now optional (sessions can span dates)
  started_by: number;
  started_by_name: string | null;
  started_at: string;
  closed_at: string | null;      // When session was closed
  is_active: boolean;
  total_revenue: number | null;  // Stored at close time
  total_orders: number | null;   // Stored at close time
}
