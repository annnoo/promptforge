import React, { useState, useEffect } from 'react';
import {
  X,
  Folder,
  Tag,
  Code,
  Check,
  FolderPlus,
  Component,
} from 'lucide-react';
import type { SectionType } from '../types';
import { TagBadge } from './TagBadge';

interface CreateTypeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (type: SectionType) => void;
  existingFolders: string[];
  initialType?: SectionType | null;
}

export const CreateTypeModal: React.FC<CreateTypeModalProps> = ({
  isOpen,
  onClose,
  onSave,
  existingFolders,
  initialType,
}) => {
  const [name, setName] = useState('');
  const [tag, setTag] = useState('');
  const [description, setDescription] = useState('');
  const [defaultBrief, setDefaultBrief] = useState('');
  const [folder, setFolder] = useState('');
  const [isNewFolder, setIsNewFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialType) {
      setName(initialType.name);
      setTag(initialType.tag);
      setDescription(initialType.description);
      setDefaultBrief(initialType.default_brief);
      setFolder(initialType.folder);
      setTags(initialType.tags || []);
      setIsNewFolder(false);
    } else {
      setName('');
      setTag('');
      setDescription('');
      setDefaultBrief('');
      setFolder(existingFolders[0] || 'Core Structure');
      setTags([]);
      setIsNewFolder(false);
      setNewFolderName('');
    }
    setError(null);
  }, [initialType, existingFolders, isOpen]);

  if (!isOpen) return null;

  const handleAddTag = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      const cleaned = tagInput.trim().toLowerCase().replace(/^#/, '');
      if (cleaned && !tags.includes(cleaned)) {
        setTags([...tags, cleaned]);
      }
      setTagInput('');
    }
  };

  const handleRemoveTag = (tToRemove: string) => {
    setTags(tags.filter((t) => t !== tToRemove));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const cleanTag = tag.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '_');
    if (!cleanTag) {
      setError('XML Tag name cannot be empty');
      return;
    }
    if (!/^[a-z_][a-z0-9_-]*$/.test(cleanTag)) {
      setError('XML tag must begin with a letter or underscore and contain only letters, numbers, hyphens, or underscores');
      return;
    }
    const finalFolder = isNewFolder ? newFolderName.trim() || 'Custom Types' : folder || 'General';

    const newType: SectionType = {
      id: initialType ? initialType.id : `type-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      name: name.trim() || cleanTag,
      tag: cleanTag,
      description: description.trim(),
      default_brief: defaultBrief.trim(),
      folder: finalFolder,
      tags,
      is_builtin: initialType ? initialType.is_builtin : false,
    };

    onSave(newType);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-lg shadow-2xl flex flex-col max-h-[90vh] overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-950/60">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-slate-800 border border-slate-700 text-amber-400">
              <Component className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-slate-100">
                {initialType ? 'Edit Section Type' : 'Create New Section Type'}
              </h2>
              <p className="text-xs text-slate-400">
                Define a modular, reusable prompt building block
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

        {/* Form Body */}
        <form onSubmit={handleSubmit} className="p-6 overflow-y-auto space-y-4 text-xs">
          {error && (
            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-300">
              {error}
            </div>
          )}

          {/* Type Name & XML Tag */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <label className="font-semibold text-slate-300">Type Name</label>
              <input
                type="text"
                placeholder="e.g. Security Policy"
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (!initialType && !tag) {
                    setTag(e.target.value.toLowerCase().replace(/\s+/g, '_').replace(/[^a-z0-9_-]/g, ''));
                  }
                }}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-100 outline-none focus:border-amber-500/50"
                required
                autoFocus
              />
            </div>

            <div className="space-y-1">
              <label className="font-semibold text-slate-300 flex items-center gap-1">
                <Code className="w-3.5 h-3.5 text-slate-400" />
                <span>XML Tag</span>
              </label>
              <div className="flex items-center bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 focus-within:border-amber-500/50 font-mono">
                <span className="text-slate-500">&lt;</span>
                <input
                  type="text"
                  placeholder="security_policy"
                  value={tag}
                  onChange={(e) => setTag(e.target.value)}
                  className="w-full bg-transparent px-1 text-amber-400 outline-none"
                  required
                />
                <span className="text-slate-500">&gt;</span>
              </div>
            </div>
          </div>

          {/* Folder / Category Selection */}
          <div className="space-y-1.5">
            <label className="font-semibold text-slate-300 flex items-center gap-1.5">
              <Folder className="w-3.5 h-3.5 text-slate-400" />
              <span>Folder / Category</span>
            </label>

            {!isNewFolder ? (
              <div className="flex items-center gap-2">
                <select
                  value={folder}
                  onChange={(e) => setFolder(e.target.value)}
                  className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 outline-none focus:border-amber-500/50"
                >
                  {existingFolders.map((f) => (
                    <option key={f} value={f}>
                      {f}
                    </option>
                  ))}
                  {!existingFolders.includes(folder) && folder && (
                    <option value={folder}>{folder}</option>
                  )}
                </select>
                <button
                  type="button"
                  onClick={() => setIsNewFolder(true)}
                  className="flex items-center gap-1 px-3 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg border border-slate-700/80 transition"
                >
                  <FolderPlus className="w-3.5 h-3.5 text-amber-400" />
                  <span>New Folder</span>
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  placeholder="Enter new folder name..."
                  value={newFolderName}
                  onChange={(e) => setNewFolderName(e.target.value)}
                  className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-100 outline-none focus:border-amber-500/50"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setIsNewFolder(false)}
                  className="px-3 py-2 text-slate-400 hover:text-white"
                >
                  Cancel
                </button>
              </div>
            )}
          </div>

          {/* Tags */}
          <div className="space-y-1.5">
            <label className="font-semibold text-slate-300 flex items-center gap-1.5">
              <Tag className="w-3.5 h-3.5 text-slate-400" />
              <span>Tags (Press Enter or comma to add)</span>
            </label>

            <div className="bg-slate-950 border border-slate-800 rounded-lg p-2 flex flex-wrap gap-1.5 min-h-[38px] items-center focus-within:border-amber-500/50">
              {tags.map((t) => (
                <TagBadge
                  key={t}
                  tag={t}
                  size="xs"
                  onRemove={() => handleRemoveTag(t)}
                />
              ))}
              <input
                type="text"
                placeholder={tags.length === 0 ? "e.g. security, backend, strict" : ""}
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={handleAddTag}
                className="flex-1 bg-transparent text-slate-200 outline-none min-w-[100px] text-xs"
              />
            </div>
          </div>

          {/* Description */}
          <div className="space-y-1">
            <label className="font-semibold text-slate-300">Description / Purpose</label>
            <input
              type="text"
              placeholder="Short explanation of when to include this section..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 outline-none focus:border-amber-500/50"
            />
          </div>

          {/* Default Template / Notes */}
          <div className="space-y-1">
            <label className="font-semibold text-slate-300">Default Template Content</label>
            <textarea
              placeholder="Default initial notes or scaffold instructions for this section..."
              value={defaultBrief}
              onChange={(e) => setDefaultBrief(e.target.value)}
              rows={4}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 font-mono text-xs text-slate-200 placeholder-slate-600 outline-none focus:border-amber-500/50 resize-y leading-relaxed"
            />
          </div>

          {/* Actions */}
          <div className="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 font-medium text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="flex items-center gap-1.5 px-5 py-2 font-bold text-slate-950 bg-amber-400 hover:bg-amber-300 rounded-xl shadow-lg transition"
            >
              <Check className="w-4 h-4" />
              <span>{initialType ? 'Update Type' : 'Save Section Type'}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
