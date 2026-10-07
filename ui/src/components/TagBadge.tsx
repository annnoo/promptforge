import React, { useState, useRef, useEffect } from 'react';
import { getTagColor, TAG_COLOR_PALETTE, type TagColorDef } from '../utils/tagColors';

interface TagBadgeProps {
  tag: string;
  customColors?: Record<string, string>;
  onRemove?: () => void;
  onClick?: () => void;
  onColorChange?: (colorId: string) => void;
  isActive?: boolean;
  size?: 'xs' | 'sm' | 'md';
  showHash?: boolean;
  className?: string;
}

export const TagBadge: React.FC<TagBadgeProps> = ({
  tag,
  customColors,
  onRemove,
  onClick,
  onColorChange,
  isActive = false,
  size = 'sm',
  showHash = true,
  className = '',
}) => {
  const [showPicker, setShowPicker] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);
  const colorDef: TagColorDef = getTagColor(tag, customColors);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (pickerRef.current && !pickerRef.current.contains(event.target as Node)) {
        setShowPicker(false);
      }
    }
    if (showPicker) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showPicker]);

  const sizeClasses = {
    xs: 'text-[10px] px-1.5 py-0.2',
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-2.5 py-1',
  }[size];

  const dotSize = {
    xs: 'w-1 h-1',
    sm: 'w-1.5 h-1.5',
    md: 'w-2 h-2',
  }[size];

  return (
    <div className="relative inline-flex items-center">
      <span
        onClick={onClick}
        className={`inline-flex items-center gap-1.5 rounded font-mono font-medium border transition-colors select-none ${
          isActive ? colorDef.activeClass : colorDef.badgeClass
        } ${sizeClasses} ${onClick ? 'cursor-pointer' : ''} ${className}`}
      >
        {/* Dot indicator, clickable to open color picker if onColorChange provided */}
        {onColorChange ? (
          <button
            type="button"
            title="Change tag color"
            onClick={(e) => {
              e.stopPropagation();
              setShowPicker(!showPicker);
            }}
            className={`rounded-full ${colorDef.dotClass} ${dotSize} hover:scale-125 transition-transform cursor-pointer focus:outline-none`}
          />
        ) : (
          <span className={`rounded-full ${colorDef.dotClass} ${dotSize}`} />
        )}

        <span>
          {showHash && '#'}
          {tag}
        </span>

        {/* Remove button */}
        {onRemove && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onRemove();
            }}
            className="ml-0.5 text-zinc-400 hover:text-zinc-200 transition-colors focus:outline-none cursor-pointer"
            title={`Remove tag ${tag}`}
          >
            ×
          </button>
        )}
      </span>

      {/* Color Picker Popover */}
      {showPicker && onColorChange && (
        <div
          ref={pickerRef}
          className="absolute left-0 top-full mt-1.5 z-50 p-2 bg-zinc-900 border border-zinc-700/80 rounded-lg shadow-2xl flex flex-wrap gap-1.5 w-40 animate-in fade-in zoom-in-95 duration-100"
        >
          <div className="w-full text-[10px] font-sans font-medium text-zinc-400 mb-1 px-1 flex justify-between items-center">
            <span>Tag Color: #{tag}</span>
          </div>
          {TAG_COLOR_PALETTE.map((pal) => (
            <button
              key={pal.id}
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onColorChange(pal.id);
                setShowPicker(false);
              }}
              title={pal.name}
              className={`w-6 h-6 rounded-md flex items-center justify-center border transition-transform hover:scale-110 cursor-pointer ${
                colorDef.id === pal.id ? 'border-white scale-105' : 'border-zinc-800'
              }`}
              style={{ backgroundColor: pal.hex + '25' }}
            >
              <span
                className="w-2.5 h-2.5 rounded-full"
                style={{ backgroundColor: pal.hex }}
              />
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
