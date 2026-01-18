import { useState, useEffect, useCallback } from "react";
import { DaySession, DaySummary, Staff, OrderWithItems } from "../types";
import {
  closeDay,
  getSalesHistory,
  getDaySummary,
  getStaff,
  verifyStaffPin,
  getOrdersByDateRange,
  createDayClosingForDate,
  getActiveSession,
} from "../hooks/useTauri";
import { SalesHistory } from "../components/SalesHistory";
import { DayClosingComponent } from "../components/DayClosing";
import { DayCloseModal } from "../components/DayCloseModal";
import { PinModal } from "../components/PinModal";
import { exportToExcel, exportToCSV, exportToText } from "../utils/exportReport";

export function Reports() {
  const [history, setHistory] = useState<DaySession[]>([]);
  const [todaySummary, setTodaySummary] = useState<DaySummary | null>(null);
  const [selectedDaySummary, setSelectedDaySummary] = useState<DaySummary | null>(null);
  const [canCloseDay, setCanCloseDay] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showCloseModal, setShowCloseModal] = useState(false);
  const [showPinModal, setShowPinModal] = useState(false);
  const [closing, setClosing] = useState(false);
  const [staff, setStaff] = useState<Staff[]>([]);
  const [selectedStaff, setSelectedStaff] = useState<Staff | null>(null);

  // Recovery mode
  const [showRecovery, setShowRecovery] = useState(false);
  const [recoveryDate, setRecoveryDate] = useState("");
  const [recoveryOrders, setRecoveryOrders] = useState<OrderWithItems[]>([]);

  const loadData = useCallback(async () => {
    try {
      const [hist, today, staffList, session] = await Promise.all([
        getSalesHistory(30),
        getDaySummary(),
        getStaff(),
        getActiveSession(),
      ]);
      setHistory(hist);
      setTodaySummary(today);
      setStaff(staffList);

      // Can close if there's an active session
      setCanCloseDay(session !== null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleOpenCloseModal = () => {
    if (staff.length === 0) {
      setError("No staff members found. Please add staff first.");
      return;
    }
    setShowCloseModal(true);
  };

  const handleRequestPin = () => {
    if (!selectedStaff) {
      setError("Please select a staff member to authorize closing.");
      return;
    }
    setShowCloseModal(false);
    setShowPinModal(true);
  };

  const handleVerifyPin = async (pin: string): Promise<boolean> => {
    if (!selectedStaff) return false;

    try {
      const valid = await verifyStaffPin(selectedStaff.id, pin);
      if (valid) {
        setShowPinModal(false);
        await handleCloseDay();
        return true;
      }
      return false;
    } catch (e) {
      setError(String(e));
      return false;
    }
  };

  const handleCloseDay = async () => {
    setClosing(true);
    try {
      await closeDay();
      setSuccess("Day closed successfully!");
      await loadData();
    } catch (e) {
      setError(String(e));
    } finally {
      setClosing(false);
      setSelectedStaff(null);
    }
  };

  const handleSelectSession = async (sessionId: number) => {
    try {
      const summary = await getDaySummary(sessionId);
      setSelectedDaySummary(summary);
    } catch (e) {
      setError(String(e));
    }
  };

  // Recovery functions
  const handleSearchRecovery = async () => {
    if (!recoveryDate) {
      setError("Please enter a date to search");
      return;
    }
    try {
      const orders = await getOrdersByDateRange(recoveryDate, recoveryDate);
      setRecoveryOrders(orders);
      if (orders.length === 0) {
        setError(`No orders found for ${recoveryDate}`);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRecoverDayClosing = async () => {
    if (!recoveryDate) return;
    try {
      await createDayClosingForDate(recoveryDate);
      setSuccess(`Day closing created for ${recoveryDate}`);
      setShowRecovery(false);
      setRecoveryDate("");
      setRecoveryOrders([]);
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  const recoveryTotal = recoveryOrders.reduce((sum, o) => sum + o.order.total, 0);

  return (
    <div className="reports-page">
      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {success && (
        <div className="success-banner">
          {success}
          <button onClick={() => setSuccess(null)}>×</button>
        </div>
      )}

      {showCloseModal && todaySummary && (
        <DayCloseModal
          summary={todaySummary}
          onConfirm={handleRequestPin}
          onCancel={() => {
            setShowCloseModal(false);
            setSelectedStaff(null);
          }}
          loading={closing}
        >
          <div className="staff-auth-section">
            <label>Authorize with staff PIN:</label>
            <select
              value={selectedStaff?.id ?? ""}
              onChange={(e) => {
                const s = staff.find((st) => st.id === Number(e.target.value));
                setSelectedStaff(s || null);
              }}
            >
              <option value="">Select staff...</option>
              {staff.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </select>
          </div>
        </DayCloseModal>
      )}

      {showPinModal && selectedStaff && (
        <PinModal
          staffName={selectedStaff.name}
          onVerify={handleVerifyPin}
          onCancel={() => {
            setShowPinModal(false);
            setSelectedStaff(null);
          }}
        />
      )}

      <div className="reports-header">
        <h2>Reports</h2>
        <button
          className="recovery-btn"
          onClick={() => setShowRecovery(!showRecovery)}
        >
          {showRecovery ? "Hide Recovery" : "Recovery Mode"}
        </button>
      </div>

      {showRecovery && (
        <div className="recovery-section">
          <h3>Recover Missing Day Closing</h3>
          <p>Search for orders on a specific date to create a missing day closing.</p>
          <div className="recovery-controls">
            <input
              type="date"
              value={recoveryDate}
              onChange={(e) => setRecoveryDate(e.target.value)}
            />
            <button onClick={handleSearchRecovery}>Search Orders</button>
          </div>

          {recoveryOrders.length > 0 && (
            <div className="recovery-results">
              <h4>Found {recoveryOrders.length} orders - Total: {recoveryTotal.toFixed(0)} ALL</h4>
              <div className="recovery-orders">
                {recoveryOrders.map((o) => (
                  <div key={o.order.id} className="recovery-order">
                    <span>#{o.order.id}</span>
                    <span>{o.order.staff_name}</span>
                    <span>Table {o.order.table_number}</span>
                    <span>{o.order.total.toFixed(0)} ALL</span>
                    <span className={`status-${o.order.status}`}>{o.order.status}</span>
                  </div>
                ))}
              </div>
              <button className="create-closing-btn" onClick={handleRecoverDayClosing}>
                Create Day Closing for {recoveryDate}
              </button>
            </div>
          )}
        </div>
      )}

      <div className="reports-content">
        <div className="reports-left">
          <DayClosingComponent
            summary={todaySummary}
            onClose={handleOpenCloseModal}
            canClose={canCloseDay}
          />
        </div>
        <div className="reports-right">
          <SalesHistory history={history} onSelectSession={handleSelectSession} />
          {selectedDaySummary && (
            <div className="selected-day">
              <h4>{selectedDaySummary.date}</h4>
              <p>Orders: {selectedDaySummary.total_orders}</p>
              <p>Revenue: {selectedDaySummary.total_revenue.toFixed(0)} ALL</p>
              <div className="export-buttons">
                <button className="export-btn primary" onClick={() => exportToExcel(selectedDaySummary)}>
                  Export Excel
                </button>
                <button className="export-btn" onClick={() => exportToCSV(selectedDaySummary)}>
                  Export CSV
                </button>
                <button className="export-btn" onClick={() => exportToText(selectedDaySummary)}>
                  Export TXT
                </button>
              </div>
              <button onClick={() => setSelectedDaySummary(null)}>Close</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
