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

  interface ExtraUsageData {
    is_enabled: boolean;
    monthly_limit: number | null;
    used_credits: number | null;
    utilization: number | null;
  }

  interface UsageData {
    five_hour: PeriodUsage;
    seven_day: PeriodUsage;
    extra_usage: ExtraUsageData;
  }

  let usage: UsageData | null = $state(null);
  let error: string | null = $state(null);

  onMount(() => {
    let unlistenUpdate: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    listen<UsageData>('usage-update', (event) => {
      usage = event.payload;
      error = null;
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>('usage-error', (event) => {
      error = event.payload;
    }).then((fn) => { unlistenError = fn; });

    invoke<UsageData | null>('get_usage').then((cached) => {
      if (cached) usage = cached;
    }).catch(() => {});

    return () => {
      unlistenUpdate?.();
      unlistenError?.();
    };
  });
</script>

<main>
  <header>
    <h1>ClaudeBar</h1>
    <span class="dot" class:online={!error && usage} class:offline={error}></span>
  </header>

  {#if error}
    <div class="error">
      {#if error.includes('Keychain') || error.includes('claude login')}
        <p>Run <code>claude login</code> to connect</p>
      {:else}
        <p>Connection error</p>
      {/if}
    </div>
  {:else if usage}
    <div class="bars">
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
    </div>
    {#if usage.extra_usage.is_enabled && usage.extra_usage.monthly_limit != null && usage.extra_usage.used_credits != null && usage.extra_usage.utilization != null}
      <ExtraUsage
        monthlyLimit={usage.extra_usage.monthly_limit}
        usedCredits={usage.extra_usage.used_credits}
        utilization={usage.extra_usage.utilization}
      />
    {/if}
    <WeeklyChart />
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
  background-color: #0f0f1a;
  color: #e2e2ea;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', sans-serif;
  font-size: 14px;
  overflow: hidden;
  -webkit-font-smoothing: antialiased;
}

main {
  padding: 18px 20px 16px;
  width: 300px;
  box-sizing: border-box;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
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
  background: #555;
}

.dot.online {
  background: #34d399;
  box-shadow: 0 0 6px rgba(52, 211, 153, 0.4);
}

.dot.offline {
  background: #ef4444;
}

.bars {
  display: flex;
  flex-direction: column;
  gap: 2px;
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
  opacity: 0.7;
}

.error code {
  background: rgba(255, 255, 255, 0.08);
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
  opacity: 0.4;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: #34d399;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
