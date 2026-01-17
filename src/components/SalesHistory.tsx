import { DayClosing } from "../types";

interface SalesHistoryProps {
  history: DayClosing[];
  onSelectDate: (date: string) => void;
}

export function SalesHistory({ history, onSelectDate }: SalesHistoryProps) {
  return (
    <div className="sales-history">
      <h3>Sales History</h3>
      {history.length === 0 ? (
        <p>No closed days yet</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Date</th>
              <th>Orders</th>
              <th>Revenue</th>
            </tr>
          </thead>
          <tbody>
            {history.map((day) => (
              <tr
                key={day.id}
                onClick={() => onSelectDate(day.date)}
                className="clickable"
              >
                <td>{day.date}</td>
                <td>{day.total_orders}</td>
                <td>{day.total_revenue.toFixed(0)} ALL</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
