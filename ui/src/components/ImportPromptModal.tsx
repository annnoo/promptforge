import React, { useState, useEffect } from 'react';
import { api } from '../api';
import type { DocumentStateDto, ImportPreviewDto } from '../types';
import { TagBadge } from './TagBadge';

interface ImportPromptModalProps {
  isOpen: boolean;
  onClose: () => void;
  onImportSuccess: (docState: DocumentStateDto) => void;
}

export const ImportPromptModal: React.FC<ImportPromptModalProps> = ({
  isOpen,
  onClose,
  onImportSuccess,
}) => {
  const [rawText, setRawText] = useState('');
  const [filePath, setFilePath] = useState<string | null>(null);
  const [importMode, setImportMode] = useState<'replace' | 'append'>('replace');
  const [preview, setPreview] = useState<ImportPreviewDto | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      setRawText('');
      setFilePath(null);
      setPreview(null);
      setError(null);
    }
  }, [isOpen]);

  // Live preview parsing on text change
  useEffect(() => {
    if (!rawText.trim()) {
      setPreview(null);
      setError(null);
      return;
    }

    const timer = setTimeout(() => {
      api
        .parseImportPreview(rawText)
        .then((res) => {
          setPreview(res);
          setError(null);
        })
        .catch((err) => {
          setPreview(null);
          setError(String(err));
        });
    }, 200);

    return () => clearTimeout(timer);
  }, [rawText]);

  if (!isOpen) return null;

  const handlePickFile = async () => {
    try {
      setError(null);
      const file = await api.pickImportFile();
      if (file) {
        setFilePath(file.path);
        setRawText(file.content);
      }
    } catch (err) {
      setError(`Failed to read file: ${err}`);
    }
  };

  const handleExecuteImport = async () => {
    if (!rawText.trim()) return;
    try {
      setIsLoading(true);
      setError(null);
      const newState = await api.importPromptContent(rawText, importMode);
      onImportSuccess(newState);
      onClose();
    } catch (err) {
      setError(`Import failed: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const formatColor: Record<string, string> = {
    json: 'text-amber-400 bg-amber-500/10 border-amber-500/30',
    xml: 'text-blue-400 bg-blue-500/10 border-blue-500/30',
    skill: 'text-violet-400 bg-violet-500/10 border-violet-500/30',
    markdown: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30',
    plain_text: 'text-zinc-400 bg-zinc-800 border-zinc-700',
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="bg-zinc-950 border border-zinc-800 rounded-xl max-w-2xl w-full flex flex-col max-h-[90vh] shadow-2xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-800/80 flex justify-between items-center bg-zinc-900/40">
          <div>
            <h2 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-blue-400" />
              Import Prompt
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Paste or open raw XML, Markdown, PromptForge JSON, or Agent Skill (<code className="text-zinc-300">SKILL.md</code>)
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto space-y-4 flex-1">
          {/* File Picker / Mode Toolbar */}
          <div className="flex flex-wrap items-center justify-between gap-3">
            <button
              type="button"
              onClick={handlePickFile}
              className="px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-200 border border-zinc-700/80 rounded text-xs font-medium transition-colors flex items-center gap-2 cursor-pointer"
            >
              <span>📂</span>
              {filePath ? 'Change File...' : 'Choose File to Import...'}
            </button>

            {filePath && (
              <span className="text-xs text-zinc-400 truncate max-w-xs font-mono">
                {filePath.split('/').pop()}
              </span>
            )}

            {/* Target Mode Segmented Control */}
            <div className="flex bg-zinc-900 border border-zinc-800 rounded p-0.5 text-xs">
              <button
                type="button"
                onClick={() => setImportMode('replace')}
                className={`px-3 py-1 font-medium rounded transition-colors ${
                  importMode === 'replace'
                    ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                New Document
              </button>
              <button
                type="button"
                onClick={() => setImportMode('append')}
                className={`px-3 py-1 font-medium rounded transition-colors ${
                  importMode === 'append'
                    ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                Append to Current
              </button>
            </div>
          </div>

          {/* Text Area */}
          <div>
            <div className="flex justify-between items-center mb-1">
              <label className="text-xs font-medium text-zinc-400">
                Prompt Content or Paste Buffer
              </label>
              {preview && (
                <span
                  className={`text-[10px] font-mono px-2 py-0.5 rounded border uppercase font-semibold ${
                    formatColor[preview.format] || formatColor.plain_text
                  }`}
                >
                  Detected: {preview.format}
                </span>
              )}
            </div>
            <textarea
              value={rawText}
              onChange={(e) => setRawText(e.target.value)}
              placeholder="Paste raw XML (<role>...</role>), Markdown (# Role\n...), PromptForge JSON, or SKILL.md content here..."
              rows={9}
              className="w-full px-3 py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 focus:outline-none focus:border-zinc-600 resize-none leading-relaxed"
            />
          </div>

          {/* Live Preview Card */}
          {preview && (
            <div className="p-3.5 bg-zinc-900/60 border border-zinc-800/80 rounded-lg space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-zinc-200">
                  {preview.title || 'Untitled Prompt'}
                </span>
                <span className="text-[11px] text-zinc-400 font-mono">
                  {preview.section_count} section{preview.section_count === 1 ? '' : 's'} parsed
                </span>
              </div>
              {preview.description && (
                <p className="text-xs text-zinc-400 line-clamp-2">
                  {preview.description}
                </p>
              )}
              <div className="flex flex-wrap gap-1.5 pt-1">
                {preview.sections.map((s, idx) => (
                  <TagBadge key={idx} tag={s.tag} size="xs" showHash={false} />
                ))}
              </div>
            </div>
          )}

          {/* Error Message */}
          {error && (
            <div className="p-2.5 rounded text-xs bg-rose-950/40 border border-rose-800/60 text-rose-300">
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-zinc-800/80 flex justify-end gap-2 bg-zinc-900/30">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 border border-zinc-800 rounded text-xs font-medium transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleExecuteImport}
            disabled={!preview || isLoading}
            className="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded text-xs font-medium transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <span>📥</span>
            {isLoading
              ? 'Importing...'
              : importMode === 'replace'
              ? 'Open as New Document'
              : `Append ${preview ? preview.section_count : ''} Sections`}
          </button>
        </div>
      </div>
    </div>
  );
};
