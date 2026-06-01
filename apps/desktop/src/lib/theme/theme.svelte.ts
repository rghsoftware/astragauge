// AstraGauge theme store — Svelte 5 runes singleton.
// Holds the current theme ('dark' | 'light', default 'dark') and applies it
// to <html> via document.documentElement.dataset.theme.
// Document access is guarded so SSR/build (no DOM) does not throw.

export type Theme = 'dark' | 'light';

const DEFAULT_THEME: Theme = 'dark';

function applyToDocument(theme: Theme): void {
  if (typeof document === 'undefined') return;
  // Dark is the default declared on :root, so we only set the attribute for light.
  if (theme === 'light') {
    document.documentElement.dataset.theme = 'light';
  } else {
    delete document.documentElement.dataset.theme;
  }
}

class ThemeStore {
  #theme = $state<Theme>(DEFAULT_THEME);

  get theme(): Theme {
    return this.#theme;
  }

  setTheme(t: Theme): void {
    this.#theme = t;
    applyToDocument(t);
  }

  toggle(): void {
    this.setTheme(this.#theme === 'dark' ? 'light' : 'dark');
  }
}

// Single shared instance imported across the app.
export const themeStore = new ThemeStore();
