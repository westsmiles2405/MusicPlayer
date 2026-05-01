import { useState } from "react";

interface PlaylistCreateDialogProps {
  open: boolean;
  onClose: () => void;
  onCreate: (name: string) => Promise<void>;
}

export function PlaylistCreateDialog({
  open,
  onClose,
  onCreate,
}: PlaylistCreateDialogProps) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  if (!open) return null;

  const trimmed = name.trim();

  const handleSubmit = async () => {
    if (!trimmed) return;
    setError(null);
    setSubmitting(true);
    try {
      await onCreate(trimmed);
      setName("");
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : "创建失败");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div
        className="dialog"
        role="dialog"
        aria-label="创建播放列表"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="dialog__title">创建播放列表</h2>
        <input
          type="text"
          aria-label="播放列表名称"
          placeholder="播放列表名称"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSubmit();
          }}
          disabled={submitting}
        />
        {error && <p className="dialog__error">{error}</p>}
        <div className="dialog__actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={!trimmed || submitting}
          >
            创建
          </button>
        </div>
      </div>
    </div>
  );
}
