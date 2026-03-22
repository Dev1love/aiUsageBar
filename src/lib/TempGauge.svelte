<script lang="ts">
  import type { TempSensor, FanInfo } from '$lib/types';

  let { temps, fans }: {
    temps: TempSensor[];
    fans: FanInfo[];
  } = $props();

  function tempColor(value: number): string {
    if (value >= 80) return 'var(--danger)';
    if (value >= 50) return 'var(--warning)';
    return 'var(--accent)';
  }
</script>

<div class="hardware-metrics">
  {#if temps.length > 0}
    <div class="temps-row">
      {#each temps as sensor}
        <div class="temp-item">
          <span class="temp-label">{sensor.name}</span>
          <span class="temp-value" style="color: {tempColor(sensor.value)}">
            {sensor.value.toFixed(0)}°C
          </span>
        </div>
      {/each}
    </div>
  {/if}

  {#if fans.length > 0}
    <div class="fans-row">
      {#each fans as fan}
        <span class="fan-item">{fan.name}: {fan.rpm} rpm</span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .hardware-metrics { display: flex; flex-direction: column; gap: 6px; }
  .temps-row { display: flex; flex-wrap: wrap; gap: 12px; }
  .temp-item { display: flex; gap: 6px; align-items: baseline; }
  .temp-label { font-size: 12px; color: var(--text-dim); }
  .temp-value { font-size: 15px; font-weight: 700; font-variant-numeric: tabular-nums; }
  .fans-row { display: flex; gap: 12px; font-size: 12px; color: var(--text-dim); }
</style>
