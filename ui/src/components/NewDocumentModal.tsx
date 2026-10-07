import React, { useState } from 'react';
import { X, FilePlus, Sparkles, LayoutTemplate, Check } from 'lucide-react';
import type { StarterTemplate } from '../types';

interface NewDocumentModalProps {
  isOpen: boolean;
  onClose: () => void;
  templates: StarterTemplate[];
  onCreate: (templateId?: string, title?: string) => void;
}

export const NewDocumentModal: React.FC<NewDocumentModalProps> = ({
  isOpen,
  onClose,
  templates,
  onCreate,
}) => {
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null);
  const [title, setTitle] = useState('');

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate(selectedTemplate || undefined, title.trim() || undefined);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl shadow-2xl flex flex-col max-h-[90vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/40">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-amber-500/10 text-amber-400">
              <FilePlus className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100">New Prompt Document</h2>
              <p className="text-xs text-slate-400">Start from a blank canvas or an architectural starter</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Body */}
        <form onSubmit={handleSubmit} className="p-6 overflow-y-auto space-y-5">
          {/* Title Input */}
          <div className="space-y-1">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block">
              Document Title
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g. Distributed Consensus Engine Prompt"
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-semibold text-slate-100 outline-none focus:border-amber-500/50"
              autoFocus
            />
          </div>

          {/* Template Selection */}
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block">
              Choose Foundation
            </label>

            <div className="space-y-2">
              {/* Blank Option */}
              <div
                onClick={() => setSelectedTemplate(null)}
                className={`p-3 rounded-xl border cursor-pointer transition flex items-center justify-between ${
                  selectedTemplate === null
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-slate-800 text-slate-300">
                    <FilePlus className="w-4 h-4 text-amber-400" />
                  </div>
                  <div>
                    <h3 className="text-xs font-bold text-slate-100">Blank Document</h3>
                    <p className="text-[11px] text-slate-400">
                      Empty document ready for your custom XML structure
                    </p>
                  </div>
                </div>
                {selectedTemplate === null && <Check className="w-4 h-4 text-amber-400" />}
              </div>

              {/* Starter Templates */}
              {templates.map((tmpl) => (
                <div
                  key={tmpl.id}
                  onClick={() => setSelectedTemplate(tmpl.id)}
                  className={`p-3 rounded-xl border cursor-pointer transition flex items-center justify-between ${
                    selectedTemplate === tmpl.id
                      ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                      : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-slate-800 text-slate-300">
                      <LayoutTemplate className="w-4 h-4 text-indigo-400" />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="text-xs font-bold text-slate-100">{tmpl.name}</h3>
                        <span className="text-[10px] font-mono bg-slate-900 px-1.5 py-0.2 rounded text-slate-400 border border-slate-800">
                          {tmpl.presets.length} sections
                        </span>
                      </div>
                      <p className="text-[11px] text-slate-400 leading-tight mt-0.5">
                        {tmpl.description}
                      </p>
                    </div>
                  </div>
                  {selectedTemplate === tmpl.id && <Check className="w-4 h-4 text-amber-400" />}
                </div>
              ))}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-800">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-xs font-medium text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="flex items-center gap-1.5 px-5 py-2 text-xs font-bold text-slate-950 bg-amber-400 hover:bg-amber-300 rounded-xl shadow-lg shadow-amber-500/20 transition"
            >
              <Sparkles className="w-4 h-4" />
              <span>Create Document</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
