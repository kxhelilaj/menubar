import { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
  currentPage: string;
  onNavigate: (page: string) => void;
}

export function Layout({ children, currentPage, onNavigate }: LayoutProps) {
  const navItems = [
    { id: "dashboard", label: "Orders" },
    { id: "products", label: "Products" },
    { id: "staff", label: "Staff" },
    { id: "reports", label: "Reports" },
  ];

  return (
    <div className="layout">
      <nav className="nav">
        {navItems.map((item) => (
          <button
            key={item.id}
            className={`nav-btn ${currentPage === item.id ? "active" : ""}`}
            onClick={() => onNavigate(item.id)}
          >
            {item.label}
          </button>
        ))}
      </nav>
      <main className="main">{children}</main>
    </div>
  );
}
