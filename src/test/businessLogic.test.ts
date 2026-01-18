import { describe, it, expect } from "vitest";
import type { Product, OrderWithItems } from "../types";

// Business logic helper functions that mirror dashboard behavior
// These test the core business rules without Tauri dependencies

interface TestCartItem {
  product: Product;
  quantity: number;
}

// Calculate cart total
function calculateCartTotal(cart: TestCartItem[]): number {
  return cart.reduce((sum, item) => sum + item.product.price * item.quantity, 0);
}

// Check if product can be added to cart (stock validation)
function canAddToCart(product: Product, currentCartQty: number, requestedQty: number): boolean {
  return product.quantity >= currentCartQty + requestedQty;
}

// Check if order can be modified (status validation)
function canModifyOrder(order: OrderWithItems): boolean {
  return order.order.status === "open";
}

// Check if day can be closed (no open orders)
function canCloseDay(openOrders: OrderWithItems[]): boolean {
  return openOrders.length === 0;
}

// Calculate order items total from order
function calculateOrderTotal(order: OrderWithItems): number {
  return order.items.reduce((sum, item) => sum + item.price_at_sale * item.quantity, 0);
}

// Check if product is low stock
function isLowStock(product: Product): boolean {
  return product.quantity <= product.low_stock_threshold;
}

// Validate table number (1-20)
function isValidTableNumber(tableNumber: number): boolean {
  return tableNumber >= 1 && tableNumber <= 20 && Number.isInteger(tableNumber);
}

describe("Cart Operations", () => {
  const mockProduct: Product = {
    id: 1,
    name: "Heineken",
    price: 5.0,
    quantity: 10,
    category_id: 1,
    category_name: "Beer",
    low_stock_threshold: 5,
    created_at: "2024-01-15T10:00:00",
  };

  describe("calculateCartTotal", () => {
    it("calculates total for single item", () => {
      const cart: TestCartItem[] = [{ product: mockProduct, quantity: 2 }];
      expect(calculateCartTotal(cart)).toBe(10.0);
    });

    it("calculates total for multiple items", () => {
      const product2 = { ...mockProduct, id: 2, name: "Corona", price: 6.0 };
      const cart: TestCartItem[] = [
        { product: mockProduct, quantity: 2 },
        { product: product2, quantity: 3 },
      ];
      expect(calculateCartTotal(cart)).toBe(28.0); // 10 + 18
    });

    it("returns 0 for empty cart", () => {
      expect(calculateCartTotal([])).toBe(0);
    });

    it("handles decimal prices correctly", () => {
      const product = { ...mockProduct, price: 5.99 };
      const cart: TestCartItem[] = [{ product, quantity: 3 }];
      expect(calculateCartTotal(cart)).toBeCloseTo(17.97, 2);
    });
  });

  describe("canAddToCart (Stock Validation)", () => {
    it("allows adding when stock is sufficient", () => {
      expect(canAddToCart(mockProduct, 0, 5)).toBe(true);
    });

    it("allows adding up to exact stock limit", () => {
      expect(canAddToCart(mockProduct, 0, 10)).toBe(true);
    });

    it("prevents adding more than available stock", () => {
      expect(canAddToCart(mockProduct, 0, 11)).toBe(false);
    });

    it("considers items already in cart", () => {
      expect(canAddToCart(mockProduct, 8, 3)).toBe(false); // 8 + 3 = 11 > 10
      expect(canAddToCart(mockProduct, 8, 2)).toBe(true); // 8 + 2 = 10 <= 10
    });

    it("prevents adding to out-of-stock product", () => {
      const outOfStock = { ...mockProduct, quantity: 0 };
      expect(canAddToCart(outOfStock, 0, 1)).toBe(false);
    });
  });
});

describe("Order Operations", () => {
  const mockOpenOrder: OrderWithItems = {
    order: {
      id: 1,
      staff_id: 1,
      staff_name: "John",
      table_number: 5,
      total: 25.0,
      customer_name: null,
      notes: null,
      status: "open",
      created_at: "2024-01-15T10:00:00",
    },
    items: [
      { id: 1, order_id: 1, product_id: 1, product_name: "Heineken", quantity: 5, price_at_sale: 5.0 },
    ],
  };

  const mockPaidOrder: OrderWithItems = {
    ...mockOpenOrder,
    order: { ...mockOpenOrder.order, status: "paid" },
  };

  describe("canModifyOrder", () => {
    it("allows modifying open orders", () => {
      expect(canModifyOrder(mockOpenOrder)).toBe(true);
    });

    it("prevents modifying paid orders", () => {
      expect(canModifyOrder(mockPaidOrder)).toBe(false);
    });
  });

  describe("calculateOrderTotal", () => {
    it("calculates correct total from items", () => {
      expect(calculateOrderTotal(mockOpenOrder)).toBe(25.0);
    });

    it("handles multiple items", () => {
      const multiItemOrder: OrderWithItems = {
        ...mockOpenOrder,
        items: [
          { id: 1, order_id: 1, product_id: 1, product_name: "Heineken", quantity: 2, price_at_sale: 5.0 },
          { id: 2, order_id: 1, product_id: 2, product_name: "Corona", quantity: 3, price_at_sale: 6.0 },
        ],
      };
      expect(calculateOrderTotal(multiItemOrder)).toBe(28.0);
    });

    it("returns 0 for order with no items", () => {
      const emptyOrder = { ...mockOpenOrder, items: [] };
      expect(calculateOrderTotal(emptyOrder)).toBe(0);
    });
  });
});

