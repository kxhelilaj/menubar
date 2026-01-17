import { CartItem } from "../types";

interface CartProps {
  items: CartItem[];
  onUpdateQuantity: (productId: number, quantity: number) => void;
  onRemove: (productId: number) => void;
  onCheckout: () => void;
  canCheckout: boolean;
  customerName: string;
  onCustomerNameChange: (name: string) => void;
  orderNotes: string;
  onOrderNotesChange: (notes: string) => void;
}

export function Cart({
  items,
  onUpdateQuantity,
  onRemove,
  onCheckout,
  canCheckout,
  customerName,
  onCustomerNameChange,
  orderNotes,
  onOrderNotesChange,
}: CartProps) {
  const total = items.reduce(
    (sum, item) => sum + item.product.price * item.quantity,
    0
  );

  return (
    <div className="cart">
      <h3>Cart</h3>
      {items.length === 0 ? (
        <p className="cart-empty">Cart is empty</p>
      ) : (
        <>
          <div className="cart-items">
            {items.map((item) => (
              <div key={item.product.id} className="cart-item">
                <div className="cart-item-info">
                  <span className="cart-item-name">{item.product.name}</span>
                  <span className="cart-item-price">
                    {(item.product.price * item.quantity).toFixed(0)} ALL
                  </span>
                </div>
                <div className="cart-item-controls">
                  <button
                    onClick={() =>
                      onUpdateQuantity(item.product.id, item.quantity - 1)
                    }
                  >
                    -
                  </button>
                  <span>{item.quantity}</span>
                  <button
                    onClick={() =>
                      onUpdateQuantity(item.product.id, item.quantity + 1)
                    }
                    disabled={item.quantity >= item.product.quantity}
                  >
                    +
                  </button>
                  <button
                    className="remove-btn"
                    onClick={() => onRemove(item.product.id)}
                  >
                    x
                  </button>
                </div>
              </div>
            ))}
          </div>
          <div className="cart-total">
            <span>Total:</span>
            <span>{total.toFixed(0)} ALL</span>
          </div>
          <div className="cart-order-info">
            <input
              type="text"
              placeholder="Customer name (optional)"
              value={customerName}
              onChange={(e) => onCustomerNameChange(e.target.value)}
              className="cart-input"
            />
            <input
              type="text"
              placeholder="Notes (optional)"
              value={orderNotes}
              onChange={(e) => onOrderNotesChange(e.target.value)}
              className="cart-input"
            />
          </div>
          <button
            className="checkout-btn"
            onClick={onCheckout}
            disabled={!canCheckout}
          >
            Checkout
          </button>
        </>
      )}
    </div>
  );
}
