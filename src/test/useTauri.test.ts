import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import * as tauri from "../hooks/useTauri";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

describe("useTauri hooks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Categories", () => {
    it("getCategories calls invoke with correct command", async () => {
      mockInvoke.mockResolvedValueOnce([{ id: 1, name: "Beer" }]);
      const result = await tauri.getCategories();
      expect(mockInvoke).toHaveBeenCalledWith("get_categories");
      expect(result).toEqual([{ id: 1, name: "Beer" }]);
    });

    it("createCategory calls invoke with name parameter", async () => {
      mockInvoke.mockResolvedValueOnce({ id: 1, name: "Wine" });
      await tauri.createCategory("Wine");
      expect(mockInvoke).toHaveBeenCalledWith("create_category", { name: "Wine" });
    });

    it("deleteCategory calls invoke with id parameter", async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await tauri.deleteCategory(1);
      expect(mockInvoke).toHaveBeenCalledWith("delete_category", { id: 1 });
    });
  });

  describe("Products", () => {
    it("getProducts returns product list", async () => {
      const mockProducts = [
        { id: 1, name: "Heineken", price: 5.0, quantity: 100, category_id: 1, low_stock_threshold: 10 },
      ];
      mockInvoke.mockResolvedValueOnce(mockProducts);
      const result = await tauri.getProducts();
      expect(result).toEqual(mockProducts);
    });

    it("createProduct sends correct product data", async () => {
      const newProduct = {
        name: "Corona",
        price: 6.0,
        quantity: 50,
        category_id: 1,
        low_stock_threshold: 5,
      };
      mockInvoke.mockResolvedValueOnce({ id: 2, ...newProduct });
      await tauri.createProduct(newProduct);
      expect(mockInvoke).toHaveBeenCalledWith("create_product", { product: newProduct });
    });

    it("getLowStock returns products below threshold", async () => {
      const lowStockProducts = [{ id: 1, name: "Guinness", quantity: 3, low_stock_threshold: 5 }];
      mockInvoke.mockResolvedValueOnce(lowStockProducts);
      const result = await tauri.getLowStock();
      expect(mockInvoke).toHaveBeenCalledWith("get_low_stock");
      expect(result).toEqual(lowStockProducts);
    });
  });

  describe("Staff", () => {
    it("verifyStaffPin calls with correct parameters", async () => {
      mockInvoke.mockResolvedValueOnce(true);
      const result = await tauri.verifyStaffPin(1, "1234");
      expect(mockInvoke).toHaveBeenCalledWith("verify_staff_pin", { id: 1, pin: "1234" });
      expect(result).toBe(true);
    });

    it("verifyStaffPin returns false for wrong PIN", async () => {
      mockInvoke.mockResolvedValueOnce(false);
      const result = await tauri.verifyStaffPin(1, "0000");
      expect(result).toBe(false);
    });
  });

  describe("Orders", () => {
    const mockOrder = {
      order: {
        id: 1,
        staff_id: 1,
        staff_name: "John",
        table_number: 5,
        total: 15.0,
        customer_name: null,
        notes: null,
        status: "open",
        created_at: "2024-01-15T10:00:00",
      },
      items: [
        { id: 1, order_id: 1, product_id: 1, product_name: "Beer", quantity: 3, price_at_sale: 5.0 },
      ],
    };

    it("createOrder sends correct order data", async () => {
      const newOrder = {
        staff_id: 1,
        table_number: 5,
        customer_name: null,
        notes: null,
        items: [{ product_id: 1, quantity: 3 }],
      };
      mockInvoke.mockResolvedValueOnce(mockOrder);
      await tauri.createOrder(newOrder);
      expect(mockInvoke).toHaveBeenCalledWith("create_order", { order: newOrder });
    });

    it("addItemsToOrder sends correct parameters", async () => {
      const items = [{ product_id: 2, quantity: 2 }];
      mockInvoke.mockResolvedValueOnce(mockOrder);
      await tauri.addItemsToOrder(1, items);
      expect(mockInvoke).toHaveBeenCalledWith("add_items_to_order", { orderId: 1, items });
    });

    it("markOrderPaid changes order status", async () => {
      const paidOrder = { ...mockOrder, order: { ...mockOrder.order, status: "paid" } };
      mockInvoke.mockResolvedValueOnce(paidOrder);
      const result = await tauri.markOrderPaid(1);
      expect(mockInvoke).toHaveBeenCalledWith("mark_order_paid", { orderId: 1 });
      expect(result.order.status).toBe("paid");
    });

    it("decreaseItemQuantity can return null when order is deleted", async () => {
      mockInvoke.mockResolvedValueOnce(null);
      const result = await tauri.decreaseItemQuantity(1);
      expect(result).toBeNull();
    });

    it("increaseItemQuantity updates item count", async () => {
      const updatedOrder = {
        ...mockOrder,
        items: [{ ...mockOrder.items[0], quantity: 4 }],
      };
      mockInvoke.mockResolvedValueOnce(updatedOrder);
      const result = await tauri.increaseItemQuantity(1);
      expect(result.items[0].quantity).toBe(4);
    });

    it("getOpenOrders returns only open orders", async () => {
      mockInvoke.mockResolvedValueOnce([mockOrder]);
      const result = await tauri.getOpenOrders();
      expect(mockInvoke).toHaveBeenCalledWith("get_open_orders");
      expect(result[0].order.status).toBe("open");
    });
  });

  describe("Day Sessions", () => {
    it("startDay creates new session", async () => {
      const mockSession = {
        id: 1,
        date: "2024-01-15",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-15T09:00:00",
        closed_at: null,
        is_active: true,
        total_revenue: null,
        total_orders: null,
      };
      mockInvoke.mockResolvedValueOnce(mockSession);
      const result = await tauri.startDay(1);
      expect(mockInvoke).toHaveBeenCalledWith("start_day", { staffId: 1 });
      expect(result.is_active).toBe(true);
    });

    it("closeDay returns session with totals", async () => {
      const closedSession = {
        id: 1,
        is_active: false,
        total_revenue: 500.0,
        total_orders: 25,
      };
      mockInvoke.mockResolvedValueOnce(closedSession);
      const result = await tauri.closeDay();
      expect(mockInvoke).toHaveBeenCalledWith("close_day");
      expect(result.is_active).toBe(false);
      expect(result.total_revenue).toBe(500.0);
    });

    it("isDayActive returns boolean", async () => {
      mockInvoke.mockResolvedValueOnce(true);
      const result = await tauri.isDayActive();
      expect(result).toBe(true);
    });

    it("getActiveSession returns null when no active session", async () => {
      mockInvoke.mockResolvedValueOnce(null);
      const result = await tauri.getActiveSession();
      expect(result).toBeNull();
    });
  });

  describe("Reports", () => {
    it("getSalesHistory accepts optional limit", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      await tauri.getSalesHistory(10);
      expect(mockInvoke).toHaveBeenCalledWith("get_sales_history", { limit: 10 });
    });

    it("getDaySummary can query by session ID", async () => {
      const mockSummary = {
        date: "2024-01-15",
        total_revenue: 300.0,
        total_orders: 15,
        orders: [],
      };
      mockInvoke.mockResolvedValueOnce(mockSummary);
      await tauri.getDaySummary(1);
      expect(mockInvoke).toHaveBeenCalledWith("get_day_summary", { sessionId: 1 });
    });

    it("getOrdersByDateRange sends date parameters", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      await tauri.getOrdersByDateRange("2024-01-01", "2024-01-31");
      expect(mockInvoke).toHaveBeenCalledWith("get_orders_by_date_range", {
        startDate: "2024-01-01",
        endDate: "2024-01-31",
      });
    });
  });
});
