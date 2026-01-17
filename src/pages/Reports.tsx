import { useState, useEffect, useCallback } from "react";
import { DayClosing, DaySummary } from "../types";
import { closeDay, getSalesHistory, getDaySummary } from "../hooks/useTauri";
import { SalesHistory } from "../components/SalesHistory";
import { DayClosingComponent } from "../components/DayClosing";
import { DayCloseModal } from "../components/DayCloseModal";

export function Reports() {
  const [history, setHistory] = useState<DayClosing[]>([]);
  const [todaySummary, setTodaySummary] = useState<DaySummary | null>(null);
  const [selectedDaySummary, setSelectedDaySummary] =
    useState<DaySummary | null>(null);
  const [canCloseDay, setCanCloseDay] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCloseModal, setShowCloseModal] = useState(false);
  const [closing, setClosing] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const [hist, today] = await Promise.all([
        getSalesHistory(30),
        getDaySummary(),
      ]);
      setHistory(hist);
      setTodaySummary(today);

      const todayDate = new Date().toISOString().split("T")[0];
      setCanCloseDay(!hist.some((h) => h.date === todayDate));
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleOpenCloseModal = () => {
    setShowCloseModal(true);
  };

  const handleCloseDay = async () => {
    setClosing(true);
    try {
      await closeDay();
      setShowCloseModal(false);
      await loadData();
    } catch (e) {
      setError(String(e));
    } finally {
      setClosing(false);
    }
  };

  const handleSelectDate = async (date: string) => {
    try {
      const summary = await getDaySummary(date);
      setSelectedDaySummary(summary);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="reports-page">
      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>x</button>
        </div>
      )}

      {showCloseModal && todaySummary && (
        <DayCloseModal
          summary={todaySummary}
          onConfirm={handleCloseDay}
          onCancel={() => setShowCloseModal(false)}
          loading={closing}
        />
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
          <SalesHistory history={history} onSelectDate={handleSelectDate} />
          {selectedDaySummary && (
            <div className="selected-day">
              <h4>{selectedDaySummary.date}</h4>
              <p>Orders: {selectedDaySummary.total_orders}</p>
              <p>Revenue: {selectedDaySummary.total_revenue.toFixed(0)} ALL</p>
              <button onClick={() => setSelectedDaySummary(null)}>Close</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
