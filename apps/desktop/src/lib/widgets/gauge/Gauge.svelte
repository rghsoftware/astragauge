<script lang="ts">
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

  // Geometry: 270-degree sweep, symmetric about the bottom gap.
  const VIEWBOX = 100;
  const CENTER = 50;
  const RADIUS = 40;
  const STROKE = 9;
  const START_ANGLE = 135; // degrees, bottom-left
  const SWEEP = 270; // degrees

  const min = $derived(typeof props.min === 'number' ? (props.min as number) : 0);
  const max = $derived(typeof props.max === 'number' ? (props.max as number) : 100);
  const label = $derived(typeof props.label === 'string' ? (props.label as string) : undefined);
  const thresholds = $derived(
    props.thresholds && typeof props.thresholds === 'object'
      ? (props.thresholds as Thresholds)
      : undefined
  );

  const hasValue = $derived(value !== null && value !== undefined && Number.isFinite(value));

  const fraction = $derived.by(() => {
    if (!hasValue) {
      return 0;
    }
    const span = max - min;
    if (span <= 0) {
      return 0;
    }
    const raw = ((value as number) - min) / span;
    return Math.min(1, Math.max(0, raw));
  });

  const valueColor = $derived.by(() => {
    if (!hasValue || !thresholds) {
      return 'var(--ag-accent)';
    }
    const v = value as number;
    if (typeof thresholds.critical === 'number' && v >= thresholds.critical) {
      return 'var(--ag-critical)';
    }
    if (typeof thresholds.warn === 'number' && v >= thresholds.warn) {
      return 'var(--ag-warn)';
    }
    return 'var(--ag-accent)';
  });

  const displayValue = $derived(hasValue ? (value as number).toFixed(1) : 'N/A');

  // Convert a polar coordinate (degrees, clockwise from positive x-axis in SVG
  // space where y grows downward) into a point on the arc circle.
  function pointAt(angleDeg: number): { x: number; y: number } {
    const rad = (angleDeg * Math.PI) / 180;
    return {
      x: CENTER + RADIUS * Math.cos(rad),
      y: CENTER + RADIUS * Math.sin(rad),
    };
  }

  function arcPath(fromAngle: number, toAngle: number): string {
    const start = pointAt(fromAngle);
    const end = pointAt(toAngle);
    const delta = toAngle - fromAngle;
    const largeArc = delta > 180 ? 1 : 0;
    // sweep flag 1 => clockwise (increasing angle) in SVG's y-down space.
    return `M ${start.x} ${start.y} A ${RADIUS} ${RADIUS} 0 ${largeArc} 1 ${end.x} ${end.y}`;
  }

  const backgroundPath = $derived(arcPath(START_ANGLE, START_ANGLE + SWEEP));
  const valuePath = $derived(arcPath(START_ANGLE, START_ANGLE + SWEEP * fraction));
</script>

<div class="gauge" class:muted={!hasValue}>
  <svg
    class="arc"
    viewBox="0 0 {VIEWBOX} {VIEWBOX}"
    width="100%"
    height="100%"
    preserveAspectRatio="xMidYMid meet"
    role="img"
    aria-label={label ? `${label} gauge` : 'gauge'}
  >
    <path
      class="track"
      d={backgroundPath}
      fill="none"
      stroke="var(--ag-border)"
      stroke-width={STROKE}
      stroke-linecap="round"
    ></path>
    {#if hasValue && fraction > 0}
      <path
        class="value-arc"
        d={valuePath}
        fill="none"
        stroke={valueColor}
        stroke-width={STROKE}
        stroke-linecap="round"
      ></path>
    {/if}
  </svg>

  <div class="readout">
    <span class="value" style:color={hasValue ? valueColor : 'var(--ag-text-secondary)'}>
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
    box-sizing: border-box;
    padding: var(--ag-widget-padding);
    background: var(--ag-surface);
    border: 1px solid var(--ag-border);
    border-radius: var(--ag-radius);
    font-family: var(--ag-font-primary);
    container-type: size;
  }

  .arc {
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
    gap: calc(var(--ag-grid) / 4);
    pointer-events: none;
    text-align: center;
  }

  .value {
    font-family: var(--ag-font-numeric);
    color: var(--ag-text-primary);
    font-size: clamp(0.9rem, 18cqmin, 2rem);
    font-weight: 600;
    line-height: 1.1;
  }

  .unit {
    font-family: var(--ag-font-primary);
    color: var(--ag-text-secondary);
    font-size: clamp(0.55rem, 8cqmin, 0.85rem);
    line-height: 1;
  }

  .label {
    font-family: var(--ag-font-primary);
    color: var(--ag-text-secondary);
    font-size: clamp(0.6rem, 9cqmin, 0.9rem);
    line-height: 1.1;
  }

  .muted .value {
    color: var(--ag-text-secondary);
  }
</style>
