<script lang="ts">
  // Compact trend line for instrument widgets. Renders recent values as a thin
  // line with an optional faint area fill. Internal component — not a panel
  // widget. Scales to its container via a fixed viewBox + preserveAspectRatio.

  interface Props {
    values: number[];
    color?: string;
    fill?: boolean;
  }

  let { values, color = 'var(--ag-accent)', fill = true }: Props = $props();

  const W = 100;
  const H = 28;

  // Map values to a polyline path within the viewBox. A flat/empty series
  // degrades gracefully to a centered baseline.
  const geometry = $derived.by(() => {
    const pts = values.filter((v) => Number.isFinite(v));
    if (pts.length < 2) {
      return { line: '', area: '' };
    }
    const min = Math.min(...pts);
    const max = Math.max(...pts);
    const span = max - min || 1;
    const stepX = W / (pts.length - 1);

    const coords = pts.map((v, i) => {
      const x = i * stepX;
      // Leave 2px headroom top/bottom so the stroke is never clipped.
      const y = H - 2 - ((v - min) / span) * (H - 4);
      return [x, y] as const;
    });

    const line = coords
      .map(([x, y], i) => `${i === 0 ? 'M' : 'L'} ${x.toFixed(2)} ${y.toFixed(2)}`)
      .join(' ');
    const area = `${line} L ${W} ${H} L 0 ${H} Z`;
    return { line, area };
  });
</script>

<svg
  class="sparkline"
  viewBox="0 0 {W} {H}"
  width="100%"
  height="100%"
  preserveAspectRatio="none"
  aria-hidden="true"
>
  {#if geometry.line}
    {#if fill}
      <path d={geometry.area} fill={color} fill-opacity="0.1" stroke="none"></path>
    {/if}
    <path
      d={geometry.line}
      fill="none"
      stroke={color}
      stroke-width="1.25"
      stroke-linecap="round"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    ></path>
  {/if}
</svg>

<style>
  .sparkline {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
