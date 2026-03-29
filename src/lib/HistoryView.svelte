<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { BatterySnapshot, NetworkDaily } from '$lib/types';

  let activeTab: 'battery' | 'network' = $state('battery');
  let batteryData: BatterySnapshot[] = $state([]);
  let networkData: NetworkDaily[] = $state([]);
  let loading = $state(true);

  const SPARK_WIDTH = 280;
  const SPARK_HEIGHT = 40;

  function formatBytes(bytes: number): string {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  }

  // Battery sparkline path (monotone cubic)
  let healthValues = $derived(batteryData.map((s) => s.health_percent));

  let latestHealth = $derived(
    batteryData.length > 0 ? batteryData[batteryData.length - 1].health_percent : null
  );
  let latestCycles = $derived(
    batteryData.length > 0 ? batteryData[batteryData.length - 1].cycle_count : null
  );

  let healthPath = $derived.by(() => {
    const values = healthValues;
    if (values.length < 2) return '';
    const len = values.length;
    const step = SPARK_WIDTH / (len - 1);
    const maxVal = 100;
    const fy = (v: number) => SPARK_HEIGHT - (Math.max(0, Math.min(v, maxVal)) / maxVal) * (SPARK_HEIGHT - 2) - 1;
    const xs = Array.from({ length: len }, (_, i) => i * step);
    const ys = values.map(fy);

    if (len === 2) return `M${xs[0]},${ys[0]} L${xs[1]},${ys[1]}`;

    const ds: number[] = [];
    const ms: number[] = [];
    for (let i = 0; i < len - 1; i++) ds.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
    ms.push(ds[0]);
    for (let i = 1; i < len - 1; i++) ms.push((ds[i - 1] + ds[i]) / 2);
    ms.push(ds[len - 2]);
    for (let i = 0; i < len - 1; i++) {
      if (Math.abs(ds[i]) < 1e-6) { ms[i] = 0; ms[i + 1] = 0; }
      else {
        const a = ms[i] / ds[i], b = ms[i + 1] / ds[i], s = a * a + b * b;
        if (s > 9) { const t = 3 / Math.sqrt(s); ms[i] = t * a * ds[i]; ms[i + 1] = t * b * ds[i]; }
      }
    }

    let d = `M${xs[0].toFixed(1)},${ys[0].toFixed(1)}`;
    for (let i = 0; i < len - 1; i++) {
      const dx = (xs[i + 1] - xs[i]) / 3;
      d += ` C${(xs[i] + dx).toFixed(1)},${(ys[i] + ms[i] * dx).toFixed(1)} ${(xs[i + 1] - dx).toFixed(1)},${(ys[i + 1] - ms[i + 1] * dx).toFixed(1)} ${xs[i + 1].toFixed(1)},${ys[i + 1].toFixed(1)}`;
    }
    return d;
  });

  let healthAreaPath = $derived(healthPath ? `${healthPath} L${SPARK_WIDTH},${SPARK_HEIGHT} L0,${SPARK_HEIGHT} Z` : '');

  // Network chart
  let networkTotal = $derived.by(() => {
    let dl = 0, ul = 0;
    for (const d of networkData) { dl += d.total_download_bytes; ul += d.total_upload_bytes; }
    return { download: dl, upload: ul };
  });

  let networkMax = $derived.by(() => {
    let max = 1;
    for (const d of networkData) {
      const total = d.total_download_bytes + d.total_upload_bytes;
      if (total > max) max = total;
    }
    return max;
  });

  // Build full 14-day array
  let networkDays = $derived.by(() => {
    const map = new Map(networkData.map((d) => [d.date, d]));
    const result: { label: string; date: string; download: number; upload: number; isToday: boolean }[] = [];
    const today = new Date();
    for (let i = 13; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const dateStr = d.toISOString().split('T')[0];
      const snap = map.get(dateStr);
      result.push({
        label: `${d.getMonth() + 1}/${d.getDate()}`,
        date: dateStr,
        download: snap?.total_download_bytes ?? 0,
        upload: snap?.total_upload_bytes ?? 0,
        isToday: i === 0,
      });
    }
    return result;
  });

  function healthColor(h: number): string {
    if (h < 70) return 'var(--danger)';
    if (h < 85) return 'var(--warning)';
    return 'var(--accent)';
  }

  onMount(() => {
    Promise.all([
      invoke<BatterySnapshot[]>('get_battery_history', { days: 30 }).catch(() => [] as BatterySnapshot[]),
      invoke<NetworkDaily[]>('get_network_daily', { days: 14 }).catch(() => [] as NetworkDaily[]),
    ]).then(([battery, network]) => {
      batteryData = battery;
      networkData = network;
      loading = false;
    });
  });
</script>

