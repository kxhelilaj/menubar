import { useState, useEffect, useCallback } from "react";
import { Product, Category, CreateProduct, UpdateProduct } from "../types";
import {
  getProducts,
  getCategories,
  createProduct,
  updateProduct,
  deleteProduct,
  createCategory,
  deleteCategory,
} from "../hooks/useTauri";
import { ProductForm } from "../components/ProductForm";
import { ConfirmModal } from "../components/ConfirmModal";

export function Products() {
  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [editingProduct, setEditingProduct] = useState<Product | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [newCategory, setNewCategory] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{
    type: "product" | "category";
    id: number;
    name: string;
  } | null>(null);

  const loadData = useCallback(async () => {
    try {
      const [prods, cats] = await Promise.all([getProducts(), getCategories()]);
      setProducts(prods);
      setCategories(cats);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSave = async (data: CreateProduct | UpdateProduct) => {
    try {
      if ("id" in data) {
        await updateProduct(data);
      } else {
        await createProduct(data);
      }
      setShowForm(false);
      setEditingProduct(null);
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async () => {
    if (!deleteConfirm) return;
    try {
      if (deleteConfirm.type === "product") {
        await deleteProduct(deleteConfirm.id);
      } else {
        await deleteCategory(deleteConfirm.id);
      }
      setDeleteConfirm(null);
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleAddCategory = async () => {
    if (!newCategory.trim()) return;
    try {
      await createCategory(newCategory.trim());
      setNewCategory("");
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  if (showForm) {
    return (
      <ProductForm
        product={editingProduct ?? undefined}
        categories={categories}
        onSave={handleSave}
        onCancel={() => {
          setShowForm(false);
          setEditingProduct(null);
        }}
      />
    );
  }

  return (
    <div className="products-page">
      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>x</button>
        </div>
      )}

      {deleteConfirm && (
        <ConfirmModal
          title={`Delete ${deleteConfirm.type === "product" ? "Product" : "Category"}`}
          message={`Are you sure you want to delete "${deleteConfirm.name}"?${
            deleteConfirm.type === "category"
              ? " Products in this category will become uncategorized."
              : ""
          }`}
          onConfirm={handleDelete}
          onCancel={() => setDeleteConfirm(null)}
          confirmText="Delete"
        />
      )}

      <section className="categories-section">
        <h3>Categories</h3>
        <div className="add-category">
          <input
            type="text"
            placeholder="New category..."
            value={newCategory}
            onChange={(e) => setNewCategory(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddCategory()}
          />
          <button onClick={handleAddCategory}>Add</button>
        </div>
        <div className="category-list">
          {categories.map((cat) => (
            <div key={cat.id} className="category-item">
              <span>{cat.name}</span>
              <button
                onClick={() =>
                  setDeleteConfirm({ type: "category", id: cat.id, name: cat.name })
                }
              >
                x
              </button>
            </div>
          ))}
        </div>
      </section>

      <section className="products-section">
        <div className="section-header">
          <h3>Products</h3>
          <button className="primary" onClick={() => setShowForm(true)}>
            Add Product
          </button>
        </div>
        <table className="products-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Price</th>
              <th>Qty</th>
              <th>Category</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {products.map((product) => (
              <tr
                key={product.id}
                className={
                  product.quantity <= product.low_stock_threshold
                    ? "low-stock"
                    : ""
                }
              >
                <td>{product.name}</td>
                <td>{product.price.toFixed(0)} ALL</td>
                <td>{product.quantity}</td>
                <td>{product.category_name ?? "-"}</td>
                <td>
                  <button
                    onClick={() => {
                      setEditingProduct(product);
                      setShowForm(true);
                    }}
                  >
                    Edit
                  </button>
                  <button
                    onClick={() =>
                      setDeleteConfirm({
                        type: "product",
                        id: product.id,
                        name: product.name,
                      })
                    }
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
