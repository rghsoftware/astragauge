import type { WidgetManifest } from '../types';

export const manifest: WidgetManifest = {
  id: 'core.gauge',
  name: 'Gauge',
  category: 'basic',
  version: 1,
  description: 'Radial arc gauge showing a value between a configurable min and max.',
  sizing: { default_w: 3, default_h: 3, min_w: 2, min_h: 2 },
  bindings: [{ key: 'value', label: 'Value', value_type: 'number', required: true }],
};
