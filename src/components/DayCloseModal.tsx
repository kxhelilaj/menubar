import { DaySummary } from "../types";
import { exportToCSV, exportToText, exportToExcel } from "../utils/exportReport";

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
  children?: React.ReactNode;
}

export function DayCloseModal({
  summary,
  onConfirm,
  onCancel,
  loading,
  children,
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
          <button className="export-btn primary" onClick={() => exportToExcel(summary)}>
            Export Excel
          </button>
          <button className="export-btn" onClick={() => exportToCSV(summary)}>
            Export CSV
          </button>
          <button className="export-btn" onClick={() => exportToText(summary)}>
            Export TXT
          </button>
        </div>

        {children}

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
