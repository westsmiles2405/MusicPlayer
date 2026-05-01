import { useEffect, useState } from "react";

interface PlaylistRenameDialogProps {
  open: boolean;
  onClose: () => void;
  onRename: (name: string) => Promise<void>;
  initialName: string;
}

export function PlaylistRenameDialog({
  open,
  onClose,
  onRename,
  initialName,
}: PlaylistRenameDialogProps) {
  const [name, setName] = useState(initialName);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setName(initialName);
      setError(null);
    }
  }, [open, initialName]);

  if (!open) return null;

  const trimmed = name.trim();

  const handleSubmit = async () => {
    if (!trimmed) return;
    setError(null);
    setSubmitting(true);
    try {
      await onRename(trimmed);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : "重命名失败");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div
        className="dialog"
        role="dialog"
        aria-label="重命名播放列表"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="dialog__title">重命名播放列表</h2>
        <input
          type="text"
          aria-label="播放列表名称"
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
            disabled={!trimmed || trimmed === initialName || submitting}
          >
            重命名
          </button>
        </div>
      </div>
    </div>
  );
}
