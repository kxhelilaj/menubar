import { Product, Category } from "../types";

interface ProductGridProps {
  products: Product[];
  categories: Category[];
  onAddToCart: (product: Product) => void;
}

export function ProductGrid({
  products,
  categories,
  onAddToCart,
}: ProductGridProps) {
  const groupedProducts = categories.reduce(
    (acc, cat) => {
      acc[cat.id] = products.filter((p) => p.category_id === cat.id);
      return acc;
    },
    {} as Record<number, Product[]>
  );

  const uncategorized = products.filter((p) => p.category_id === null);

  return (
    <div className="product-grid">
      {categories.map((category) => (
        <div key={category.id} className="category-section">
          <h3>{category.name}</h3>
          <div className="products">
            {groupedProducts[category.id]?.map((product) => (
              <button
                key={product.id}
                className={`product-btn ${product.quantity === 0 ? "out-of-stock" : ""}`}
                onClick={() => onAddToCart(product)}
                disabled={product.quantity === 0}
              >
                <span className="product-name">{product.name}</span>
                <span className="product-price">
                  {product.price.toFixed(0)} ALL
                </span>
                <span className="product-qty">({product.quantity})</span>
              </button>
            ))}
          </div>
        </div>
      ))}
      {uncategorized.length > 0 && (
        <div className="category-section">
          <h3>Other</h3>
          <div className="products">
            {uncategorized.map((product) => (
              <button
                key={product.id}
                className={`product-btn ${product.quantity === 0 ? "out-of-stock" : ""}`}
                onClick={() => onAddToCart(product)}
                disabled={product.quantity === 0}
              >
                <span className="product-name">{product.name}</span>
                <span className="product-price">
                  {product.price.toFixed(0)} ALL
                </span>
                <span className="product-qty">({product.quantity})</span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
