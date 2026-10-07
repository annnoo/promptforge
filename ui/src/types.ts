export interface PromptSection {
  id: string;
  tag: string;
  brief: string;
  refined?: string | null;
  locked: boolean;
  enabled: boolean;
  tags?: string[];
}

export interface PromptDocument {
  schema_version: number;
  id: string;
  title: string;
  description: string;
  sections: PromptSection[];
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
  timeout_seconds: number;
}

export interface AppConfig {
  active_provider: ProviderType;
  openai_compatible: OpenAiCompatibleConfig;
  recent_files: string[];
  theme: string;
  custom_types?: SectionType[];
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
  | { type: 'Undo' }
  | { type: 'Redo' };
