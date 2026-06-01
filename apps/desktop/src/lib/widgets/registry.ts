import type { Component } from 'svelte';
import type { WidgetManifest } from './types';

import Gauge from './gauge/Gauge.svelte';
import { manifest as gaugeManifest } from './gauge/manifest';
import Stat from './stat/Stat.svelte';
import { manifest as statManifest } from './stat/manifest';

// Props every widget component receives (see WIDGET MODEL contract).
export interface WidgetProps {
  value: number | null;
  unit?: string;
  props?: Record<string, unknown>;
  history?: number[];
}

export interface WidgetEntry {
  manifest: WidgetManifest;
  component: Component<WidgetProps>;
}

// Map widget type id -> { manifest, component }.
const registry: Record<string, WidgetEntry> = {
  [gaugeManifest.id]: {
    manifest: gaugeManifest,
    component: Gauge as unknown as Component<WidgetProps>,
  },
  [statManifest.id]: {
    manifest: statManifest,
    component: Stat as unknown as Component<WidgetProps>,
  },
};

export function getWidget(type: string): WidgetEntry | undefined {
  return registry[type];
}
