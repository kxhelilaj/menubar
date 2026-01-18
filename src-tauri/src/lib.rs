mod commands;
mod db;
mod models;

#[cfg(test)]
mod tests;

use commands::{categories, orders, products, reports, staff};
use db::Database;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database
            let db = Database::new(&app.handle()).expect("Failed to create database");
            db.initialize().expect("Failed to initialize database");
            app.manage(db);

            // Create tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    if event.id == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Categories
            categories::get_categories,
            categories::create_category,
            categories::delete_category,
            // Products
            products::get_products,
            products::create_product,
            products::update_product,
            products::delete_product,
            products::get_low_stock,
            // Staff
            staff::get_staff,
            staff::create_staff,
            staff::delete_staff,
            staff::verify_staff_pin,
            // Orders
            orders::create_order,
            orders::get_today_orders,
            orders::get_open_orders,
            orders::get_order,
            orders::add_items_to_order,
            orders::mark_order_paid,
            orders::decrease_item_quantity,
            orders::increase_item_quantity,
            orders::update_order_notes,
            // Reports
            reports::close_day,
            reports::get_sales_history,
            reports::get_day_summary,
            reports::get_orders_by_date_range,
            reports::create_day_closing_for_date,
            // Day Sessions
            reports::get_active_session,
            reports::start_day,
            reports::is_day_active,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
