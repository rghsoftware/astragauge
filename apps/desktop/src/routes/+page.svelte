<script lang="ts">
  import { onMount } from 'svelte';
  import { sensors } from '$lib/data/sensors.svelte';
  import { themeStore } from '$lib/theme/theme.svelte';
  import PanelGrid from '$lib/panel/PanelGrid.svelte';
  import type { PanelDocument } from '$lib/panel/types';
  import demoPanel from '$lib/panel/demo.panel.json';

  const demo = demoPanel as unknown as PanelDocument;

  onMount(() => {
    sensors.start();
  });
</script>

<div class="page">
  <header class="header">
    <span class="brand">AstraGauge</span>
    <button class="theme-toggle" type="button" onclick={() => themeStore.toggle()}>
      {themeStore.theme === 'dark' ? 'Light' : 'Dark'} mode
    </button>
  </header>

  <main class="content">
    <PanelGrid panel={demo} />
  </main>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background: var(--ag-background);
    font-family: var(--ag-font-primary);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: calc(var(--ag-grid) * 1.5) calc(var(--ag-grid) * 2);
    border-bottom: 1px solid var(--ag-border);
    background: var(--ag-surface);
  }

  .brand {
    font-family: var(--ag-font-primary);
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--ag-text-primary);
    letter-spacing: 0.02em;
  }

  .theme-toggle {
    font-family: var(--ag-font-primary);
    font-size: 0.85rem;
    color: var(--ag-text-primary);
    background: var(--ag-surface-raised);
    border: 1px solid var(--ag-border);
    border-radius: var(--ag-radius);
    padding: calc(var(--ag-grid) * 0.75) calc(var(--ag-grid) * 1.25);
    cursor: pointer;
  }

  .theme-toggle:hover {
    border-color: var(--ag-accent);
    color: var(--ag-accent);
  }

  .content {
    flex: 1;
    min-height: 0;
  }
</style>
