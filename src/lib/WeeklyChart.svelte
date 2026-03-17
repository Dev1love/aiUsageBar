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
        util: snap ? snap.five_hour_util : 0,
        isToday: i === 0,
      });
    }
    return result;
  });

  function barColor(util: number): string {
    const pct = Math.round(util * 100);
    if (pct >= 95) return '#ef4444';
    if (pct >= 80) return '#facc15';
    return '#4ade80';
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
            style="height: {Math.round(day.util * 100)}%; background-color: {barColor(day.util)}"
          ></div>
        </div>
        <span class="day-label">{day.label}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .weekly-chart {
    margin-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    padding-top: 12px;
  }
  .chart-label {
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 8px;
  }
  .chart {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    height: 80px;
    gap: 4px;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
  }
  .bar-track {
    width: 100%;
    max-width: 30px;
    height: 60px;
    background: #2a2a3e;
    border-radius: 3px;
    display: flex;
    align-items: flex-end;
    overflow: hidden;
  }
  .bar-fill {
    width: 100%;
    border-radius: 3px;
    transition: height 0.3s ease;
    min-height: 0;
  }
  .day-label {
    font-size: 10px;
    opacity: 0.5;
    margin-top: 4px;
  }
  .today .day-label {
    opacity: 1;
    font-weight: 600;
  }
  .today .bar-track {
    outline: 1px solid rgba(255, 255, 255, 0.15);
  }
</style>
