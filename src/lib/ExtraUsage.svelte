<script lang="ts">
  let { monthlyLimit, usedCredits, utilization }: {
    monthlyLimit: number;
    usedCredits: number;
    utilization: number;
  } = $props();

  // API returns utilization as 0-100 percentage
  let percent = $derived(Math.round(utilization));

  let barColor = $derived(
    utilization >= 95 ? '#ef4444' :
    utilization >= 80 ? '#f59e0b' :
    '#34d399'
  );

  let barGlow = $derived(
    utilization >= 95 ? 'rgba(239, 68, 68, 0.3)' :
    utilization >= 80 ? 'rgba(245, 158, 11, 0.3)' :
    'rgba(52, 211, 153, 0.15)'
  );

  let usedFormatted = $derived(`$${usedCredits.toFixed(0)}`);
  let limitFormatted = $derived(`$${monthlyLimit.toFixed(0)}`);
</script>

<div class="extra-usage">
  <div class="header">
    <span class="label">Extra usage</span>
    <span class="percent" style="color: {barColor}">{percent}%</span>
  </div>
  <div class="track">
    <div
      class="fill"
      style="width: {Math.min(percent, 100)}%; background: {barColor}; box-shadow: 0 0 8px {barGlow}"
    ></div>
  </div>
  <div class="credits">{usedFormatted} / {limitFormatted}</div>
</div>

<style>
  .extra-usage {
    margin-bottom: 14px;
    padding-top: 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 6px;
  }
  .label {
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.7;
  }
  .percent {
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .track {
    height: 6px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 3px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.5s ease, background 0.3s ease;
    min-width: 2px;
  }
  .credits {
    font-size: 11px;
    opacity: 0.4;
    margin-top: 4px;
    text-align: right;
  }
</style>