describe("Day Session Operations", () => {
  describe("canCloseDay", () => {
    const mockOpenOrder: OrderWithItems = {
      order: {
        id: 1,
        staff_id: 1,
        staff_name: "John",
        table_number: 5,
        total: 25.0,
        customer_name: null,
        notes: null,
        status: "open",
        created_at: "2024-01-15T10:00:00",
      },
      items: [],
    };

    it("allows closing day with no open orders", () => {
      expect(canCloseDay([])).toBe(true);
    });

    it("prevents closing day with open orders", () => {
      expect(canCloseDay([mockOpenOrder])).toBe(false);
    });

    it("prevents closing day with multiple open orders", () => {
      const order2 = { ...mockOpenOrder, order: { ...mockOpenOrder.order, id: 2, table_number: 6 } };
      expect(canCloseDay([mockOpenOrder, order2])).toBe(false);
    });
  });
});

describe("Stock Management", () => {
  describe("isLowStock", () => {
    it("returns true when quantity equals threshold", () => {
      const product: Product = {
        id: 1,
        name: "Test",
        price: 5.0,
        quantity: 5,
        category_id: 1,
        category_name: "Test",
        low_stock_threshold: 5,
        created_at: "",
      };
      expect(isLowStock(product)).toBe(true);
    });

    it("returns true when quantity is below threshold", () => {
      const product: Product = {
        id: 1,
        name: "Test",
        price: 5.0,
        quantity: 3,
        category_id: 1,
        category_name: "Test",
        low_stock_threshold: 5,
        created_at: "",
      };
      expect(isLowStock(product)).toBe(true);
    });

    it("returns false when quantity is above threshold", () => {
      const product: Product = {
        id: 1,
        name: "Test",
        price: 5.0,
        quantity: 10,
        category_id: 1,
        category_name: "Test",
        low_stock_threshold: 5,
        created_at: "",
      };
      expect(isLowStock(product)).toBe(false);
    });

    it("handles zero threshold", () => {
      const product: Product = {
        id: 1,
        name: "Test",
        price: 5.0,
        quantity: 0,
        category_id: 1,
        category_name: "Test",
        low_stock_threshold: 0,
        created_at: "",
      };
      expect(isLowStock(product)).toBe(true);
    });
  });
});

describe("Table Validation", () => {
  describe("isValidTableNumber", () => {
    it("accepts valid table numbers 1-20", () => {
      expect(isValidTableNumber(1)).toBe(true);
      expect(isValidTableNumber(10)).toBe(true);
      expect(isValidTableNumber(20)).toBe(true);
    });

    it("rejects table number 0", () => {
      expect(isValidTableNumber(0)).toBe(false);
    });

    it("rejects negative table numbers", () => {
      expect(isValidTableNumber(-1)).toBe(false);
    });

    it("rejects table numbers above 20", () => {
      expect(isValidTableNumber(21)).toBe(false);
    });

    it("rejects non-integer table numbers", () => {
      expect(isValidTableNumber(5.5)).toBe(false);
    });
  });
});

describe("Revenue Calculations", () => {
  it("calculates daily revenue correctly", () => {
    const orders: OrderWithItems[] = [
      {
        order: { id: 1, staff_id: 1, table_number: 1, total: 50.0, status: "paid", created_at: "", staff_name: null, customer_name: null, notes: null },
        items: [],
      },
      {
        order: { id: 2, staff_id: 1, table_number: 2, total: 75.0, status: "paid", created_at: "", staff_name: null, customer_name: null, notes: null },
        items: [],
      },
      {
        order: { id: 3, staff_id: 1, table_number: 3, total: 25.0, status: "paid", created_at: "", staff_name: null, customer_name: null, notes: null },
        items: [],
      },
    ];

    const totalRevenue = orders.reduce((sum, o) => sum + o.order.total, 0);
    expect(totalRevenue).toBe(150.0);
  });

  it("only counts paid orders for revenue", () => {
    const orders: OrderWithItems[] = [
      {
        order: { id: 1, staff_id: 1, table_number: 1, total: 50.0, status: "paid", created_at: "", staff_name: null, customer_name: null, notes: null },
        items: [],
      },
      {
        order: { id: 2, staff_id: 1, table_number: 2, total: 75.0, status: "open", created_at: "", staff_name: null, customer_name: null, notes: null },
        items: [],
      },
    ];

    const paidRevenue = orders
      .filter(o => o.order.status === "paid")
      .reduce((sum, o) => sum + o.order.total, 0);
    expect(paidRevenue).toBe(50.0);
  });
});
