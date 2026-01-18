import { DaySession } from "../types";

interface SalesHistoryProps {
  history: DaySession[];
  onSelectSession: (sessionId: number) => void;
}

// Format session time range for display
function formatSessionTimeRange(session: DaySession): string {
  const startDate = new Date(session.started_at);
  const endDate = session.closed_at ? new Date(session.closed_at) : null;

  const formatDate = (d: Date) => {
    const month = d.toLocaleString("en-US", { month: "short" });
    const day = d.getDate();
    return `${month} ${day}`;
  };

  const formatTime = (d: Date) => {
    return d.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
  };

  const startDateStr = formatDate(startDate);
  const startTimeStr = formatTime(startDate);

  if (!endDate) {
    return `${startDateStr}, ${startTimeStr}`;
  }

  const endDateStr = formatDate(endDate);
  const endTimeStr = formatTime(endDate);

  // Check if session spans multiple days
  if (startDateStr !== endDateStr) {
    return `${startDateStr}, ${startTimeStr} - ${endDateStr}, ${endTimeStr}`;
  }

  return `${startDateStr}, ${startTimeStr} - ${endTimeStr}`;
}

export function SalesHistory({ history, onSelectSession }: SalesHistoryProps) {
  return (
    <div className="sales-history">
      <h3>Sales History</h3>
      {history.length === 0 ? (
        <p>No closed sessions yet</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Session</th>
              <th>Orders</th>
              <th>Revenue</th>
            </tr>
          </thead>
          <tbody>
            {history.map((session) => (
              <tr
                key={session.id}
                onClick={() => onSelectSession(session.id)}
                className="clickable"
              >
                <td>{formatSessionTimeRange(session)}</td>
                <td>{session.total_orders ?? 0}</td>
                <td>{(session.total_revenue ?? 0).toFixed(0)} ALL</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
