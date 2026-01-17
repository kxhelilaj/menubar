import { useState, useEffect, useCallback } from "react";
import { Staff as StaffType, CreateStaff } from "../types";
import { getStaff, createStaff, deleteStaff } from "../hooks/useTauri";
import { ConfirmModal } from "../components/ConfirmModal";

export function Staff() {
  const [staffList, setStaffList] = useState<StaffType[]>([]);
  const [name, setName] = useState("");
  const [pin, setPin] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{
    id: number;
    name: string;
  } | null>(null);

  const loadData = useCallback(async () => {
    try {
      const staff = await getStaff();
      setStaffList(staff);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleAdd = async () => {
    if (!name.trim()) return;
    try {
      const data: CreateStaff = {
        name: name.trim(),
        pin: pin.trim() || null,
      };
      await createStaff(data);
      setName("");
      setPin("");
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async () => {
    if (!deleteConfirm) return;
    try {
      await deleteStaff(deleteConfirm.id);
      setDeleteConfirm(null);
      await loadData();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="staff-page">
      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>x</button>
        </div>
      )}

      {deleteConfirm && (
        <ConfirmModal
          title="Delete Staff Member"
          message={`Are you sure you want to delete "${deleteConfirm.name}"?`}
          onConfirm={handleDelete}
          onCancel={() => setDeleteConfirm(null)}
          confirmText="Delete"
        />
      )}

      <section className="add-staff-section">
        <h3>Add Staff Member</h3>
        <div className="add-staff-form">
          <input
            type="text"
            placeholder="Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <input
            type="text"
            placeholder="PIN (optional)"
            value={pin}
            onChange={(e) => setPin(e.target.value)}
          />
          <button className="primary" onClick={handleAdd}>
            Add
          </button>
        </div>
      </section>

      <section className="staff-list-section">
        <h3>Staff Members</h3>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>PIN</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {staffList.map((staff) => (
              <tr key={staff.id}>
                <td>{staff.name}</td>
                <td>{staff.pin ? "****" : "-"}</td>
                <td>
                  <button
                    onClick={() =>
                      setDeleteConfirm({ id: staff.id, name: staff.name })
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
