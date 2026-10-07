export interface TagColorDef {
  id: string;
  name: string;
  badgeClass: string;
  dotClass: string;
  activeClass: string;
  hex: string;
}

export const TAG_COLOR_PALETTE: TagColorDef[] = [
  {
    id: 'blue',
    name: 'Blue',
    badgeClass: 'bg-blue-500/10 text-blue-400 border-blue-500/25 hover:border-blue-500/50',
    dotClass: 'bg-blue-400',
    activeClass: 'bg-blue-500/20 text-blue-300 border-blue-500/50',
    hex: '#60a5fa',
  },
  {
    id: 'emerald',
    name: 'Emerald',
    badgeClass: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/25 hover:border-emerald-500/50',
    dotClass: 'bg-emerald-400',
    activeClass: 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50',
    hex: '#34d399',
  },
  {
    id: 'violet',
    name: 'Violet',
    badgeClass: 'bg-violet-500/10 text-violet-400 border-violet-500/25 hover:border-violet-500/50',
    dotClass: 'bg-violet-400',
    activeClass: 'bg-violet-500/20 text-violet-300 border-violet-500/50',
    hex: '#a78bfa',
  },
  {
    id: 'amber',
    name: 'Amber',
    badgeClass: 'bg-amber-500/10 text-amber-400 border-amber-500/25 hover:border-amber-500/50',
    dotClass: 'bg-amber-400',
    activeClass: 'bg-amber-500/20 text-amber-300 border-amber-500/50',
    hex: '#fbbf24',
  },
  {
    id: 'rose',
    name: 'Rose',
    badgeClass: 'bg-rose-500/10 text-rose-400 border-rose-500/25 hover:border-rose-500/50',
    dotClass: 'bg-rose-400',
    activeClass: 'bg-rose-500/20 text-rose-300 border-rose-500/50',
    hex: '#fb7185',
  },
  {
    id: 'cyan',
    name: 'Cyan',
    badgeClass: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/25 hover:border-cyan-500/50',
    dotClass: 'bg-cyan-400',
    activeClass: 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50',
    hex: '#22d3ee',
  },
  {
    id: 'orange',
    name: 'Orange',
    badgeClass: 'bg-orange-500/10 text-orange-400 border-orange-500/25 hover:border-orange-500/50',
    dotClass: 'bg-orange-400',
    activeClass: 'bg-orange-500/20 text-orange-300 border-orange-500/50',
    hex: '#fb923c',
  },
  {
    id: 'pink',
    name: 'Pink',
    badgeClass: 'bg-pink-500/10 text-pink-400 border-pink-500/25 hover:border-pink-500/50',
    dotClass: 'bg-pink-400',
    activeClass: 'bg-pink-500/20 text-pink-300 border-pink-500/50',
    hex: '#f472b6',
  },
  {
    id: 'indigo',
    name: 'Indigo',
    badgeClass: 'bg-indigo-500/10 text-indigo-400 border-indigo-500/25 hover:border-indigo-500/50',
    dotClass: 'bg-indigo-400',
    activeClass: 'bg-indigo-500/20 text-indigo-300 border-indigo-500/50',
    hex: '#818cf8',
  },
  {
    id: 'zinc',
    name: 'Zinc',
    badgeClass: 'bg-zinc-800 text-zinc-300 border-zinc-700 hover:border-zinc-500',
    dotClass: 'bg-zinc-400',
    activeClass: 'bg-zinc-700 text-zinc-200 border-zinc-500',
    hex: '#a1a1aa',
  },
];

/**
 * Returns a stable, deterministic color from the palette for a tag name,
 * unless overridden by customColors.
 */
export function getTagColor(
  tagName: string,
  customColors?: Record<string, string>
): TagColorDef {
  const normalized = tagName.trim().toLowerCase();

  // Check custom override first
  if (customColors && customColors[normalized]) {
    const customId = customColors[normalized];
    const match = TAG_COLOR_PALETTE.find((c) => c.id === customId);
    if (match) return match;
  }

  // Common semantic mappings
  if (normalized.includes('persona') || normalized.includes('role')) {
    return TAG_COLOR_PALETTE[2]; // violet
  }
  if (normalized.includes('arch') || normalized.includes('system') || normalized.includes('tech')) {
    return TAG_COLOR_PALETTE[0]; // blue
  }
  if (normalized.includes('rule') || normalized.includes('bound') || normalized.includes('warn')) {
    return TAG_COLOR_PALETTE[3]; // amber
  }
  if (normalized.includes('limit') || normalized.includes('constraint') || normalized.includes('non_goal')) {
    return TAG_COLOR_PALETTE[4]; // rose
  }
  if (normalized.includes('data') || normalized.includes('schema') || normalized.includes('spec')) {
    return TAG_COLOR_PALETTE[5]; // cyan
  }
  if (normalized.includes('task') || normalized.includes('action') || normalized.includes('exec')) {
    return TAG_COLOR_PALETTE[1]; // emerald
  }
  if (normalized.includes('output') || normalized.includes('format')) {
    return TAG_COLOR_PALETTE[6]; // orange
  }

  // Deterministic DJB2-like hash for everything else
  let hash = 0;
  for (let i = 0; i < normalized.length; i++) {
    hash = (hash << 5) - hash + normalized.charCodeAt(i);
    hash |= 0;
  }
  const index = Math.abs(hash) % TAG_COLOR_PALETTE.length;
  return TAG_COLOR_PALETTE[index];
}
