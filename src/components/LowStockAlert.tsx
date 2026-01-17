import { Product } from "../types";

interface LowStockAlertProps {
  products: Product[];
  onDismiss: () => void;
}

export function LowStockAlert({ products, onDismiss }: LowStockAlertProps) {
  if (products.length === 0) return null;

  return (
    <div className="low-stock-alert">
      <div className="alert-header">
        <span>Low Stock Warning</span>
        <button onClick={onDismiss}>x</button>
      </div>
      <ul>
        {products.map((p) => (
          <li key={p.id}>
            {p.name}: {p.quantity} left
          </li>
        ))}
      </ul>
    </div>
  );
}
