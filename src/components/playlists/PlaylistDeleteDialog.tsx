import { useState } from "react";

interface PlaylistDeleteDialogProps {
  open: boolean;
  onClose: () => void;
  onDelete: () => Promise<void>;
  playlistName: string;
}

export function PlaylistDeleteDialog({
  open,
  onClose,
  onDelete,
  playlistName,
}: PlaylistDeleteDialogProps) {
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  if (!open) return null;

  const handleSubmit = async () => {
    setError(null);
    setSubmitting(true);
    try {
      await onDelete();
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : "删除失败");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div
        className="dialog"
        role="dialog"
        aria-label="删除播放列表"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="dialog__title">删除播放列表</h2>
        <p>
          确定要删除播放列表「{playlistName}」吗？此操作不可撤销。
        </p>
        {error && <p className="dialog__error">{error}</p>}
        <div className="dialog__actions">
          <button type="button" onClick={onClose} disabled={submitting}>
            取消
          </button>
          <button type="button" onClick={handleSubmit} disabled={submitting}>
            删除
          </button>
        </div>
      </div>
    </div>
  );
}
