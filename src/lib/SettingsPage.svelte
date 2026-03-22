<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { UserSettings, SystemMetrics } from '$lib/types';

  let { settings, systemMetrics, onClose, onSave }: {
    settings: UserSettings;
    systemMetrics: SystemMetrics | null;
    onClose: () => void;
    onSave: (s: UserSettings) => void;
  } = $props();

  let local = $state(JSON.parse(JSON.stringify(settings)));

  // Determine which items are available based on current system metrics
  let disabledItems = $derived.by(() => {
    const disabled = new Set<string>();
    if (!systemMetrics) return disabled;
    const hasCpuTemp = systemMetrics.temps.some(t => t.name.toLowerCase().includes('cpu'));
    const hasGpuTemp = systemMetrics.temps.some(t => t.name.toLowerCase().includes('gpu'));
    const hasFans = systemMetrics.fans.length > 0;
    const hasBattery = systemMetrics.battery != null;
    if (!hasCpuTemp) disabled.add('temp_cpu');
    if (!hasGpuTemp) disabled.add('temp_gpu');
    if (!hasFans) disabled.add('fan');
    if (!hasBattery) disabled.add('battery');
    return disabled;
  });

  const availableItems = [
    { key: 'cpu', label: 'CPU %' },
    { key: 'ram', label: 'RAM' },
    { key: 'temp_cpu', label: 'CPU Temp' },
    { key: 'temp_gpu', label: 'GPU Temp' },
    { key: 'fan', label: 'Fan RPM' },
    { key: 'battery', label: 'Battery' },
    { key: 'net_down', label: 'Download' },
    { key: 'net_up', label: 'Upload' },
    { key: 'disk_free', label: 'Disk Free' },
    { key: 'ai_session', label: 'AI Session' },
    { key: 'ai_weekly', label: 'AI Weekly' },
  ];

  function toggleItem(key: string) {
    if (disabledItems.has(key)) return;
    const idx = local.tray.items.indexOf(key);
    if (idx >= 0) {
      local.tray.items = local.tray.items.filter((k: string) => k !== key);
    } else {
      local.tray.items = [...local.tray.items, key];
    }
  }

  // Build preview showing formatted labels, not raw keys
  let previewText = $derived.by(() => {
    const labelMap: Record<string, string> = {};
    for (const item of availableItems) {
      labelMap[item.key] = item.label;
    }
    return local.tray.items
      .filter((k: string) => !disabledItems.has(k))
      .map((k: string) => labelMap[k] || k)
      .join(local.tray.separator);
  });

  async function save() {
    // Remove disabled items before saving
    local.tray.items = local.tray.items.filter((k: string) => !disabledItems.has(k));
    try {
      await invoke('save_settings_cmd', { newSettings: local });
      onSave(local);
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }
</script>

<div class="settings-page">
  <div class="settings-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <h2>Settings</h2>
  </div>

  <div class="settings-section">
    <h3>Tray Display</h3>
    <div class="tray-items">
      {#each availableItems as item}
        {@const isDisabled = disabledItems.has(item.key)}
        <label class="tray-item" class:disabled={isDisabled}>
          <input
            type="checkbox"
            checked={local.tray.items.includes(item.key)}
            disabled={isDisabled}
            onchange={() => toggleItem(item.key)}
          />
          <span>{item.label}</span>
          {#if isDisabled}
            <span class="unavailable">N/A</span>
          {/if}
        </label>
      {/each}
    </div>

    <div class="tray-preview">
      <span class="preview-label">Preview:</span>
      <span class="preview-text">
        {previewText || '(empty)'}
      </span>
    </div>
  </div>

  <div class="settings-section">
    <h3>Polling Interval</h3>
    <label class="slider-label">
      System metrics: {local.polling.system_interval_sec}s
      <input
        type="range"
        min="1"
        max="30"
        bind:value={local.polling.system_interval_sec}
      />
    </label>
  </div>

  <button class="save-btn" onclick={save}>Save</button>
</div>

<style>
  .settings-page { padding: 4px 0; }
  .settings-header { display: flex; align-items: center; gap: 8px; margin-bottom: 16px; }
  .settings-header h2 { font-size: 15px; margin: 0; font-weight: 600; }
  .back-btn { background: none; border: none; color: #60a5fa; cursor: pointer; font-size: 13px; padding: 2px 6px; border-radius: 4px; }
  .back-btn:hover { background: rgba(96, 165, 250, 0.1); }
  .settings-section { margin-bottom: 16px; }
  .settings-section h3 { font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; opacity: 0.5; margin: 0 0 8px; }
  .tray-items { display: flex; flex-direction: column; gap: 6px; }
  .tray-item { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  .tray-item.disabled { opacity: 0.3; cursor: not-allowed; }
  .tray-item input { accent-color: #34d399; }
  .tray-item input:disabled { accent-color: #555; }
  .unavailable { font-size: 10px; color: #666; margin-left: auto; }
  .tray-preview { margin-top: 10px; padding: 8px; background: rgba(255, 255, 255, 0.04); border-radius: 6px; font-size: 12px; }
  .preview-label { opacity: 0.4; margin-right: 6px; }
  .preview-text { font-family: 'SF Mono', monospace; color: #34d399; }
  .slider-label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .slider-label input[type="range"] { width: 100%; accent-color: #34d399; }
  .save-btn { width: 100%; padding: 8px; background: #34d399; color: #0f0f1a; border: none; border-radius: 6px; font-weight: 600; font-size: 13px; cursor: pointer; }
  .save-btn:hover { background: #2ec48a; }
</style>
