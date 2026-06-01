import type { WidgetManifest } from '../types';

export const manifest: WidgetManifest = {
  id: 'core.stat',
  name: 'Stat Tile',
  category: 'basic',
  version: 1,
  description: 'Displays a single numeric value as a stat tile.',
  sizing: { default_w: 3, default_h: 2, min_w: 2, min_h: 1 },
  bindings: [{ key: 'value', label: 'Value', value_type: 'number', required: true }],
};
