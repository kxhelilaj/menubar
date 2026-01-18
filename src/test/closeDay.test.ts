import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import * as tauri from "../hooks/useTauri";
import type { OrderWithItems, DaySession } from "../types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

describe("Close Day Functionality", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Prerequisites for Closing Day", () => {
    it("requires an active session to close day", async () => {
      // No active session returns error
      mockInvoke.mockRejectedValueOnce(new Error("No active day session to close."));

      await expect(tauri.closeDay()).rejects.toThrow("No active day session to close.");
    });

    it("blocks close when there are open orders", async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error("Cannot close day: 2 tables are still open. Close all tables first.")
      );

      await expect(tauri.closeDay()).rejects.toThrow("tables are still open");
    });

    it("blocks close when session has no orders", async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error("No orders found for this session. Cannot close an empty day.")
      );

      await expect(tauri.closeDay()).rejects.toThrow("Cannot close an empty day");
    });

    it("allows close when all orders are paid", async () => {
      const closedSession: DaySession = {
        id: 1,
        date: "2024-01-15",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-15T09:00:00",
        closed_at: "2024-01-15T22:00:00",
        is_active: false,
        total_revenue: 500.0,
        total_orders: 20,
      };

      mockInvoke.mockResolvedValueOnce(closedSession);

      const result = await tauri.closeDay();
      expect(result.is_active).toBe(false);
      expect(result.closed_at).toBeDefined();
    });
  });

  describe("Close Day Returns Correct Data", () => {
    it("returns session with totals calculated", async () => {
      const closedSession: DaySession = {
        id: 1,
        date: "2024-01-15",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-15T09:00:00",
        closed_at: "2024-01-15T22:00:00",
        is_active: false,
        total_revenue: 1250.75,
        total_orders: 45,
      };

      mockInvoke.mockResolvedValueOnce(closedSession);

      const result = await tauri.closeDay();
      expect(result.total_revenue).toBe(1250.75);
      expect(result.total_orders).toBe(45);
    });

    it("returns closed_at timestamp", async () => {
      const closedSession: DaySession = {
        id: 1,
        date: "2024-01-15",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-15T09:00:00",
        closed_at: "2024-01-15T22:30:45",
        is_active: false,
        total_revenue: 100.0,
        total_orders: 5,
      };

      mockInvoke.mockResolvedValueOnce(closedSession);

      const result = await tauri.closeDay();
      expect(result.closed_at).toBe("2024-01-15T22:30:45");
    });

    it("session is marked as inactive", async () => {
      const closedSession: DaySession = {
        id: 1,
        date: "2024-01-15",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-15T09:00:00",
        closed_at: "2024-01-15T22:00:00",
        is_active: false,
        total_revenue: 100.0,
        total_orders: 5,
      };

      mockInvoke.mockResolvedValueOnce(closedSession);

      const result = await tauri.closeDay();
      expect(result.is_active).toBe(false);
    });
  });

  describe("Day Session Management", () => {
    it("prevents starting new session while one is active", async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error("A day session is already active. Close it first.")
      );

      await expect(tauri.startDay(1)).rejects.toThrow("already active");
    });

    it("allows starting new session after previous closed", async () => {
      const newSession: DaySession = {
        id: 2,
        date: "2024-01-16",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-16T09:00:00",
        closed_at: null,
        is_active: true,
        total_revenue: null,
        total_orders: null,
      };

      mockInvoke.mockResolvedValueOnce(newSession);

      const result = await tauri.startDay(1);
      expect(result.is_active).toBe(true);
      expect(result.id).toBe(2);
    });

    it("isDayActive returns false when no session", async () => {
      mockInvoke.mockResolvedValueOnce(false);

      const result = await tauri.isDayActive();
      expect(result).toBe(false);
    });

    it("isDayActive returns true when session active", async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const result = await tauri.isDayActive();
      expect(result).toBe(true);
    });

    it("getActiveSession returns null when no session", async () => {
      mockInvoke.mockResolvedValueOnce(null);

      const result = await tauri.getActiveSession();
      expect(result).toBeNull();
    });

    it("getActiveSession returns session data", async () => {
      const activeSession: DaySession = {
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

      mockInvoke.mockResolvedValueOnce(activeSession);

      const result = await tauri.getActiveSession();
      expect(result?.is_active).toBe(true);
      expect(result?.started_by_name).toBe("John");
    });
  });

  describe("Recovery Mode", () => {
    it("creates closing for missed date", async () => {
      const recoverySession: DaySession = {
        id: 10,
        date: "2024-01-10",
        started_by: 1,
        started_by_name: "John",
        started_at: "2024-01-10T00:00:00",
        closed_at: "2024-01-15T10:00:00",
        is_active: false,
        total_revenue: 350.0,
        total_orders: 15,
      };

      mockInvoke.mockResolvedValueOnce(recoverySession);

      const result = await tauri.createDayClosingForDate("2024-01-10");
      expect(result.date).toBe("2024-01-10");
      expect(result.is_active).toBe(false);
    });

    it("rejects duplicate recovery for same date", async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error("A closed session already exists for 2024-01-10")
      );

      await expect(tauri.createDayClosingForDate("2024-01-10")).rejects.toThrow(
        "already exists"
      );
    });

    it("rejects recovery for date with no orders", async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error("No orders found for 2024-01-20")
      );

      await expect(tauri.createDayClosingForDate("2024-01-20")).rejects.toThrow(
        "No orders found"
      );
    });
  });

  describe("Sales History", () => {
    it("returns only closed sessions", async () => {
      const history: DaySession[] = [
        {
          id: 1,
          date: "2024-01-14",
          started_by: 1,
          started_by_name: "John",
          started_at: "2024-01-14T09:00:00",
          closed_at: "2024-01-14T22:00:00",
          is_active: false,
          total_revenue: 500.0,
          total_orders: 20,
        },
        {
          id: 2,
          date: "2024-01-13",
          started_by: 2,
          started_by_name: "Jane",
          started_at: "2024-01-13T09:00:00",
          closed_at: "2024-01-13T21:00:00",
          is_active: false,
          total_revenue: 450.0,
          total_orders: 18,
        },
      ];

      mockInvoke.mockResolvedValueOnce(history);

      const result = await tauri.getSalesHistory(30);
      expect(result.every((s) => s.is_active === false)).toBe(true);
      expect(result.every((s) => s.closed_at !== null)).toBe(true);
    });

    it("respects limit parameter", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await tauri.getSalesHistory(10);
      expect(mockInvoke).toHaveBeenCalledWith("get_sales_history", { limit: 10 });
    });
  });

  describe("Day Summary", () => {
    it("returns summary for active session", async () => {
      const summary = {
        date: "2024-01-15",
        total_revenue: 250.0,
        total_orders: 10,
        orders: [],
      };

      mockInvoke.mockResolvedValueOnce(summary);

      const result = await tauri.getDaySummary();
      expect(result.total_revenue).toBe(250.0);
      expect(result.total_orders).toBe(10);
    });

    it("returns summary for specific session by ID", async () => {
      const summary = {
        date: "2024-01-10",
        total_revenue: 500.0,
        total_orders: 25,
        orders: [],
      };

      mockInvoke.mockResolvedValueOnce(summary);

      await tauri.getDaySummary(5);
      expect(mockInvoke).toHaveBeenCalledWith("get_day_summary", { sessionId: 5 });
    });

    it("returns empty summary when no active session", async () => {
      const summary = {
        date: "2024-01-15",
        total_revenue: 0.0,
        total_orders: 0,
        orders: [],
      };

      mockInvoke.mockResolvedValueOnce(summary);

      const result = await tauri.getDaySummary();
      expect(result.total_orders).toBe(0);
      expect(result.total_revenue).toBe(0);
    });
  });
});

