import React, { useState, useEffect } from 'react';
import { api } from '../api';
import type { DocumentStateDto, SavedPromptSummary } from '../types';
import { TagBadge } from './TagBadge';

interface PromptLibraryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onLoadPrompt: (docState: DocumentStateDto) => void;
  onSaveCurrentToLibrary: () => Promise<void>;
  customColors?: Record<string, string>;
}

export const PromptLibraryModal: React.FC<PromptLibraryModalProps> = ({
  isOpen,
  onClose,
  onLoadPrompt,
  onSaveCurrentToLibrary,
  customColors,
}) => {
  const [prompts, setPrompts] = useState<SavedPromptSummary[]>([]);
  const [search, setSearch] = useState('');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSavingCurrent, setIsSavingCurrent] = useState(false);
  const [statusMsg, setStatusMsg] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const fetchPrompts = async () => {
    try {
      setIsLoading(true);
      const list = await api.listLibraryPrompts();
      setPrompts(list);
    } catch (err) {
      console.error('Failed to load prompts library:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      fetchPrompts();
      setSearch('');
      setSelectedTag(null);
      setStatusMsg(null);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  // Gather unique tags across all library prompts
  const allTags = Array.from(new Set(prompts.flatMap((p) => p.tags))).sort();

  const filteredPrompts = prompts.filter((p) => {
    const matchesSearch =
      !search ||
      p.title.toLowerCase().includes(search.toLowerCase()) ||
      p.description.toLowerCase().includes(search.toLowerCase()) ||
      p.tags.some((t) => t.toLowerCase().includes(search.toLowerCase()));

    const matchesTag = !selectedTag || p.tags.includes(selectedTag);
    return matchesSearch && matchesTag;
  });

  const handleSaveCurrent = async () => {
    try {
      setIsSavingCurrent(true);
      setStatusMsg(null);
      await onSaveCurrentToLibrary();
      await fetchPrompts();
      setStatusMsg({ text: 'Current prompt saved to Library!', type: 'success' });
    } catch (err) {
      setStatusMsg({ text: `Failed to save: ${err}`, type: 'error' });
    } finally {
      setIsSavingCurrent(false);
    }
  };

  const handleLoad = async (id: string) => {
    try {
      const docState = await api.loadFromLibrary(id);
      onLoadPrompt(docState);
      onClose();
    } catch (err) {
      setStatusMsg({ text: `Failed to open prompt: ${err}`, type: 'error' });
    }
  };

  const handleDelete = async (id: string, title: string) => {
    if (!confirm(`Delete "${title}" from your prompt library?`)) return;
    try {
      await api.deleteFromLibrary(id);
      setPrompts((prev) => prev.filter((p) => p.id !== id));
      setStatusMsg({ text: `Deleted "${title}"`, type: 'success' });
    } catch (err) {
      setStatusMsg({ text: `Failed to delete: ${err}`, type: 'error' });
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="bg-zinc-950 border border-zinc-800 rounded-xl max-w-3xl w-full flex flex-col max-h-[90vh] shadow-2xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-800/80 flex justify-between items-center bg-zinc-900/40">
          <div>
            <h2 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
              Prompt Library
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Saved reusable prompts in your local PromptForge workspace.
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
          >
            ✕
          </button>
        </div>

        {/* Toolbar */}
        <div className="px-6 py-3 border-b border-zinc-800/60 bg-zinc-900/20 flex flex-wrap items-center justify-between gap-3">
          <div className="flex-1 min-w-[200px]">
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search library prompts by name, description, or tag..."
              className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded text-xs text-zinc-200 focus:outline-none focus:border-zinc-600"
            />
          </div>

          <button
            type="button"
            onClick={handleSaveCurrent}
            disabled={isSavingCurrent}
            className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-xs font-medium transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
          >
            <span>💾</span>
            {isSavingCurrent ? 'Saving...' : 'Save Active Prompt to Library'}
          </button>
        </div>

        {/* Tag Filters */}
        {allTags.length > 0 && (
          <div className="px-6 py-2 border-b border-zinc-800/40 bg-zinc-950 flex flex-wrap items-center gap-1.5 overflow-x-auto">
            <span className="text-[11px] text-zinc-500 mr-1">Filter:</span>
            <button
              onClick={() => setSelectedTag(null)}
              className={`px-2 py-0.5 text-xs rounded border transition-colors cursor-pointer ${
                selectedTag === null
                  ? 'bg-zinc-800 text-zinc-200 border-zinc-600'
                  : 'bg-zinc-900 text-zinc-400 border-zinc-800 hover:border-zinc-700'
              }`}
            >
              All ({prompts.length})
            </button>
            {allTags.map((tag) => (
              <TagBadge
                key={tag}
                tag={tag}
                size="xs"
                customColors={customColors}
                isActive={selectedTag === tag}
                onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
              />
            ))}
          </div>
        )}

        {/* Status Notification */}
        {statusMsg && (
          <div
            className={`mx-6 mt-3 p-2.5 rounded text-xs border ${
              statusMsg.type === 'success'
                ? 'bg-emerald-950/40 border-emerald-800/60 text-emerald-300'
                : 'bg-rose-950/40 border-rose-800/60 text-rose-300'
            }`}
          >
            {statusMsg.text}
          </div>
        )}

        {/* Prompt List */}
        <div className="p-6 overflow-y-auto space-y-3 flex-1">
          {isLoading ? (
            <div className="py-12 text-center text-xs text-zinc-500">
              Loading prompt library...
            </div>
          ) : filteredPrompts.length === 0 ? (
            <div className="py-12 text-center border border-dashed border-zinc-800 rounded-lg p-6">
              <span className="text-2xl mb-2 block">📚</span>
              <p className="text-sm font-medium text-zinc-300">
                {search || selectedTag ? 'No matching prompts found' : 'No prompts in library yet'}
              </p>
              <p className="text-xs text-zinc-500 mt-1 max-w-sm mx-auto">
                {search || selectedTag
                  ? 'Try clearing the search query or tag filter.'
                  : 'Click "Save Active Prompt to Library" above to save your structured prompt for quick reuse.'}
              </p>
            </div>
          ) : (
            filteredPrompts.map((prompt) => (
              <div
                key={prompt.id}
                className="p-4 bg-zinc-900/40 border border-zinc-800/80 hover:border-zinc-700 rounded-xl transition-all flex flex-col sm:flex-row sm:items-center justify-between gap-4 group"
              >
                <div className="space-y-1.5 flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-medium text-zinc-200 truncate">
                      {prompt.title}
                    </h3>
                    <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700">
                      {prompt.section_count} sections
                    </span>
                  </div>

                  {prompt.description && (
                    <p className="text-xs text-zinc-400 line-clamp-1">
                      {prompt.description}
                    </p>
                  )}

                  {prompt.tags.length > 0 && (
                    <div className="flex flex-wrap gap-1 pt-1">
                      {prompt.tags.map((tag) => (
                        <TagBadge
                          key={tag}
                          tag={tag}
                          size="xs"
                          customColors={customColors}
                        />
                      ))}
                    </div>
                  )}
                </div>

                <div className="flex items-center gap-2 shrink-0">
                  <button
                    type="button"
                    onClick={() => handleLoad(prompt.id)}
                    className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 rounded text-xs font-medium transition-colors cursor-pointer"
                  >
                    Open
                  </button>
                  <button
                    type="button"
                    onClick={() => handleDelete(prompt.id, prompt.title)}
                    className="px-2 py-1.5 text-zinc-500 hover:text-rose-400 hover:bg-rose-950/20 rounded text-xs transition-colors cursor-pointer"
                    title="Delete prompt"
                  >
                    🗑
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-zinc-800/80 flex justify-between items-center bg-zinc-900/30 text-xs text-zinc-500">
          <span>{filteredPrompts.length} of {prompts.length} prompts</span>
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 border border-zinc-800 rounded text-xs font-medium transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