<div class="history">
  <div class="tabs">
    <button class="tab" class:active={activeTab === 'battery'} onclick={() => activeTab = 'battery'}>Battery</button>
    <button class="tab" class:active={activeTab === 'network'} onclick={() => activeTab = 'network'}>Network</button>
  </div>

  {#if loading}
    <div class="history-loading">Loading...</div>
  {:else if activeTab === 'battery'}
    <div class="battery-tab">
      {#if batteryData.length === 0}
        <div class="empty">No battery data yet</div>
      {:else}
        <div class="spark-header">
          <span class="spark-label">Health (30 days)</span>
          {#if latestHealth != null}
            <span class="spark-value" style="color: {healthColor(latestHealth)}">{latestHealth.toFixed(0)}%</span>
          {/if}
        </div>
        <svg
          viewBox="0 0 {SPARK_WIDTH} {SPARK_HEIGHT}"
          preserveAspectRatio="none"
          class="spark-chart"
          style="height: {SPARK_HEIGHT}px"
        >
          {#if healthAreaPath}
            <defs>
              <linearGradient id="grad-health" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.3" />
                <stop offset="100%" stop-color="var(--accent)" stop-opacity="0.03" />
              </linearGradient>
            </defs>
            <path d={healthAreaPath} fill="url(#grad-health)" />
            <path d={healthPath} fill="none" stroke="var(--accent)" stroke-width="1.5" />
          {:else}
            <line x1="0" y1={SPARK_HEIGHT} x2={SPARK_WIDTH} y2={SPARK_HEIGHT} stroke="var(--track)" stroke-width="1" />
          {/if}
        </svg>
        {#if latestCycles != null}
          <div class="battery-stats">
            <span>{latestCycles} cycles</span>
          </div>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="network-tab">
      {#if networkData.length === 0}
        <div class="empty">No network data yet</div>
      {:else}
        <div class="net-summary">
          <span class="net-total"><span class="net-dl-dot"></span> {formatBytes(networkTotal.download)}</span>
          <span class="net-total"><span class="net-ul-dot"></span> {formatBytes(networkTotal.upload)}</span>
        </div>
        <div class="net-chart">
          {#each networkDays as day}
            <div class="net-bar-col" class:today={day.isToday}>
              <div class="net-bar-track">
                <div
                  class="net-bar-fill net-bar-ul"
                  style="height: {networkMax > 0 ? ((day.upload) / networkMax) * 100 : 0}%"
                ></div>
                <div
                  class="net-bar-fill net-bar-dl"
                  style="height: {networkMax > 0 ? ((day.download) / networkMax) * 100 : 0}%"
                ></div>
              </div>
              <span class="net-day-label">{day.label}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .history {
    padding: 0;
  }
  .tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 12px;
  }
  .tab {
    flex: 1;
    padding: 5px 0;
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: var(--track);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-dim);
    cursor: pointer;
    transition: all 0.15s;
  }
  .tab.active {
    background: var(--bg-secondary);
    color: var(--accent);
    border-color: var(--accent);
  }
  .tab:hover:not(.active) {
    color: var(--text);
  }
  .history-loading {
    font-size: 12px;
    color: var(--text-dim);
    text-align: center;
    padding: 16px 0;
  }
  .empty {
    font-size: 12px;
    color: var(--text-dim);
    text-align: center;
    padding: 16px 0;
  }

  /* Battery tab */
  .spark-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 4px;
  }
  .spark-label {
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-dim);
  }
  .spark-value {
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .spark-chart {
    width: 100%;
    display: block;
    border-radius: 4px;
    background: var(--track);
  }
  .battery-stats {
    font-size: 11px;
    color: var(--text-dim);
    text-align: right;
    margin-top: 4px;
  }

  /* Network tab */
  .net-summary {
    display: flex;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .net-total {
    font-size: 12px;
    color: var(--text-dim);
    display: flex;
    align-items: center;
    gap: 4px;
    font-variant-numeric: tabular-nums;
  }
  .net-dl-dot, .net-ul-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .net-dl-dot { background: var(--net-down); }
  .net-ul-dot { background: var(--net-up); }
  .net-chart {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    height: 72px;
    gap: 3px;
  }
  .net-bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
  }
  .net-bar-track {
    width: 100%;
    max-width: 18px;
    height: 52px;
    background: var(--track);
    border-radius: 3px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    overflow: hidden;
  }
  .net-bar-fill {
    width: 100%;
    transition: height 0.5s ease;
    min-height: 0;
  }
  .net-bar-dl { background: var(--net-down); border-radius: 0 0 3px 3px; }
  .net-bar-ul { background: var(--net-up); }
  .net-day-label {
    font-size: 8px;
    color: var(--text-dim);
    margin-top: 4px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .today .net-day-label {
    color: var(--text);
    font-weight: 600;
  }
  .today .net-bar-track {
    outline: 1px solid var(--border);
  }
</style>
