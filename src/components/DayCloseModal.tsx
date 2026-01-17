import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { DaySummary } from "../types";

interface ProductSummary {
  name: string;
  quantity: number;
  revenue: number;
}

interface DayCloseModalProps {
  summary: DaySummary;
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

export function DayCloseModal({
  summary,
  onConfirm,
  onCancel,
  loading,
}: DayCloseModalProps) {
  // Aggregate products sold
  const productSummary: ProductSummary[] = [];
  const productMap = new Map<string, ProductSummary>();

  summary.orders.forEach((order) => {
    order.items.forEach((item) => {
      const name = item.product_name || "Unknown";
      const existing = productMap.get(name);
      if (existing) {
        existing.quantity += item.quantity;
        existing.revenue += item.price_at_sale * item.quantity;
      } else {
        productMap.set(name, {
          name,
          quantity: item.quantity,
          revenue: item.price_at_sale * item.quantity,
        });
      }
    });
  });

  productMap.forEach((value) => productSummary.push(value));
  productSummary.sort((a, b) => b.revenue - a.revenue);

  const exportToCSV = async () => {
    // Use semicolon separator for better Excel compatibility
    const lines = [
      `Date;${summary.date}`,
      `Total Orders;${summary.total_orders}`,
      `Total Revenue;${summary.total_revenue.toFixed(0)} ALL`,
      ``,
      `Product;Quantity;Revenue (ALL)`,
      ...productSummary.map((p) =>
        `${p.name};${p.quantity};${p.revenue.toFixed(0)}`
      ),
      ``,
      `TOTAL;${productSummary.reduce((sum, p) => sum + p.quantity, 0)};${summary.total_revenue.toFixed(0)}`,
    ];

    const csvContent = lines.join("\n");

    try {
      const filePath = await save({
        defaultPath: `day-report-${summary.date}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, csvContent);
        alert("CSV saved successfully!");
      }
    } catch (e) {
      alert("Failed to save CSV: " + e);
      console.error("Failed to save CSV:", e);
    }
  };

  const exportToText = async () => {
    const lines = [
      `═══════════════════════════════════════`,
      `       DAY REPORT - ${summary.date}`,
      `═══════════════════════════════════════`,
      ``,
      `Total Orders: ${summary.total_orders}`,
      `Total Revenue: ${summary.total_revenue.toFixed(0)} ALL`,
      ``,
      `───────────────────────────────────────`,
      `PRODUCTS SOLD`,
      `───────────────────────────────────────`,
      ``,
      ...productSummary.map(
        (p) =>
          `${p.name.padEnd(20)} x${p.quantity.toString().padStart(3)}  ${p.revenue.toFixed(0).padStart(8)} ALL`
      ),
      ``,
      `───────────────────────────────────────`,
      `TOTAL: ${summary.total_revenue.toFixed(0)} ALL`,
      `═══════════════════════════════════════`,
    ];

    const textContent = lines.join("\n");

    try {
      const filePath = await save({
        defaultPath: `day-report-${summary.date}.txt`,
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, textContent);
        alert("TXT saved successfully!");
      }
    } catch (e) {
      alert("Failed to save TXT: " + e);
      console.error("Failed to save TXT:", e);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h3>Close Day - {summary.date}</h3>
        <p>Review today's sales before closing the day.</p>

        <div className="summary-stats">
          <div className="stat">
            <span className="stat-label">Orders</span>
            <span className="stat-value">{summary.total_orders}</span>
          </div>
          <div className="stat">
            <span className="stat-label">Revenue</span>
            <span className="stat-value">
              {summary.total_revenue.toFixed(0)} ALL
            </span>
          </div>
        </div>

        {productSummary.length > 0 && (
          <div className="day-close-summary">
            <h4>Products Sold</h4>
            <table className="product-summary-table">
              <thead>
                <tr>
                  <th>Product</th>
                  <th>Qty</th>
                  <th>Revenue</th>
                </tr>
              </thead>
              <tbody>
                {productSummary.map((p) => (
                  <tr key={p.name}>
                    <td>{p.name}</td>
                    <td>{p.quantity}</td>
                    <td>{p.revenue.toFixed(0)} ALL</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <div className="export-buttons">
          <button className="export-btn" onClick={exportToCSV}>
            Export CSV
          </button>
          <button className="export-btn" onClick={exportToText}>
            Export TXT
          </button>
        </div>

        <div className="modal-actions">
          <button onClick={onCancel} disabled={loading}>
            Cancel
          </button>
          <button className="primary" onClick={onConfirm} disabled={loading}>
            {loading ? "Closing..." : "Close Day"}
          </button>
        </div>
      </div>
    </div>
  );
}
