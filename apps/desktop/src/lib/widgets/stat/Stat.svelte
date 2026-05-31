<script lang="ts">
  import Sparkline from '../Sparkline.svelte';
  import { formatValue } from '../format';

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
  const showTrend = $derived(props.trend !== false);
  const decimals = $derived(
    typeof props.decimals === 'number' ? (props.decimals as number) : undefined
  );
  const thresholds = $derived(
    props.thresholds && typeof props.thresholds === 'object'
      ? (props.thresholds as Thresholds)
      : undefined
  );

  const isNull = $derived(value === null || value === undefined || Number.isNaN(value as number));
  const formatted = $derived(isNull ? 'N/A' : formatValue(value as number, decimals));

  const valueColor = $derived.by(() => {
    if (isNull) return 'var(--ag-text-secondary)';
    const v = value as number;
    if (thresholds) {
      if (typeof thresholds.critical === 'number' && v >= thresholds.critical)
        return 'var(--ag-critical)';
      if (typeof thresholds.warn === 'number' && v >= thresholds.warn) return 'var(--ag-warn)';
    }
    return 'var(--ag-text-primary)';
  });

  const hasTrend = $derived(showTrend && history.length > 1);
</script>

<div class="stat">
  {#if label}
    <div class="label">{label}</div>
  {/if}
  <div class="value-row">
    <span class="ag-numeric value" style:color={valueColor}>{formatted}</span>
    {#if !isNull && showUnit && unit}
      <span class="unit">{unit}</span>
    {/if}
  </div>
  {#if hasTrend}
    <div class="trend">
      <Sparkline values={history} color={valueColor} />
    </div>
  {/if}
</div>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: calc(var(--ag-grid) / 2);
    width: 100%;
    height: 100%;
    min-width: 0;
    overflow: hidden;
  }

  .label {
    font-family: var(--ag-font-primary);
    font-size: clamp(0.6rem, 1.4vw, 0.72rem);
    color: var(--ag-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .value-row {
    display: flex;
    align-items: baseline;
    gap: calc(var(--ag-grid) / 2);
    min-width: 0;
  }

  .value {
    font-size: clamp(1.5rem, 4.4vw, 2.6rem);
    font-weight: 600;
    line-height: 1.05;
  }

  .unit {
    font-family: var(--ag-font-primary);
    font-size: clamp(0.7rem, 1.6vw, 0.9rem);
    color: var(--ag-text-secondary);
  }

  .trend {
    height: 28px;
    margin-top: auto;
    opacity: 0.9;
  }
</style>
