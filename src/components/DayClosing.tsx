import { DaySummary } from "../types";

interface DayClosingProps {
  summary: DaySummary | null;
  onClose: () => void;
  canClose: boolean;
}

export function DayClosingComponent({
  summary,
  onClose,
  canClose,
}: DayClosingProps) {
  if (!summary) return null;

  return (
    <div className="day-closing">
      <h3>Today's Summary</h3>
      <div className="summary-stats">
        <div className="stat">
          <span className="stat-label">Total Orders</span>
          <span className="stat-value">{summary.total_orders}</span>
        </div>
        <div className="stat">
          <span className="stat-label">Total Revenue</span>
          <span className="stat-value">{summary.total_revenue.toFixed(0)} ALL</span>
        </div>
      </div>
      {summary.orders.length > 0 && (
        <div className="today-orders">
          <h4>Orders</h4>
          {summary.orders.map((o) => (
            <div key={o.order.id} className="order-summary">
              <div className="order-header">
                <span>#{o.order.id}</span>
                <span>{o.order.staff_name}</span>
                <span>{o.order.total.toFixed(0)} ALL</span>
              </div>
              <div className="order-items">
                {o.items.map((item) => (
                  <span key={item.id}>
                    {item.quantity}x {item.product_name}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
      {canClose && (
        <button className="close-day-btn" onClick={onClose}>
          Close Day
        </button>
      )}
    </div>
  );
}
