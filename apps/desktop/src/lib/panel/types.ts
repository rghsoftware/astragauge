export interface PanelGrid {
  columns: number;
  rowHeight: number;
  spacing: number;
}

export interface PanelWidget {
  id: string;
  type: string;
  x: number;
  y: number;
  w: number;
  h: number;
  props?: Record<string, unknown>;
  bindings: Record<string, string>;
}

export interface PanelDocument {
  version: number;
  name: string;
  theme: string;
  grid: PanelGrid;
  widgets: PanelWidget[];
}
