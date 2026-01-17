import { useState, useEffect, useCallback } from "react";
import { Product, Category, Staff, CartItem, OrderWithItems } from "../types";
import {
  getProducts,
  getCategories,
  getStaff,
  createOrder,
  getLowStock,
  getOpenOrders,
  addItemsToOrder,
  markOrderPaid,
  verifyStaffPin,
} from "../hooks/useTauri";
import { StaffSelector } from "../components/StaffSelector";
import { ProductGrid } from "../components/ProductGrid";
import { LowStockAlert } from "../components/LowStockAlert";
import { PinModal } from "../components/PinModal";

const TOTAL_TABLES = 20;
const STAFF_STORAGE_KEY = "menubar_selected_staff_id";

export function Dashboard() {
  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [staff, setStaff] = useState<Staff[]>([]);
  const [selectedStaff, setSelectedStaff] = useState<Staff | null>(null);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [lowStock, setLowStock] = useState<Product[]>([]);
  const [showLowStock, setShowLowStock] = useState(true);
  const [openTables, setOpenTables] = useState<OrderWithItems[]>([]);
  const [selectedTableNumber, setSelectedTableNumber] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [showPinModal, setShowPinModal] = useState(false);

  // Get the order for a specific table number
  const getTableOrder = useCallback((tableNum: number): OrderWithItems | undefined => {
    return openTables.find((t) => t.order.table_number === tableNum);
  }, [openTables]);

  // Get the currently selected table's order
  const selectedTableOrder = selectedTableNumber ? getTableOrder(selectedTableNumber) : undefined;

  const loadData = useCallback(async () => {
    try {
      const [prods, cats, staffList, lowStockList, tables] = await Promise.all([
        getProducts(),
        getCategories(),
        getStaff(),
        getLowStock(),
        getOpenOrders(),
      ]);
      setProducts(prods);
      setCategories(cats);
      setStaff(staffList);
      setLowStock(lowStockList);
      setOpenTables(tables);

      // Restore selected staff from localStorage
      const savedStaffId = localStorage.getItem(STAFF_STORAGE_KEY);
      if (savedStaffId && !selectedStaff) {
        const savedStaff = staffList.find((s) => s.id === Number(savedStaffId));
        if (savedStaff) {
          setSelectedStaff(savedStaff);
        }
      }
    } catch (e) {
      console.error("loadData error:", e);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [selectedStaff]);

  useEffect(() => {
    loadData();
  }, []);

  // Persist staff selection
  const handleStaffSelect = (staff: Staff) => {
    setSelectedStaff(staff);
    localStorage.setItem(STAFF_STORAGE_KEY, String(staff.id));
  };

  const addToCart = (product: Product) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.product.id === product.id);
      if (existing) {
        if (existing.quantity >= product.quantity) return prev;
        return prev.map((item) =>
          item.product.id === product.id
            ? { ...item, quantity: item.quantity + 1 }
            : item
        );
      }
      return [...prev, { product, quantity: 1 }];
    });
  };

  const updateQuantity = (productId: number, quantity: number) => {
    if (quantity <= 0) {
      setCart((prev) => prev.filter((item) => item.product.id !== productId));
    } else {
      setCart((prev) =>
        prev.map((item) =>
          item.product.id === productId ? { ...item, quantity } : item
        )
      );
    }
  };

  const removeFromCart = (productId: number) => {
    setCart((prev) => prev.filter((item) => item.product.id !== productId));
  };

  // Handle clicking on a table
  const handleTableClick = (tableNum: number) => {
    // If clicking the already selected table, deselect it
    if (selectedTableNumber === tableNum) {
      setSelectedTableNumber(null);
      return;
    }

    // Select the table (whether empty or occupied)
    setSelectedTableNumber(tableNum);
  };

  // Add cart items to selected table
  const handleAddToTable = async () => {
    if (!selectedTableNumber || cart.length === 0 || !selectedStaff) return;

    try {
      if (selectedTableOrder) {
        // Table already has an order - add items to it
        await addItemsToOrder(
          selectedTableOrder.order.id,
          cart.map((item) => ({
            product_id: item.product.id,
            quantity: item.quantity,
          }))
        );
      } else {
        // Table is empty - create new order with items
        await createOrder({
          staff_id: selectedStaff.id,
          table_number: selectedTableNumber,
          customer_name: null,
          notes: null,
          items: cart.map((item) => ({
            product_id: item.product.id,
            quantity: item.quantity,
          })),
        });
      }
      setCart([]);
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  // Request to close table - show PIN modal
  const handleCloseTable = () => {
    if (!selectedTableOrder) return;
    setShowPinModal(true);
  };

  // Verify PIN and close table
  const handleVerifyPin = async (pin: string): Promise<boolean> => {
    if (!selectedTableOrder) return false;

    try {
      const valid = await verifyStaffPin(selectedTableOrder.order.staff_id, pin);
      if (valid) {
        await markOrderPaid(selectedTableOrder.order.id);
        setSelectedTableNumber(null);
        setShowPinModal(false);
        await loadData();
        return true;
      }
      return false;
    } catch (e) {
      setError(String(e));
      return false;
    }
  };

  const cartTotal = cart.reduce(
    (sum, item) => sum + item.product.price * item.quantity,
    0
  );

  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  return (
    <div className="dashboard-tables">
      {showPinModal && selectedTableOrder && (
        <PinModal
          staffName={selectedTableOrder.order.staff_name || "Staff"}
          onVerify={handleVerifyPin}
          onCancel={() => setShowPinModal(false)}
        />
      )}

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {showLowStock && lowStock.length > 0 && (
        <LowStockAlert products={lowStock} onDismiss={() => setShowLowStock(false)} />
      )}

      {/* Header with staff selector */}
      <div className="dash-header">
        <StaffSelector
          staff={staff}
          selectedStaff={selectedStaff}
          onSelect={handleStaffSelect}
        />
      </div>

      <div className="dash-body">
        {/* LEFT: Tables Grid */}
        <div className="tables-section tables-section-wide">
          <div className="section-title">
            <h3>Tables</h3>
            <span className="badge">{openTables.filter(t => t.items.length > 0).length} occupied</span>
          </div>

          <div className="tables-grid-fixed">
            {Array.from({ length: TOTAL_TABLES }, (_, i) => i + 1).map((tableNum) => {
              const order = getTableOrder(tableNum);
              const isSelected = selectedTableNumber === tableNum;
              // Only show as occupied if it has items (not just an empty order)
              const hasItems = order && order.items.length > 0;

              return (
                <div
                  key={tableNum}
                  className={`table-card-fixed ${hasItems ? "occupied" : "empty"} ${isSelected ? "active" : ""}`}
                  onClick={() => handleTableClick(tableNum)}
                >
                  <div className="table-card-number">Table {tableNum}</div>
                  {hasItems ? (
                    <>
                      <div className="table-card-items">{order.items.length} items</div>
                      <div className="table-card-total">{order.order.total.toFixed(0)} ALL</div>
                    </>
                  ) : (
                    <div className="table-card-status">{isSelected ? "Selected" : "Available"}</div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* MIDDLE: Products */}
        <div className="products-section">
          <ProductGrid
            products={products}
            categories={categories}
            onAddToCart={addToCart}
          />
        </div>

        {/* RIGHT: Current Order */}
        <div className="order-section">
          <div className="section-title">
            <h3>{selectedTableNumber ? `Table ${selectedTableNumber}` : "Select a Table"}</h3>
          </div>

          {/* Show selected table's existing items */}
          {selectedTableOrder && selectedTableOrder.items.length > 0 && (
            <div className="existing-items">
              <div className="existing-items-title">Current Items</div>
              {selectedTableOrder.items.map((item) => (
                <div key={item.id} className="existing-item">
                  <span>{item.quantity}× {item.product_name}</span>
                  <span>{(item.price_at_sale * item.quantity).toFixed(0)}</span>
                </div>
              ))}
              <div className="existing-items-total">
                <span>Subtotal</span>
                <span>{selectedTableOrder.order.total.toFixed(0)} ALL</span>
              </div>
            </div>
          )}

          {/* Cart for new items */}
          <div className="cart-area">
            {cart.length > 0 && selectedTableOrder && (
              <div className="new-items-title">Add to order:</div>
            )}

            {cart.map((item) => (
              <div key={item.product.id} className="cart-row">
                <div className="cart-row-info">
                  <span className="cart-row-name">{item.product.name}</span>
                  <span className="cart-row-price">
                    {(item.product.price * item.quantity).toFixed(0)}
                  </span>
                </div>
                <div className="cart-row-controls">
                  <button onClick={() => updateQuantity(item.product.id, item.quantity - 1)}>−</button>
                  <span>{item.quantity}</span>
                  <button
                    onClick={() => updateQuantity(item.product.id, item.quantity + 1)}
                    disabled={item.quantity >= item.product.quantity}
                  >+</button>
                  <button className="remove" onClick={() => removeFromCart(item.product.id)}>×</button>
                </div>
              </div>
            ))}

            {cart.length > 0 && (
              <div className="cart-subtotal">
                <span>New items:</span>
                <span>{cartTotal.toFixed(0)} ALL</span>
              </div>
            )}

            {!selectedTableNumber && (
              <div className="empty-cart">Select a table to start</div>
            )}

            {selectedTableNumber && !selectedTableOrder && cart.length === 0 && (
              <div className="empty-cart">Add products to this table</div>
            )}

            {selectedTableOrder && cart.length === 0 && selectedTableOrder.items.length === 0 && (
              <div className="empty-cart">Add products to this table</div>
            )}
          </div>

          {/* Action Buttons */}
          <div className="order-actions">
            {selectedTableNumber ? (
              <>
                {cart.length > 0 && selectedStaff && (
                  <button className="btn-add" onClick={handleAddToTable}>
                    Add to Table
                  </button>
                )}
                {selectedTableOrder && selectedTableOrder.items.length > 0 && (
                  <button className="btn-close" onClick={handleCloseTable}>
                    Close Table ({selectedTableOrder.order.total.toFixed(0)} ALL)
                  </button>
                )}
                <button className="btn-deselect" onClick={() => setSelectedTableNumber(null)}>
                  Deselect
                </button>
              </>
            ) : (
              <div className="no-table-hint">
                {!selectedStaff ? "Select staff first, then click a table" : "Click a table to select it"}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
