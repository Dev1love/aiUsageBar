<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface DailySnapshot {
    date: string;
    five_hour_util: number;
    seven_day_util: number;
    extra_usage_util: number;
  }

  let snapshots: DailySnapshot[] = $state([]);

  // Build a full 7-day array, filling gaps with zero
  let days = $derived.by(() => {
    const result: { label: string; util: number; isToday: boolean }[] = [];
    const today = new Date();
    const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    const snapshotMap = new Map(snapshots.map((s) => [s.date, s]));

    for (let i = 6; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const dateStr = d.toISOString().split('T')[0];
      const snap = snapshotMap.get(dateStr);
      result.push({
        label: dayNames[d.getDay()],
        // five_hour_util comes from API as 0-100 percentage
        util: snap ? snap.five_hour_util : 0,
        isToday: i === 0,
      });
    }
    return result;
  });

  function barColor(util: number): string {
    if (util >= 95) return 'var(--danger)';
    if (util >= 80) return 'var(--warning)';
    return 'var(--accent)';
  }

  onMount(() => {
    invoke<DailySnapshot[]>('get_history', { days: 7 }).then((data) => {
      if (data) snapshots = data;
    }).catch(() => {});
  });
</script>

<div class="weekly-chart">
  <div class="chart-label">Last 7 days</div>
  <div class="chart">
    {#each days as day}
      <div class="bar-col" class:today={day.isToday}>
        <div class="bar-track">
          <div
            class="bar-fill"
            style="height: {Math.min(day.util, 100)}%; background: {barColor(day.util)}"
          ></div>
        </div>
        <span class="day-label">{day.label}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .weekly-chart {
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
  .chart-label {
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-dim);
    margin-bottom: 10px;
  }
  .chart {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    height: 72px;
    gap: 6px;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
  }
  .bar-track {
    width: 100%;
    max-width: 28px;
    height: 52px;
    background: var(--track);
    border-radius: 4px;
    display: flex;
    align-items: flex-end;
    overflow: hidden;
  }
  .bar-fill {
    width: 100%;
    border-radius: 4px;
    transition: height 0.5s ease;
  }
  .day-label {
    font-size: 10px;
    color: var(--text-dim);
    margin-top: 6px;
    font-variant-numeric: tabular-nums;
  }
  .today .day-label {
    color: var(--text);
    font-weight: 600;
  }
  .today .bar-track {
    outline: 1px solid var(--border);
  }
</style>
