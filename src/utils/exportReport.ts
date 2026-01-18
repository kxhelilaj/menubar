import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile, writeFile } from "@tauri-apps/plugin-fs";
import { DaySummary } from "../types";
import * as XLSX from "xlsx";

interface ProductSummary {
  name: string;
  quantity: number;
  revenue: number;
}

function aggregateProducts(summary: DaySummary): ProductSummary[] {
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

  const productSummary: ProductSummary[] = [];
  productMap.forEach((value) => productSummary.push(value));
  productSummary.sort((a, b) => b.revenue - a.revenue);

  return productSummary;
}

export async function exportToCSV(summary: DaySummary): Promise<boolean> {
  const productSummary = aggregateProducts(summary);

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
      return true;
    }
    return false;
  } catch (e) {
    alert("Failed to save CSV: " + e);
    console.error("Failed to save CSV:", e);
    return false;
  }
}

export async function exportToText(summary: DaySummary): Promise<boolean> {
  const productSummary = aggregateProducts(summary);

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
      return true;
    }
    return false;
  } catch (e) {
    alert("Failed to save TXT: " + e);
    console.error("Failed to save TXT:", e);
    return false;
  }
}

export async function exportToExcel(summary: DaySummary): Promise<boolean> {
  const productSummary = aggregateProducts(summary);

  try {
    const wb = XLSX.utils.book_new();

    // Summary sheet
    const summaryData: (string | number | null)[][] = [
      ["Day Report", summary.date],
      [],
      ["Total Orders", summary.total_orders],
      ["Total Revenue (ALL)", summary.total_revenue],
      [],
      ["Products Sold"],
      ["Product", "Quantity", "Revenue (ALL)"],
      ...productSummary.map((p) => [p.name, p.quantity, p.revenue]),
      [],
      ["TOTAL", productSummary.reduce((sum, p) => sum + p.quantity, 0), summary.total_revenue],
    ];

    const summarySheet = XLSX.utils.aoa_to_sheet(summaryData);
    summarySheet["!cols"] = [
      { wch: 25 },
      { wch: 12 },
      { wch: 15 },
    ];
    XLSX.utils.book_append_sheet(wb, summarySheet, "Summary");

    // Orders sheet
    const ordersData: (string | number | null)[][] = [
      ["Order Details"],
      [],
      ["Order ID", "Table", "Staff", "Customer", "Total (ALL)", "Status", "Time"],
      ...summary.orders.map((o) => [
        o.order.id,
        o.order.table_number,
        o.order.staff_name || "-",
        o.order.customer_name || "-",
        o.order.total,
        o.order.status,
        o.order.created_at,
      ]),
    ];

    const ordersSheet = XLSX.utils.aoa_to_sheet(ordersData);
    ordersSheet["!cols"] = [
      { wch: 10 },
      { wch: 8 },
      { wch: 15 },
      { wch: 20 },
      { wch: 12 },
      { wch: 10 },
      { wch: 20 },
    ];
    XLSX.utils.book_append_sheet(wb, ordersSheet, "Orders");

    // Items sheet
    const itemsData: (string | number | null)[][] = [
      ["Items Sold"],
      [],
      ["Order ID", "Table", "Product", "Quantity", "Unit Price (ALL)", "Subtotal (ALL)"],
    ];

    summary.orders.forEach((o) => {
      o.items.forEach((item) => {
        itemsData.push([
          o.order.id,
          o.order.table_number,
          item.product_name || "Unknown",
          item.quantity,
          item.price_at_sale,
          item.quantity * item.price_at_sale,
        ]);
      });
    });

    const itemsSheet = XLSX.utils.aoa_to_sheet(itemsData);
    itemsSheet["!cols"] = [
      { wch: 10 },
      { wch: 8 },
      { wch: 25 },
      { wch: 10 },
      { wch: 15 },
      { wch: 15 },
    ];
    XLSX.utils.book_append_sheet(wb, itemsSheet, "Items");

    const excelBuffer = XLSX.write(wb, { bookType: "xlsx", type: "array" });

    const filePath = await save({
      defaultPath: `day-report-${summary.date}.xlsx`,
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
    });

    if (filePath) {
      await writeFile(filePath, new Uint8Array(excelBuffer));
      alert("Excel file saved successfully!");
      return true;
    }
    return false;
  } catch (e) {
    alert("Failed to save Excel: " + e);
    console.error("Failed to save Excel:", e);
    return false;
  }
}
