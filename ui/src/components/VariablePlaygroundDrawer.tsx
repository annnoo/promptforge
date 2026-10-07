import React, { useState, useEffect } from 'react';
import { api } from '../api';
import type { DocumentStateDto, ScenarioPreviewDto, TestScenario } from '../types';
import { TagBadge } from './TagBadge';

interface VariablePlaygroundDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  docState: DocumentStateDto;
  onSaveScenario: (scenario: TestScenario) => void;
  onDeleteScenario: (id: string) => void;
  customColors?: Record<string, string>;
}

export const VariablePlaygroundDrawer: React.FC<VariablePlaygroundDrawerProps> = ({
  isOpen,
  onClose,
  docState,
  onSaveScenario,
  onDeleteScenario,
  customColors,
}) => {
  const [detectedVariables, setDetectedVariables] = useState<string[]>([]);
  const [selectedScenarioId, setSelectedScenarioId] = useState<string | null>(null);
  const [scenarioNameInput, setScenarioNameInput] = useState('');
  const [scenarioDescInput, setScenarioDescInput] = useState('');
  const [variableValues, setVariableValues] = useState<Record<string, string>>({});
  const [preview, setPreview] = useState<ScenarioPreviewDto | null>(null);
  const [copied, setCopied] = useState(false);
  const [isEditingScenario, setIsEditingScenario] = useState(false);

  const scenarios = docState.document.scenarios || [];

  // Fetch detected variables when opened or doc changes
  useEffect(() => {
    if (isOpen) {
      api.getDocumentVariables().then((vars) => {
        setDetectedVariables(vars);
      });
    }
  }, [isOpen, docState.document.sections]);

  // Sync active scenario selection
  useEffect(() => {
    if (!isOpen) return;
    setIsEditingScenario(false);
    if (selectedScenarioId) {
      const current = scenarios.find((s) => s.id === selectedScenarioId);
      if (current) {
        setScenarioNameInput(current.name);
        setScenarioDescInput(current.description || '');
        setVariableValues(current.variables || {});
        return;
      }
    }
    // Default to first scenario or empty draft
    if (scenarios.length > 0) {
      setSelectedScenarioId(scenarios[0].id);
      setScenarioNameInput(scenarios[0].name);
      setScenarioDescInput(scenarios[0].description || '');
      setVariableValues(scenarios[0].variables || {});
    } else {
      setSelectedScenarioId(null);
      setScenarioNameInput('Default Test Scenario');
      setScenarioDescInput('Sample input values for prompt evaluation');
      setVariableValues({});
    }
  }, [isOpen, selectedScenarioId, scenarios]);

  // Update live interpolation preview
  useEffect(() => {
    if (!isOpen) return;
    let isCurrent = true;
    api
      .renderScenarioPreview(
        selectedScenarioId || undefined,
        variableValues,
        'final',
        false
      )
      .then((res) => {
        if (isCurrent) setPreview(res);
      })
      .catch((err) => {
        console.error('Failed to render scenario preview:', err);
      });
    return () => {
      isCurrent = false;
    };
  }, [isOpen, selectedScenarioId, variableValues, docState.document.sections]);

  if (!isOpen) return null;

  const handleVariableChange = (varName: string, val: string) => {
    setVariableValues((prev) => ({
      ...prev,
      [varName]: val,
    }));
  };

  const handleSaveCurrentScenario = () => {
    const id = selectedScenarioId || `scen-${Date.now()}`;
    const scenario: TestScenario = {
      id,
      name: scenarioNameInput.trim() || 'Untitled Scenario',
      description: scenarioDescInput.trim() || null,
      variables: variableValues,
    };
    onSaveScenario(scenario);
    setSelectedScenarioId(id);
    setIsEditingScenario(false);
  };

  const handleCreateNewScenario = () => {
    const newId = `scen-${Date.now()}`;
    const initialVals: Record<string, string> = {};
    for (const v of detectedVariables) {
      initialVals[v] = '';
    }
    const newScenario: TestScenario = {
      id: newId,
      name: `Scenario ${scenarios.length + 1}`,
      description: 'Test case parameters',
      variables: initialVals,
    };
    onSaveScenario(newScenario);
    setSelectedScenarioId(newId);
    setScenarioNameInput(newScenario.name);
    setScenarioDescInput(newScenario.description || '');
    setVariableValues(initialVals);
  };

  const handleCopyResolvedPrompt = async () => {
    if (!preview) return;
    try {
      await navigator.clipboard.writeText(preview.rendered_xml);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      console.error('Failed to copy to clipboard');
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="bg-zinc-950 border border-zinc-800 rounded-xl max-w-5xl w-full flex flex-col max-h-[92vh] shadow-2xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-800/80 flex justify-between items-center bg-zinc-900/40">
          <div>
            <h2 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-cyan-400" />
              Dynamic Prompt Variables &amp; Scenario Matrix
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Simulate prompt variations with <code className="text-cyan-400 font-mono">{'{{variables}}'}</code> and test scenarios without editing base instructions.
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-500 hover:text-zinc-300 transition-colors p-1"
          >
            ✕
          </button>
        </div>

        {/* Content Body: 2 Columns */}
        <div className="flex-1 overflow-hidden grid grid-cols-1 md:grid-cols-12">
          {/* Left Column (5 cols): Variables & Scenarios Setup */}
          <div className="md:col-span-5 border-r border-zinc-800/80 p-5 overflow-y-auto space-y-4 bg-zinc-900/20">
            {/* Detected Variables Tags */}
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <span className="text-xs font-semibold text-zinc-300">
                  Detected Variables ({detectedVariables.length})
                </span>
                <span className="text-[10px] text-zinc-500 font-mono">
                  Syntax: {'{{variable_name}}'}
                </span>
              </div>

              {detectedVariables.length === 0 ? (
                <div className="p-3 bg-zinc-900/60 border border-dashed border-zinc-800 rounded-lg text-xs text-zinc-500 leading-relaxed">
                  No variables found in active sections. Add <code className="text-cyan-400 font-mono">{'{{variable_name}}'}</code> anywhere in your section text to test dynamic inputs.
                </div>
              ) : (
                <div className="flex flex-wrap gap-1.5">
                  {detectedVariables.map((v) => (
                    <TagBadge
                      key={v}
                      tag={v}
                      size="xs"
                      showHash={false}
                      customColors={customColors}
                    />
                  ))}
                </div>
              )}
            </div>

            {/* Scenarios Selector Bar */}
            <div className="pt-2 border-t border-zinc-800/60">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-zinc-300">Test Scenarios</span>
                <button
                  type="button"
                  onClick={handleCreateNewScenario}
                  className="px-2 py-0.5 text-xs font-medium text-cyan-400 bg-cyan-950/40 border border-cyan-800/60 hover:bg-cyan-900/40 rounded transition-colors cursor-pointer"
                >
                  + New Scenario
                </button>
              </div>

              {scenarios.length > 0 && (
                <div className="flex gap-1.5 overflow-x-auto pb-1 scrollbar-none">
                  {scenarios.map((sc) => (
                    <button
                      key={sc.id}
                      type="button"
                      onClick={() => setSelectedScenarioId(sc.id)}
                      className={`px-2.5 py-1 text-xs rounded border transition-colors shrink-0 cursor-pointer ${
                        selectedScenarioId === sc.id
                          ? 'bg-zinc-800 text-zinc-100 border-zinc-600 font-medium'
                          : 'bg-zinc-900 text-zinc-400 border-zinc-800 hover:border-zinc-700'
                      }`}
                    >
                      {sc.name}
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Scenario Details Form */}
            <div className="space-y-3 pt-1">
              <div>
                <label className="block text-[11px] font-medium text-zinc-400 mb-1">
                  Scenario Name
                </label>
                <input
                  type="text"
                  value={scenarioNameInput}
                  onChange={(e) => {
                    setScenarioNameInput(e.target.value);
                    setIsEditingScenario(true);
                  }}
                  className="w-full px-2.5 py-1 bg-zinc-900 border border-zinc-800 rounded text-xs text-zinc-200 focus:outline-none focus:border-zinc-600"
                />
              </div>

              {/* Variable Value Inputs */}
              <div>
                <span className="block text-[11px] font-medium text-zinc-400 mb-1.5">
                  Variable Values for Scenario
                </span>

                {detectedVariables.length === 0 ? (
                  <div className="text-xs text-zinc-500 italic">
                    Add variables in section briefs to assign values.
                  </div>
                ) : (
                  <div className="space-y-2">
                    {detectedVariables.map((v) => (
                      <div key={v} className="space-y-0.5">
                        <div className="flex justify-between items-center">
                          <span className="text-[11px] font-mono text-cyan-400">
                            {'{' + '{' + v + '}' + '}'}
                          </span>
                        </div>
                        <textarea
                          rows={2}
                          value={variableValues[v] || ''}
                          onChange={(e) => {
                            handleVariableChange(v, e.target.value);
                            setIsEditingScenario(true);
                          }}
                          placeholder={`Value for {{${v}}}...`}
                          className="w-full px-2 py-1 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200 font-mono focus:outline-none focus:border-zinc-700 resize-none leading-relaxed"
                        />
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Scenario Actions */}
              <div className="flex gap-2 pt-2">
                <button
                  type="button"
                  onClick={handleSaveCurrentScenario}
                  className="flex-1 px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white rounded text-xs font-medium transition-colors cursor-pointer flex items-center justify-center gap-1.5"
                >
                  <span>Save Scenario</span>
                  {isEditingScenario && (
                    <span className="w-1.5 h-1.5 rounded-full bg-cyan-200 animate-pulse" />
                  )}
                </button>
                {selectedScenarioId && scenarios.length > 1 && (
                  <button
                    type="button"
                    onClick={() => {
                      if (confirm('Delete this test scenario?')) {
                        onDeleteScenario(selectedScenarioId);
                        setSelectedScenarioId(null);
                      }
                    }}
                    className="px-2.5 py-1.5 bg-zinc-900 hover:bg-rose-950/40 text-zinc-400 hover:text-rose-300 border border-zinc-800 rounded text-xs transition-colors cursor-pointer"
                    title="Delete Scenario"
                  >
                    🗑
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Right Column (7 cols): Live Telemetry & Interpolated XML Preview */}
          <div className="md:col-span-7 p-5 flex flex-col overflow-hidden bg-zinc-950 space-y-3">
            {/* Telemetry Bar */}
            <div className="flex flex-wrap items-center justify-between gap-2 p-2.5 bg-zinc-900/60 border border-zinc-800/80 rounded-lg text-xs">
              <div className="flex items-center gap-3">
                <span className="text-zinc-400">
                  Tokens (Est.):{' '}
                  <strong className="text-cyan-400 font-mono">
                    ~{preview?.estimated_tokens || 0}
                  </strong>
                </span>
                <span className="text-zinc-400">
                  Words: <strong className="text-zinc-200 font-mono">{preview?.word_count || 0}</strong>
                </span>
                <span className="text-zinc-400">
                  Chars: <strong className="text-zinc-200 font-mono">{preview?.char_count || 0}</strong>
                </span>
              </div>

              {preview && preview.unresolved_variables.length > 0 && (
                <span className="text-[10px] text-amber-400 font-mono bg-amber-950/40 px-2 py-0.5 rounded border border-amber-800/60">
                  {preview.unresolved_variables.length} unresolved
                </span>
              )}
            </div>

            {/* Resolved Preview Code Area */}
            <div className="flex-1 flex flex-col min-h-0">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-semibold text-zinc-300">
                  Resolved Prompt Output ({scenarioNameInput || 'Default'})
                </span>
                <button
                  type="button"
                  onClick={handleCopyResolvedPrompt}
                  className="px-2 py-0.5 text-xs text-zinc-300 hover:text-white bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded transition-colors cursor-pointer"
                >
                  {copied ? '✓ Copied!' : '📋 Copy Resolved'}
                </button>
              </div>

              <pre className="flex-1 p-3.5 bg-zinc-900/80 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 overflow-auto select-text leading-relaxed">
                {preview?.rendered_xml || 'Rendering scenario...'}
              </pre>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-zinc-800/80 flex justify-between items-center bg-zinc-900/30">
          <span className="text-xs text-zinc-500">
            {scenarios.length} test scenario{scenarios.length === 1 ? '' : 's'} stored in document
          </span>
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 rounded text-xs font-medium transition-colors cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
