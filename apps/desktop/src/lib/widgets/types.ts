export interface WidgetSizing {
  default_w: number;
  default_h: number;
  min_w: number;
  min_h: number;
}

export interface Thresholds {
  warn?: number;
  critical?: number;
}

export interface WidgetBinding {
  key: string;
  label: string;
  value_type: 'number' | 'string' | 'boolean';
  required: boolean;
}

export interface WidgetManifest {
  id: string;
  name: string;
  category: string;
  version: number;
  description: string;
  sizing: WidgetSizing;
  bindings: WidgetBinding[];
}
