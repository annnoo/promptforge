import React, { useState, useEffect } from 'react';
import {
  X,
  Sparkles,
  ShieldCheck,
  Maximize2,
  FileSearch,
  Copy,
  Check,
  ArrowRight,
  Loader2,
  AlertTriangle,
  Bot,
  ClipboardPaste,
} from 'lucide-react';
import type { AppConfig, DocumentStateDto } from '../types';
import { api } from '../api';

interface RefineModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: AppConfig;
  onRefinementSuccess: (newState: DocumentStateDto) => void;
}

export const RefineModal: React.FC<RefineModalProps> = ({
  isOpen,
  onClose,
  config,
  onRefinementSuccess,
}) => {
  const [mode, setMode] = useState<'conservative' | 'expand' | 'critique'>('conservative');
  const [manualTab, setManualTab] = useState<'export' | 'import'>('export');
  const [manualPromptText, setManualPromptText] = useState('');
  const [manualResponseInput, setManualResponseInput] = useState('');
  const [isCopied, setIsCopied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Generate manual request whenever mode changes
  useEffect(() => {
    if (isOpen) {
      setErrorMessage(null);
      api
        .generateManualRequest(mode)
        .then((txt) => setManualPromptText(txt))
        .catch((err) => console.error('Error generating manual prompt:', err));
    }
  }, [isOpen, mode]);

  if (!isOpen) return null;

  const isAutomated = config.active_provider === 'openai_compatible';

  const handleRunAutomated = async () => {
    setIsLoading(true);
    setErrorMessage(null);
    try {
      const newState = await api.executeAutomatedRefinement(mode);
      onRefinementSuccess(newState);
      onClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopyManualPrompt = async () => {
    try {
      await navigator.clipboard.writeText(manualPromptText);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const handleApplyManualResponse = async () => {
    if (!manualResponseInput.trim()) return;
    setIsLoading(true);
    setErrorMessage(null);
    try {
      const newState = await api.applyManualResponse(manualResponseInput.trim());
      onRefinementSuccess(newState);
      onClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-2xl shadow-2xl flex flex-col max-h-[90vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Modal Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/60">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-slate-800 border border-slate-700 text-amber-400">
              <Sparkles className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100">Prompt Optimization & Refinement</h2>
              <p className="text-xs text-slate-400">
                Enhance prompt precision, eliminate ambiguity, and resolve structural gaps
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

        {/* Modal Body */}
        <div className="p-6 overflow-y-auto space-y-6">
          {/* Error message */}
          {errorMessage && (
            <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-xs flex items-start gap-2">
              <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-red-400" />
              <span>{errorMessage}</span>
            </div>
          )}

          {/* Step 1: Mode Selector */}
          <div>
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block mb-2">
              1. Choose Refinement Strategy
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {/* Conservative */}
              <button
                type="button"
                onClick={() => setMode('conservative')}
                className={`p-3 rounded-xl border text-left transition flex flex-col gap-1.5 ${
                  mode === 'conservative'
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-2">
                  <ShieldCheck
                    className={`w-4 h-4 ${
                      mode === 'conservative' ? 'text-amber-400' : 'text-slate-400'
                    }`}
                  />
                  <span className="text-xs font-bold text-slate-100">Conservative</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-normal">
                  Tightens language, removes ambiguity, fixes structure, preserves intent.
                </p>
              </button>

              {/* Expand */}
              <button
                type="button"
                onClick={() => setMode('expand')}
                className={`p-3 rounded-xl border text-left transition flex flex-col gap-1.5 ${
                  mode === 'expand'
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Maximize2
                    className={`w-4 h-4 ${mode === 'expand' ? 'text-amber-400' : 'text-slate-400'}`}
                  />
                  <span className="text-xs font-bold text-slate-100">Expand</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-normal">
                  Generates missing sections, adds edge cases, constraints, and rich examples.
                </p>
              </button>

              {/* Critique */}
              <button
                type="button"
                onClick={() => setMode('critique')}
                className={`p-3 rounded-xl border text-left transition flex flex-col gap-1.5 ${
                  mode === 'critique'
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-2">
                  <FileSearch
                    className={`w-4 h-4 ${mode === 'critique' ? 'text-amber-400' : 'text-slate-400'}`}
                  />
                  <span className="text-xs font-bold text-slate-100">Critique</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-normal">
                  Produces comprehensive critique, identifies risks, and suggests fixes.
                </p>
              </button>
            </div>
          </div>

          {/* Step 2: Provider Execution */}
          <div>
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block mb-2">
              2. Execution Method ({isAutomated ? 'Automated HTTP API' : 'Manual / Offline Mode'})
            </label>

            {isAutomated ? (
              <div className="p-4 rounded-xl bg-slate-800/50 border border-slate-700/60 space-y-4">
                <div className="flex items-center justify-between text-xs">
                  <div className="flex items-center gap-2">
                    <Bot className="w-4 h-4 text-amber-400" />
                    <span className="font-semibold text-slate-200">
                      Configured Model: {config.openai_compatible.model}
                    </span>
                  </div>
                  <span className="text-slate-400 font-mono text-[11px]">
                    {config.openai_compatible.base_url}
                  </span>
                </div>

                <p className="text-xs text-slate-400 leading-relaxed">
                  PromptForge will send your structured prompt document directly to the OpenAI-compatible endpoint using the API key in environment variable{' '}
                  <code className="text-amber-300 bg-slate-900 px-1 py-0.5 rounded border border-slate-700">
                    {config.openai_compatible.api_key_env_var}
                  </code>
                  .
                </p>

                <button
                  onClick={handleRunAutomated}
                  disabled={isLoading}
                  className="w-full flex items-center justify-center gap-2 py-2.5 bg-amber-400 hover:bg-amber-300 disabled:opacity-50 text-slate-950 font-bold text-xs rounded-xl shadow transition"
                >
                  {isLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>Refining prompt via LLM...</span>
                    </>
                  ) : (
                    <>
                      <Sparkles className="w-4 h-4" />
                      <span>Execute Automated Refinement</span>
                    </>
                  )}
                </button>
              </div>
            ) : (
              /* Manual Mode: Zero external setup required, completely local/offline */
              <div className="space-y-3">
                <div className="flex border-b border-slate-800 bg-slate-950/60 p-1 rounded-lg text-xs font-semibold gap-1">
                  <button
                    onClick={() => setManualTab('export')}
                    className={`flex-1 py-1.5 rounded-md flex items-center justify-center gap-1.5 transition ${
                      manualTab === 'export'
                        ? 'bg-slate-800 text-amber-400'
                        : 'text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <Copy className="w-3.5 h-3.5" />
                    <span>Step 1: Copy Instructions & Prompt</span>
                  </button>
                  <button
                    onClick={() => setManualTab('import')}
                    className={`flex-1 py-1.5 rounded-md flex items-center justify-center gap-1.5 transition ${
                      manualTab === 'import'
                        ? 'bg-slate-800 text-amber-400'
                        : 'text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <ClipboardPaste className="w-3.5 h-3.5" />
                    <span>Step 2: Paste LLM Response</span>
                  </button>
                </div>

                {manualTab === 'export' ? (
                  <div className="space-y-3">
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Copy the generated prompt packet below and paste it into ChatGPT, Claude, DeepSeek, or Gemini in your browser.
                    </p>
                    <div className="relative">
                      <textarea
                        readOnly
                        value={manualPromptText}
                        rows={7}
                        className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs font-mono text-slate-300 resize-none outline-none"
                      />
                      <button
                        onClick={handleCopyManualPrompt}
                        className="absolute right-3 top-3 flex items-center gap-1.5 px-3 py-1.5 bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-xs rounded-lg shadow transition"
                      >
                        {isCopied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                        <span>{isCopied ? 'Copied Packet!' : 'Copy to Clipboard'}</span>
                      </button>
                    </div>
                    <div className="flex justify-end">
                      <button
                        onClick={() => setManualTab('import')}
                        className="flex items-center gap-1.5 text-xs text-amber-400 hover:text-amber-300 font-semibold"
                      >
                        <span>Next: Paste LLM Response</span>
                        <ArrowRight className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-3">
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Paste the response or JSON changeset returned by the external LLM below:
                    </p>
                    <textarea
                      placeholder='Paste the model response or ```json { "changes": [...] } ``` here...'
                      value={manualResponseInput}
                      onChange={(e) => setManualResponseInput(e.target.value)}
                      rows={8}
                      className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs font-mono text-slate-200 placeholder-slate-600 outline-none focus:border-amber-500/50 resize-none"
                    />
                    <button
                      onClick={handleApplyManualResponse}
                      disabled={isLoading || !manualResponseInput.trim()}
                      className="w-full flex items-center justify-center gap-2 py-2.5 bg-amber-500 hover:bg-amber-400 disabled:opacity-40 text-slate-950 font-bold text-xs rounded-xl shadow transition"
                    >
                      {isLoading ? (
                        <>
                          <Loader2 className="w-4 h-4 animate-spin" />
                          <span>Validating & Parsing Response...</span>
                        </>
                      ) : (
                        <>
                          <Sparkles className="w-4 h-4" />
                          <span>Parse & Review Proposed Changes</span>
                        </>
                      )}
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
