import React, { useState, useEffect } from 'react';
import {
  X,
  Check,
  CheckCheck,
  Ban,
  PlusCircle,
  AlertTriangle,
  HelpCircle,
  MessageSquareQuote,
  Sparkles,
  GitCompare,
} from 'lucide-react';
import type {
  PendingRefinements,
  PromptDocument,
  DiffLine,
  DocumentStateDto,
} from '../types';
import { api } from '../api';

interface DiffReviewModalProps {
  isOpen: boolean;
  onClose: () => void;
  pending: PendingRefinements | null;
  document: PromptDocument;
  onUpdateState: (newState: DocumentStateDto) => void;
}

export const DiffReviewModal: React.FC<DiffReviewModalProps> = ({
  isOpen,
  onClose,
  pending,
  document,
  onUpdateState,
}) => {
  const [diffsBySection, setDiffsBySection] = useState<Record<string, DiffLine[]>>({});

  // Compute diffs for all proposed changes when pending changes arrive
  useEffect(() => {
    if (!pending || !isOpen) return;

    for (const change of pending.changes) {
      const origSec = document.sections.find((s) => s.id === change.section_id);
      const origText = origSec ? (origSec.refined || origSec.brief) : '';
      api
        .computeDiff(origText, change.proposed_text)
        .then((diff) => {
          setDiffsBySection((prev) => ({ ...prev, [change.section_id]: diff }));
        })
        .catch((err) => console.error('Diff calculation error:', err));
    }
  }, [pending, document, isOpen]);

  if (!isOpen || !pending) return null;

  const handleAcceptOne = async (sectionId: string) => {
    try {
      const newState = await api.acceptRefinement(sectionId);
      onUpdateState(newState);
      if (!newState.pending_refinements || newState.pending_refinements.changes.length === 0) {
        onClose();
      }
    } catch (err) {
      console.error('Failed to accept refinement:', err);
    }
  };

  const handleRejectOne = async (sectionId: string) => {
    try {
      const newState = await api.rejectRefinement(sectionId);
      onUpdateState(newState);
      if (!newState.pending_refinements || newState.pending_refinements.changes.length === 0) {
        onClose();
      }
    } catch (err) {
      console.error('Failed to reject refinement:', err);
    }
  };

  const handleAcceptSuggested = async (index: number) => {
    try {
      const newState = await api.acceptSuggestedSection(index);
      onUpdateState(newState);
    } catch (err) {
      console.error('Failed to accept suggested section:', err);
    }
  };

  const handleAcceptAll = async () => {
    try {
      const newState = await api.acceptAllRefinements();
      onUpdateState(newState);
      onClose();
    } catch (err) {
      console.error('Failed to accept all refinements:', err);
    }
  };

  const handleRejectAll = async () => {
    try {
      const newState = await api.rejectAllRefinements();
      onUpdateState(newState);
      onClose();
    } catch (err) {
      console.error('Failed to reject all refinements:', err);
    }
  };

  const hasChanges = pending.changes.length > 0;
  const hasSuggestions = pending.suggested_sections.length > 0;

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-4xl shadow-2xl flex flex-col max-h-[92vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Modal Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/60">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-400">
              <GitCompare className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100 flex items-center gap-2">
                <span>Refinement Review</span>
                <span className="text-xs font-mono font-medium px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30">
                  {pending.changes.length} changes, {pending.suggested_sections.length} suggestions
                </span>
              </h2>
              <p className="text-xs text-slate-400">
                Inspect proposed modifications line-by-line and accept or reject granularly
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Modal Scrollable Content */}
        <div className="p-6 overflow-y-auto space-y-6">
          {/* Critique Banner */}
          {pending.critique && (
            <div className="p-4 rounded-xl bg-indigo-950/20 border border-indigo-500/30 space-y-1.5">
              <div className="flex items-center gap-2 text-indigo-300 font-semibold text-xs">
                <MessageSquareQuote className="w-4 h-4 text-indigo-400" />
                <span>AI Architectural Critique</span>
              </div>
              <p className="text-xs text-indigo-100/90 leading-relaxed font-sans whitespace-pre-wrap">
                {pending.critique}
              </p>
            </div>
          )}

          {/* Warnings Banner */}
          {pending.warnings && pending.warnings.length > 0 && (
            <div className="p-3.5 rounded-xl bg-amber-950/20 border border-amber-500/30 space-y-1">
              <div className="flex items-center gap-2 text-amber-300 font-semibold text-xs">
                <AlertTriangle className="w-4 h-4 text-amber-400" />
                <span>Potential Hazards / Inconsistencies</span>
              </div>
              <ul className="list-disc list-inside text-xs text-amber-200/80 space-y-0.5">
                {pending.warnings.map((w, idx) => (
                  <li key={idx}>{w}</li>
                ))}
              </ul>
            </div>
          )}

          {/* Open Questions */}
          {pending.open_questions && pending.open_questions.length > 0 && (
            <div className="p-3.5 rounded-xl bg-blue-950/20 border border-blue-500/30 space-y-1">
              <div className="flex items-center gap-2 text-blue-300 font-semibold text-xs">
                <HelpCircle className="w-4 h-4 text-blue-400" />
                <span>Open Questions For Clarification</span>
              </div>
              <ul className="list-disc list-inside text-xs text-blue-200/80 space-y-0.5">
                {pending.open_questions.map((q, idx) => (
                  <li key={idx}>{q}</li>
                ))}
              </ul>
            </div>
          )}

          {/* Section Changes */}
          {hasChanges && (
            <div className="space-y-4">
              <h3 className="text-xs font-bold text-slate-300 uppercase tracking-wider">
                Proposed Section Modifications ({pending.changes.length})
              </h3>

              <div className="space-y-4">
                {pending.changes.map((change) => {
                  const sec = document.sections.find((s) => s.id === change.section_id);
                  const tagName = sec ? sec.tag : 'unknown';
                  const diff = diffsBySection[change.section_id];

                  return (
                    <div
                      key={change.section_id}
                      className="rounded-xl border border-slate-800 bg-slate-950/50 overflow-hidden"
                    >
                      {/* Section Change Header */}
                      <div className="flex items-center justify-between px-4 py-2.5 bg-slate-900/80 border-b border-slate-800">
                        <div className="flex items-center gap-2">
                          <span className="text-xs font-mono font-bold text-amber-400">
                            &lt;{tagName}&gt;
                          </span>
                          <span className="text-xs text-slate-400 font-sans">
                            {change.reason}
                          </span>
                        </div>

                        <div className="flex items-center gap-1.5">
                          <button
                            onClick={() => handleAcceptOne(change.section_id)}
                            className="flex items-center gap-1 px-2.5 py-1 text-xs font-semibold text-emerald-300 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/30 rounded-lg transition"
                          >
                            <Check className="w-3.5 h-3.5 text-emerald-400" />
                            <span>Accept</span>
                          </button>
                          <button
                            onClick={() => handleRejectOne(change.section_id)}
                            className="flex items-center gap-1 px-2.5 py-1 text-xs font-semibold text-slate-400 hover:text-red-400 hover:bg-red-500/10 border border-slate-700/60 rounded-lg transition"
                          >
                            <X className="w-3.5 h-3.5" />
                            <span>Reject</span>
                          </button>
                        </div>
                      </div>

                      {/* Diff View */}
                      <div className="p-3 font-mono text-xs max-h-56 overflow-y-auto space-y-0.5 bg-slate-950">
                        {diff ? (
                          diff.map((line, idx) => (
                            <div
                              key={idx}
                              className={`px-2 py-0.5 rounded flex items-start gap-2 ${
                                line.tag === 'Delete'
                                  ? 'bg-red-500/15 text-red-300'
                                  : line.tag === 'Insert'
                                  ? 'bg-emerald-500/15 text-emerald-300'
                                  : 'text-slate-400'
                              }`}
                            >
                              <span className="select-none font-bold opacity-60 w-3">
                                {line.tag === 'Delete' ? '-' : line.tag === 'Insert' ? '+' : ' '}
                              </span>
                              <span className="whitespace-pre-wrap break-all flex-1">
                                {line.text || ' '}
                              </span>
                            </div>
                          ))
                        ) : (
                          <div className="text-slate-500 italic py-2">Computing diff...</div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Suggested New Sections */}
          {hasSuggestions && (
            <div className="space-y-3">
              <h3 className="text-xs font-bold text-slate-300 uppercase tracking-wider flex items-center gap-2">
                <Sparkles className="w-3.5 h-3.5 text-purple-400" />
                <span>Suggested New Sections ({pending.suggested_sections.length})</span>
              </h3>

              <div className="space-y-3">
                {pending.suggested_sections.map((sug, idx) => (
                  <div
                    key={idx}
                    className="p-3.5 rounded-xl border border-purple-900/30 bg-purple-950/10 flex flex-col gap-2"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-mono font-bold text-purple-300">
                          &lt;{sug.tag}&gt;
                        </span>
                        <span className="text-xs text-purple-200/80 font-sans">
                          {sug.reason}
                        </span>
                      </div>
                      <button
                        onClick={() => handleAcceptSuggested(idx)}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-white bg-purple-600 hover:bg-purple-500 rounded-lg shadow-sm transition"
                      >
                        <PlusCircle className="w-3.5 h-3.5" />
                        <span>Add Section</span>
                      </button>
                    </div>
                    <pre className="font-mono text-xs text-slate-300 bg-slate-950/70 p-2.5 rounded-lg whitespace-pre-wrap leading-relaxed border border-purple-900/20">
                      {sug.content}
                    </pre>
                  </div>
                ))}
              </div>
            </div>
          )}

          {!hasChanges && !hasSuggestions && (
            <div className="py-8 text-center text-slate-400 text-xs">
              No further pending section changes to review.
            </div>
          )}
        </div>

        {/* Modal Bottom Actions */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-slate-800 bg-slate-950/80">
          <button
            onClick={onClose}
            className="px-4 py-2 text-xs font-medium text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition"
          >
            Close & Review Later
          </button>

          <div className="flex items-center gap-2">
            <button
              onClick={handleRejectAll}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold text-slate-400 hover:text-red-400 hover:bg-red-500/10 border border-slate-800 rounded-xl transition"
            >
              <Ban className="w-3.5 h-3.5" />
              <span>Reject All</span>
            </button>
            <button
              onClick={handleAcceptAll}
              className="flex items-center gap-1.5 px-5 py-2 text-xs font-bold text-slate-950 bg-emerald-400 hover:bg-emerald-300 rounded-xl shadow-lg shadow-emerald-500/20 transition"
            >
              <CheckCheck className="w-4 h-4" />
              <span>Accept All Changes</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
