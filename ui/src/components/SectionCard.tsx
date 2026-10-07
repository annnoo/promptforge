import React, { useState } from 'react';
import {
  GripVertical,
  ChevronUp,
  ChevronDown,
  Lock,
  Unlock,
  Eye,
  EyeOff,
  Copy,
  Trash2,
  GitCompare,
  RotateCcw,
  Edit2,
  Plus,
  CheckCircle2,
} from 'lucide-react';
import type { PromptSection, DiffLine } from '../types';
import { api } from '../api';
import { TagBadge } from './TagBadge';

interface SectionCardProps {
  section: PromptSection;
  index: number;
  totalSections: number;
  hasPendingChange: boolean;
  customColors?: Record<string, string>;
  onTagColorChange?: (tag: string, color: string) => void;
  onUpdateBrief: (id: string, text: string) => void;
  onRenameTag: (id: string, tag: string) => void;
  onUpdateTags: (id: string, tags: string[]) => void;
  onToggleLock: (id: string) => void;
  onToggleEnabled: (id: string) => void;
  onMoveUp: (id: string) => void;
  onMoveDown: (id: string) => void;
  onDuplicate: (id: string) => void;
  onRemove: (id: string) => void;
  onClearRefinement: (id: string) => void;
}

export const SectionCard: React.FC<SectionCardProps> = ({
  section,
  index,
  totalSections,
  hasPendingChange,
  customColors,
  onTagColorChange,
  onUpdateBrief,
  onRenameTag,
  onUpdateTags,
  onToggleLock,
  onToggleEnabled,
  onMoveUp,
  onMoveDown,
  onDuplicate,
  onRemove,
  onClearRefinement,
}) => {
  const [isEditingTag, setIsEditingTag] = useState(false);
  const [tagInput, setTagInput] = useState(section.tag);
  const [activeTab, setActiveTab] = useState<'brief' | 'refined' | 'diff'>('brief');
  const [diffLines, setDiffLines] = useState<DiffLine[] | null>(null);
  const [isLoadingDiff, setIsLoadingDiff] = useState(false);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [newTagInput, setNewTagInput] = useState('');

  // Sync tag if section changes
  React.useEffect(() => {
    setTagInput(section.tag);
  }, [section.tag]);

  const handleTagSubmit = () => {
    setIsEditingTag(false);
    const cleaned = tagInput.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '_');
    if (cleaned && cleaned !== section.tag) {
      onRenameTag(section.id, cleaned);
    } else {
      setTagInput(section.tag);
    }
  };

  const handleAddTagSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const cleaned = newTagInput.trim().toLowerCase().replace(/^#/, '');
    if (cleaned) {
      const current = section.tags || [];
      if (!current.includes(cleaned)) {
        onUpdateTags(section.id, [...current, cleaned]);
      }
    }
    setNewTagInput('');
    setIsAddingTag(false);
  };

  const handleRemoveTag = (tToRemove: string) => {
    const current = section.tags || [];
    onUpdateTags(
      section.id,
      current.filter((t) => t !== tToRemove)
    );
  };

  const handleLoadDiff = async () => {
    setActiveTab('diff');
    if (section.refined) {
      setIsLoadingDiff(true);
      try {
        const diff = await api.computeDiff(section.brief, section.refined);
        setDiffLines(diff);
      } catch (err) {
        console.error('Failed to compute diff:', err);
      } finally {
        setIsLoadingDiff(false);
      }
    }
  };

  // Metrics
  const currentText = activeTab === 'refined' && section.refined ? section.refined : section.brief;
  const charCount = currentText.length;
  const wordCount = currentText.trim() ? currentText.trim().split(/\s+/).length : 0;

  return (
    <div
      id={`section-${section.id}`}
      className={`rounded-xl border transition-all duration-150 ${
        !section.enabled
          ? 'bg-slate-900/40 border-slate-800/60 opacity-60'
          : hasPendingChange
          ? 'bg-slate-900/95 border-amber-500/50 shadow-md shadow-amber-500/5'
          : 'bg-slate-900/90 border-slate-800 hover:border-slate-700 shadow-sm'
      }`}
    >
      {/* Card Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800 bg-slate-950/60 rounded-t-xl">
        {/* Left: Drag Handle, Number, Tag, and Section Labels */}
        <div className="flex items-center gap-2 min-w-0 flex-wrap">
          <span className="text-slate-600 hover:text-slate-400 cursor-grab shrink-0">
            <GripVertical className="w-4 h-4" />
          </span>

          <span className="text-[11px] font-mono text-slate-500 font-semibold min-w-4 shrink-0">
            #{index + 1}
          </span>

          {/* Tag Name Badge */}
          {isEditingTag ? (
            <div className="flex items-center bg-slate-900 border border-slate-600 rounded px-1.5 py-0.5">
              <span className="text-xs font-mono text-slate-500">&lt;</span>
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onBlur={handleTagSubmit}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleTagSubmit();
                  if (e.key === 'Escape') {
                    setTagInput(section.tag);
                    setIsEditingTag(false);
                  }
                }}
                autoFocus
                className="bg-transparent text-xs font-mono font-bold text-amber-400 outline-none w-28"
              />
              <span className="text-xs font-mono text-slate-500">&gt;</span>
            </div>
          ) : (
            <button
              onClick={() => !section.locked && setIsEditingTag(true)}
              disabled={section.locked}
              className="flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-slate-800/80 hover:bg-slate-800 text-slate-200 border border-slate-700 transition group"
              title={section.locked ? 'Section is locked' : 'Click to rename XML tag'}
            >
              <span className="text-xs font-mono font-semibold text-amber-400">&lt;{section.tag}&gt;</span>
              {!section.locked && (
                <Edit2 className="w-2.5 h-2.5 opacity-0 group-hover:opacity-100 text-slate-400 transition-opacity" />
              )}
            </button>
          )}

          {/* Tags Chips Bar */}
          <div className="flex items-center gap-1 flex-wrap">
            {section.tags &&
              section.tags.map((t) => (
                <TagBadge
                  key={t}
                  tag={t}
                  size="xs"
                  customColors={customColors}
                  onRemove={!section.locked ? () => handleRemoveTag(t) : undefined}
                  onColorChange={
                    onTagColorChange
                      ? (colorId) => onTagColorChange(t, colorId)
                      : undefined
                  }
                />
              ))}

            {/* Add tag button / input */}
            {!section.locked && (
              isAddingTag ? (
                <form onSubmit={handleAddTagSubmit} className="flex items-center">
                  <input
                    type="text"
                    placeholder="tag"
                    value={newTagInput}
                    onChange={(e) => setNewTagInput(e.target.value)}
                    onBlur={() => {
                      if (!newTagInput.trim()) setIsAddingTag(false);
                    }}
                    autoFocus
                    className="bg-slate-950 border border-slate-700 rounded px-1.5 py-0.2 text-[10px] text-slate-200 outline-none w-16"
                  />
                </form>
              ) : (
                <button
                  onClick={() => setIsAddingTag(true)}
                  className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-slate-800/50 hover:bg-slate-800 text-slate-500 hover:text-slate-300 text-[10px] border border-dashed border-slate-700 transition"
                  title="Add tag to section"
                >
                  <Plus className="w-2.5 h-2.5" />
                  <span>tag</span>
                </button>
              )
            )}
          </div>

          {/* Status Badges */}
          {section.refined && (
            <span className="flex items-center gap-1 text-[10px] font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 rounded-md">
              <CheckCircle2 className="w-3 h-3" />
              Refined
            </span>
          )}

          {hasPendingChange && (
            <span className="flex items-center gap-1 text-[10px] font-semibold text-amber-300 bg-amber-500/10 border border-amber-500/30 px-2 py-0.5 rounded-md">
              Pending Change
            </span>
          )}
        </div>

        {/* Right: Actions Toolbar */}
        <div className="flex items-center gap-1 shrink-0">
          {/* Section view tabs if refined exists */}
          {section.refined && (
            <div className="flex items-center bg-slate-900 border border-slate-800 rounded-lg p-0.5 mr-2">
              <button
                onClick={() => setActiveTab('brief')}
                className={`px-2 py-0.5 text-[11px] font-medium rounded transition ${
                  activeTab === 'brief'
                    ? 'bg-slate-800 text-slate-100 font-semibold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                Draft
              </button>
              <button
                onClick={() => setActiveTab('refined')}
                className={`px-2 py-0.5 text-[11px] font-medium rounded transition ${
                  activeTab === 'refined'
                    ? 'bg-emerald-950 text-emerald-200 border border-emerald-500/30 font-semibold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                Refined
              </button>
              <button
                onClick={handleLoadDiff}
                className={`px-2 py-0.5 text-[11px] font-medium rounded transition flex items-center gap-1 ${
                  activeTab === 'diff'
                    ? 'bg-slate-800 text-amber-400 font-semibold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                <GitCompare className="w-3 h-3" />
                Diff
              </button>
            </div>
          )}

          {/* Enabled Toggle */}
          <button
            onClick={() => onToggleEnabled(section.id)}
            className={`p-1.5 rounded-md transition ${
              section.enabled
                ? 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
                : 'text-slate-600 bg-slate-950 hover:text-slate-400'
            }`}
            title={section.enabled ? 'Disable section (exclude from XML)' : 'Enable section'}
          >
            {section.enabled ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
          </button>

          {/* Lock Toggle */}
          <button
            onClick={() => onToggleLock(section.id)}
            className={`p-1.5 rounded-md transition ${
              section.locked
                ? 'text-amber-400 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
            }`}
            title={section.locked ? 'Unlock section' : 'Lock section (protect from modification)'}
          >
            {section.locked ? <Lock className="w-3.5 h-3.5" /> : <Unlock className="w-3.5 h-3.5" />}
          </button>

          {/* Move Up / Down */}
          <button
            onClick={() => onMoveUp(section.id)}
            disabled={index === 0}
            className="p-1.5 rounded-md text-slate-400 hover:text-slate-200 hover:bg-slate-800 disabled:opacity-20 disabled:pointer-events-none transition"
            title="Move Section Up"
          >
            <ChevronUp className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => onMoveDown(section.id)}
            disabled={index === totalSections - 1}
            className="p-1.5 rounded-md text-slate-400 hover:text-slate-200 hover:bg-slate-800 disabled:opacity-20 disabled:pointer-events-none transition"
            title="Move Section Down"
          >
            <ChevronDown className="w-3.5 h-3.5" />
          </button>

          {/* Duplicate */}
          <button
            onClick={() => onDuplicate(section.id)}
            className="p-1.5 rounded-md text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition"
            title="Duplicate Section"
          >
            <Copy className="w-3.5 h-3.5" />
          </button>

          {/* Delete */}
          <button
            onClick={() => onRemove(section.id)}
            className="p-1.5 rounded-md text-slate-500 hover:text-red-400 hover:bg-red-500/10 transition"
            title="Delete Section"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Card Content Area */}
      <div className="p-3.5">
        {/* VIEW 1: BRIEF EDITOR */}
        {activeTab === 'brief' && (
          <div>
            <textarea
              value={section.brief}
              onChange={(e) => onUpdateBrief(section.id, e.target.value)}
              disabled={section.locked}
              rows={Math.min(10, Math.max(3, section.brief.split('\n').length))}
              placeholder="Enter instructions, notes, constraints, or examples for this section..."
              className="w-full bg-slate-950/70 border border-slate-800 rounded-lg p-3 text-xs font-mono text-slate-200 placeholder-slate-600 outline-none focus:border-slate-700 resize-y leading-relaxed disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>
        )}

        {/* VIEW 2: REFINED VIEW */}
        {activeTab === 'refined' && section.refined && (
          <div className="space-y-2">
            <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-200 font-mono whitespace-pre-wrap leading-relaxed">
              {section.refined}
            </div>
            <div className="flex items-center justify-between pt-1">
              <span className="text-[11px] text-emerald-400 font-medium flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" />
                Active in Final XML output
              </span>
              <button
                onClick={() => onClearRefinement(section.id)}
                className="flex items-center gap-1 text-[11px] text-slate-400 hover:text-red-400 hover:bg-slate-800 px-2 py-1 rounded transition"
                title="Discard refined version and revert to original draft brief"
              >
                <RotateCcw className="w-3 h-3" />
                <span>Revert to Draft</span>
              </button>
            </div>
          </div>
        )}

        {/* VIEW 3: DIFF VIEW */}
        {activeTab === 'diff' && (
          <div className="space-y-2">
            {isLoadingDiff ? (
              <div className="py-6 text-center text-xs text-slate-400">Computing diff...</div>
            ) : diffLines ? (
              <div className="rounded-lg border border-slate-800 bg-slate-950 font-mono text-xs overflow-hidden">
                <div className="px-3 py-1.5 bg-slate-900 border-b border-slate-800 text-[10px] text-slate-400 flex items-center justify-between">
                  <span>Line Diff (Draft Brief → Refined)</span>
                  <div className="flex gap-2">
                    <span className="text-red-400">- Removed</span>
                    <span className="text-emerald-400">+ Added</span>
                  </div>
                </div>
                <div className="p-2 space-y-0.5 max-h-64 overflow-y-auto">
                  {diffLines.map((line, idx) => (
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
                      <span className="whitespace-pre-wrap break-all flex-1">{line.text || ' '}</span>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        )}

        {/* Card Footer: Metrics & Details */}
        <div className="flex items-center justify-between mt-2 pt-2 border-t border-slate-800/60 text-[11px] text-slate-500">
          <div className="flex items-center gap-3 font-mono">
            <span>{charCount} chars</span>
            <span>•</span>
            <span>{wordCount} words</span>
            <span>•</span>
            <span>~{Math.ceil(charCount / 4)} tokens</span>
          </div>

          <div className="flex items-center gap-2">
            {section.locked && (
              <span className="text-[10px] text-amber-400/80 font-medium">Locked</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
