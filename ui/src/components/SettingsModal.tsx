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
  Eye,
  EyeOff,
  Zap,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Bot,
  Sparkles,
  Terminal,
} from 'lucide-react';
import type { AppConfig, ProviderType } from '../types';
import { api } from '../api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: AppConfig;
  onSaveConfig: (newConfig: AppConfig) => void;
}

interface PresetOption {
  id: string;
  name: string;
  badge: string;
  icon: React.ReactNode;
  baseUrl: string;
  model: string;
  envVar: string;
  description: string;
}

const PRESETS: PresetOption[] = [
  {
    id: 'openai',
    name: 'OpenAI / ChatGPT',
    badge: 'Official',
    icon: <Sparkles className="w-3.5 h-3.5 text-emerald-400" />,
    baseUrl: 'https://api.openai.com/v1',
    model: 'gpt-4o',
    envVar: 'OPENAI_API_KEY',
    description: 'Direct API connection for GPT-4o, o1, o3-mini models',
  },
  {
    id: 'copilot',
    name: 'GitHub Copilot / Codex',
    badge: 'Copilot',
    icon: <Bot className="w-3.5 h-3.5 text-indigo-400" />,
    baseUrl: 'https://api.githubcopilot.com',
    model: 'gpt-4o',
    envVar: 'COPILOT_API_KEY',
    description: 'Use your GitHub Copilot subscription / proxy endpoint',
  },
  {
    id: 'ollama',
    name: 'Ollama / Local LLM',
    badge: 'Offline',
    icon: <Terminal className="w-3.5 h-3.5 text-amber-400" />,
    baseUrl: 'http://localhost:11434/v1',
    model: 'deepseek-r1',
    envVar: 'OLLAMA_API_KEY',
    description: 'Zero cost, 100% local privacy (DeepSeek R1, Llama 3, Qwen)',
  },
  {
    id: 'openrouter',
    name: 'OpenRouter / Anthropic',
    badge: 'Multi-Model',
    icon: <Zap className="w-3.5 h-3.5 text-cyan-400" />,
    baseUrl: 'https://openrouter.ai/api/v1',
    model: 'anthropic/claude-3.5-sonnet',
    envVar: 'OPENROUTER_API_KEY',
    description: 'Unified router for Claude 3.5 Sonnet, Gemini, and DeepSeek',
  },
];

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  config,
  onSaveConfig,
}) => {
  const [provider, setProvider] = useState<ProviderType>(config.active_provider);
  const [selectedPreset, setSelectedPreset] = useState<string>(
    config.openai_compatible.preset || 'openai'
  );
  const [baseUrl, setBaseUrl] = useState(config.openai_compatible.base_url);
  const [model, setModel] = useState(config.openai_compatible.model);
  const [envVar, setEnvVar] = useState(config.openai_compatible.api_key_env_var);
  const [apiKey, setApiKey] = useState(config.openai_compatible.api_key || '');
  const [showApiKey, setShowApiKey] = useState(false);
  const [timeout, setTimeoutSec] = useState(config.openai_compatible.timeout_seconds);

  // Test connection state
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  if (!isOpen) return null;

  const applyPreset = (preset: PresetOption) => {
    setSelectedPreset(preset.id);
    setBaseUrl(preset.baseUrl);
    setModel(preset.model);
    setEnvVar(preset.envVar);
    setTestResult(null);
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const msg = await api.testAiConnection({
        base_url: baseUrl.trim(),
        model: model.trim(),
        api_key_env_var: envVar.trim(),
        api_key: apiKey.trim() || null,
        preset: selectedPreset,
        timeout_seconds: Number(timeout) || 60,
      });
      setTestResult({ success: true, message: msg });
    } catch (err: any) {
      setTestResult({
        success: false,
        message: err?.message || String(err) || 'Failed to connect to LLM provider',
      });
    } finally {
      setTesting(false);
    }
  };

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    const updated: AppConfig = {
      ...config,
      active_provider: provider,
      openai_compatible: {
        base_url: baseUrl.trim(),
        model: model.trim(),
        api_key_env_var: envVar.trim(),
        api_key: apiKey.trim() || null,
        preset: selectedPreset,
        timeout_seconds: Number(timeout) || 60,
      },
    };
    onSaveConfig(updated);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl shadow-2xl flex flex-col max-h-[92vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/40">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-slate-800 text-slate-200">
              <Settings className="w-5 h-5 text-amber-400" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100">Workbench AI Settings</h2>
              <p className="text-xs text-slate-400">Configure LLM refinement backends & connections</p>
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
        <form onSubmit={handleSave} className="p-6 overflow-y-auto space-y-6">
          {/* Provider Mode Selection */}
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block">
              Refinement Engine Mode
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
                  <span className="text-xs font-bold text-slate-100">Manual / Web Paste</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-tight">
                  Generate prompt packets to copy into ChatGPT web UI, Claude, or Copilot chat.
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
                  <span className="text-xs font-bold text-slate-100">Automated AI API</span>
                </div>
                <p className="text-[11px] text-slate-400 leading-tight">
                  1-Click automated refinement via OpenAI, Copilot, Ollama, or OpenRouter.
                </p>
              </button>
            </div>
          </div>

          {/* Automated AI Provider Configuration */}
          {provider === 'openai_compatible' && (
            <div className="space-y-4">
              {/* Presets Grid */}
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-300 uppercase tracking-wider block">
                  Quick Provider Presets
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {PRESETS.map((p) => {
                    const isSelected = selectedPreset === p.id;
                    return (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => applyPreset(p)}
                        className={`p-2.5 rounded-xl border text-left flex flex-col gap-1 transition ${
                          isSelected
                            ? 'bg-amber-500/15 border-amber-500/70 ring-1 ring-amber-500/30'
                            : 'bg-slate-950/60 border-slate-800 hover:border-slate-700'
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-1.5 font-bold text-xs text-slate-200">
                            {p.icon}
                            <span>{p.name}</span>
                          </div>
                          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">
                            {p.badge}
                          </span>
                        </div>
                        <p className="text-[10px] text-slate-400 leading-tight line-clamp-1">
                          {p.description}
                        </p>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Endpoint Details */}
              <div className="p-4 rounded-xl bg-slate-800/40 border border-slate-800 space-y-4">
                {/* Base URL */}
                <div className="space-y-1">
                  <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                    <Server className="w-3.5 h-3.5 text-slate-400" />
                    <span>API Base Endpoint URL</span>
                  </label>
                  <input
                    type="text"
                    value={baseUrl}
                    onChange={(e) => {
                      setBaseUrl(e.target.value);
                      setSelectedPreset('custom');
                    }}
                    placeholder="https://api.openai.com/v1"
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 outline-none focus:border-amber-500/50"
                    required
                  />
                </div>

                {/* Model */}
                <div className="space-y-1">
                  <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                    <Cpu className="w-3.5 h-3.5 text-slate-400" />
                    <span>Model Name / ID</span>
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

                {/* API Key (Direct) */}
                <div className="space-y-1">
                  <label className="text-xs font-semibold text-slate-300 flex items-center justify-between">
                    <span className="flex items-center gap-1.5">
                      <Key className="w-3.5 h-3.5 text-amber-400" />
                      <span>API Key / Secret Token (Direct)</span>
                    </span>
                    <span className="text-[10px] text-slate-400 font-normal">
                      Stored locally in config
                    </span>
                  </label>
                  <div className="relative">
                    <input
                      type={showApiKey ? 'text' : 'password'}
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="Paste your API key (e.g. sk-...) or leave blank to use env var"
                      className="w-full bg-slate-950 border border-slate-800 rounded-lg pl-3 pr-10 py-2 text-xs font-mono text-amber-300 outline-none focus:border-amber-500/50"
                    />
                    <button
                      type="button"
                      onClick={() => setShowApiKey(!showApiKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-slate-400 hover:text-white"
                    >
                      {showApiKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                    </button>
                  </div>
                </div>

                {/* API Key Env Var (Fallback) */}
                <div className="space-y-1">
                  <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                    <Terminal className="w-3.5 h-3.5 text-slate-400" />
                    <span>Fallback Environment Variable Name</span>
                  </label>
                  <input
                    type="text"
                    value={envVar}
                    onChange={(e) => setEnvVar(e.target.value)}
                    placeholder="OPENAI_API_KEY"
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-300 outline-none focus:border-amber-500/50"
                    required
                  />
                  <p className="text-[10px] text-slate-500">
                    If no direct key is entered above, PromptForge reads from process environment.
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

                {/* Test Connection Button */}
                <div className="pt-2 border-t border-slate-800/80 flex flex-col gap-2">
                  <button
                    type="button"
                    onClick={handleTestConnection}
                    disabled={testing}
                    className="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold text-xs flex items-center justify-center gap-2 border border-slate-700/60 transition disabled:opacity-50"
                  >
                    {testing ? (
                      <>
                        <Loader2 className="w-3.5 h-3.5 animate-spin text-amber-400" />
                        <span>Testing LLM Connection...</span>
                      </>
                    ) : (
                      <>
                        <Zap className="w-3.5 h-3.5 text-amber-400" />
                        <span>Test AI Endpoint Connection</span>
                      </>
                    )}
                  </button>

                  {testResult && (
                    <div
                      className={`p-2.5 rounded-lg border text-xs flex items-start gap-2 animate-in fade-in duration-150 ${
                        testResult.success
                          ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-300'
                          : 'bg-rose-950/40 border-rose-500/40 text-rose-300'
                      }`}
                    >
                      {testResult.success ? (
                        <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                      ) : (
                        <AlertCircle className="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
                      )}
                      <div className="break-all">{testResult.message}</div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
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
