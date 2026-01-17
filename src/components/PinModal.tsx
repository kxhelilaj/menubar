import { useState } from "react";

interface PinModalProps {
  staffName: string;
  onVerify: (pin: string) => Promise<boolean>;
  onCancel: () => void;
}

export function PinModal({ staffName, onVerify, onCancel }: PinModalProps) {
  const [pin, setPin] = useState("");
  const [error, setError] = useState(false);
  const [verifying, setVerifying] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pin || verifying) return;

    setVerifying(true);
    setError(false);

    const valid = await onVerify(pin);
    if (!valid) {
      setError(true);
      setPin("");
    }
    setVerifying(false);
  };

  const handleKeyPress = (num: string) => {
    if (pin.length < 4) {
      setPin((prev) => prev + num);
      setError(false);
    }
  };

  const handleBackspace = () => {
    setPin((prev) => prev.slice(0, -1));
    setError(false);
  };

  const handleClear = () => {
    setPin("");
    setError(false);
  };

  return (
    <div className="modal-overlay">
      <div className="pin-modal">
        <h3>Enter PIN</h3>
        <p className="pin-staff-name">{staffName}</p>

        <form onSubmit={handleSubmit}>
          <div className={`pin-display ${error ? "error" : ""}`}>
            {[0, 1, 2, 3].map((i) => (
              <div key={i} className={`pin-dot ${pin.length > i ? "filled" : ""}`} />
            ))}
          </div>

          {error && <div className="pin-error">Invalid PIN</div>}

          <div className="pin-keypad">
            {["1", "2", "3", "4", "5", "6", "7", "8", "9", "C", "0", "←"].map((key) => (
              <button
                key={key}
                type="button"
                className={`pin-key ${key === "C" || key === "←" ? "pin-key-action" : ""}`}
                onClick={() => {
                  if (key === "←") handleBackspace();
                  else if (key === "C") handleClear();
                  else handleKeyPress(key);
                }}
              >
                {key}
              </button>
            ))}
          </div>

          <div className="pin-actions">
            <button type="button" className="btn-cancel" onClick={onCancel}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn-confirm"
              disabled={pin.length !== 4 || verifying}
            >
              {verifying ? "..." : "Confirm"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
