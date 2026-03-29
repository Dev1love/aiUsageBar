<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import UsageBar from '$lib/UsageBar.svelte';
  import ExtraUsage from '$lib/ExtraUsage.svelte';
  import WeeklyChart from '$lib/WeeklyChart.svelte';
  import SystemSection from '$lib/SystemSection.svelte';
  import SparkChart from '$lib/SparkChart.svelte';
  import CpuCores from '$lib/CpuCores.svelte';
  import TempGauge from '$lib/TempGauge.svelte';
  import BluetoothList from '$lib/BluetoothList.svelte';
  import HistoryView from '$lib/HistoryView.svelte';
  import SettingsPage from '$lib/SettingsPage.svelte';
  import type { AllUsage, SystemMetrics, UserSettings } from '$lib/types';
  import { applyTheme, type ThemeName } from '$lib/themes';

  let usage: AllUsage | null = $state(null);
  let showSettings = $state(false);
  let error: string | null = $state(null);
  let systemMetrics: SystemMetrics | null = $state(null);
  let currentSettings: UserSettings | null = $state(null);

  // Sparkline history (ring buffer, max 60 samples)
  const MAX_HISTORY = 60;
  let cpuHistory: number[] = $state([]);
  let ramHistory: number[] = $state([]);
  let netDownHistory: number[] = $state([]);
  let netUpHistory: number[] = $state([]);
  let perCoreHistory: number[][] = $state([]);

  function pushHistory(arr: number[], value: number): number[] {
    const next = [...arr, value];
    return next.length > MAX_HISTORY ? next.slice(next.length - MAX_HISTORY) : next;
  }

  function formatSpeed(bytes: number): string {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB/s`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB/s`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
    return `${bytes} B/s`;
  }

  // Chart display mode
  let chartMode = $derived((currentSettings as UserSettings | null)?.popup?.chart_mode ?? 'spark');

  // Section visibility helper
  function sectionVisible(key: string): boolean {
    const cfg = currentSettings?.popup?.sections?.[key];
    return cfg?.visible ?? true;
  }

  // Sorted section keys for rendering order
  let sortedSectionKeys = $derived.by(() => {
    const sections = currentSettings?.popup?.sections;
    if (!sections) return ['ai_claude', 'ai_codex', 'weekly_chart', 'compute', 'storage_network', 'hardware', 'bluetooth', 'history'];
    return Object.entries(sections)
      .sort(([, a], [, b]) => a.order - b.order)
      .map(([key]) => key);
  });

  onMount(() => {
    // Apply default theme immediately
    applyTheme('glass');
    let unlistenUpdate: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;
    let unlistenSystem: (() => void) | undefined;

    listen<AllUsage>('usage-update', (event) => {
      usage = event.payload;
      error = null;
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>('usage-error', (event) => {
      error = event.payload;
    }).then((fn) => { unlistenError = fn; });

    listen<SystemMetrics>('system-update', (event) => {
      systemMetrics = event.payload;
      const m = event.payload;
      cpuHistory = pushHistory(cpuHistory, m.cpu.overall);
      ramHistory = pushHistory(ramHistory, (m.ram.used_gb / m.ram.total_gb) * 100);
      netDownHistory = pushHistory(netDownHistory, m.network.download_speed);
      netUpHistory = pushHistory(netUpHistory, m.network.upload_speed);
      // Per-core history (mutate in place to avoid full re-render)
      if (m.cpu.per_core.length > 0) {
        if (perCoreHistory.length !== m.cpu.per_core.length) {
          perCoreHistory = m.cpu.per_core.map((v: number) => [v]);
        } else {
          for (let i = 0; i < m.cpu.per_core.length; i++) {
            perCoreHistory[i] = pushHistory(perCoreHistory[i], m.cpu.per_core[i]);
          }
        }
      }
    }).then((fn) => { unlistenSystem = fn; });

    invoke<AllUsage | null>('get_usage').then((cached) => {
      if (cached) usage = cached;
    }).catch(() => {});

    invoke<UserSettings>('get_settings').then((s) => {
      currentSettings = s;
      if (s.theme) applyTheme(s.theme as ThemeName);
    }).catch((e) => { console.error('Failed to load settings:', e); });

    return () => {
      unlistenUpdate?.();
      unlistenError?.();
      unlistenSystem?.();
    };
  });
</script>

<main>
  <header>
    <h1>VibeUsageBar</h1>
    <div class="header-right">
      <span class="dot" class:online={!error && usage} class:offline={error}></span>
      <button class="gear-btn" aria-label="Settings" onclick={() => {
        if (!showSettings && !currentSettings) {
          invoke<UserSettings>('get_settings').then((s) => {
            currentSettings = s;
            showSettings = true;
          }).catch((e) => { console.error('Failed to load settings on open:', e); showSettings = true; });
        } else {
          showSettings = !showSettings;
        }
      }}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
    </div>
  </header>

  {#if showSettings}
    {#if currentSettings}
      <SettingsPage
        settings={currentSettings}
        {systemMetrics}
        onClose={() => showSettings = false}
        onSave={(s) => { currentSettings = s; if (s.theme) applyTheme(s.theme as ThemeName); showSettings = false; }}
      />
    {:else}
      <div class="loading">
        <div class="spinner"></div>
        <p>Loading settings...</p>
      </div>
    {/if}
  {:else if error && !usage}
    <div class="error">
      {#if error.includes('Keychain') || error.includes('claude login')}
        <p>Run <code>claude login</code> to connect</p>
      {:else}
        <p>{error}</p>
      {/if}
    </div>
  {:else if usage}
    {#each sortedSectionKeys as sectionKey (sectionKey)}
      {#if sectionKey === 'ai_claude' && sectionVisible('ai_claude') && usage.claude}
        <SystemSection title="AI Usage — Claude">
          <UsageBar
            label="5-hour session"
            utilization={usage.claude.five_hour.utilization}
            resetsAt={usage.claude.five_hour.resets_at}
          />
          <UsageBar
            label="7-day weekly"
            utilization={usage.claude.seven_day.utilization}
            resetsAt={usage.claude.seven_day.resets_at}
          />
          {#if usage.claude.extra_usage.is_enabled && usage.claude.extra_usage.monthly_limit != null && usage.claude.extra_usage.used_credits != null && usage.claude.extra_usage.utilization != null}
            <ExtraUsage
              monthlyLimit={usage.claude.extra_usage.monthly_limit}
              usedCredits={usage.claude.extra_usage.used_credits}
              utilization={usage.claude.extra_usage.utilization}
            />
          {/if}
        </SystemSection>
      {/if}

      {#if sectionKey === 'ai_codex' && sectionVisible('ai_codex') && usage.codex}
        <SystemSection title="AI Usage — Codex">
          <UsageBar
            label="5-hour session"
            utilization={usage.codex.primary.utilization}
            resetsAt={usage.codex.primary.resets_at}
          />
          {#if usage.codex.secondary}
            <UsageBar
              label="Weekly"
              utilization={usage.codex.secondary.utilization}
              resetsAt={usage.codex.secondary.resets_at}
            />
          {/if}
          {#if usage.codex.credits}
            <div class="codex-credits">
              <span class="credits-label">Credits</span>
              <span class="credits-value" class:low={!usage.codex.credits.has_credits}>
                ${usage.codex.credits.remaining.toFixed(0)}
              </span>
            </div>
          {/if}
        </SystemSection>
      {/if}

      {#if sectionKey === 'weekly_chart' && sectionVisible('weekly_chart')}
        <WeeklyChart />
      {/if}

      {#if sectionKey === 'compute' && sectionVisible('compute') && systemMetrics}
        <SystemSection title="Compute">
          {#if chartMode === 'spark'}
            <SparkChart
              label="CPU"
              values={cpuHistory}
              current={systemMetrics.cpu.overall}
            />
            <SparkChart
              label="RAM"
              values={ramHistory}
              current={(systemMetrics.ram.used_gb / systemMetrics.ram.total_gb) * 100}
              detail="{systemMetrics.ram.used_gb.toFixed(1)} / {systemMetrics.ram.total_gb.toFixed(0)} GB"
            />
          {:else}
            <UsageBar label="CPU" utilization={systemMetrics.cpu.overall} />
            <UsageBar
              label="RAM"
              utilization={(systemMetrics.ram.used_gb / systemMetrics.ram.total_gb) * 100}
            />
            <div class="metric-detail">
              {systemMetrics.ram.used_gb.toFixed(1)} / {systemMetrics.ram.total_gb.toFixed(0)} GB
            </div>
          {/if}
          {#if currentSettings?.popup?.show_per_core !== false && systemMetrics.cpu.per_core.length > 0}
            <CpuCores cores={systemMetrics.cpu.per_core} history={perCoreHistory} />
          {/if}
        </SystemSection>
      {/if}

      {#if sectionKey === 'storage_network' && sectionVisible('storage_network') && systemMetrics}
        <SystemSection title="Storage & Network">
          <UsageBar
            label="Disk"
            utilization={(systemMetrics.disk.used_gb / systemMetrics.disk.total_gb) * 100}
          />
          <div class="metric-detail">
            {systemMetrics.disk.used_gb.toFixed(0)} / {systemMetrics.disk.total_gb.toFixed(0)} GB
          </div>
          {#if chartMode === 'spark'}
            <SparkChart
              label="Network ↓"
              values={netDownHistory}
              current={systemMetrics.network.download_speed}
              formattedValue={formatSpeed(systemMetrics.network.download_speed)}
              color="var(--net-down)"
              maxValue={Math.max(1024, ...netDownHistory)}
              height={32}
            />
            <SparkChart
              label="Network ↑"
              values={netUpHistory}
              current={systemMetrics.network.upload_speed}
              formattedValue={formatSpeed(systemMetrics.network.upload_speed)}
              color="var(--net-up)"
              maxValue={Math.max(1024, ...netUpHistory)}
              height={32}
            />
          {:else}
            <div class="network-bar">
              <span class="net-label">↓ {formatSpeed(systemMetrics.network.download_speed)}</span>
              <span class="net-label net-up">↑ {formatSpeed(systemMetrics.network.upload_speed)}</span>
            </div>
          {/if}
        </SystemSection>
      {/if}

      {#if sectionKey === 'hardware' && sectionVisible('hardware') && systemMetrics && (systemMetrics.battery || systemMetrics.temps.length > 0 || systemMetrics.fans.length > 0)}
        <SystemSection title="Hardware">
          <TempGauge temps={systemMetrics.temps} fans={systemMetrics.fans} />
          {#if systemMetrics.battery}
            <div class="battery-info">
              <span class="battery-icon">{systemMetrics.battery.charging ? '⚡' : '🔋'}</span>
              <span class="battery-percent">{systemMetrics.battery.percent.toFixed(0)}%</span>
              <span class="battery-detail">
                Health {systemMetrics.battery.health_percent.toFixed(0)}%
                · {systemMetrics.battery.cycle_count} cycles
              </span>
            </div>
          {/if}
        </SystemSection>
      {/if}

      {#if sectionKey === 'bluetooth' && sectionVisible('bluetooth') && systemMetrics && systemMetrics.bluetooth.length > 0}
        <SystemSection title="Bluetooth">
          <BluetoothList devices={systemMetrics.bluetooth} />
        </SystemSection>
      {/if}

      {#if sectionKey === 'history' && sectionVisible('history')}
        <SystemSection title="History">
          <HistoryView />
        </SystemSection>
      {/if}
    {/each}
  {:else}
    <div class="loading">
      <div class="spinner"></div>
      <p>Connecting...</p>
    </div>
  {/if}
</main>

<style>
:global(html) {
  background: transparent;
}

:global(body) {
  margin: 0;
  padding: 0;
  background-color: var(--bg);
  color: var(--text);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', sans-serif;
  font-size: 14px;
  overflow-x: hidden;
  overflow-y: auto;
  -webkit-font-smoothing: antialiased;
  position: relative;
}

:global(body::before) {
  content: '';
  position: fixed;
  inset: 0;
  -webkit-backdrop-filter: var(--backdrop, none);
  backdrop-filter: var(--backdrop, none);
  z-index: -1;
  pointer-events: none;
}

main {
  padding: 18px 20px 16px;
  width: 350px;
  box-sizing: border-box;
  position: relative;
  z-index: 0;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

h1 {
  font-size: 15px;
  margin: 0;
  font-weight: 600;
  letter-spacing: -0.3px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-dim);
}

.dot.online {
  background: var(--accent);
  box-shadow: 0 0 6px var(--accent-glow);
}

.dot.offline {
  background: var(--danger);
}


.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gear-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}

.gear-btn:hover {
  color: var(--text);
  background: var(--btn-hover);
}

.codex-credits {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  font-size: 12px;
  color: var(--text-dim);
  margin-top: 4px;
}

.credits-value {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--accent);
}

.credits-value.low {
  color: var(--danger);
}

.error {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.15);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}

.error p {
  margin: 0;
  font-size: 13px;
  color: var(--text-dim);
}

.error code {
  background: var(--btn-hover);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px 0;
}

.loading p {
  margin: 0;
  font-size: 13px;
  color: var(--text-dim);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.metric-detail {
  font-size: 11px;
  color: var(--text-dim);
  text-align: right;
  margin-top: -8px;
  margin-bottom: 8px;
}

.battery-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
}
.battery-icon {
  font-size: 16px;
}
.battery-percent {
  font-weight: 700;
  font-size: 18px;
  font-variant-numeric: tabular-nums;
  color: var(--accent);
}
.battery-detail {
  font-size: 11px;
  color: var(--text-dim);
}

.network-bar {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  padding: 2px 0;
}
.net-label { color: var(--net-down); }
.net-label.net-up { color: var(--net-up); }
</style>
