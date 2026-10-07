import React, { useState, useMemo } from 'react';
import {
  Folder,
  FolderOpen,
  Plus,
  Search,
  LayoutTemplate,
  ListOrdered,
  ChevronDown,
  ChevronRight,
  Lock,
  EyeOff,
  Edit2,
  Trash2,
  Library,
  Sparkles,
} from 'lucide-react';
import type { SectionType, StarterTemplate, PromptSection } from '../types';

interface PresetsSidebarProps {
  sectionTypes: SectionType[];
  templates: StarterTemplate[];
  currentSections: PromptSection[];
  onInsertType: (type: SectionType) => void;
  onOpenCreateType: () => void;
  onEditType: (type: SectionType) => void;
  onDeleteType: (id: string) => void;
  onSelectTemplate: (templateId: string) => void;
  onScrollToSection: (sectionId: string) => void;
}

export const PresetsSidebar: React.FC<PresetsSidebarProps> = ({
  sectionTypes,
  templates,
  currentSections,
  onInsertType,
  onOpenCreateType,
  onEditType,
  onDeleteType,
  onSelectTemplate,
  onScrollToSection,
}) => {
  const [activeTab, setActiveTab] = useState<'types' | 'templates' | 'outline'>('types');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [collapsedFolders, setCollapsedFolders] = useState<Record<string, boolean>>({});

  // Compute all unique tags across section types
  const allTags = useMemo(() => {
    const set = new Set<string>();
    for (const st of sectionTypes) {
      if (st.tags) {
        for (const t of st.tags) {
          set.add(t);
        }
      }
    }
    return Array.from(set).sort();
  }, [sectionTypes]);

  // Filter section types by search and tag
  const filteredTypes = useMemo(() => {
    return sectionTypes.filter((st) => {
      const q = searchQuery.toLowerCase().trim();
      const matchesSearch =
        !q ||
        st.name.toLowerCase().includes(q) ||
        st.tag.toLowerCase().includes(q) ||
        st.description.toLowerCase().includes(q) ||
        st.folder.toLowerCase().includes(q) ||
        (st.tags && st.tags.some((t) => t.toLowerCase().includes(q)));

      const matchesTag = !selectedTag || (st.tags && st.tags.includes(selectedTag));
      return matchesSearch && matchesTag;
    });
  }, [sectionTypes, searchQuery, selectedTag]);

  // Group filtered section types by folder
  const groupedFolders = useMemo(() => {
    const groups: Record<string, SectionType[]> = {};
    for (const st of filteredTypes) {
      const f = st.folder || 'General';
      if (!groups[f]) groups[f] = [];
      groups[f].push(st);
    }
    return groups;
  }, [filteredTypes]);

  const toggleFolder = (folderName: string) => {
    setCollapsedFolders((prev) => ({
      ...prev,
      [folderName]: !prev[folderName],
    }));
  };

  return (
    <aside className="w-80 border-r border-slate-800 bg-slate-900/70 flex flex-col h-full select-none shrink-0">
      {/* Top Tab Bar */}
      <div className="flex border-b border-slate-800 bg-slate-900/90 text-xs font-medium text-slate-400 p-1.5 gap-1">
        <button
          onClick={() => setActiveTab('types')}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md transition ${
            activeTab === 'types'
              ? 'bg-slate-800 text-slate-100 font-semibold shadow-sm'
              : 'hover:text-slate-200 hover:bg-slate-800/40'
          }`}
          title="Section Types & Components Library"
        >
          <Library className="w-3.5 h-3.5 text-amber-400" />
          <span>Types</span>
        </button>
        <button
          onClick={() => setActiveTab('templates')}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md transition ${
            activeTab === 'templates'
              ? 'bg-slate-800 text-slate-100 font-semibold shadow-sm'
              : 'hover:text-slate-200 hover:bg-slate-800/40'
          }`}
          title="Starter Templates"
        >
          <LayoutTemplate className="w-3.5 h-3.5 text-slate-400" />
          <span>Templates</span>
        </button>
        <button
          onClick={() => setActiveTab('outline')}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md transition ${
            activeTab === 'outline'
              ? 'bg-slate-800 text-slate-100 font-semibold shadow-sm'
              : 'hover:text-slate-200 hover:bg-slate-800/40'
          }`}
          title="Document Outline"
        >
          <ListOrdered className="w-3.5 h-3.5 text-slate-400" />
          <span>Outline</span>
        </button>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto flex flex-col">
        {/* TAB 1: TYPES & COMPONENT LIBRARY */}
        {activeTab === 'types' && (
          <div className="p-3 space-y-3 flex-1 flex flex-col">
            {/* Search & New Type action */}
            <div className="flex items-center gap-2">
              <div className="relative flex-1">
                <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                <input
                  type="text"
                  placeholder="Search types, tags..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg pl-8 pr-2.5 py-1.5 text-xs text-slate-200 placeholder-slate-500 outline-none focus:border-slate-700 transition"
                />
              </div>

              <button
                onClick={onOpenCreateType}
                className="flex items-center gap-1 px-2.5 py-1.5 bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded-lg text-xs font-semibold transition shrink-0"
                title="Create a new custom Section Type"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>New</span>
              </button>
            </div>

            {/* Tags Filter Chips */}
            {allTags.length > 0 && (
              <div className="flex items-center gap-1 overflow-x-auto pb-1 text-[11px] scrollbar-none">
                <button
                  onClick={() => setSelectedTag(null)}
                  className={`px-2 py-0.5 rounded-md transition shrink-0 ${
                    selectedTag === null
                      ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30 font-medium'
                      : 'text-slate-400 hover:text-slate-200 bg-slate-800/40 border border-transparent'
                  }`}
                >
                  All
                </button>
                {allTags.map((tag) => (
                  <button
                    key={tag}
                    onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
                    className={`px-2 py-0.5 rounded-md transition shrink-0 ${
                      selectedTag === tag
                        ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30 font-medium'
                        : 'text-slate-400 hover:text-slate-200 bg-slate-800/40 border border-transparent'
                    }`}
                  >
                    #{tag}
                  </button>
                ))}
              </div>
            )}

            {/* Folder Tree Groups */}
            <div className="space-y-2 flex-1">
              {Object.keys(groupedFolders).length === 0 ? (
                <div className="py-8 text-center text-xs text-slate-500 italic">
                  No section types match criteria.
                </div>
              ) : (
                Object.entries(groupedFolders).map(([folderName, typesInFolder]) => {
                  const isCollapsed = collapsedFolders[folderName];
                  return (
                    <div
                      key={folderName}
                      className="rounded-xl border border-slate-800/80 bg-slate-950/40 overflow-hidden"
                    >
                      {/* Folder Accordion Header */}
                      <button
                        onClick={() => toggleFolder(folderName)}
                        className="w-full flex items-center justify-between px-3 py-2 bg-slate-800/40 hover:bg-slate-800/70 text-slate-300 hover:text-white transition text-xs font-semibold"
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          {isCollapsed ? (
                            <Folder className="w-3.5 h-3.5 text-amber-400/80" />
                          ) : (
                            <FolderOpen className="w-3.5 h-3.5 text-amber-400" />
                          )}
                          <span className="truncate">{folderName}</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-slate-500">
                          <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-slate-900 border border-slate-800">
                            {typesInFolder.length}
                          </span>
                          {isCollapsed ? (
                            <ChevronRight className="w-3.5 h-3.5" />
                          ) : (
                            <ChevronDown className="w-3.5 h-3.5" />
                          )}
                        </div>
                      </button>

                      {/* Folder Content: Section Types List */}
                      {!isCollapsed && (
                        <div className="p-2 space-y-1.5 bg-slate-950/20">
                          {typesInFolder.map((st) => {
                            const isPresent = currentSections.some((s) => s.tag === st.tag);
                            return (
                              <div
                                key={st.id}
                                className="group p-2 rounded-lg bg-slate-900/60 hover:bg-slate-850 border border-slate-800/80 hover:border-slate-700 transition flex flex-col gap-1.5"
                              >
                                <div className="flex items-center justify-between">
                                  <div className="flex items-center gap-1.5 min-w-0">
                                    <span className="text-xs font-semibold text-slate-200 truncate">
                                      {st.name}
                                    </span>
                                    <span className="text-[10px] font-mono text-amber-400 bg-slate-950 px-1 py-0.2 rounded border border-slate-800 shrink-0">
                                      &lt;{st.tag}&gt;
                                    </span>
                                  </div>

                                  {/* Action Buttons */}
                                  <div className="flex items-center gap-1 shrink-0">
                                    {!st.is_builtin && (
                                      <>
                                        <button
                                          onClick={() => onEditType(st)}
                                          className="p-1 rounded text-slate-500 hover:text-slate-200 hover:bg-slate-800 transition"
                                          title="Edit Section Type"
                                        >
                                          <Edit2 className="w-3 h-3" />
                                        </button>
                                        <button
                                          onClick={() => onDeleteType(st.id)}
                                          className="p-1 rounded text-slate-500 hover:text-red-400 hover:bg-red-500/10 transition"
                                          title="Delete Section Type"
                                        >
                                          <Trash2 className="w-3 h-3" />
                                        </button>
                                      </>
                                    )}

                                    <button
                                      onClick={() => onInsertType(st)}
                                      className="flex items-center gap-0.5 px-2 py-0.5 rounded bg-slate-800 hover:bg-amber-400 hover:text-slate-950 text-slate-300 text-[11px] font-semibold transition"
                                      title={`Insert <${st.tag}> into document`}
                                    >
                                      <Plus className="w-3 h-3" />
                                      <span>Add</span>
                                    </button>
                                  </div>
                                </div>

                                {st.description && (
                                  <p className="text-[11px] text-slate-400 leading-tight line-clamp-2">
                                    {st.description}
                                  </p>
                                )}

                                {/* Tags pills */}
                                {st.tags && st.tags.length > 0 && (
                                  <div className="flex flex-wrap gap-1 mt-0.5">
                                    {st.tags.map((t) => (
                                      <span
                                        key={t}
                                        className="text-[10px] text-slate-400 bg-slate-950 px-1.5 py-0.2 rounded border border-slate-800/80"
                                      >
                                        #{t}
                                      </span>
                                    ))}
                                  </div>
                                )}

                                {isPresent && (
                                  <span className="text-[10px] text-slate-500 italic">
                                    In document
                                  </span>
                                )}
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}

        {/* TAB 2: STARTER TEMPLATES */}
        {activeTab === 'templates' && (
          <div className="p-3 space-y-3">
            <p className="text-xs text-slate-400 leading-relaxed">
              Instantiate a full architectural prompt scaffolding.
            </p>
            <div className="space-y-2.5">
              {templates.map((tmpl) => (
                <div
                  key={tmpl.id}
                  className="p-3 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 transition flex flex-col gap-2"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-bold text-slate-100">{tmpl.name}</span>
                    <span className="text-[10px] text-slate-400 bg-slate-900 px-1.5 py-0.5 rounded border border-slate-800">
                      {tmpl.presets.length} sections
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-400 leading-normal">{tmpl.description}</p>
                  <div className="flex flex-wrap gap-1">
                    {tmpl.presets.map((p) => (
                      <span
                        key={p.tag}
                        className="text-[10px] font-mono bg-slate-900 text-slate-300 px-1.5 py-0.5 rounded border border-slate-800"
                      >
                        &lt;{p.tag}&gt;
                      </span>
                    ))}
                  </div>
                  <button
                    onClick={() => onSelectTemplate(tmpl.id)}
                    className="mt-1 flex items-center justify-center gap-1.5 w-full py-1.5 text-xs font-semibold text-slate-200 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition"
                  >
                    <span>Use Template</span>
                    <ChevronRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* TAB 3: DOCUMENT OUTLINE */}
        {activeTab === 'outline' && (
          <div className="p-3 space-y-2">
            <div className="flex items-center justify-between text-xs text-slate-400 pb-1 border-b border-slate-800">
              <span className="font-semibold text-slate-300">Document Outline</span>
              <span className="font-mono">{currentSections.length} sections</span>
            </div>
            {currentSections.length === 0 ? (
              <p className="text-xs text-slate-500 italic py-6 text-center">No sections in document</p>
            ) : (
              <div className="space-y-1">
                {currentSections.map((sec, idx) => (
                  <button
                    key={sec.id}
                    onClick={() => onScrollToSection(sec.id)}
                    className="w-full flex items-center justify-between p-2 rounded-lg bg-slate-950/40 hover:bg-slate-800 border border-slate-800/80 transition text-left group"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="text-[10px] font-mono text-slate-500 w-4">{idx + 1}.</span>
                      <span className="text-xs font-mono font-medium text-slate-200 truncate">
                        &lt;{sec.tag}&gt;
                      </span>
                    </div>
                    <div className="flex items-center gap-1 shrink-0 text-slate-500">
                      {sec.tags && sec.tags.length > 0 && (
                        <span className="text-[10px] text-slate-400 font-sans">
                          #{sec.tags[0]}
                        </span>
                      )}
                      {sec.refined && (
                        <span title="Refined version available">
                          <Sparkles className="w-3 h-3 text-emerald-400" />
                        </span>
                      )}
                      {sec.locked && (
                        <span title="Section is locked">
                          <Lock className="w-3 h-3 text-amber-400" />
                        </span>
                      )}
                      {!sec.enabled && (
                        <span title="Section is disabled">
                          <EyeOff className="w-3 h-3 text-slate-500" />
                        </span>
                      )}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </aside>
  );
};
