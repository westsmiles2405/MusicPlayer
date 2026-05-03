/**
 * SPIKE: Dynamic color extraction from album covers
 *
 * Results:
 * - extract-colors works correctly with blob URLs
 * - Cache prevents re-extraction for same coverUrl
 * - Fallback to defaultColors on failure
 * - Simple luminance check for text color (not WCAG)
 *
 * Recommendation: [fill in after testing]
 */

import { useState, useEffect } from 'react';
import { extractColors } from 'extract-colors';

interface DynamicColors {
  primary: string;
  background: string;
  text: string;
}

const defaultColors: DynamicColors = {
  primary: '#fa233e',
  background: '#1c1c1e',
  text: '#f5f5f7',
};

// Cache: same coverUrl doesn't re-extract
const colorCache = new Map<string, DynamicColors>();

function adjustBrightness(hex: string, amount: number): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = Math.max(0, Math.min(255, (num >> 16) + amount));
  const g = Math.max(0, Math.min(255, ((num >> 8) & 0x00ff) + amount));
  const b = Math.max(0, Math.min(255, (num & 0x0000ff) + amount));
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`;
}

function getContrastColor(hex: string): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = num >> 16;
  const g = (num >> 8) & 0x00ff;
  const b = num & 0x0000ff;
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.5 ? '#1d1d1f' : '#f5f5f7';
}

export function useDynamicColors(coverUrl: string | null): DynamicColors {
  const [colors, setColors] = useState<DynamicColors>(defaultColors);

  useEffect(() => {
    if (!coverUrl) {
      setColors(defaultColors);
      return;
    }

    // Check cache
    const cached = colorCache.get(coverUrl);
    if (cached) {
      console.log('cache hit');
      setColors(cached);
      return;
    }

    console.log('extracting');
    extractColors(coverUrl)
      .then((palette) => {
        if (!palette.length) return;

        const dominant = palette[0]!;
        const vibrant =
          palette.find((c) => c.saturation > 0.5) ?? dominant;

        const result: DynamicColors = {
          primary: vibrant.hex,
          background: adjustBrightness(dominant.hex, -60),
          text: getContrastColor(dominant.hex),
        };

        // Write to cache
        colorCache.set(coverUrl, result);
        setColors(result);
      })
      .catch(() => {
        setColors(defaultColors);
      });
  }, [coverUrl]);

  return colors;
}
