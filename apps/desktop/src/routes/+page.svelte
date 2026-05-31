<script lang="ts">
  import { onMount } from 'svelte';
  import { sensors } from '$lib/data/sensors.svelte';
  import { themeStore } from '$lib/theme/theme.svelte';
  import PanelGrid from '$lib/panel/PanelGrid.svelte';
  import type { PanelDocument } from '$lib/panel/types';
  import demoPanel from '$lib/panel/demo.panel.json';

  const demo = demoPanel as unknown as PanelDocument;

  // Diegetic status rail: a live clock and session uptime, like a real sensor
  // panel — not a web app bar.
  let now = $state(new Date());
  const startedAt = Date.now();

  onMount(() => {
    sensors.start();
    const id = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(id);
  });

  const clock = $derived(now.toLocaleTimeString(undefined, { hour12: false }));

  const uptime = $derived.by(() => {
    const s = Math.max(0, Math.floor((now.getTime() - startedAt) / 1000));
    const hh = String(Math.floor(s / 3600)).padStart(2, '0');
    const mm = String(Math.floor((s % 3600) / 60)).padStart(2, '0');
    const ss = String(s % 60).padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  });
</script>

<div class="app">
  <div class="rail">
    <div class="rail-left">
      <span class="mark" aria-hidden="true">✦</span>
      <span class="panel-name">{demo.name}</span>
    </div>
    <div class="rail-right">
      <span class="meta"
        ><span class="meta-label">UP</span> <span class="ag-numeric">{uptime}</span></span
      >
      <span class="ag-numeric clock">{clock}</span>
      <button
        class="theme-toggle"
        type="button"
        aria-label="Toggle theme"
        title="Toggle light / dark"
        onclick={() => themeStore.toggle()}
      >
        {themeStore.theme === 'dark' ? '◐' : '◑'}
      </button>
    </div>
  </div>

  <main class="surface">
    <PanelGrid panel={demo} />
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100%;
    background: var(--ag-background);
    color: var(--ag-text-primary);
    overflow: hidden;
  }

  .rail {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 34px;
    padding: 0 calc(var(--ag-grid) * 1.5);
    border-bottom: 1px solid var(--ag-hairline);
    background: var(--ag-surface);
    user-select: none;
  }

  .rail-left,
  .rail-right {
    display: flex;
    align-items: center;
    gap: calc(var(--ag-grid) * 1.5);
  }

  .mark {
    color: var(--ag-accent);
    font-size: 0.8rem;
  }

  .panel-name {
    font-family: var(--ag-font-primary);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--ag-text-secondary);
  }

  .meta {
    font-size: 0.72rem;
    color: var(--ag-text-secondary);
    letter-spacing: 0.04em;
  }

  .meta-label {
    opacity: 0.7;
  }

  .clock {
    font-size: 0.78rem;
    color: var(--ag-text-primary);
    letter-spacing: 0.02em;
  }

  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    font-size: 0.85rem;
    line-height: 1;
    color: var(--ag-text-secondary);
    background: transparent;
    border: 1px solid var(--ag-hairline);
    border-radius: var(--ag-radius);
    cursor: pointer;
    transition:
      color 0.15s ease,
      border-color 0.15s ease;
  }

  .theme-toggle:hover {
    color: var(--ag-accent);
    border-color: var(--ag-accent-dim);
  }

  .surface {
    position: relative;
    flex: 1 1 auto;
    min-height: 0;
    padding: calc(var(--ag-grid) * 1.5);
    overflow: hidden;
    /* Constellation / precision-grid motif: a faint dot field with a calm
       glow toward the top. Restrained, not decorative noise. */
    background-image:
      radial-gradient(circle at 50% -10%, var(--ag-accent-dim), transparent 55%),
      radial-gradient(var(--ag-tick) 0.5px, transparent 0.5px);
    background-size:
      100% 100%,
      22px 22px;
    background-position:
      center top,
      center center;
  }
</style>
