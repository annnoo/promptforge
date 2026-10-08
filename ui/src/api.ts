import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
  AppConfig,
  Command,
  DiffLine,
  DocumentStateDto,
  ImportFileContent,
  ImportPreviewDto,
  OpenAiCompatibleConfig,
  SavedPromptSummary,
  ScenarioPreviewDto,
  SectionPreset,
  SectionType,
  SnapshotComparison,
  StarterTemplate,
} from './types';

// Fallback mock state for standalone web browser preview
const MOCK_PRESETS: SectionPreset[] = [
  {
    name: 'Role',
    tag: 'role',
    description: 'The persona, seniority, or specialized perspective the LLM should adopt.',
    default_brief: 'Expert software engineer and systems architect specializing in high-reliability Rust.',
  },
  {
    name: 'Context',
    tag: 'context',
    description: 'Background information, domain knowledge, or existing system setup.',
    default_brief: 'We are architecting a high-throughput event processing pipeline with strict zero-allocation limits.',
  },
  {
    name: 'Objective',
    tag: 'objective',
    description: 'High-level goal or business outcome to achieve.',
    default_brief: 'Design a resilient actor-based state machine capable of processing 250k msgs/sec.',
  },
  {
    name: 'Task',
    tag: 'task',
    description: 'Specific, actionable instructions for what the LLM must execute or produce.',
    default_brief: '1. Detail memory layout and cache lines.\n2. Provide Tokio/crossbeam channel topology.\n3. Include concrete error handling and backpressure strategies.',
  },
  {
    name: 'Requirements',
    tag: 'requirements',
    description: 'Functional and technical requirements that must be met.',
    default_brief: '- Zero dynamic heap allocation in hot loops\n- Comprehensive fuzz tests with cargo-fuzz\n- Graceful shutdown under SIGINT/SIGTERM',
  },
  {
    name: 'Constraints',
    tag: 'constraints',
    description: 'Boundaries, limitations, things to avoid, or mandatory restrictions.',
    default_brief: '- Do not use unsafe blocks without formal invariant proofs\n- No third-party network crates outside tokio and hyper\n- Rust edition 2021',
  },
  {
    name: 'Input',
    tag: 'input',
    description: 'Data, schema, code, or materials provided for the task.',
    default_brief: 'Protocol buffers schema definitions and sample telemetry payloads.',
  },
  {
    name: 'Output Format',
    tag: 'output_format',
    description: 'Exact structure, format, schema, or packaging expected.',
    default_brief: 'Markdown architecture RFC document with embedded Rust snippets and ASCII sequence diagrams.',
  },
  {
    name: 'Examples',
    tag: 'examples',
    description: 'Few-shot demonstrations or expected input/output pairs.',
    default_brief: 'Example input telemetry packet and corresponding parsed ring-buffer entry.',
  },
  {
    name: 'Chain of Thought',
    tag: 'chain_of_thought',
    description: 'Reasoning process or step-by-step thinking instructions.',
    default_brief: 'Analyze concurrency hazards first, verify queue contention, then draft data structures.',
  },
  {
    name: 'Evaluation',
    tag: 'evaluation',
    description: 'Acceptance tests, self-check criteria, or verification steps.',
    default_brief: 'Verify that zero unwrap() calls exist and memory consumption does not exceed 64MB.',
  },
  {
    name: 'Style',
    tag: 'style',
    description: 'Tone, voice, formatting, and prose style preferences.',
    default_brief: 'Concise, rigorous, technical prose with exact type signatures and performance notes.',
  },
  {
    name: 'Negative Rules',
    tag: 'negative_rules',
    description: 'Explicit list of what the LLM must NEVER do or output.',
    default_brief: '- Never output generic boilerplate or filler phrases\n- Never skip edge cases or error branches',
  },
];

