<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import UsageBar from '$lib/UsageBar.svelte';
  import ExtraUsage from '$lib/ExtraUsage.svelte';
  import WeeklyChart from '$lib/WeeklyChart.svelte';

  interface PeriodUsage {
    utilization: number;
    resets_at: string;
  }

  interface ExtraUsage {
    is_enabled: boolean;
    monthly_limit: number | null;
    used_credits: number | null;
    utilization: number | null;
  }

  interface UsageData {
    five_hour: PeriodUsage;
    seven_day: PeriodUsage;
    extra_usage: ExtraUsage;
  }

  let usage: UsageData | null = $state(null);
  let error: string | null = $state(null);

  onMount(() => {
    let unlistenUpdate: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    // Listen for live updates
    listen<UsageData>('usage-update', (event) => {
      usage = event.payload;
      error = null;
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>('usage-error', (event) => {
      error = event.payload;
    }).then((fn) => { unlistenError = fn; });

    // Fetch cached data on mount
    invoke<UsageData | null>('get_usage').then((cached) => {
      if (cached) usage = cached;
    }).catch(() => {
      // No cached data yet, will come via events
    });

    return () => {
      unlistenUpdate?.();
      unlistenError?.();
    };
  });
</script>

<main>
  <h1>ClaudeBar</h1>

  {#if error}
    <div class="error">
      {#if error.includes('Keychain') || error.includes('claude login')}
        Run <code>claude login</code> first
      {:else}
        Connection error
      {/if}
    </div>
  {:else if usage}
    <UsageBar
      label="5-hour session"
      utilization={usage.five_hour.utilization}
      resetsAt={usage.five_hour.resets_at}
    />
    <UsageBar
      label="7-day weekly"
      utilization={usage.seven_day.utilization}
      resetsAt={usage.seven_day.resets_at}
    />
    {#if usage.extra_usage.is_enabled && usage.extra_usage.monthly_limit != null && usage.extra_usage.used_credits != null && usage.extra_usage.utilization != null}
      <ExtraUsage
        monthlyLimit={usage.extra_usage.monthly_limit}
        usedCredits={usage.extra_usage.used_credits}
        utilization={usage.extra_usage.utilization}
      />
    {/if}
    <WeeklyChart />
  {:else}
    <p class="loading">Loading usage data…</p>
  {/if}
</main>

<style>
:global(body) {
  margin: 0;
  padding: 0;
  background-color: #1a1a2e;
  color: #e0e0e0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: 14px;
  overflow: hidden;
}

main {
  padding: 16px;
  width: 320px;
  box-sizing: border-box;
}

h1 {
  font-size: 16px;
  margin: 0 0 16px 0;
  font-weight: 600;
}

.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  padding: 12px;
  font-size: 13px;
}

.error code {
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
}

.loading {
  margin: 0;
  opacity: 0.5;
}
</style>
