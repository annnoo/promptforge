import React, { useState } from 'react';
import {
  X,
  Settings,
  Server,
  Key,
  Clock,
  Check,
  Cpu,
  FileText,
} from 'lucide-react';
import type { AppConfig, ProviderType } from '../types';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: AppConfig;
  onSaveConfig: (newConfig: AppConfig) => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  config,
  onSaveConfig,
}) => {
  const [provider, setProvider] = useState<ProviderType>(config.active_provider);
  const [baseUrl, setBaseUrl] = useState(config.openai_compatible.base_url);
  const [model, setModel] = useState(config.openai_compatible.model);
  const [envVar, setEnvVar] = useState(config.openai_compatible.api_key_env_var);
  const [timeout, setTimeoutSec] = useState(config.openai_compatible.timeout_seconds);

  if (!isOpen) return null;

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    const updated: AppConfig = {
      ...config,
      active_provider: provider,
      openai_compatible: {
        base_url: baseUrl.trim(),
        model: model.trim(),
        api_key_env_var: envVar.trim(),
        timeout_seconds: Number(timeout) || 60,
      },
    };
    onSaveConfig(updated);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-lg shadow-2xl flex flex-col max-h-[90vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/40">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-slate-800 text-slate-200">
              <Settings className="w-5 h-5 text-amber-400" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100">Workbench Settings</h2>
              <p className="text-xs text-slate-400">Configure LLM refinement provider and options</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Form Body */}
        <form onSubmit={handleSave} className="p-6 overflow-y-auto space-y-5">
          {/* Provider Selection */}
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block">
              Refinement Provider
            </label>
            <div className="grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => setProvider('manual')}
                className={`p-3 rounded-xl border text-left flex flex-col gap-1 transition ${
                  provider === 'manual'
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-2">
                  <FileText
                    className={`w-4 h-4 ${provider === 'manual' ? 'text-amber-400' : 'text-slate-400'}`}
                  />
                  <span className="text-xs font-bold text-slate-100">Manual / Offline</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-tight">
                  Zero network calls. Copy-paste prompt packet into any web LLM.
                </p>
              </button>

              <button
                type="button"
                onClick={() => setProvider('openai_compatible')}
                className={`p-3 rounded-xl border text-left flex flex-col gap-1 transition ${
                  provider === 'openai_compatible'
                    ? 'bg-amber-500/10 border-amber-500/60 ring-1 ring-amber-500/30'
                    : 'bg-slate-800/40 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Server
                    className={`w-4 h-4 ${
                      provider === 'openai_compatible' ? 'text-amber-400' : 'text-slate-400'
                    }`}
                  />
                  <span className="text-xs font-bold text-slate-100">OpenAI Compatible</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-tight">
                  Automated HTTP calls via OpenAI, OpenRouter, or compatible endpoints.
                </p>
              </button>
            </div>
          </div>

          {/* OpenAI Configuration */}
          {provider === 'openai_compatible' && (
            <div className="p-4 rounded-xl bg-slate-800/40 border border-slate-800 space-y-4">
              {/* Base URL */}
              <div className="space-y-1">
                <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                  <Server className="w-3.5 h-3.5 text-slate-400" />
                  <span>API Base URL</span>
                </label>
                <input
                  type="text"
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder="https://api.openai.com/v1"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 outline-none focus:border-amber-500/50"
                  required
                />
              </div>

              {/* Model */}
              <div className="space-y-1">
                <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                  <Cpu className="w-3.5 h-3.5 text-slate-400" />
                  <span>Model Identifier</span>
                </label>
                <input
                  type="text"
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  placeholder="gpt-4o"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 outline-none focus:border-amber-500/50"
                  required
                />
              </div>

              {/* API Key Env Var */}
              <div className="space-y-1">
                <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                  <Key className="w-3.5 h-3.5 text-slate-400" />
                  <span>API Key Environment Variable</span>
                </label>
                <input
                  type="text"
                  value={envVar}
                  onChange={(e) => setEnvVar(e.target.value)}
                  placeholder="OPENAI_API_KEY"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-amber-300 outline-none focus:border-amber-500/50"
                  required
                />
                <p className="text-[10px] text-slate-500">
                  PromptForge reads your API key safely from this local environment variable.
                </p>
              </div>

              {/* Timeout */}
              <div className="space-y-1">
                <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                  <Clock className="w-3.5 h-3.5 text-slate-400" />
                  <span>Request Timeout (Seconds)</span>
                </label>
                <input
                  type="number"
                  min="5"
                  max="300"
                  value={timeout}
                  onChange={(e) => setTimeoutSec(Number(e.target.value))}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 outline-none focus:border-amber-500/50"
                  required
                />
              </div>
            </div>
          )}

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
              <Check className="w-4 h-4" />
              <span>Save Configuration</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