const MOCK_TEMPLATES: StarterTemplate[] = [
  {
    id: 'general',
    name: 'General Task',
    description: 'Well-structured foundation for general-purpose AI tasks.',
    presets: [
      MOCK_PRESETS[0], // role
      MOCK_PRESETS[1], // context
      MOCK_PRESETS[3], // task
      MOCK_PRESETS[5], // constraints
      MOCK_PRESETS[7], // output_format
    ],
  },
  {
    id: 'engineering',
    name: 'Software Engineering Task',
    description: 'Specialized prompt structure for software architecture, coding, and debugging tasks.',
    presets: [
      MOCK_PRESETS[0], // role
      MOCK_PRESETS[1], // context
      MOCK_PRESETS[2], // objective
      MOCK_PRESETS[3], // task
      MOCK_PRESETS[4], // requirements
      MOCK_PRESETS[5], // constraints
      MOCK_PRESETS[10], // evaluation
    ],
  },
  {
    id: 'research',
    name: 'Research and Analysis',
    description: 'Structured prompt for deep-dive investigation, synthesis, and report generation.',
    presets: [
      MOCK_PRESETS[0], // role
      MOCK_PRESETS[1], // context
      MOCK_PRESETS[3], // task
      MOCK_PRESETS[6], // input
      MOCK_PRESETS[5], // constraints
      MOCK_PRESETS[7], // output_format
      MOCK_PRESETS[11], // style
    ],
  },
];

let mockSectionTypes: SectionType[] = [
  {
    id: 'builtin-role',
    name: 'Role',
    tag: 'role',
    description: 'The persona, seniority, or specialized perspective the LLM should adopt.',
    default_brief: 'Expert software engineer and systems architect specializing in high-reliability Rust.',
    folder: 'Core Structure',
    tags: ['persona', 'architecture'],
    is_builtin: true,
  },
  {
    id: 'builtin-context',
    name: 'Context',
    tag: 'context',
    description: 'Background information, domain knowledge, or existing system setup.',
    default_brief: 'We are architecting a high-throughput event processing pipeline with strict zero-allocation limits.',
    folder: 'Core Structure',
    tags: ['background', 'domain'],
    is_builtin: true,
  },
  {
    id: 'builtin-objective',
    name: 'Objective',
    tag: 'objective',
    description: 'High-level goal or business outcome to achieve.',
    default_brief: 'Design a resilient actor-based state machine capable of processing 250k msgs/sec.',
    folder: 'Core Structure',
    tags: ['goals', 'business'],
    is_builtin: true,
  },
  {
    id: 'builtin-task',
    name: 'Task',
    tag: 'task',
    description: 'Specific, actionable instructions for what the LLM must execute or produce.',
    default_brief: '1. Detail memory layout and cache lines.\n2. Provide Tokio/crossbeam channel topology.\n3. Include concrete error handling and backpressure strategies.',
    folder: 'Core Structure',
    tags: ['instructions', 'core'],
    is_builtin: true,
  },
  {
    id: 'builtin-requirements',
    name: 'Requirements',
    tag: 'requirements',
    description: 'Functional and technical requirements that must be met.',
    default_brief: '- Zero dynamic heap allocation in hot loops\n- Comprehensive fuzz tests with cargo-fuzz\n- Graceful shutdown under SIGINT/SIGTERM',
    folder: 'Rules & Boundaries',
    tags: ['specs', 'criteria'],
    is_builtin: true,
  },
  {
    id: 'builtin-constraints',
    name: 'Constraints',
    tag: 'constraints',
    description: 'Boundaries, limitations, things to avoid, or mandatory restrictions.',
    default_brief: '- Do not use unsafe blocks without formal invariant proofs\n- No third-party network crates outside tokio and hyper\n- Rust edition 2021',
    folder: 'Rules & Boundaries',
    tags: ['limits', 'strict'],
    is_builtin: true,
  },
  {
    id: 'builtin-negative-rules',
    name: 'Negative Rules',
    tag: 'negative_rules',
    description: 'Explicit negative guardrails and prohibited patterns or outputs.',
    default_brief: '- Never use deprecated APIs\n- Never produce markdown code without syntax language tags\n- Never assume unverified external inputs',
    folder: 'Rules & Boundaries',
    tags: ['safety', 'prohibitions'],
    is_builtin: true,
  },
  {
    id: 'builtin-input',
    name: 'Input',
    tag: 'input',
    description: 'Data, schema, code, or materials provided for the task.',
    default_brief: 'Protocol buffers schema definitions and sample telemetry payloads.',
    folder: 'Data & Schema',
    tags: ['data', 'schema'],
    is_builtin: true,
  },
  {
    id: 'builtin-output-format',
    name: 'Output Format',
    tag: 'output_format',
    description: 'Exact structure, format, schema, or packaging expected.',
    default_brief: 'Markdown architecture RFC document with embedded Rust snippets and ASCII sequence diagrams.',
    folder: 'Data & Schema',
    tags: ['format', 'schema'],
    is_builtin: true,
  },
  {
    id: 'builtin-examples',
    name: 'Examples',
    tag: 'examples',
    description: 'Few-shot demonstrations or expected input/output pairs.',
    default_brief: 'Example input telemetry packet and corresponding parsed ring-buffer entry.',
    folder: 'Data & Schema',
    tags: ['few-shot', 'patterns'],
    is_builtin: true,
  },
  {
    id: 'builtin-chain-of-thought',
    name: 'Chain of Thought',
    tag: 'chain_of_thought',
    description: 'Reasoning process or step-by-step thinking instructions.',
    default_brief: 'Analyze concurrency hazards first, verify queue contention, then draft data structures.',
    folder: 'Reasoning & Quality',
    tags: ['reasoning', 'logic'],
    is_builtin: true,
  },
  {
    id: 'builtin-evaluation',
    name: 'Evaluation',
    tag: 'evaluation',
    description: 'Acceptance tests, self-check criteria, or verification steps.',
    default_brief: 'Verify that zero unwrap() calls exist and memory consumption does not exceed 64MB.',
    folder: 'Reasoning & Quality',
    tags: ['testing', 'qa'],
    is_builtin: true,
  },
  {
    id: 'builtin-style',
    name: 'Style',
    tag: 'style',
    description: 'Tone, voice, formatting, and prose style preferences.',
    default_brief: 'Concise, rigorous, technical prose with exact type signatures and performance notes.',
    folder: 'Reasoning & Quality',
    tags: ['tone', 'guidelines'],
    is_builtin: true,
  },
];

