<script lang="ts">
  import type { BtDevice } from '$lib/types';
  let { devices }: { devices: BtDevice[] } = $props();
</script>

{#if devices.length > 0}
  <div class="bt-list">
    {#each devices as device}
      <div class="bt-item">
        <span class="bt-dot" class:connected={device.connected}></span>
        <span class="bt-name">{device.name}</span>
        <span class="bt-battery">
          {device.battery != null ? `${device.battery}%` : '--'}
        </span>
      </div>
    {/each}
  </div>
{:else}
  <div class="bt-empty">No devices</div>
{/if}

<style>
  .bt-list { display: flex; flex-direction: column; gap: 6px; }
  .bt-item { display: flex; align-items: center; gap: 8px; font-size: 13px; }
  .bt-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--text-dim); flex-shrink: 0; }
  .bt-dot.connected { background: var(--accent); }
  .bt-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bt-battery { font-variant-numeric: tabular-nums; color: var(--text-dim); font-size: 12px; }
  .bt-empty { font-size: 12px; color: var(--text-dim); }
</style>
