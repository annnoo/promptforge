import React, { useState, useEffect } from 'react';
import {
  Code,
  Copy,
  Check,
  Download,
  Sliders,
  Layers,
  Sparkles,
  Info,
} from 'lucide-react';
import type { PromptDocument } from '../types';
import { api } from '../api';

interface XmlPreviewPanelProps {
  document: PromptDocument;
  onExportToFile: (stage: 'draft' | 'final', clean: boolean) => void;
}

export const XmlPreviewPanel: React.FC<XmlPreviewPanelProps> = ({
  document,
  onExportToFile,
}) => {
  const [stage, setStage] = useState<'draft' | 'final'>('draft');
  const [clean, setClean] = useState(true);
  const [xmlContent, setXmlContent] = useState('');
  const [copied, setCopied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    let isCancelled = false;
    setIsLoading(true);
    api
      .renderXmlContent(stage, clean)
      .then((xml) => {
        if (!isCancelled) {
          setXmlContent(xml);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        console.error('Failed to render XML:', err);
        if (!isCancelled) setIsLoading(false);
      });

    return () => {
      isCancelled = true;
    };
  }, [document, stage, clean]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(xmlContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const enabledSections = document.sections.filter((s) => s.enabled);
  const refinedSections = document.sections.filter((s) => s.refined);
  const totalChars = xmlContent.length;
  const estTokens = Math.ceil(totalChars / 4);

  return (
    <aside className="w-96 border-l border-slate-800 bg-slate-900/60 flex flex-col h-full select-none shrink-0">
      {/* Panel Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-slate-800 bg-slate-900/90">
        <div className="flex items-center gap-2">
          <Code className="w-4 h-4 text-amber-400" />
          <span className="text-xs font-bold text-slate-100 uppercase tracking-wider">
            XML Output
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          {/* Copy Button */}
          <button
            onClick={handleCopy}
            className={`flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-lg transition ${
              copied
                ? 'bg-emerald-500 text-slate-950 shadow-sm'
                : 'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700'
            }`}
            title="Copy prompt XML to clipboard"
          >
            {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5 text-slate-400" />}
            <span>{copied ? 'Copied!' : 'Copy'}</span>
          </button>

          {/* Export Button */}
          <button
            onClick={() => onExportToFile(stage, clean)}
            className="p-1.5 text-slate-400 hover:text-slate-100 hover:bg-slate-800 rounded-lg border border-slate-700/60 transition"
            title="Export XML to File"
          >
            <Download className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Control Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 bg-slate-950/40 border-b border-slate-800/80 text-xs">
        {/* Stage Switcher */}
        <div className="flex items-center bg-slate-900 border border-slate-800 rounded-lg p-0.5">
          <button
            onClick={() => setStage('draft')}
            className={`px-2 py-0.5 rounded text-[11px] font-medium transition ${
              stage === 'draft'
                ? 'bg-slate-800 text-slate-100'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Draft
          </button>
          <button
            onClick={() => setStage('final')}
            className={`px-2 py-0.5 rounded text-[11px] font-medium transition flex items-center gap-1 ${
              stage === 'final'
                ? 'bg-purple-900/50 text-purple-200 border border-purple-500/30'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Sparkles className="w-3 h-3 text-purple-400" />
            <span>Final</span>
          </button>
        </div>

        {/* Clean Mode Toggle */}
        <button
          onClick={() => setClean(!clean)}
          className={`flex items-center gap-1 px-2 py-0.5 text-[11px] font-medium rounded-lg border transition ${
            clean
              ? 'bg-slate-800/80 border-slate-700 text-slate-300'
              : 'bg-amber-500/10 border-amber-500/30 text-amber-300'
          }`}
          title="Toggle inclusion of internal section IDs in XML tags"
        >
          <Sliders className="w-3 h-3" />
          <span>{clean ? 'Clean Tags' : 'With IDs'}</span>
        </button>
      </div>

      {/* XML Code Container */}
      <div className="flex-1 overflow-auto p-3 font-mono text-xs leading-relaxed bg-slate-950/90 text-slate-300 select-text">
        {isLoading ? (
          <div className="text-slate-500 italic py-4 text-center">Rendering XML...</div>
        ) : (
          <pre className="whitespace-pre-wrap break-all font-mono">
            {xmlContent || '<prompt>\n  <!-- No enabled sections -->\n</prompt>'}
          </pre>
        )}
      </div>

      {/* Metrics / Inspector Footer */}
      <div className="p-3 border-t border-slate-800 bg-slate-950/70 text-[11px] space-y-2">
        <div className="flex items-center justify-between text-slate-400">
          <div className="flex items-center gap-1.5">
            <Layers className="w-3.5 h-3.5 text-slate-500" />
            <span>Sections Active</span>
          </div>
          <span className="font-mono font-semibold text-slate-200">
            {enabledSections.length} / {document.sections.length}
          </span>
        </div>

        <div className="flex items-center justify-between text-slate-400">
          <span>Refined Sections</span>
          <span className="font-mono font-semibold text-purple-300">
            {refinedSections.length} of {document.sections.length}
          </span>
        </div>

        <div className="flex items-center justify-between text-slate-400">
          <span>Character Count</span>
          <span className="font-mono font-semibold text-slate-200">{totalChars}</span>
        </div>

        <div className="flex items-center justify-between text-slate-400">
          <span>Estimated Tokens</span>
          <span className="font-mono font-bold text-amber-400">~{estTokens}</span>
        </div>

        {stage === 'draft' && refinedSections.length > 0 && (
          <div className="p-2 rounded-lg bg-purple-950/30 border border-purple-800/40 text-[10px] text-purple-300 flex items-start gap-1.5">
            <Info className="w-3.5 h-3.5 shrink-0 mt-0.5 text-purple-400" />
            <span>
              You have {refinedSections.length} refined section(s). Switch to <strong>Final</strong> stage above to preview them.
            </span>
          </div>
        )}
      </div>
    </aside>
  );
};
