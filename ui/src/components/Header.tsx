import React, { useState } from 'react';
import {
  Flame,
  FilePlus,
  FolderOpen,
  Save,
  Undo2,
  Redo2,
  Sparkles,
  Settings,
  Check,
  AlertCircle,
  Clock,
  BookOpen,
  Upload,
  Share2,
  Variable,
  History,
} from 'lucide-react';
import type { DocumentStateDto } from '../types';

interface HeaderProps {
  state: DocumentStateDto;
  onUpdateTitle: (title: string) => void;
  onUpdateDescription: (description: string) => void;
  onNew: () => void;
  onOpen: () => void;
  onSave: () => void;
  onSaveAs: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onOpenLibrary: () => void;
  onOpenImport: () => void;
  onOpenExportSkill: () => void;
  onOpenVariables: () => void;
  onOpenHistory: () => void;
  onOpenRefine: () => void;
  onOpenReview: () => void;
  onOpenSettings: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  state,
  onUpdateTitle,
  onUpdateDescription,
  onNew,
  onOpen,
  onSave,
  onSaveAs,
  onUndo,
  onRedo,
  onOpenLibrary,
  onOpenImport,
  onOpenExportSkill,
  onOpenVariables,
  onOpenHistory,
  onOpenRefine,
  onOpenReview,
  onOpenSettings,
}) => {
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [titleInput, setTitleInput] = useState(state.document.title);
  const [isEditingDesc, setIsEditingDesc] = useState(false);
  const [descInput, setDescInput] = useState(state.document.description);

  // Sync inputs if document changes externally
  React.useEffect(() => {
    setTitleInput(state.document.title);
  }, [state.document.title]);

  React.useEffect(() => {
    setDescInput(state.document.description);
  }, [state.document.description]);

  const handleTitleSubmit = () => {
    setIsEditingTitle(false);
    if (titleInput.trim() && titleInput !== state.document.title) {
      onUpdateTitle(titleInput.trim());
    } else {
      setTitleInput(state.document.title);
    }
  };

  const handleDescSubmit = () => {
    setIsEditingDesc(false);
    if (descInput !== state.document.description) {
      onUpdateDescription(descInput);
    }
  };

  const pendingCount =
    (state.pending_refinements?.changes.length || 0) +
    (state.pending_refinements?.suggested_sections.length || 0);

  const scenarioCount = state.document.scenarios?.length || 0;
  const snapshotCount = state.document.snapshots?.length || 0;

  return (
    <header className="h-14 border-b border-slate-800 bg-slate-900/95 backdrop-blur-md px-4 flex items-center justify-between select-none z-20 shrink-0">
      {/* Left: Branding & Document Info */}
      <div className="flex items-center gap-3 min-w-0">
        <div className="flex items-center gap-2 pr-3 border-r border-slate-800">
          <div className="w-7 h-7 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-amber-400 shadow-sm">
            <Flame className="w-3.5 h-3.5" />
          </div>
          <span className="font-bold tracking-tight text-xs text-slate-100 hidden sm:inline uppercase">
            Prompt<span className="text-amber-400">Forge</span>
          </span>
        </div>

        {/* Title & Description inline editors */}
        <div className="flex flex-col min-w-0">
          <div className="flex items-center gap-2">
            {isEditingTitle ? (
              <input
                type="text"
                value={titleInput}
                onChange={(e) => setTitleInput(e.target.value)}
                onBlur={handleTitleSubmit}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleTitleSubmit();
                  if (e.key === 'Escape') {
                    setTitleInput(state.document.title);
                    setIsEditingTitle(false);
                  }
                }}
                autoFocus
                className="bg-slate-800 text-sm font-semibold text-white px-2 py-0.5 rounded outline-none border border-amber-500/50 min-w-[200px]"
              />
            ) : (
              <button
                onClick={() => setIsEditingTitle(true)}
                className="text-sm font-semibold text-slate-200 hover:text-white hover:bg-slate-800/60 px-1.5 py-0.5 rounded transition-colors truncate max-w-xs sm:max-w-md text-left"
                title="Click to rename document"
              >
                {state.document.title || 'Untitled Prompt'}
              </button>
            )}

            {/* Dirty badge */}
            {state.dirty ? (
              <span
                className="flex items-center gap-1 text-[11px] font-medium text-amber-400 bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.5 rounded-full"
                title="Document has unsaved modifications"
              >
                <Clock className="w-3 h-3" />
                Unsaved
              </span>
            ) : (
              <span
                className="flex items-center gap-1 text-[11px] font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5 rounded-full"
                title="Saved to disk"
              >
                <Check className="w-3 h-3" />
                Saved
              </span>
            )}
          </div>

          {/* Description line */}
          <div className="text-[11px] text-slate-400 truncate max-w-sm sm:max-w-lg">
            {isEditingDesc ? (
              <input
                type="text"
                value={descInput}
                onChange={(e) => setDescInput(e.target.value)}
                onBlur={handleDescSubmit}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleDescSubmit();
                  if (e.key === 'Escape') {
                    setDescInput(state.document.description);
                    setIsEditingDesc(false);
                  }
                }}
                autoFocus
                className="bg-slate-800 text-[11px] text-slate-200 px-1.5 py-0.2 rounded outline-none border border-slate-600 w-full"
              />
            ) : (
              <button
                onClick={() => setIsEditingDesc(true)}
                className="hover:text-slate-200 transition-colors text-left truncate w-full"
                title="Click to edit document description"
              >
                {state.document.description || 'Add prompt description or objective...'}
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Right: Action Buttons */}
      <div className="flex items-center gap-1.5">
        {/* Undo / Redo */}
        <div className="flex items-center bg-slate-800/60 rounded-lg p-0.5 border border-slate-700/50">
          <button
            onClick={onUndo}
            disabled={!state.can_undo}
            className="p-1.5 rounded text-slate-400 hover:text-slate-100 disabled:opacity-30 disabled:pointer-events-none hover:bg-slate-700 transition"
            title="Undo (Ctrl+Z)"
          >
            <Undo2 className="w-4 h-4" />
          </button>
          <button
            onClick={onRedo}
            disabled={!state.can_redo}
            className="p-1.5 rounded text-slate-400 hover:text-slate-100 disabled:opacity-30 disabled:pointer-events-none hover:bg-slate-700 transition"
            title="Redo (Ctrl+Y)"
          >
            <Redo2 className="w-4 h-4" />
          </button>
        </div>

        {/* File actions */}
        <button
          onClick={onNew}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="New Prompt Document (Ctrl+N)"
        >
          <FilePlus className="w-3.5 h-3.5 text-slate-400" />
          <span className="hidden md:inline">New</span>
        </button>

        <button
          onClick={onOpen}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Open Document (Ctrl+O)"
        >
          <FolderOpen className="w-3.5 h-3.5 text-slate-400" />
          <span className="hidden md:inline">Open</span>
        </button>

        <div className="flex items-center bg-slate-800 rounded-lg border border-slate-700/60 overflow-hidden">
          <button
            onClick={onSave}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-700 transition"
            title="Save Document (Ctrl+S)"
          >
            <Save className="w-3.5 h-3.5 text-amber-400" />
            <span className="hidden md:inline">Save</span>
          </button>
          <div className="w-[1px] h-4 bg-slate-700" />
          <button
            onClick={onSaveAs}
            className="px-1.5 py-1.5 text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-700 transition"
            title="Save As..."
          >
            <span className="text-[10px]">As</span>
          </button>
        </div>

        <button
          onClick={onOpenLibrary}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Open Prompt Library"
        >
          <BookOpen className="w-3.5 h-3.5 text-emerald-400" />
          <span className="hidden md:inline">Library</span>
        </button>

        <button
          onClick={onOpenImport}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Import Prompt (XML, Markdown, JSON, Skill)"
        >
          <Upload className="w-3.5 h-3.5 text-blue-400" />
          <span className="hidden md:inline">Import</span>
        </button>

        <button
          onClick={onOpenExportSkill}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Export prompt as Agent Skill (SKILL.md)"
        >
          <Share2 className="w-3.5 h-3.5 text-violet-400" />
          <span className="hidden md:inline">Skill</span>
        </button>

        <button
          onClick={onOpenVariables}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Dynamic Variables & Scenario Matrix (Interpolation & Telemetry)"
        >
          <Variable className="w-3.5 h-3.5 text-cyan-400" />
          <span className="hidden md:inline">Variables</span>
          {scenarioCount > 0 && (
            <span className="ml-0.5 px-1.5 py-0.2 text-[10px] bg-slate-700 text-slate-300 rounded-full font-mono">
              {scenarioCount}
            </span>
          )}
        </button>

        <button
          onClick={onOpenHistory}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700/60 transition"
          title="Prompt Revision History & Visual Diff Branching"
        >
          <History className="w-3.5 h-3.5 text-amber-400" />
          <span className="hidden md:inline">History</span>
          {snapshotCount > 0 && (
            <span className="ml-0.5 px-1.5 py-0.2 text-[10px] bg-slate-700 text-slate-300 rounded-full font-mono">
              {snapshotCount}
            </span>
          )}
        </button>

        {/* Review Changes button (if pending) */}
        {pendingCount > 0 && (
          <button
            onClick={onOpenReview}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-amber-300 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 rounded-lg transition"
            title="Review pending refinement modifications"
          >
            <AlertCircle className="w-3.5 h-3.5 text-amber-400" />
            <span>Review Changes ({pendingCount})</span>
          </button>
        )}

        {/* Refine button */}
        <button
          onClick={onOpenRefine}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-950 bg-amber-400 hover:bg-amber-300 rounded-lg shadow-sm transition active:scale-95"
          title="Refine and optimize prompt"
        >
          <Sparkles className="w-3.5 h-3.5 text-slate-950" />
          <span>Refine</span>
        </button>

        {/* Settings button */}
        <button
          onClick={onOpenSettings}
          className="p-1.5 text-slate-400 hover:text-slate-100 hover:bg-slate-800 rounded-lg border border-transparent hover:border-slate-700 transition"
          title="Workbench Settings"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
