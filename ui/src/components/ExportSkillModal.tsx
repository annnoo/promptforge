import React, { useState, useEffect } from 'react';
import { api } from '../api';

interface ExportSkillModalProps {
  isOpen: boolean;
  onClose: () => void;
  defaultTitle: string;
  defaultDescription: string;
}

export const ExportSkillModal: React.FC<ExportSkillModalProps> = ({
  isOpen,
  onClose,
  defaultTitle,
  defaultDescription,
}) => {
  const [skillName, setSkillName] = useState('');
  const [skillDescription, setSkillDescription] = useState('');
  const [stage, setStage] = useState<'final' | 'draft'>('final');
  const [content, setContent] = useState('');
  const [copied, setCopied] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [statusMsg, setStatusMsg] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  useEffect(() => {
    if (isOpen) {
      const slug = defaultTitle
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '');
      setSkillName(slug || 'custom-prompt-skill');
      setSkillDescription(defaultDescription || `Agent skill for ${defaultTitle}`);
      setStatusMsg(null);
      setCopied(false);
    }
  }, [isOpen, defaultTitle, defaultDescription]);

  useEffect(() => {
    if (!isOpen) return;
    let isCurrent = true;
    api
      .generateSkillContent(stage, skillName, skillDescription)
      .then((generated) => {
        if (isCurrent) setContent(generated);
      })
      .catch((err) => {
        console.error('Failed to generate skill preview:', err);
      });
    return () => {
      isCurrent = false;
    };
  }, [isOpen, stage, skillName, skillDescription]);

  if (!isOpen) return null;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setStatusMsg({ text: 'Failed to copy to clipboard', type: 'error' });
    }
  };

  const handleSaveToFile = async () => {
    try {
      setIsExporting(true);
      setStatusMsg(null);
      const defaultFileName = `${skillName || 'skill'}.SKILL.md`;
      const savePath = await api.pickSaveSkillFile(defaultFileName);
      if (savePath) {
        await api.exportSkillToFile(savePath, stage, skillName, skillDescription);
        setStatusMsg({ text: `Exported successfully to ${savePath}`, type: 'success' });
      }
    } catch (err) {
      setStatusMsg({ text: `Export failed: ${err}`, type: 'error' });
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="bg-zinc-950 border border-zinc-800 rounded-xl max-w-3xl w-full flex flex-col max-h-[90vh] shadow-2xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-800/80 flex justify-between items-center bg-zinc-900/40">
          <div>
            <h2 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-violet-400" />
              Export as Agent Skill
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Exports prompt as a standard <code className="text-zinc-300">SKILL.md</code> with YAML frontmatter for Antigravity, Claude, and OpenAI agents.
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
          {/* Settings Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1">
                Skill Identifier (kebab-case)
              </label>
              <input
                type="text"
                value={skillName}
                onChange={(e) => setSkillName(e.target.value)}
                placeholder="e.g. system-architect-review"
                className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded text-sm text-zinc-200 focus:outline-none focus:border-zinc-600 font-mono"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1">
                Render Stage
              </label>
              <div className="flex bg-zinc-900 border border-zinc-800 rounded p-0.5">
                <button
                  type="button"
                  onClick={() => setStage('final')}
                  className={`flex-1 py-1 text-xs font-medium rounded transition-colors ${
                    stage === 'final'
                      ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                      : 'text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  Final (Clean / Accepted)
                </button>
                <button
                  type="button"
                  onClick={() => setStage('draft')}
                  className={`flex-1 py-1 text-xs font-medium rounded transition-colors ${
                    stage === 'draft'
                      ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                      : 'text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  Draft (Original Briefs)
                </button>
              </div>
            </div>
          </div>

          <div>
            <label className="block text-xs font-medium text-zinc-400 mb-1">
              Skill Description
            </label>
            <input
              type="text"
              value={skillDescription}
              onChange={(e) => setSkillDescription(e.target.value)}
              placeholder="Brief summary of when agents should activate this skill"
              className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded text-sm text-zinc-200 focus:outline-none focus:border-zinc-600"
            />
          </div>

          {/* Status Message */}
          {statusMsg && (
            <div
              className={`p-2.5 rounded text-xs border ${
                statusMsg.type === 'success'
                  ? 'bg-emerald-950/40 border-emerald-800/60 text-emerald-300'
                  : 'bg-rose-950/40 border-rose-800/60 text-rose-300'
              }`}
            >
              {statusMsg.text}
            </div>
          )}

          {/* Code Preview */}
          <div>
            <div className="flex justify-between items-center mb-1">
              <label className="text-xs font-medium text-zinc-400">
                Generated <span className="font-mono text-zinc-300">SKILL.md</span> Preview
              </label>
              <span className="text-[11px] text-zinc-500 font-mono">
                {content.split('\n').length} lines
              </span>
            </div>
            <pre className="p-3 bg-zinc-900/90 border border-zinc-800/80 rounded-lg text-xs font-mono text-zinc-300 overflow-x-auto max-h-64 select-text leading-relaxed">
              {content}
            </pre>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-zinc-800/80 flex justify-between items-center bg-zinc-900/30">
          <button
            type="button"
            onClick={handleCopy}
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 rounded text-xs font-medium transition-colors flex items-center gap-1.5 cursor-pointer"
          >
            {copied ? (
              <>
                <span className="text-emerald-400">✓</span> Copied to Clipboard!
              </>
            ) : (
              <>
                <span>📋</span> Copy Markdown
              </>
            )}
          </button>

          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 border border-zinc-800 rounded text-xs font-medium transition-colors cursor-pointer"
            >
              Close
            </button>
            <button
              type="button"
              onClick={handleSaveToFile}
              disabled={isExporting}
              className="px-4 py-1.5 bg-violet-600 hover:bg-violet-500 text-white rounded text-xs font-medium transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
            >
              <span>💾</span>
              {isExporting ? 'Saving...' : 'Save SKILL.md...'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
