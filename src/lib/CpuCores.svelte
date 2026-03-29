<script lang="ts">
  let { cores, history }: {
    cores: number[];
    history: number[][];
  } = $props();

  const WIDTH = 60;
  const HEIGHT = 20;

  function miniPath(values: number[]): string {
    if (values.length < 2) return '';
    const len = values.length;
    const step = WIDTH / (len - 1);
    const y = (v: number) => HEIGHT - (Math.min(Math.max(v, 0), 100) / 100) * (HEIGHT - 1) - 0.5;

    let d = `M0,${y(values[0]).toFixed(1)}`;
    for (let i = 1; i < len; i++) {
      const px = i * step;
      d += ` L${px.toFixed(1)},${y(values[i]).toFixed(1)}`;
    }
    return d;
  }

  function areaPath(line: string): string {
    if (!line) return '';
    return `${line} L${WIDTH},${HEIGHT} L0,${HEIGHT} Z`;
  }

  function coreColor(util: number): string {
    if (util >= 95) return 'var(--danger)';
    if (util >= 80) return 'var(--warning)';
    return 'var(--accent)';
  }
</script>

<div class="cores-grid">
  {#each cores as util, i (i)}
    <div class="core-cell">
      <div class="core-header">
        <span class="core-label">{i + 1}</span>
        <span class="core-value" style="color: {coreColor(util)}">{Math.round(util)}%</span>
      </div>
      <svg viewBox="0 0 {WIDTH} {HEIGHT}" preserveAspectRatio="none" class="core-chart">
        {#if history[i] && history[i].length >= 2}
          {@const line = miniPath(history[i])}
          <path d={areaPath(line)} fill={coreColor(util)} fill-opacity="0.15" />
          <path d={line} fill="none" stroke={coreColor(util)} stroke-width="1" />
        {:else}
          <line x1="0" y1={HEIGHT} x2={WIDTH} y2={HEIGHT} stroke="var(--track)" stroke-width="0.5" />
        {/if}
      </svg>
    </div>
  {/each}
</div>

<style>
  .cores-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(68px, 1fr));
    gap: 6px;
    margin-top: 8px;
  }
  .core-cell {
    background: var(--bg-secondary);
    border-radius: 4px;
    padding: 4px 5px 2px;
    overflow: hidden;
  }
  .core-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 2px;
  }
  .core-label {
    font-size: 9px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
  }
  .core-value {
    font-size: 11px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .core-chart {
    width: 100%;
    height: 20px;
    display: block;
    border-radius: 2px;
  }
</style>
