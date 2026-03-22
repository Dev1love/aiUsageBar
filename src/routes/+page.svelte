<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import UsageBar from '$lib/UsageBar.svelte';
  import ExtraUsage from '$lib/ExtraUsage.svelte';
  import WeeklyChart from '$lib/WeeklyChart.svelte';
  import SystemSection from '$lib/SystemSection.svelte';
  import TempGauge from '$lib/TempGauge.svelte';
  import BluetoothList from '$lib/BluetoothList.svelte';
  import SettingsPage from '$lib/SettingsPage.svelte';
  import type { AllUsage, SystemMetrics, UserSettings } from '$lib/types';
  import NetworkSpeed from '$lib/NetworkSpeed.svelte';
  import { applyTheme, type ThemeName } from '$lib/themes';

  let usage: AllUsage | null = $state(null);
  let showSettings = $state(false);
  let error: string | null = $state(null);
  let systemMetrics: SystemMetrics | null = $state(null);
  let currentSettings: UserSettings | null = $state(null);

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
    <SystemSection title="AI Usage">
      {#if usage.claude}
        <div class="provider-block">
          <div class="provider-label">Claude Code</div>
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
        </div>
      {/if}

      {#if usage.codex}
        <div class="provider-block">
          <div class="provider-label">Codex CLI</div>
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
        </div>
      {/if}
    </SystemSection>

    <WeeklyChart />

    {#if systemMetrics}
      <SystemSection title="Compute">
        <UsageBar label="CPU" utilization={systemMetrics.cpu.overall} />
        <UsageBar
          label="RAM"
          utilization={(systemMetrics.ram.used_gb / systemMetrics.ram.total_gb) * 100}
        />
        <div class="metric-detail">
          {systemMetrics.ram.used_gb.toFixed(1)} / {systemMetrics.ram.total_gb.toFixed(0)} GB
        </div>
      </SystemSection>

      <SystemSection title="Storage & Network">
        <UsageBar
          label="Disk"
          utilization={(systemMetrics.disk.used_gb / systemMetrics.disk.total_gb) * 100}
        />
        <div class="metric-detail">
          {systemMetrics.disk.used_gb.toFixed(0)} / {systemMetrics.disk.total_gb.toFixed(0)} GB
        </div>
        <NetworkSpeed
          download={systemMetrics.network.download_speed}
          upload={systemMetrics.network.upload_speed}
        />
      </SystemSection>

      {#if systemMetrics.battery || systemMetrics.temps.length > 0 || systemMetrics.fans.length > 0}
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

      {#if systemMetrics.bluetooth.length > 0}
        <SystemSection title="Bluetooth">
          <BluetoothList devices={systemMetrics.bluetooth} />
        </SystemSection>
      {/if}
    {/if}
  {:else}
    <div class="loading">
      <div class="spinner"></div>
      <p>Connecting...</p>
    </div>
  {/if}
</main>

<style>
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
  -webkit-backdrop-filter: var(--backdrop, none);
  backdrop-filter: var(--backdrop, none);
}

main {
  padding: 18px 20px 16px;
  width: 350px;
  box-sizing: border-box;
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

.provider-block {
  margin-bottom: 8px;
}

.provider-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--text-dim);
  margin-bottom: 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border);
}

.provider-block + .provider-block {
  padding-top: 8px;
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
</style>
