import { Staff } from "../types";

interface StaffSelectorProps {
  staff: Staff[];
  selectedStaff: Staff | null;
  onSelect: (staff: Staff) => void;
}

export function StaffSelector({
  staff,
  selectedStaff,
  onSelect,
}: StaffSelectorProps) {
  return (
    <div className="staff-selector">
      <label>Staff:</label>
      <select
        value={selectedStaff?.id ?? ""}
        onChange={(e) => {
          const selected = staff.find((s) => s.id === Number(e.target.value));
          if (selected) onSelect(selected);
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
  );
}
