import { useState, useEffect } from "react";
import { Product, Category, CreateProduct, UpdateProduct } from "../types";

interface ProductFormProps {
  product?: Product;
  categories: Category[];
  onSave: (product: CreateProduct | UpdateProduct) => void;
  onCancel: () => void;
}

export function ProductForm({
  product,
  categories,
  onSave,
  onCancel,
}: ProductFormProps) {
  const [name, setName] = useState(product?.name ?? "");
  const [price, setPrice] = useState(product?.price?.toString() ?? "");
  const [quantity, setQuantity] = useState(product?.quantity?.toString() ?? "0");
  const [categoryId, setCategoryId] = useState<number | null>(
    product?.category_id ?? null
  );
  const [threshold, setThreshold] = useState(
    product?.low_stock_threshold?.toString() ?? "5"
  );

  useEffect(() => {
    if (product) {
      setName(product.name);
      setPrice(product.price.toString());
      setQuantity(product.quantity.toString());
      setCategoryId(product.category_id);
      setThreshold(product.low_stock_threshold.toString());
    }
  }, [product]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const data = {
      name,
      price: parseFloat(price),
      quantity: parseInt(quantity),
      category_id: categoryId,
      low_stock_threshold: parseInt(threshold),
    };

    if (product) {
      onSave({ ...data, id: product.id } as UpdateProduct);
    } else {
      onSave(data as CreateProduct);
    }
  };

  return (
    <form className="product-form" onSubmit={handleSubmit}>
      <h3>{product ? "Edit Product" : "Add Product"}</h3>
      <div className="form-group">
        <label>Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
        />
      </div>
      <div className="form-group">
        <label>Price</label>
        <input
          type="number"
          step="0.01"
          min="0"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          required
        />
      </div>
      <div className="form-group">
        <label>Quantity</label>
        <input
          type="number"
          min="0"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
          required
        />
      </div>
      <div className="form-group">
        <label>Category</label>
        <select
          value={categoryId ?? ""}
          onChange={(e) =>
            setCategoryId(e.target.value ? Number(e.target.value) : null)
          }
        >
          <option value="">No category</option>
          {categories.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
      </div>
      <div className="form-group">
        <label>Low Stock Threshold</label>
        <input
          type="number"
          min="0"
          value={threshold}
          onChange={(e) => setThreshold(e.target.value)}
          required
        />
      </div>
      <div className="form-actions">
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="primary">
          Save
        </button>
      </div>
    </form>
  );
}
