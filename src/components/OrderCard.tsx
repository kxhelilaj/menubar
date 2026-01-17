import { OrderWithItems } from "../types";

interface OrderCardProps {
  order: OrderWithItems;
}

export function OrderCard({ order }: OrderCardProps) {
  const { order: o, items } = order;
  const time = new Date(o.created_at).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });

  return (
    <div className="order-card-new">
      <div className="order-card-top">
        <div className="order-badge">#{o.id}</div>
        <div className="order-time">{time}</div>
      </div>

      <div className="order-card-main">
        <div className="order-customer-name">
          {o.customer_name || "Walk-in Customer"}
        </div>

        <div className="order-items-list">
          {items.map((item) => (
            <div key={item.id} className="order-item-row">
              <span className="item-qty">{item.quantity}x</span>
              <span className="item-name">{item.product_name}</span>
              <span className="item-price">
                {(item.price_at_sale * item.quantity).toFixed(0)}
              </span>
            </div>
          ))}
        </div>

        {o.notes && (
          <div className="order-notes-box">
            <span className="notes-label">Note:</span> {o.notes}
          </div>
        )}
      </div>

      <div className="order-card-bottom">
        <div className="order-staff-info">
          <span className="staff-label">Staff:</span> {o.staff_name}
        </div>
        <div className="order-total-amount">{o.total.toFixed(0)} ALL</div>
      </div>
    </div>
  );
}
