<script lang="ts">
  import type { PanelDocument } from './types';
  import { getWidget } from '$lib/widgets/registry';
  import { sensors } from '$lib/data/sensors.svelte';

  let { panel }: { panel: PanelDocument } = $props();
</script>

<div
  class="panel-grid"
  style:grid-template-columns="repeat({panel.grid.columns}, 1fr)"
  style:grid-auto-rows="{panel.grid.rowHeight}px"
  style:gap="{panel.grid.spacing}px"
>
  {#each panel.widgets as widget (widget.id)}
    {@const entry = getWidget(widget.type)}
    {@const valueId = widget.bindings.value}
    <div
      class="cell"
      style:grid-column="{widget.x + 1} / span {widget.w}"
      style:grid-row="{widget.y + 1} / span {widget.h}"
    >
      {#if entry}
        {@const Widget = entry.component}
        <svelte:boundary>
          <Widget
            value={sensors.getValue(valueId)}
            unit={sensors.getReading(valueId)?.unit}
            props={widget.props}
            history={sensors.getHistory(valueId)}
          />
          {#snippet failed()}
            <div class="fallback error">sensor unavailable</div>
          {/snippet}
        </svelte:boundary>
      {:else}
        <div class="fallback">unknown widget</div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .panel-grid {
    display: grid;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
  }

  .cell {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: flex;
    padding: calc(var(--ag-grid) * 1.5);
    background: var(--ag-surface);
    border: 1px solid var(--ag-hairline);
    border-radius: var(--ag-radius);
    /* A faint top accent hairline reads as an instrument module header rule. */
    box-shadow: inset 0 1px 0 0 var(--ag-hairline);
  }

  .cell > :global(*) {
    flex: 1;
    min-width: 0;
  }

  .fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    font-family: var(--ag-font-primary);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ag-text-secondary);
    text-align: center;
  }

  .fallback.error {
    color: var(--ag-critical);
  }
</style>
