<script lang="ts">
  let { label, utilization, resetsAt }: {
    label: string;
    utilization: number;
    resetsAt: string;
  } = $props();

  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => { now = Date.now(); }, 60_000);
    return () => clearInterval(timer);
  });

  let percent = $derived(Math.round(utilization * 100));

  let barColor = $derived(
    percent >= 95 ? '#ef4444' :
    percent >= 80 ? '#facc15' :
    '#4ade80'
  );

  let countdown = $derived.by(() => {
    const resetMs = new Date(resetsAt).getTime();
    const diff = resetMs - now;
    if (diff <= 0) return 'now';
    const hours = Math.floor(diff / 3_600_000);
    const minutes = Math.floor((diff % 3_600_000) / 60_000);
    return `${hours}h ${minutes}m`;
  });
</script>

<div class="usage-bar">
  <div class="header">
    <span class="label">{label}</span>
    <span class="percent">{percent}%</span>
  </div>
  <div class="track">
    <div class="fill" style="width: {percent}%; background-color: {barColor}"></div>
  </div>
  <div class="reset">Resets in {countdown}</div>
</div>

<style>
  .usage-bar {
    margin-bottom: 16px;
  }
  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 6px;
    font-size: 13px;
  }
  .label {
    font-weight: 500;
  }
  .percent {
    opacity: 0.8;
  }
  .track {
    height: 8px;
    background: #2a2a3e;
    border-radius: 4px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease, background-color 0.3s ease;
    min-width: 2px;
  }
  .reset {
    font-size: 11px;
    opacity: 0.5;
    margin-top: 4px;
  }
</style>
