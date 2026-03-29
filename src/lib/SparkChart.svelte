<script lang="ts">
  let { values, label, current, unit = '%', color = 'var(--accent)', height = 40, maxValue = 100, detail = '', formattedValue = '' }: {
    values: number[];
    label: string;
    current: number;
    unit?: string;
    color?: string;
    height?: number;
    maxValue?: number;
    detail?: string;
    formattedValue?: string;
  } = $props();

  const WIDTH = 280;
  // Safe ID for SVG gradient (no spaces/special chars)
  let gradId = $derived('grad-' + label.replace(/[^a-zA-Z0-9]/g, '_'));

  let barColor = $derived(
    current >= 95 ? 'var(--danger)' :
    current >= 80 ? 'var(--warning)' :
    color
  );

  // Monotone cubic spline for smooth curves
  let path = $derived.by(() => {
    if (values.length < 2) return '';
    const len = values.length;
    const step = WIDTH / (len - 1);
    const clamp = (v: number) => Math.max(0, Math.min(v, maxValue));
    const fy = (v: number) => height - (clamp(v) / maxValue) * (height - 2) - 1;

    const xs = Array.from({ length: len }, (_, i) => i * step);
    const ys = values.map(fy);

    if (len === 2) {
      return `M${xs[0]},${ys[0]} L${xs[1]},${ys[1]}`;
    }

    // Compute tangents (monotone)
    const ds: number[] = [];
    const ms: number[] = [];
    for (let i = 0; i < len - 1; i++) {
      ds.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
    }
    ms.push(ds[0]);
    for (let i = 1; i < len - 1; i++) {
      ms.push((ds[i - 1] + ds[i]) / 2);
    }
    ms.push(ds[len - 2]);

    // Monotone constraint
    for (let i = 0; i < len - 1; i++) {
      if (Math.abs(ds[i]) < 1e-6) {
        ms[i] = 0;
        ms[i + 1] = 0;
      } else {
        const a = ms[i] / ds[i];
        const b = ms[i + 1] / ds[i];
        const s = a * a + b * b;
        if (s > 9) {
          const t = 3 / Math.sqrt(s);
          ms[i] = t * a * ds[i];
          ms[i + 1] = t * b * ds[i];
        }
      }
    }

    // Build cubic bezier path
    let d = `M${xs[0].toFixed(1)},${ys[0].toFixed(1)}`;
    for (let i = 0; i < len - 1; i++) {
      const dx = (xs[i + 1] - xs[i]) / 3;
      const cx1 = xs[i] + dx;
      const cy1 = ys[i] + ms[i] * dx;
      const cx2 = xs[i + 1] - dx;
      const cy2 = ys[i + 1] - ms[i + 1] * dx;
      d += ` C${cx1.toFixed(1)},${cy1.toFixed(1)} ${cx2.toFixed(1)},${cy2.toFixed(1)} ${xs[i + 1].toFixed(1)},${ys[i + 1].toFixed(1)}`;
    }
    return d;
  });

  let areaPath = $derived.by(() => {
    if (!path) return '';
    return `${path} L${WIDTH},${height} L0,${height} Z`;
  });
</script>

<div class="spark">
  <div class="header">
    <span class="label">{label}</span>
    <span class="value" style="color: {barColor}">
      {formattedValue || `${Math.round(current)}${unit}`}
    </span>
  </div>
  <svg
    viewBox="0 0 {WIDTH} {height}"
    preserveAspectRatio="none"
    class="chart"
    style="height: {height}px"
  >
    {#if areaPath}
      <defs>
        <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color={barColor} stop-opacity="0.3" />
          <stop offset="100%" stop-color={barColor} stop-opacity="0.03" />
        </linearGradient>
      </defs>
      <path d={areaPath} fill="url(#{gradId})" />
      <path d={path} fill="none" stroke={barColor} stroke-width="1.5" />
    {:else}
      <line x1="0" y1={height} x2={WIDTH} y2={height} stroke="var(--track)" stroke-width="1" />
    {/if}
  </svg>
  {#if detail}
    <div class="detail">{detail}</div>
  {/if}
</div>

<style>
  .spark {
    margin-bottom: 12px;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 4px;
  }
  .label {
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-dim);
  }
  .value {
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .chart {
    width: 100%;
    display: block;
    border-radius: 4px;
    background: var(--track);
  }
  .detail {
    font-size: 11px;
    color: var(--text-dim);
    text-align: right;
    margin-top: 2px;
  }
</style>
