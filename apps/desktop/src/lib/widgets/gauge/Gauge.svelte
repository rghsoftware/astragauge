<script lang="ts">
  import { formatValue } from '../format';

  interface Thresholds {
    warn?: number;
    critical?: number;
  }

  interface Props {
    value: number | null;
    unit?: string;
    props?: Record<string, unknown>;
    history?: number[];
  }

  let { value, unit, props = {} }: Props = $props();

  // Precision-dial geometry: a 250° sweep with a symmetric bottom gap.
  const CENTER = 50;
  const RADIUS = 40;
  const START_ANGLE = 145; // bottom-left
  const SWEEP = 250;
  const TICKS = 11; // major tick marks around the dial

  const min = $derived(typeof props.min === 'number' ? (props.min as number) : 0);
  const max = $derived(typeof props.max === 'number' ? (props.max as number) : 100);
  const label = $derived(typeof props.label === 'string' ? (props.label as string) : undefined);
  const decimals = $derived(
    typeof props.decimals === 'number' ? (props.decimals as number) : undefined
  );
  const thresholds = $derived(
    props.thresholds && typeof props.thresholds === 'object'
      ? (props.thresholds as Thresholds)
      : undefined
  );

  const hasValue = $derived(value !== null && value !== undefined && Number.isFinite(value));

  const fraction = $derived.by(() => {
    if (!hasValue) return 0;
    const span = max - min;
    if (span <= 0) return 0;
    return Math.min(1, Math.max(0, ((value as number) - min) / span));
  });

  const valueColor = $derived.by(() => {
    if (!hasValue || !thresholds) return 'var(--ag-accent)';
    const v = value as number;
    if (typeof thresholds.critical === 'number' && v >= thresholds.critical)
      return 'var(--ag-critical)';
    if (typeof thresholds.warn === 'number' && v >= thresholds.warn) return 'var(--ag-warn)';
    return 'var(--ag-accent)';
  });

  const displayValue = $derived(hasValue ? formatValue(value as number, decimals) : 'N/A');

  function pointAt(angleDeg: number, radius: number): { x: number; y: number } {
    const rad = (angleDeg * Math.PI) / 180;
    return { x: CENTER + radius * Math.cos(rad), y: CENTER + radius * Math.sin(rad) };
  }

  function arcPath(fromAngle: number, toAngle: number): string {
    const start = pointAt(fromAngle, RADIUS);
    const end = pointAt(toAngle, RADIUS);
    const largeArc = toAngle - fromAngle > 180 ? 1 : 0;
    return `M ${start.x} ${start.y} A ${RADIUS} ${RADIUS} 0 ${largeArc} 1 ${end.x} ${end.y}`;
  }

  const trackPath = $derived(arcPath(START_ANGLE, START_ANGLE + SWEEP));
  const valuePath = $derived(arcPath(START_ANGLE, START_ANGLE + SWEEP * fraction));
  const pointer = $derived(pointAt(START_ANGLE + SWEEP * fraction, RADIUS));

  // Fine tick marks: short radial hairlines just inside the arc.
  const ticks = $derived.by(() =>
    Array.from({ length: TICKS }, (_, i) => {
      const angle = START_ANGLE + (SWEEP * i) / (TICKS - 1);
      const outer = pointAt(angle, RADIUS - 1);
      const inner = pointAt(angle, RADIUS - 5);
      return { x1: inner.x, y1: inner.y, x2: outer.x, y2: outer.y };
    })
  );
</script>

<div class="gauge" class:muted={!hasValue}>
  <svg
    class="dial"
    viewBox="0 0 100 100"
    width="100%"
    height="100%"
    preserveAspectRatio="xMidYMid meet"
    role="img"
    aria-label={label ? `${label} gauge` : 'gauge'}
  >
    <path d={trackPath} fill="none" stroke="var(--ag-track)" stroke-width="3"></path>
    {#each ticks as t}
      <line x1={t.x1} y1={t.y1} x2={t.x2} y2={t.y2} stroke="var(--ag-tick)" stroke-width="0.8"
      ></line>
    {/each}
    {#if hasValue && fraction > 0}
      <path d={valuePath} fill="none" stroke={valueColor} stroke-width="3.5" stroke-linecap="round"
      ></path>
      <circle cx={pointer.x} cy={pointer.y} r="2.4" fill={valueColor}></circle>
    {/if}
  </svg>

  <div class="readout">
    <span class="ag-numeric value" style:color={hasValue ? valueColor : 'var(--ag-text-secondary)'}>
      {displayValue}
    </span>
    {#if hasValue && unit}
      <span class="unit">{unit}</span>
    {/if}
    {#if label}
      <span class="label">{label}</span>
    {/if}
  </div>
</div>

<style>
  .gauge {
    position: relative;
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    container-type: size;
  }

  .dial {
    display: block;
    width: 100%;
    height: 100%;
  }

  .readout {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
    pointer-events: none;
    text-align: center;
  }

  .value {
    color: var(--ag-text-primary);
    font-size: clamp(0.9rem, 22cqmin, 2.4rem);
    font-weight: 600;
    line-height: 1.05;
  }

  .unit {
    font-family: var(--ag-font-primary);
    color: var(--ag-text-secondary);
    font-size: clamp(0.5rem, 7cqmin, 0.8rem);
    line-height: 1;
  }

  .label {
    font-family: var(--ag-font-primary);
    color: var(--ag-text-secondary);
    font-size: clamp(0.55rem, 8cqmin, 0.78rem);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    margin-top: 2px;
    line-height: 1.1;
  }
</style>
