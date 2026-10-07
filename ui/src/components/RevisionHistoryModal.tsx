import React, { useState, useEffect } from 'react';
import { api } from '../api';
import type { DocumentStateDto, SnapshotComparison } from '../types';

interface RevisionHistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  docState: DocumentStateDto;
  onCreateSnapshot: (name: string, description?: string | null) => void;
  onRestoreSnapshot: (snapshotId: string) => void;
  onDeleteSnapshot: (snapshotId: string) => void;
  onForkSnapshot: (snapshotId: string, newTitle: string) => void;
}

export const RevisionHistoryModal: React.FC<RevisionHistoryModalProps> = ({
  isOpen,
  onClose,
  docState,
  onCreateSnapshot,
  onRestoreSnapshot,
  onDeleteSnapshot,
  onForkSnapshot,
}) => {
  const [selectedSnapshotId, setSelectedSnapshotId] = useState<string | null>(null);
  const [snapshotNameInput, setSnapshotNameInput] = useState('');
  const [snapshotDescInput, setSnapshotDescInput] = useState('');
  const [comparison, setComparison] = useState<SnapshotComparison | null>(null);
  const [isLoadingDiff, setIsLoadingDiff] = useState(false);
  const [statusMsg, setStatusMsg] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const snapshots = docState.document.snapshots || [];

  // Default selection
  useEffect(() => {
    if (isOpen) {
      if (snapshots.length > 0 && !selectedSnapshotId) {
        setSelectedSnapshotId(snapshots[snapshots.length - 1].id);
      }
      setSnapshotNameInput('');
      setSnapshotDescInput('');
      setStatusMsg(null);
    }
  }, [isOpen, snapshots.length]);

  // Load comparison diff when selected snapshot changes
  useEffect(() => {
    if (!isOpen || !selectedSnapshotId) {
      setComparison(null);
      return;
    }
    let isCurrent = true;
    setIsLoadingDiff(true);
    api
      .compareSnapshot(selectedSnapshotId)
      .then((comp) => {
        if (isCurrent) setComparison(comp);
      })
      .catch((err) => {
        console.error('Failed to compare snapshot:', err);
      })
      .finally(() => {
        if (isCurrent) setIsLoadingDiff(false);
      });

    return () => {
      isCurrent = false;
    };
  }, [isOpen, selectedSnapshotId, docState.document.sections, docState.document.title]);

  if (!isOpen) return null;

  const handleCreateSnapshot = () => {
    const name = snapshotNameInput.trim() || `Revision ${snapshots.length + 1}`;
    onCreateSnapshot(name, snapshotDescInput.trim() || null);
    setSnapshotNameInput('');
    setSnapshotDescInput('');
    setStatusMsg({ text: `Created snapshot "${name}"`, type: 'success' });
  };

  const handleRestore = (id: string, name: string) => {
    if (confirm(`Restore prompt to snapshot "${name}"? Current canvas will be replaced (can be undone with Ctrl+Z).`)) {
      onRestoreSnapshot(id);
      setStatusMsg({ text: `Restored to "${name}"`, type: 'success' });
    }
  };

  const handleFork = (id: string, currentName: string) => {
    const newTitle = prompt(
      'Enter new prompt document title for fork:',
      `${docState.document.title} (${currentName} Branch)`
    );
    if (newTitle && newTitle.trim()) {
      onForkSnapshot(id, newTitle.trim());
      onClose();
    }
  };

  const selectedSnapshot = snapshots.find((s) => s.id === selectedSnapshotId);

  const statusBadge = (status: string) => {
    switch (status) {
      case 'added':
        return <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-950/60 text-emerald-400 border border-emerald-800/80 uppercase font-semibold">Added</span>;
      case 'removed':
        return <span className="text-[10px] px-1.5 py-0.5 rounded bg-rose-950/60 text-rose-400 border border-rose-800/80 uppercase font-semibold">Removed</span>;
      case 'modified':
        return <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-950/60 text-amber-400 border border-amber-800/80 uppercase font-semibold">Modified</span>;
      default:
        return <span className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 uppercase font-semibold">Unchanged</span>;
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="bg-zinc-950 border border-zinc-800 rounded-xl max-w-5xl w-full flex flex-col max-h-[92vh] shadow-2xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-800/80 flex justify-between items-center bg-zinc-900/40">
          <div>
            <h2 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-violet-400" />
              Revision History &amp; Visual Diff Inspector
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Tag named milestones, inspect visual diffs against historical drafts, and branch or restore versions.
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
          >
            ✕
          </button>
        </div>

        {/* Status Message */}
        {statusMsg && (
          <div
            className={`mx-6 mt-3 p-2 rounded text-xs border ${
              statusMsg.type === 'success'
                ? 'bg-emerald-950/40 border-emerald-800/60 text-emerald-300'
                : 'bg-rose-950/40 border-rose-800/60 text-rose-300'
            }`}
          >
            {statusMsg.text}
          </div>
        )}

        {/* Content Body */}
        <div className="flex-1 overflow-hidden grid grid-cols-1 md:grid-cols-12">
          {/* Left Column (4 cols): Snapshot List & Create Form */}
          <div className="md:col-span-4 border-r border-zinc-800/80 p-4 overflow-y-auto space-y-4 bg-zinc-900/20">
            {/* Create Snapshot Box */}
            <div className="p-3 bg-zinc-900/70 border border-zinc-800 rounded-lg space-y-2">
              <span className="text-xs font-semibold text-zinc-200 block">
                Capture New Snapshot
              </span>
              <input
                type="text"
                placeholder="e.g. v1.1 Chain of Thought"
                value={snapshotNameInput}
                onChange={(e) => setSnapshotNameInput(e.target.value)}
                className="w-full px-2.5 py-1 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200 focus:outline-none focus:border-zinc-700"
              />
              <input
                type="text"
                placeholder="Optional notes or rationale..."
                value={snapshotDescInput}
                onChange={(e) => setSnapshotDescInput(e.target.value)}
                className="w-full px-2.5 py-1 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200 focus:outline-none focus:border-zinc-700"
              />
              <button
                type="button"
                onClick={handleCreateSnapshot}
                className="w-full py-1.5 bg-violet-600 hover:bg-violet-500 text-white rounded text-xs font-medium transition-colors cursor-pointer"
              >
                + Snapshot Current State
              </button>
            </div>

            {/* Snapshots List */}
            <div className="space-y-1.5">
              <span className="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider block px-1">
                Saved Snapshots ({snapshots.length})
              </span>

              {snapshots.length === 0 ? (
                <div className="p-4 text-center text-xs text-zinc-500 border border-dashed border-zinc-800 rounded-lg">
                  No snapshots captured yet. Capture your first milestone above.
                </div>
              ) : (
                snapshots.map((snap) => (
                  <div
                    key={snap.id}
                    onClick={() => setSelectedSnapshotId(snap.id)}
                    className={`p-3 rounded-lg border transition-all cursor-pointer ${
                      selectedSnapshotId === snap.id
                        ? 'bg-zinc-800/90 border-violet-500/50 shadow-sm'
                        : 'bg-zinc-900/40 border-zinc-800/80 hover:border-zinc-700'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-zinc-200 truncate">
                        {snap.name}
                      </span>
                      <span className="text-[10px] font-mono text-zinc-500">
                        {snap.sections.length} sec
                      </span>
                    </div>
                    {snap.description && (
                      <p className="text-[11px] text-zinc-400 line-clamp-1 mt-0.5">
                        {snap.description}
                      </p>
                    )}
                    <span className="text-[10px] text-zinc-500 font-mono block mt-1">
                      {snap.created_at}
                    </span>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Right Column (8 cols): Snapshot Details & Visual Diff Breakdown */}
          <div className="md:col-span-8 p-5 flex flex-col overflow-hidden bg-zinc-950 space-y-3">
            {!selectedSnapshot ? (
              <div className="h-full flex items-center justify-center text-xs text-zinc-500 italic">
                Select a snapshot on the left to inspect differences.
              </div>
            ) : (
              <>
                {/* Snapshot Header & Actions */}
                <div className="p-3 bg-zinc-900/60 border border-zinc-800 rounded-lg flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-sm font-semibold text-zinc-200">
                        {selectedSnapshot.name}
                      </h3>
                      <span className="text-[10px] font-mono text-zinc-400 bg-zinc-800 px-1.5 py-0.5 rounded border border-zinc-700">
                        {selectedSnapshot.created_at}
                      </span>
                    </div>
                    {selectedSnapshot.description && (
                      <p className="text-xs text-zinc-400 mt-0.5">
                        {selectedSnapshot.description}
                      </p>
                    )}
                  </div>

                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() => handleRestore(selectedSnapshot.id, selectedSnapshot.name)}
                      className="px-2.5 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 rounded text-xs font-medium transition-colors cursor-pointer"
                      title="Restore canvas to this revision"
                    >
                      ↺ Restore
                    </button>
                    <button
                      type="button"
                      onClick={() => handleFork(selectedSnapshot.id, selectedSnapshot.name)}
                      className="px-2.5 py-1.5 bg-violet-600/90 hover:bg-violet-600 text-white rounded text-xs font-medium transition-colors cursor-pointer"
                      title="Fork into a new prompt branch"
                    >
                      ⎇ Fork Branch...
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        if (confirm(`Delete snapshot "${selectedSnapshot.name}"?`)) {
                          onDeleteSnapshot(selectedSnapshot.id);
                          setSelectedSnapshotId(null);
                        }
                      }}
                      className="p-1.5 text-zinc-500 hover:text-rose-400 hover:bg-rose-950/20 rounded transition-colors cursor-pointer"
                      title="Delete Snapshot"
                    >
                      🗑
                    </button>
                  </div>
                </div>

                {/* Section Diff Breakdown */}
                <div className="flex-1 overflow-y-auto space-y-2 pr-1">
                  <div className="flex items-center justify-between text-xs text-zinc-400 pb-1">
                    <span className="font-semibold">
                      Comparison vs Active Canvas ({comparison?.section_diffs.length || 0} sections)
                    </span>
                    {comparison?.is_title_changed && (
                      <span className="text-[11px] text-amber-400 font-mono">
                        Title changed: &quot;{comparison.snapshot_title}&quot; → &quot;{comparison.current_title}&quot;
                      </span>
                    )}
                  </div>

                  {isLoadingDiff ? (
                    <div className="py-8 text-center text-xs text-zinc-500">
                      Computing visual differences...
                    </div>
                  ) : comparison?.section_diffs.map((diff, idx) => (
                    <div
                      key={idx}
                      className="p-3 bg-zinc-900/40 border border-zinc-800/80 rounded-lg space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-xs font-mono font-semibold text-zinc-200">
                          &lt;{diff.tag}&gt;
                        </span>
                        {statusBadge(diff.status)}
                      </div>

                      {diff.status === 'modified' ? (
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs font-mono pt-1">
                          <div className="p-2 bg-rose-950/20 border border-rose-900/40 rounded text-rose-300">
                            <span className="text-[10px] text-rose-400/80 uppercase font-sans font-semibold block mb-0.5">
                              Snapshot Version
                            </span>
                            <pre className="whitespace-pre-wrap">{diff.snapshot_brief}</pre>
                          </div>
                          <div className="p-2 bg-emerald-950/20 border border-emerald-900/40 rounded text-emerald-300">
                            <span className="text-[10px] text-emerald-400/80 uppercase font-sans font-semibold block mb-0.5">
                              Current Version
                            </span>
                            <pre className="whitespace-pre-wrap">{diff.current_brief}</pre>
                          </div>
                        </div>
                      ) : diff.status === 'added' ? (
                        <div className="p-2 bg-emerald-950/20 border border-emerald-900/40 rounded text-xs font-mono text-emerald-300">
                          <span className="text-[10px] text-emerald-400/80 uppercase font-sans font-semibold block mb-0.5">
                            Added in Active Canvas
                          </span>
                          <pre className="whitespace-pre-wrap">{diff.current_brief}</pre>
                        </div>
                      ) : diff.status === 'removed' ? (
                        <div className="p-2 bg-rose-950/20 border border-rose-900/40 rounded text-xs font-mono text-rose-300">
                          <span className="text-[10px] text-rose-400/80 uppercase font-sans font-semibold block mb-0.5">
                            Present in Snapshot, Removed in Active
                          </span>
                          <pre className="whitespace-pre-wrap">{diff.snapshot_brief}</pre>
                        </div>
                      ) : (
                        <p className="text-xs text-zinc-400 line-clamp-2 font-mono">
                          {diff.current_brief}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-zinc-800/80 flex justify-between items-center bg-zinc-900/30">
          <span className="text-xs text-zinc-500">
            {snapshots.length} historical revision{snapshots.length === 1 ? '' : 's'} preserved
          </span>
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 rounded text-xs font-medium transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
