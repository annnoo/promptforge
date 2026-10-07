import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Plus,
  Layers,
  Info,
  CheckCircle,
  AlertCircle,
  X,
  FilePlus,
  Tag,
} from 'lucide-react';
import type {
  AppConfig,
  Command,
  DocumentStateDto,
  SectionType,
  StarterTemplate,
  TestScenario,
} from './types';
import { api } from './api';
import { Header } from './components/Header';
import { PresetsSidebar } from './components/PresetsSidebar';
import { SectionCard } from './components/SectionCard';
import { XmlPreviewPanel } from './components/XmlPreviewPanel';
import { RefineModal } from './components/RefineModal';
import { DiffReviewModal } from './components/DiffReviewModal';
import { SettingsModal } from './components/SettingsModal';
import { NewDocumentModal } from './components/NewDocumentModal';
import { CreateTypeModal } from './components/CreateTypeModal';
import { ExportSkillModal } from './components/ExportSkillModal';
import { ImportPromptModal } from './components/ImportPromptModal';
import { PromptLibraryModal } from './components/PromptLibraryModal';
import { VariablePlaygroundDrawer } from './components/VariablePlaygroundDrawer';
import { RevisionHistoryModal } from './components/RevisionHistoryModal';
import { TagBadge } from './components/TagBadge';

interface ToastInfo {
  id: number;
  message: string;
  type: 'info' | 'success' | 'error';
}