let mockState: DocumentStateDto = {
  document: {
    schema_version: 1,
    id: 'b1234567-89ab-cdef-0123-456789abcdef',
    title: 'High-Performance Rust Architecture RFC',
    description: 'Prompt specification for designing zero-allocation event processing microservices.',
    sections: [
      {
        id: '11111111-1111-1111-1111-111111111111',
        tag: 'role',
        brief: 'Expert software engineer and systems architect specializing in high-reliability Rust.',
        refined: 'You are a Principal Systems Architect and Staff Rust Engineer with over a decade of production experience designing ultra-low latency, mission-critical infrastructure. You possess deep knowledge of the Linux kernel, cache hierarchy, memory-mapped I/O, and lock-free concurrency primitives.',
        locked: false,
        enabled: true,
      },
      {
        id: '22222222-2222-2222-2222-222222222222',
        tag: 'context',
        brief: 'We are architecting a high-throughput event processing pipeline with strict zero-allocation limits.',
        refined: null,
        locked: false,
        enabled: true,
      },
      {
        id: '33333333-3333-3333-3333-333333333333',
        tag: 'task',
        brief: '1. Detail memory layout and cache lines.\n2. Provide Tokio/crossbeam channel topology.\n3. Include concrete error handling and backpressure strategies.',
        refined: null,
        locked: false,
        enabled: true,
      },
      {
        id: '44444444-4444-4444-4444-444444444444',
        tag: 'constraints',
        brief: '- Do not use unsafe blocks without formal invariant proofs\n- No third-party network crates outside tokio and hyper\n- Rust edition 2021',
        refined: null,
        locked: true,
        enabled: true,
      },
    ],
  },
  dirty: false,
  current_file_path: 'examples/software_architecture.prompt.json',
  can_undo: true,
  can_redo: false,
  pending_refinements: null,
};