describe("Close Day Business Logic Validation", () => {
  // Helper functions that mirror actual business logic

  function canCloseDay(openOrders: OrderWithItems[]): { canClose: boolean; reason?: string } {
    if (openOrders.length > 0) {
      return {
        canClose: false,
        reason: `Cannot close day: ${openOrders.length} tables are still open. Close all tables first.`,
      };
    }
    return { canClose: true };
  }

  function calculateSessionTotals(orders: OrderWithItems[]): { revenue: number; count: number } {
    const paidOrders = orders.filter((o) => o.order.status === "paid");
    return {
      revenue: paidOrders.reduce((sum, o) => sum + o.order.total, 0),
      count: paidOrders.length,
    };
  }

  describe("canCloseDay", () => {
    it("returns false with open orders", () => {
      const openOrders: OrderWithItems[] = [
        {
          order: {
            id: 1,
            staff_id: 1,
            table_number: 1,
            total: 50.0,
            status: "open",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
      ];

      const result = canCloseDay(openOrders);
      expect(result.canClose).toBe(false);
      expect(result.reason).toContain("1 tables are still open");
    });

    it("returns false with multiple open orders", () => {
      const openOrders: OrderWithItems[] = [
        {
          order: {
            id: 1,
            staff_id: 1,
            table_number: 1,
            total: 50.0,
            status: "open",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 2,
            staff_id: 1,
            table_number: 5,
            total: 75.0,
            status: "open",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 3,
            staff_id: 1,
            table_number: 10,
            total: 100.0,
            status: "open",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
      ];

      const result = canCloseDay(openOrders);
      expect(result.canClose).toBe(false);
      expect(result.reason).toContain("3 tables are still open");
    });

    it("returns true when no open orders", () => {
      const result = canCloseDay([]);
      expect(result.canClose).toBe(true);
      expect(result.reason).toBeUndefined();
    });
  });

  describe("calculateSessionTotals", () => {
    it("calculates revenue from paid orders only", () => {
      const orders: OrderWithItems[] = [
        {
          order: {
            id: 1,
            staff_id: 1,
            table_number: 1,
            total: 100.0,
            status: "paid",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 2,
            staff_id: 1,
            table_number: 2,
            total: 50.0,
            status: "open",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 3,
            staff_id: 1,
            table_number: 3,
            total: 75.0,
            status: "paid",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
      ];

      const result = calculateSessionTotals(orders);
      expect(result.revenue).toBe(175.0); // 100 + 75, excluding open order
      expect(result.count).toBe(2);
    });

    it("returns zero for no orders", () => {
      const result = calculateSessionTotals([]);
      expect(result.revenue).toBe(0);
      expect(result.count).toBe(0);
    });

    it("handles decimal precision", () => {
      const orders: OrderWithItems[] = [
        {
          order: {
            id: 1,
            staff_id: 1,
            table_number: 1,
            total: 33.33,
            status: "paid",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 2,
            staff_id: 1,
            table_number: 2,
            total: 33.33,
            status: "paid",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
        {
          order: {
            id: 3,
            staff_id: 1,
            table_number: 3,
            total: 33.34,
            status: "paid",
            created_at: "",
            staff_name: null,
            customer_name: null,
            notes: null,
          },
          items: [],
        },
      ];

      const result = calculateSessionTotals(orders);
      expect(result.revenue).toBeCloseTo(100.0, 2);
    });
  });
});
