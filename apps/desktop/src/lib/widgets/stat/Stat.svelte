<script lang="ts">
  interface Thresholds {
    warn?: number;
    critical?: number;
  }

  let {
    value = null,
    unit = undefined,
    props = {},
    history = [],
  }: {
    value?: number | null;
    unit?: string;
    props?: Record<string, unknown>;
    history?: number[];
  } = $props();

  const label = $derived(typeof props.label === 'string' ? (props.label as string) : undefined);

  const showUnit = $derived(props.show_unit !== false);

  const thresholds = $derived(
    props.thresholds && typeof props.thresholds === 'object'
      ? (props.thresholds as Thresholds)
      : undefined
  );

  const formatted = $derived(
    value === null || value === undefined || Number.isNaN(value) ? 'N/A' : value.toFixed(1)
  );

  const isNull = $derived(value === null || value === undefined || Number.isNaN(value));

  const valueColor = $derived.by(() => {
    if (isNull) return 'var(--ag-text-secondary)';
    const v = value as number;
    const t = thresholds;
    if (t) {
      if (typeof t.critical === 'number' && v >= t.critical) return 'var(--ag-critical)';
      if (typeof t.warn === 'number' && v >= t.warn) return 'var(--ag-warn)';
    }
    return 'var(--ag-text-primary)';
  });
</script>

<div class="stat">
  {#if label}
    <div class="label">{label}</div>
  {/if}
  <div class="value-row">
    <span class="value" style:color={valueColor}>{formatted}</span>
    {#if !isNull && showUnit && unit}
      <span class="unit">{unit}</span>
    {/if}
  </div>
</div>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: calc(var(--ag-grid) / 2);
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: var(--ag-widget-padding);
    background: var(--ag-surface);
    border: 1px solid var(--ag-border);
    border-radius: var(--ag-radius);
    overflow: hidden;
  }

  .label {
    font-family: var(--ag-font-primary);
    font-size: 0.75rem;
    color: var(--ag-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .value-row {
    display: flex;
    align-items: baseline;
    gap: calc(var(--ag-grid) / 2);
    min-width: 0;
  }

  .value {
    font-family: var(--ag-font-numeric);
    font-size: 2rem;
    font-weight: 600;
    line-height: 1.1;
  }

  .unit {
    font-family: var(--ag-font-primary);
    font-size: 0.875rem;
    color: var(--ag-text-secondary);
  }
</style>