let mockConfig: AppConfig = {
  active_provider: 'manual',
  openai_compatible: {
    base_url: 'https://api.openai.com/v1',
    model: 'gpt-4o',
    api_key_env_var: 'OPENAI_API_KEY',
    timeout_seconds: 60,
  },
  recent_files: ['examples/software_architecture.prompt.json'],
  theme: 'dark',
};

export const api = {
  async getDocumentState(): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('get_document_state');
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async executeCommand(command: Command): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('execute_command', { command });
    }
    // Simple mock execution
    mockState.dirty = true;
    if (command.type === 'UpdateTitle') {
      mockState.document.title = command.title;
    } else if (command.type === 'UpdateDescription') {
      mockState.document.description = command.description;
    } else if (command.type === 'UpdateBrief') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.brief = command.text;
    } else if (command.type === 'RenameSection') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.tag = command.tag;
    } else if (command.type === 'ToggleLock') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.locked = !sec.locked;
    } else if (command.type === 'ToggleEnabled') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.enabled = !sec.enabled;
    } else if (command.type === 'RemoveSection') {
      mockState.document.sections = mockState.document.sections.filter((s) => s.id !== command.id);
    } else if (command.type === 'DuplicateSection') {
      const idx = mockState.document.sections.findIndex((s) => s.id === command.id);
      if (idx !== -1) {
        const orig = mockState.document.sections[idx];
        const copy = {
          ...orig,
          id: crypto.randomUUID ? crypto.randomUUID() : `sec-${Date.now()}`,
          tag: `${orig.tag}_copy`,
        };
        mockState.document.sections.splice(idx + 1, 0, copy);
      }
    } else if (command.type === 'MoveUp') {
      const idx = mockState.document.sections.findIndex((s) => s.id === command.id);
      if (idx > 0) {
        const item = mockState.document.sections.splice(idx, 1)[0];
        mockState.document.sections.splice(idx - 1, 0, item);
      }
    } else if (command.type === 'MoveDown') {
      const idx = mockState.document.sections.findIndex((s) => s.id === command.id);
      if (idx !== -1 && idx < mockState.document.sections.length - 1) {
        const item = mockState.document.sections.splice(idx, 1)[0];
        mockState.document.sections.splice(idx + 1, 0, item);
      }
    } else if (command.type === 'AddSection') {
      mockState.document.sections.push({
        id: crypto.randomUUID ? crypto.randomUUID() : `sec-${Date.now()}`,
        tag: command.tag,
        brief: command.brief,
        refined: null,
        locked: false,
        enabled: true,
      });
    } else if (command.type === 'AddPreset') {
      const p = MOCK_PRESETS.find((x) => x.tag === command.preset_tag);
      if (p) {
        mockState.document.sections.push({
          id: crypto.randomUUID ? crypto.randomUUID() : `sec-${Date.now()}`,
          tag: p.tag,
          brief: p.default_brief,
          refined: null,
          locked: false,
          enabled: true,
        });
      }
    } else if (command.type === 'UpdateTags') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.tags = command.tags;
    } else if (command.type === 'ClearRefinement') {
      const sec = mockState.document.sections.find((s) => s.id === command.id);
      if (sec) sec.refined = null;
    } else if (command.type === 'SaveScenario') {
      if (!mockState.document.scenarios) mockState.document.scenarios = [];
      const idx = mockState.document.scenarios.findIndex((s) => s.id === command.scenario.id);
      if (idx !== -1) {
        mockState.document.scenarios[idx] = command.scenario;
      } else {
        mockState.document.scenarios.push(command.scenario);
      }
    } else if (command.type === 'DeleteScenario') {
      if (mockState.document.scenarios) {
        mockState.document.scenarios = mockState.document.scenarios.filter((s) => s.id !== command.id);
      }
    } else if (command.type === 'CreateSnapshot') {
      if (!mockState.document.snapshots) mockState.document.snapshots = [];
      mockState.document.snapshots.push({
        id: crypto.randomUUID ? crypto.randomUUID() : `snap-${Date.now()}`,
        name: command.name,
        description: command.description,
        created_at: new Date().toISOString(),
        title: mockState.document.title,
        sections: JSON.parse(JSON.stringify(mockState.document.sections)),
      });
    } else if (command.type === 'RestoreSnapshot') {
      const snap = mockState.document.snapshots?.find((s) => s.id === command.id);
      if (snap) {
        mockState.document.title = snap.title;
        mockState.document.sections = JSON.parse(JSON.stringify(snap.sections));
      }
    } else if (command.type === 'DeleteSnapshot') {
      if (mockState.document.snapshots) {
        mockState.document.snapshots = mockState.document.snapshots.filter((s) => s.id !== command.id);
      }
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async newDocument(templateId?: string, title?: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('new_document', {
        templateId: templateId || null,
        title: title || null,
      });
    }
    const tmpl = templateId ? MOCK_TEMPLATES.find((t) => t.id === templateId) : null;
    mockState = {
      document: {
        schema_version: 1,
        id: crypto.randomUUID ? crypto.randomUUID() : `doc-${Date.now()}`,
        title: title || (tmpl ? tmpl.name : 'Untitled Prompt'),
        description: tmpl ? tmpl.description : '',
        sections: tmpl
          ? tmpl.presets.map((p) => ({
              id: crypto.randomUUID ? crypto.randomUUID() : `sec-${Math.random()}`,
              tag: p.tag,
              brief: p.default_brief,
              refined: null,
              locked: false,
              enabled: true,
            }))
          : [],
      },
      dirty: false,
      current_file_path: null,
      can_undo: false,
      can_redo: false,
      pending_refinements: null,
    };
    return JSON.parse(JSON.stringify(mockState));
  },

  async loadDocument(path: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('load_document', { path });
    }
    mockState.current_file_path = path;
    mockState.dirty = false;
    return JSON.parse(JSON.stringify(mockState));
  },

  async saveDocument(path?: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('save_document', { path: path || null });
    }
    if (path) mockState.current_file_path = path;
    mockState.dirty = false;
    return JSON.parse(JSON.stringify(mockState));
  },

  async renderXmlContent(stage: 'draft' | 'final', clean: boolean): Promise<string> {
    if (isTauri()) {
      return invoke<string>('render_xml_content', { stage, clean });
    }
    const lines: string[] = ['<prompt>'];
    for (const sec of mockState.document.sections) {
      if (!sec.enabled) continue;
      const text = stage === 'final' && sec.refined ? sec.refined : sec.brief;
      if (clean) {
        lines.push(`  <${sec.tag}>\n    ${text.split('\n').join('\n    ')}\n  </${sec.tag}>`);
      } else {
        lines.push(`  <${sec.tag} id="${sec.id}">\n    ${text.split('\n').join('\n    ')}\n  </${sec.tag}>`);
      }
    }
    lines.push('</prompt>');
    return lines.join('\n');
  },

  async getPresets(): Promise<SectionPreset[]> {
    if (isTauri()) {
      return invoke<SectionPreset[]>('get_presets');
    }
    return MOCK_PRESETS;
  },

  async getSectionTypes(): Promise<SectionType[]> {
    if (isTauri()) {
      return invoke<SectionType[]>('get_section_types');
    }
    return JSON.parse(JSON.stringify(mockSectionTypes));
  },

  async saveSectionType(sectionType: SectionType): Promise<SectionType[]> {
    if (isTauri()) {
      return invoke<SectionType[]>('save_section_type', { sectionType });
    }
    const idx = mockSectionTypes.findIndex((t) => t.id === sectionType.id);
    if (idx !== -1) {
      mockSectionTypes[idx] = sectionType;
    } else {
      mockSectionTypes.push(sectionType);
    }
    return JSON.parse(JSON.stringify(mockSectionTypes));
  },

  async deleteSectionType(id: string): Promise<SectionType[]> {
    if (isTauri()) {
      return invoke<SectionType[]>('delete_section_type', { id });
    }
    mockSectionTypes = mockSectionTypes.filter((t) => t.id !== id);
    return JSON.parse(JSON.stringify(mockSectionTypes));
  },

  async getTemplates(): Promise<StarterTemplate[]> {
    if (isTauri()) {
      return invoke<StarterTemplate[]>('get_templates');
    }
    return MOCK_TEMPLATES;
  },

  async getConfig(): Promise<AppConfig> {
    if (isTauri()) {
      return invoke<AppConfig>('get_config');
    }
    return JSON.parse(JSON.stringify(mockConfig));
  },

  async saveConfig(config: AppConfig): Promise<void> {
    if (isTauri()) {
      return invoke<void>('save_config', { config });
    }
    mockConfig = JSON.parse(JSON.stringify(config));
  },

  async testAiConnection(config: OpenAiCompatibleConfig): Promise<string> {
    if (isTauri()) {
      return invoke<string>('test_ai_connection', { config });
    }
    await new Promise((r) => setTimeout(r, 600));
    if (config.base_url.includes('invalid')) {
      throw new Error('Could not resolve host name (mock connection error)');
    }
    return `Successfully connected to endpoint! Model '${config.model}' is ready.`;
  },

  async generateManualRequest(mode: string): Promise<string> {
    if (isTauri()) {
      return invoke<string>('generate_manual_request', { mode });
    }
    return `PROMPT REFINEMENT INSTRUCTIONS (${mode.toUpperCase()} MODE)\n\nPlease refine the following prompt document by enhancing clarity, removing ambiguity, and completing constraints.\n\nReturn your refined recommendations in the standardized PromptForge JSON changeset format.`;
  },

  async applyManualResponse(responseText: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('apply_manual_response', { responseText });
    }
    // Mock diff
    const targetSec = mockState.document.sections[1] || mockState.document.sections[0];
    if (targetSec) {
      mockState.pending_refinements = {
        changes: [
          {
            section_id: targetSec.id,
            proposed_text: `${targetSec.brief}\n\n[Refined with strict performance boundaries, zero-allocation buffers, and tokio tracing metrics.]`,
            reason: 'Strengthened operational context and runtime boundaries.',
          },
        ],
        suggested_sections: [
          {
            tag: 'telemetry',
            content: 'Emit structured JSON spans at INFO level with latency histograms.',
            reason: 'Missing observability requirement in the original specification.',
          },
        ],
        open_questions: ['What is the maximum allowable P99 latency in microseconds?'],
        warnings: ['Ensure crossbeam ring buffer capacities are power-of-two.'],
        critique: 'The prompt establishes strong architectural constraints but lacked telemetry specifications.',
      };
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async executeAutomatedRefinement(mode: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('execute_automated_refinement', { mode });
    }
    return this.applyManualResponse('');
  },

  async acceptRefinement(sectionId: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('accept_refinement', { sectionId });
    }
    if (mockState.pending_refinements) {
      const change = mockState.pending_refinements.changes.find((c) => c.section_id === sectionId);
      if (change) {
        const sec = mockState.document.sections.find((s) => s.id === sectionId);
        if (sec) sec.refined = change.proposed_text;
        mockState.pending_refinements.changes = mockState.pending_refinements.changes.filter(
          (c) => c.section_id !== sectionId
        );
      }
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async rejectRefinement(sectionId: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('reject_refinement', { sectionId });
    }
    if (mockState.pending_refinements) {
      mockState.pending_refinements.changes = mockState.pending_refinements.changes.filter(
        (c) => c.section_id !== sectionId
      );
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async acceptAllRefinements(): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('accept_all_refinements');
    }
    if (mockState.pending_refinements) {
      for (const change of mockState.pending_refinements.changes) {
        const sec = mockState.document.sections.find((s) => s.id === change.section_id);
        if (sec) sec.refined = change.proposed_text;
      }
      mockState.pending_refinements = null;
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async rejectAllRefinements(): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('reject_all_refinements');
    }
    mockState.pending_refinements = null;
    return JSON.parse(JSON.stringify(mockState));
  },

  async acceptSuggestedSection(index: number): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('accept_suggested_section', { index });
    }
    if (mockState.pending_refinements && mockState.pending_refinements.suggested_sections[index]) {
      const sug = mockState.pending_refinements.suggested_sections.splice(index, 1)[0];
      mockState.document.sections.push({
        id: crypto.randomUUID ? crypto.randomUUID() : `sec-${Date.now()}`,
        tag: sug.tag,
        brief: sug.content,
        refined: sug.content,
        locked: false,
        enabled: true,
      });
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async pickOpenFile(): Promise<string | null> {
    if (isTauri()) {
      return invoke<string | null>('pick_open_file');
    }
    return prompt('Enter document file path to load:', 'examples/software_architecture.prompt.json');
  },

  async pickSaveFile(defaultName?: string): Promise<string | null> {
    if (isTauri()) {
      return invoke<string | null>('pick_save_file', { defaultName: defaultName || null });
    }
    return prompt('Enter save destination path:', defaultName || 'my_prompt.prompt.json');
  },

  async exportXmlToFile(path: string, stage: string, clean: boolean): Promise<void> {
    if (isTauri()) {
      return invoke<void>('export_xml_to_file', { path, stage, clean });
    }
    console.log(`Mock exported XML to ${path} (stage: ${stage}, clean: ${clean})`);
  },

  async computeDiff(original: string, refined: string): Promise<DiffLine[]> {
    if (isTauri()) {
      return invoke<DiffLine[]>('compute_diff', { original, refined });
    }
    const origLines = original.split('\n');
    const refLines = refined.split('\n');
    const diff: DiffLine[] = [];
    const max = Math.max(origLines.length, refLines.length);
    for (let i = 0; i < max; i++) {
      if (origLines[i] === refLines[i]) {
        if (origLines[i] !== undefined) diff.push({ tag: 'Equal', text: origLines[i] });
      } else {
        if (origLines[i] !== undefined) diff.push({ tag: 'Delete', text: origLines[i] });
        if (refLines[i] !== undefined) diff.push({ tag: 'Insert', text: refLines[i] });
      }
    }
    return diff;
  },

  async setTagColor(tag: string, color: string): Promise<AppConfig> {
    if (isTauri()) {
      return invoke<AppConfig>('set_tag_color', { tag, color });
    }
    if (!mockConfig.tag_colors) mockConfig.tag_colors = {};
    mockConfig.tag_colors[tag] = color;
    return JSON.parse(JSON.stringify(mockConfig));
  },

  async generateSkillContent(stage: string, name?: string, description?: string): Promise<string> {
    if (isTauri()) {
      return invoke<string>('generate_skill_content', {
        stage,
        name: name || null,
        description: description || null,
      });
    }
    const skillName = (name || mockState.document.title).toLowerCase().replace(/[^a-z0-9]+/g, '-');
    return `---
name: ${skillName}
description: ${description || mockState.document.description || 'Agent skill'}
---

# ${mockState.document.title}

${mockState.document.description}

## Prompt Instructions
\`\`\`xml
<prompt>
${mockState.document.sections.map(s => `  <${s.tag}>\n    ${s.brief}\n  </${s.tag}>`).join('\n')}
</prompt>
\`\`\`
`;
  },

  async exportSkillToFile(
    path: string,
    stage: string,
    name?: string,
    description?: string
  ): Promise<void> {
    if (isTauri()) {
      return invoke<void>('export_skill_to_file', {
        path,
        stage,
        name: name || null,
        description: description || null,
      });
    }
    console.log(`Mock exported skill to ${path}`);
  },

  async pickSaveSkillFile(defaultName?: string): Promise<string | null> {
    if (isTauri()) {
      return invoke<string | null>('pick_save_skill_file', { defaultName: defaultName || null });
    }
    return prompt('Enter skill destination file:', defaultName || 'SKILL.md');
  },

  async parseImportPreview(content: string): Promise<ImportPreviewDto> {
    if (isTauri()) {
      return invoke<ImportPreviewDto>('parse_import_preview', { content });
    }
    return {
      format: 'markdown',
      title: 'Imported Prompt',
      description: 'Mock imported preview',
      section_count: 2,
      sections: [
        {
          id: 'mock-1',
          tag: 'role',
          brief: 'Mock Role',
          refined: null,
          locked: false,
          enabled: true,
        },
        {
          id: 'mock-2',
          tag: 'task',
          brief: 'Mock Task',
          refined: null,
          locked: false,
          enabled: true,
        },
      ],
    };
  },

  async importPromptContent(content: string, mode: 'replace' | 'append'): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('import_prompt_content', { content, mode });
    }
    mockState.dirty = true;
    return JSON.parse(JSON.stringify(mockState));
  },

  async pickImportFile(): Promise<ImportFileContent | null> {
    if (isTauri()) {
      return invoke<ImportFileContent | null>('pick_import_file');
    }
    return null;
  },

  async listLibraryPrompts(): Promise<SavedPromptSummary[]> {
    if (isTauri()) {
      return invoke<SavedPromptSummary[]>('list_library_prompts');
    }
    return [
      {
        id: mockState.document.id,
        title: mockState.document.title,
        description: mockState.document.description,
        section_count: mockState.document.sections.length,
        tags: ['architecture', 'rust', 'concurrency'],
        file_path: 'mock/path/prompt.json',
        updated_at: 'Just now',
      },
    ];
  },

  async saveToLibrary(): Promise<SavedPromptSummary> {
    if (isTauri()) {
      return invoke<SavedPromptSummary>('save_to_library');
    }
    return {
      id: mockState.document.id,
      title: mockState.document.title,
      description: mockState.document.description,
      section_count: mockState.document.sections.length,
      tags: ['architecture', 'rust'],
      file_path: 'mock/path/saved.prompt.json',
      updated_at: 'Just now',
    };
  },

  async loadFromLibrary(id: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('load_from_library', { id });
    }
    return JSON.parse(JSON.stringify(mockState));
  },

  async deleteFromLibrary(id: string): Promise<void> {
    if (isTauri()) {
      return invoke<void>('delete_from_library', { id });
    }
    console.log(`Mock deleted prompt ${id} from library`);
  },

  async getDocumentVariables(): Promise<string[]> {
    if (isTauri()) {
      return invoke<string[]>('get_document_variables');
    }
    const vars = new Set<string>();
    for (const s of mockState.document.sections) {
      const matches = s.brief.match(/\{\{([^}]+)\}\}/g);
      if (matches) {
        for (const m of matches) {
          vars.add(m.replace(/[{}]/g, '').trim());
        }
      }
    }
    return Array.from(vars);
  },

  async renderScenarioPreview(
    scenarioId?: string,
    customValues?: Record<string, string>,
    stage = 'final',
    clean = false
  ): Promise<ScenarioPreviewDto> {
    if (isTauri()) {
      return invoke<ScenarioPreviewDto>('render_scenario_preview', {
        scenarioId: scenarioId || null,
        customValues: customValues || null,
        stage,
        clean,
      });
    }
    const xml = `<prompt>\n  <role>Mock Scenario Render</role>\n</prompt>`;
    return {
      rendered_xml: xml,
      char_count: xml.length,
      word_count: 5,
      estimated_tokens: 12,
      all_variables: ['user_query', 'target_platform'],
      unresolved_variables: [],
    };
  },

  async compareSnapshot(snapshotId: string): Promise<SnapshotComparison> {
    if (isTauri()) {
      return invoke<SnapshotComparison>('compare_snapshot', { snapshotId });
    }
    return {
      snapshot_id: snapshotId,
      snapshot_name: 'v1.0 Baseline',
      snapshot_title: 'Original Title',
      current_title: mockState.document.title,
      is_title_changed: false,
      section_diffs: [
        {
          tag: 'role',
          status: 'unchanged',
          snapshot_brief: 'Senior Engineer',
          current_brief: 'Senior Engineer',
        },
      ],
    };
  },

  async forkSnapshot(snapshotId: string, newTitle: string): Promise<DocumentStateDto> {
    if (isTauri()) {
      return invoke<DocumentStateDto>('fork_snapshot', { snapshotId, newTitle });
    }
    mockState.document.title = newTitle;
    mockState.current_file_path = null;
    mockState.dirty = true;
    return JSON.parse(JSON.stringify(mockState));
  },
};