export const App: React.FC = () => {
  const [docState, setDocState] = useState<DocumentStateDto | null>(null);
  const [sectionTypes, setSectionTypes] = useState<SectionType[]>([]);
  const [templates, setTemplates] = useState<StarterTemplate[]>([]);
  const [config, setConfig] = useState<AppConfig | null>(null);

  // Modals
  const [isRefineModalOpen, setIsRefineModalOpen] = useState(false);
  const [isDiffReviewModalOpen, setIsDiffReviewModalOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [isNewDocModalOpen, setIsNewDocModalOpen] = useState(false);
  const [isCreateTypeModalOpen, setIsCreateTypeModalOpen] = useState(false);
  const [isExportSkillModalOpen, setIsExportSkillModalOpen] = useState(false);
  const [isImportPromptModalOpen, setIsImportPromptModalOpen] = useState(false);
  const [isPromptLibraryModalOpen, setIsPromptLibraryModalOpen] = useState(false);
  const [isVariablesDrawerOpen, setIsVariablesDrawerOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [editingType, setEditingType] = useState<SectionType | null>(null);

  // Filter sections by tag on canvas
  const [selectedCanvasTag, setSelectedCanvasTag] = useState<string | null>(null);

  // Toast notifications
  const [toasts, setToasts] = useState<ToastInfo[]>([]);

  const showToast = useCallback((message: string, type: 'info' | 'success' | 'error' = 'info') => {
    const id = Date.now() + Math.random();
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 3500);
  }, []);

  // Initial load
  useEffect(() => {
    Promise.all([
      api.getDocumentState(),
      api.getSectionTypes(),
      api.getTemplates(),
      api.getConfig(),
    ])
      .then(([state, types, tmpl, cfg]) => {
        setDocState(state);
        setSectionTypes(types);
        setTemplates(tmpl);
        setConfig(cfg);
      })
      .catch((err) => {
        console.error('Initialization error:', err);
        showToast('Failed to initialize workbench: ' + err, 'error');
      });
  }, [showToast]);

  // Execute command helper
  const exec = useCallback(
    async (command: Command) => {
      try {
        const nextState = await api.executeCommand(command);
        setDocState(nextState);
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast(msg, 'error');
      }
    },
    [showToast]
  );

  // File operations
  const handleOpen = useCallback(async () => {
    try {
      const path = await api.pickOpenFile();
      if (!path) return;
      const nextState = await api.loadDocument(path);
      setDocState(nextState);
      showToast(`Loaded ${path.split(/[/\\]/).pop()}`, 'success');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast('Open failed: ' + msg, 'error');
    }
  }, [showToast]);

  const handleSaveAs = useCallback(async () => {
    if (!docState) return;
    try {
      const defaultName = docState.document.title
        ? `${docState.document.title.toLowerCase().replace(/[^a-z0-9_-]/g, '_')}.prompt.json`
        : 'prompt.prompt.json';
      const path = await api.pickSaveFile(defaultName);
      if (!path) return;
      const nextState = await api.saveDocument(path);
      setDocState(nextState);
      showToast(`Saved to ${path.split(/[/\\]/).pop()}`, 'success');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast('Save As failed: ' + msg, 'error');
    }
  }, [docState, showToast]);

  const handleSave = useCallback(async () => {
    if (!docState) return;
    if (!docState.current_file_path) {
      return handleSaveAs();
    }
    try {
      const nextState = await api.saveDocument();
      setDocState(nextState);
      showToast('Document saved successfully', 'success');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast('Save failed: ' + msg, 'error');
    }
  }, [docState, handleSaveAs, showToast]);

  const handleCreateDocument = useCallback(
    async (templateId?: string, title?: string) => {
      try {
        const nextState = await api.newDocument(templateId, title);
        setDocState(nextState);
        showToast('Created new prompt document', 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('New document failed: ' + msg, 'error');
      }
    },
    [showToast]
  );

  const handleExportToFile = useCallback(
    async (stage: 'draft' | 'final', clean: boolean) => {
      if (!docState) return;
      try {
        const defaultName = `${(docState.document.title || 'prompt')
          .toLowerCase()
          .replace(/[^a-z0-9_-]/g, '_')}.${stage}.xml`;
        const path = await api.pickSaveFile(defaultName);
        if (!path) return;
        await api.exportXmlToFile(path, stage, clean);
        showToast(`Exported XML to ${path.split(/[/\\]/).pop()}`, 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Export failed: ' + msg, 'error');
      }
    },
    [docState, showToast]
  );

  const handleSaveConfig = useCallback(
    async (newConfig: AppConfig) => {
      try {
        await api.saveConfig(newConfig);
        setConfig(newConfig);
        showToast('Configuration saved', 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Failed to save config: ' + msg, 'error');
      }
    },
    [showToast]
  );

  // Section Type management handlers
  const handleInsertType = useCallback(
    async (type: SectionType) => {
      try {
        await exec({ type: 'AddSection', tag: type.tag, brief: type.default_brief });
        showToast(`Added <${type.tag}> section`, 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Failed to insert section: ' + msg, 'error');
      }
    },
    [exec, showToast]
  );

  const handleSaveSectionType = useCallback(
    async (type: SectionType) => {
      try {
        const updatedList = await api.saveSectionType(type);
        setSectionTypes(updatedList);
        showToast(`Saved Section Type "${type.name}"`, 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Failed to save type: ' + msg, 'error');
      }
    },
    [showToast]
  );

  const handleDeleteSectionType = useCallback(
    async (id: string) => {
      try {
        const updatedList = await api.deleteSectionType(id);
        setSectionTypes(updatedList);
        showToast('Section Type deleted', 'info');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Failed to delete type: ' + msg, 'error');
      }
    },
    [showToast]
  );

  const handleTagColorChange = useCallback(
    async (tag: string, color: string) => {
      try {
        const newConfig = await api.setTagColor(tag, color);
        setConfig(newConfig);
        showToast(`Updated color for #${tag}`, 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Failed to save tag color: ' + msg, 'error');
      }
    },
    [showToast]
  );

  const handleSaveCurrentToLibrary = useCallback(async () => {
    if (!docState) return;
    try {
      const summary = await api.saveToLibrary();
      showToast(`Saved "${summary.title}" to Library`, 'success');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast('Save to Library failed: ' + msg, 'error');
      throw err;
    }
  }, [docState, showToast]);

  // Existing folders list
  const existingFolders = useMemo(() => {
    const set = new Set<string>();
    for (const st of sectionTypes) {
      if (st.folder) set.add(st.folder);
    }
    return Array.from(set).sort();
  }, [sectionTypes]);

  // Unique tags across document sections for canvas filter
  const docSectionTags = useMemo(() => {
    if (!docState) return [];
    const set = new Set<string>();
    for (const sec of docState.document.sections) {
      if (sec.tags) {
        for (const t of sec.tags) {
          set.add(t);
        }
      }
    }
    return Array.from(set).sort();
  }, [docState]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const mod = isMac ? e.metaKey : e.ctrlKey;

      if (mod && e.key.toLowerCase() === 's') {
        e.preventDefault();
        handleSave();
      } else if (mod && !e.shiftKey && e.key.toLowerCase() === 'z') {
        e.preventDefault();
        if (docState?.can_undo) exec({ type: 'Undo' });
      } else if (
        (mod && e.shiftKey && e.key.toLowerCase() === 'z') ||
        (mod && e.key.toLowerCase() === 'y')
      ) {
        e.preventDefault();
        if (docState?.can_redo) exec({ type: 'Redo' });
      } else if (mod && e.key.toLowerCase() === 'n') {
        e.preventDefault();
        setIsNewDocModalOpen(true);
      } else if (mod && e.key.toLowerCase() === 'o') {
        e.preventDefault();
        handleOpen();
      } else if (mod && e.key.toLowerCase() === 'r') {
        e.preventDefault();
        setIsRefineModalOpen(true);
      } else if (e.key === 'Escape') {
        setIsRefineModalOpen(false);
        setIsDiffReviewModalOpen(false);
        setIsSettingsModalOpen(false);
        setIsNewDocModalOpen(false);
        setIsCreateTypeModalOpen(false);
        setIsExportSkillModalOpen(false);
        setIsImportPromptModalOpen(false);
        setIsPromptLibraryModalOpen(false);
        setIsVariablesDrawerOpen(false);
        setIsHistoryModalOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [docState, exec, handleOpen, handleSave]);

  const handleSaveScenario = useCallback(
    async (scenario: TestScenario) => {
      await exec({ type: 'SaveScenario', scenario });
      showToast(`Saved test scenario "${scenario.name}"`, 'success');
    },
    [exec, showToast]
  );

  const handleDeleteScenario = useCallback(
    async (id: string) => {
      await exec({ type: 'DeleteScenario', id });
      showToast('Deleted test scenario', 'info');
    },
    [exec, showToast]
  );

  const handleCreateSnapshot = useCallback(
    async (name: string, description?: string | null) => {
      await exec({ type: 'CreateSnapshot', name, description: description || null });
      showToast(`Created snapshot "${name}"`, 'success');
    },
    [exec, showToast]
  );

  const handleRestoreSnapshot = useCallback(
    async (snapshotId: string) => {
      await exec({ type: 'RestoreSnapshot', id: snapshotId });
      showToast('Restored snapshot successfully', 'success');
    },
    [exec, showToast]
  );

  const handleDeleteSnapshot = useCallback(
    async (snapshotId: string) => {
      await exec({ type: 'DeleteSnapshot', id: snapshotId });
      showToast('Deleted snapshot', 'info');
    },
    [exec, showToast]
  );

  const handleForkSnapshot = useCallback(
    async (snapshotId: string, newTitle: string) => {
      try {
        const nextState = await api.forkSnapshot(snapshotId, newTitle);
        setDocState(nextState);
        setIsHistoryModalOpen(false);
        showToast(`Forked snapshot into new prompt "${newTitle}"`, 'success');
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        showToast('Fork failed: ' + msg, 'error');
      }
    },
    [showToast]
  );

  const scrollToSection = (id: string) => {
    const el = document.getElementById(`section-${id}`);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      el.classList.add('ring-2', 'ring-amber-400');
      setTimeout(() => el.classList.remove('ring-2', 'ring-amber-400'), 1200);
    }
  };

  if (!docState || !config) {
    return (
      <div className="h-screen w-screen flex flex-col items-center justify-center bg-slate-950 text-slate-300 select-none">
        <div className="w-8 h-8 border-2 border-amber-400 border-t-transparent rounded-full animate-spin mb-3" />
        <p className="text-xs font-mono text-slate-400">Loading PromptForge Workbench...</p>
      </div>
    );
  }

  const sections = docState.document.sections;
  const pendingMap = new Set(
    docState.pending_refinements?.changes.map((c) => c.section_id) || []
  );

  const displayedSections = selectedCanvasTag
    ? sections.filter((s) => s.tags && s.tags.includes(selectedCanvasTag))
    : sections;

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden bg-slate-950 text-slate-100 select-none font-sans">
      {/* 1. Header */}
      <Header
        state={docState}
        onUpdateTitle={(title) => exec({ type: 'UpdateTitle', title })}
        onUpdateDescription={(description) => exec({ type: 'UpdateDescription', description })}
        onNew={() => setIsNewDocModalOpen(true)}
        onOpen={handleOpen}
        onSave={handleSave}
        onSaveAs={handleSaveAs}
        onUndo={() => exec({ type: 'Undo' })}
        onRedo={() => exec({ type: 'Redo' })}
        onOpenLibrary={() => setIsPromptLibraryModalOpen(true)}
        onOpenImport={() => setIsImportPromptModalOpen(true)}
        onOpenExportSkill={() => setIsExportSkillModalOpen(true)}
        onOpenVariables={() => setIsVariablesDrawerOpen(true)}
        onOpenHistory={() => setIsHistoryModalOpen(true)}
        onOpenRefine={() => setIsRefineModalOpen(true)}
        onOpenReview={() => setIsDiffReviewModalOpen(true)}
        onOpenSettings={() => setIsSettingsModalOpen(true)}
      />

      {/* 2. Main Workbench 3-Column Layout */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Column: Section Types Manager */}
        <PresetsSidebar
          sectionTypes={sectionTypes}
          templates={templates}
          currentSections={sections}
          customColors={config?.tag_colors}
          onTagColorChange={handleTagColorChange}
          onInsertType={handleInsertType}
          onOpenCreateType={() => {
            setEditingType(null);
            setIsCreateTypeModalOpen(true);
          }}
          onEditType={(type) => {
            setEditingType(type);
            setIsCreateTypeModalOpen(true);
          }}
          onDeleteType={handleDeleteSectionType}
          onSelectTemplate={(templateId) => handleCreateDocument(templateId)}
          onScrollToSection={scrollToSection}
        />

        {/* Center Column: Section Canvas (Interactive Editor) */}
        <main className="flex-1 flex flex-col overflow-hidden bg-slate-950">
          {/* Canvas Sub-header */}
          <div className="px-6 py-2.5 border-b border-slate-800 bg-slate-900/60 flex items-center justify-between text-xs">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <span className="font-semibold text-slate-200">Sections</span>
                <span className="text-[11px] font-mono text-slate-400 bg-slate-800 px-2 py-0.5 rounded-md border border-slate-700">
                  {sections.length} total
                </span>
              </div>

              {/* Tag filters for document sections */}
              {docSectionTags.length > 0 && (
                <div className="flex items-center gap-1.5 pl-2 border-l border-slate-800">
                  <Tag className="w-3 h-3 text-slate-500" />
                  <button
                    onClick={() => setSelectedCanvasTag(null)}
                    className={`px-2 py-0.5 rounded text-[10px] border transition ${
                      selectedCanvasTag === null
                        ? 'bg-zinc-800 text-zinc-100 border-zinc-600 font-semibold'
                        : 'text-zinc-400 hover:text-zinc-200 bg-zinc-900 border-zinc-800'
                    }`}
                  >
                    All
                  </button>
                  {docSectionTags.map((tag) => (
                    <TagBadge
                      key={tag}
                      tag={tag}
                      size="xs"
                      customColors={config?.tag_colors}
                      isActive={selectedCanvasTag === tag}
                      onClick={() =>
                        setSelectedCanvasTag(selectedCanvasTag === tag ? null : tag)
                      }
                      onColorChange={(colorId) => handleTagColorChange(tag, colorId)}
                    />
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={() => exec({ type: 'AddSection', tag: 'custom_section', brief: '' })}
                className="flex items-center gap-1.5 px-3 py-1 bg-slate-800 hover:bg-slate-750 text-slate-200 font-semibold text-xs rounded-lg border border-slate-700 transition"
              >
                <Plus className="w-3.5 h-3.5 text-amber-400" />
                <span>Add Section</span>
              </button>
            </div>
          </div>

          {/* Section Canvas List */}
          <div className="flex-1 overflow-y-auto p-6 space-y-4">
            {sections.length === 0 ? (
              /* Empty Canvas State */
              <div className="h-full flex flex-col items-center justify-center text-center max-w-md mx-auto p-6 space-y-4 my-auto">
                <div className="w-12 h-12 rounded-2xl bg-slate-900 border border-slate-800 flex items-center justify-center text-amber-400 shadow-md">
                  <Layers className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-slate-100">No prompt sections yet</h3>
                  <p className="text-xs text-slate-400 mt-1 leading-relaxed">
                    Build your prompt by choosing section types from the library on the left, or instantiate an architectural template.
                  </p>
                </div>
                <div className="flex flex-col sm:flex-row items-center gap-2 w-full pt-1">
                  <button
                    onClick={() => setIsNewDocModalOpen(true)}
                    className="flex-1 flex items-center justify-center gap-2 py-2 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold text-xs border border-slate-700 transition w-full"
                  >
                    <FilePlus className="w-4 h-4 text-amber-400" />
                    <span>Choose Template</span>
                  </button>
                  <button
                    onClick={() => exec({ type: 'AddPreset', preset_tag: 'role' })}
                    className="flex-1 flex items-center justify-center gap-2 py-2 px-4 rounded-xl bg-amber-400 hover:bg-amber-300 text-slate-950 font-bold text-xs transition w-full"
                  >
                    <Plus className="w-4 h-4" />
                    <span>Add &lt;role&gt;</span>
                  </button>
                </div>
              </div>
            ) : (
              /* Active Sections List */
              displayedSections.map((section) => {
                const actualIndex = sections.findIndex((s) => s.id === section.id);
                return (
                  <SectionCard
                    key={section.id}
                    section={section}
                    index={actualIndex}
                    totalSections={sections.length}
                    hasPendingChange={pendingMap.has(section.id)}
                    customColors={config?.tag_colors}
                    onTagColorChange={handleTagColorChange}
                    onUpdateBrief={(id, text) => exec({ type: 'UpdateBrief', id, text })}
                    onRenameTag={(id, tag) => exec({ type: 'RenameSection', id, tag })}
                    onUpdateTags={(id, tags) => exec({ type: 'UpdateTags', id, tags })}
                    onToggleLock={(id) => exec({ type: 'ToggleLock', id })}
                    onToggleEnabled={(id) => exec({ type: 'ToggleEnabled', id })}
                    onMoveUp={(id) => exec({ type: 'MoveUp', id })}
                    onMoveDown={(id) => exec({ type: 'MoveDown', id })}
                    onDuplicate={(id) => exec({ type: 'DuplicateSection', id })}
                    onRemove={(id) => exec({ type: 'RemoveSection', id })}
                    onClearRefinement={(id) => exec({ type: 'ClearRefinement', id })}
                  />
                );
              })
            )}
          </div>
        </main>

        {/* Right Column: Live XML Preview & Inspector */}
        <XmlPreviewPanel
          document={docState.document}
          onExportToFile={handleExportToFile}
        />
      </div>

      {/* 3. Dialogs & Modals */}
      <CreateTypeModal
        isOpen={isCreateTypeModalOpen}
        onClose={() => {
          setIsCreateTypeModalOpen(false);
          setEditingType(null);
        }}
        onSave={handleSaveSectionType}
        existingFolders={existingFolders}
        initialType={editingType}
      />

      <RefineModal
        isOpen={isRefineModalOpen}
        onClose={() => setIsRefineModalOpen(false)}
        config={config}
        onRefinementSuccess={(newState) => {
          setDocState(newState);
          setIsDiffReviewModalOpen(true);
          showToast('Refinement complete! Review proposed modifications.', 'success');
        }}
      />

      <DiffReviewModal
        isOpen={isDiffReviewModalOpen}
        onClose={() => setIsDiffReviewModalOpen(false)}
        pending={docState.pending_refinements}
        document={docState.document}
        onUpdateState={(newState) => setDocState(newState)}
      />

      <SettingsModal
        isOpen={isSettingsModalOpen}
        onClose={() => setIsSettingsModalOpen(false)}
        config={config}
        onSaveConfig={handleSaveConfig}
      />

      <NewDocumentModal
        isOpen={isNewDocModalOpen}
        onClose={() => setIsNewDocModalOpen(false)}
        templates={templates}
        onCreate={handleCreateDocument}
      />

      <ExportSkillModal
        isOpen={isExportSkillModalOpen}
        onClose={() => setIsExportSkillModalOpen(false)}
        defaultTitle={docState?.document.title || ''}
        defaultDescription={docState?.document.description || ''}
      />

      <ImportPromptModal
        isOpen={isImportPromptModalOpen}
        onClose={() => setIsImportPromptModalOpen(false)}
        onImportSuccess={(newState) => {
          setDocState(newState);
          showToast('Prompt imported successfully', 'success');
        }}
      />

      <PromptLibraryModal
        isOpen={isPromptLibraryModalOpen}
        onClose={() => setIsPromptLibraryModalOpen(false)}
        onLoadPrompt={(newState) => {
          setDocState(newState);
          showToast(`Opened "${newState.document.title}" from Library`, 'success');
        }}
        onSaveCurrentToLibrary={handleSaveCurrentToLibrary}
        customColors={config?.tag_colors}
      />

      <VariablePlaygroundDrawer
        isOpen={isVariablesDrawerOpen}
        onClose={() => setIsVariablesDrawerOpen(false)}
        docState={docState}
        onSaveScenario={handleSaveScenario}
        onDeleteScenario={handleDeleteScenario}
        customColors={config?.tag_colors}
      />

      <RevisionHistoryModal
        isOpen={isHistoryModalOpen}
        onClose={() => setIsHistoryModalOpen(false)}
        docState={docState}
        onCreateSnapshot={handleCreateSnapshot}
        onRestoreSnapshot={handleRestoreSnapshot}
        onDeleteSnapshot={handleDeleteSnapshot}
        onForkSnapshot={handleForkSnapshot}
      />

      {/* 4. Global Toast Notifications */}
      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm pointer-events-none">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`pointer-events-auto flex items-center justify-between gap-3 px-4 py-2.5 rounded-xl shadow-xl border text-xs font-medium animate-in slide-in-from-bottom-3 duration-200 ${
              t.type === 'success'
                ? 'bg-slate-900/95 text-slate-200 border-emerald-500/40'
                : t.type === 'error'
                ? 'bg-slate-900/95 text-red-200 border-red-500/40'
                : 'bg-slate-900/95 text-slate-200 border-slate-700'
            }`}
          >
            <div className="flex items-center gap-2">
              {t.type === 'success' && <CheckCircle className="w-4 h-4 text-emerald-400 shrink-0" />}
              {t.type === 'error' && <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />}
              {t.type === 'info' && <Info className="w-4 h-4 text-amber-400 shrink-0" />}
              <span className="leading-tight">{t.message}</span>
            </div>
            <button
              onClick={() => setToasts((prev) => prev.filter((item) => item.id !== t.id))}
              className="text-slate-400 hover:text-white"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
export default App;
