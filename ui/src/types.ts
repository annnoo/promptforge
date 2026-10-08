export interface PromptSection {
  id: string;
  tag: string;
  brief: string;
  refined?: string | null;
  locked: boolean;
  enabled: boolean;
  tags?: string[];
}

export interface TestScenario {
  id: string;
  name: string;
  description?: string | null;
  variables: Record<string, string>;
}

export interface DocumentSnapshot {
  id: string;
  name: string;
  description?: string | null;
  created_at: string;
  title: string;
  sections: PromptSection[];
}

export interface PromptDocument {
  schema_version: number;
  id: string;
  title: string;
  description: string;
  sections: PromptSection[];
  scenarios?: TestScenario[];
  snapshots?: DocumentSnapshot[];
}

export interface ProposedSectionChange {
  section_id: string;
  proposed_text: string;
  reason: string;
}

export interface SuggestedSection {
  tag: string;
  content: string;
  reason: string;
}

export interface PendingRefinements {
  changes: ProposedSectionChange[];
  suggested_sections: SuggestedSection[];
  open_questions: string[];
  warnings: string[];
  critique?: string | null;
}

export interface DocumentStateDto {
  document: PromptDocument;
  dirty: boolean;
  current_file_path: string | null;
  can_undo: boolean;
  can_redo: boolean;
  pending_refinements: PendingRefinements | null;
}

export interface SectionPreset {
  name: string;
  tag: string;
  description: string;
  default_brief: string;
}

export interface SectionType {
  id: string;
  name: string;
  tag: string;
  description: string;
  default_brief: string;
  folder: string;
  tags: string[];
  is_builtin: boolean;
}

export interface StarterTemplate {
  id: string;
  name: string;
  description: string;
  presets: SectionPreset[];
}

export type ProviderType = 'manual' | 'openai_compatible';

export interface OpenAiCompatibleConfig {
  base_url: string;
  model: string;
  api_key_env_var: string;
  api_key?: string | null;
  preset?: string | null;
  timeout_seconds: number;
}

export interface AppConfig {
  active_provider: ProviderType;
  openai_compatible: OpenAiCompatibleConfig;
  recent_files: string[];
  theme: string;
  custom_types?: SectionType[];
  tag_colors?: Record<string, string>;
}

export type ImportFormat = 'json' | 'xml' | 'skill' | 'markdown' | 'plain_text';

export interface ImportPreviewDto {
  format: ImportFormat;
  title: string;
  description: string;
  section_count: number;
  sections: PromptSection[];
}

export interface ImportFileContent {
  path: string;
  file_name: string;
  content: string;
}

export interface SavedPromptSummary {
  id: string;
  title: string;
  description: string;
  section_count: number;
  tags: string[];
  file_path: string;
  updated_at: string;
}

export interface ScenarioPreviewDto {
  rendered_xml: string;
  char_count: number;
  word_count: number;
  estimated_tokens: number;
  all_variables: string[];
  unresolved_variables: string[];
}

export type SectionDiffStatus = 'added' | 'removed' | 'modified' | 'unchanged';

export interface SectionDiffSummary {
  tag: string;
  status: SectionDiffStatus;
  snapshot_brief?: string | null;
  current_brief?: string | null;
}

export interface SnapshotComparison {
  snapshot_id: string;
  snapshot_name: string;
  snapshot_title: string;
  current_title: string;
  is_title_changed: boolean;
  section_diffs: SectionDiffSummary[];
}

export type DiffTag = 'Equal' | 'Delete' | 'Insert';

export interface DiffLine {
  tag: DiffTag;
  text: string;
}

export type Command =
  | { type: 'AddSection'; tag: string; brief: string }
  | { type: 'AddPreset'; preset_tag: string }
  | { type: 'RemoveSection'; id: string }
  | { type: 'DuplicateSection'; id: string }
  | { type: 'MoveSection'; id: string; to: number }
  | { type: 'MoveUp'; id: string }
  | { type: 'MoveDown'; id: string }
  | { type: 'RenameSection'; id: string; tag: string }
  | { type: 'UpdateBrief'; id: string; text: string }
  | { type: 'UpdateTags'; id: string; tags: string[] }
  | { type: 'UpdateTitle'; title: string }
  | { type: 'UpdateDescription'; description: string }
  | { type: 'ToggleLock'; id: string }
  | { type: 'ToggleEnabled'; id: string }
  | { type: 'SetRefined'; id: string; refined: string | null }
  | { type: 'ClearRefinement'; id: string }
  | { type: 'ApplyRefinement'; id: string }
  | { type: 'RejectRefinement'; id: string }
  | { type: 'ApplyAllRefinements' }
  | { type: 'RejectAllRefinements' }
  | { type: 'SaveScenario'; scenario: TestScenario }
  | { type: 'DeleteScenario'; id: string }
  | { type: 'CreateSnapshot'; name: string; description?: string | null }
  | { type: 'RestoreSnapshot'; id: string }
  | { type: 'DeleteSnapshot'; id: string }
  | { type: 'Undo' }
  | { type: 'Redo' };
