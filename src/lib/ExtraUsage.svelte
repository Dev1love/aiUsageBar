<script lang="ts">
  let { monthlyLimit, usedCredits, utilization }: {
    monthlyLimit: number;
    usedCredits: number;
    utilization: number;
  } = $props();

  let percent = $derived(Math.round(utilization * 100));

  let barColor = $derived(
    percent >= 95 ? '#ef4444' :
    percent >= 80 ? '#facc15' :
    '#4ade80'
  );

  let usedFormatted = $derived(`$${usedCredits.toFixed(2)}`);
  let limitFormatted = $derived(`$${monthlyLimit.toFixed(2)}`);
</script>

<div class="extra-usage">
  <div class="header">
    <span class="label">Extra usage</span>
    <span class="percent">{percent}%</span>
  </div>
  <div class="track">
    <div class="fill" style="width: {percent}%; background-color: {barColor}"></div>
  </div>
  <div class="credits">{usedFormatted} / {limitFormatted} used</div>
</div>

<style>
  .extra-usage {
    margin-bottom: 16px;
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
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
  .credits {
    font-size: 11px;
    opacity: 0.5;
    margin-top: 4px;
  }
</style>
