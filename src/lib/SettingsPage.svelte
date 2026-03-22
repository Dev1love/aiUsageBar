<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { UserSettings, SystemMetrics } from '$lib/types';
  import { applyTheme, type ThemeName } from '$lib/themes';

  let { settings, systemMetrics, onClose, onSave }: {
    settings: UserSettings;
    systemMetrics: SystemMetrics | null;
    onClose: () => void;
    onSave: (s: UserSettings) => void;
  } = $props();

  const initialSettings = JSON.parse(JSON.stringify(settings));
  if (!initialSettings.theme) initialSettings.theme = 'glass';
  let local = $state(initialSettings);

  const themeOptions = [
    { key: 'glass', label: 'Glass', bg: 'rgba(30,30,30,0.55)', accent: 'rgba(255,255,255,0.85)' },
    { key: 'glass-blue', label: 'Glass Blue', bg: 'rgba(15,25,60,0.5)', accent: 'rgba(120,180,255,0.9)' },
    { key: 'hacker', label: 'Hacker', bg: '#000000', accent: '#00ff41' },
    { key: 'midnight', label: 'Midnight', bg: '#0f0f1a', accent: '#34d399' },
    { key: 'cyberpunk', label: 'Cyberpunk', bg: '#0d0011', accent: '#bf5af2' },
    { key: 'frost', label: 'Frost', bg: '#f0f4f8', accent: '#3182ce' },
  ];

  function previewTheme(key: string) {
    applyTheme(key as ThemeName);
  }

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
    <button class="back-btn" onclick={() => { save(); onClose(); }}>← Back</button>
    <h2>Settings</h2>
  </div>

  <div class="settings-section">
    <h3>Theme</h3>
    <div class="theme-picker">
      {#each themeOptions as theme}
        <button
          class="theme-swatch"
          class:active={local.theme === theme.key}
          style="background: {theme.bg}; border-color: {theme.accent}"
          onclick={() => { local.theme = theme.key; previewTheme(theme.key); }}
          title={theme.label}
        >
          <span class="swatch-dot" style="background: {theme.accent}"></span>
        </button>
      {/each}
    </div>
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
  .back-btn { background: none; border: none; color: var(--accent); cursor: pointer; font-size: 13px; padding: 2px 6px; border-radius: 4px; }
  .back-btn:hover { background: var(--btn-hover); }
  .settings-section { margin-bottom: 16px; }
  .settings-section h3 { font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin: 0 0 8px; }
  .tray-items { display: flex; flex-direction: column; gap: 6px; }
  .tray-item { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  .tray-item.disabled { opacity: 0.3; cursor: not-allowed; }
  .tray-item input { accent-color: var(--accent); }
  .tray-item input:disabled { accent-color: var(--text-dim); }
  .unavailable { font-size: 10px; color: var(--text-dim); margin-left: auto; }
  .tray-preview { margin-top: 10px; padding: 8px; background: var(--bg-secondary); border-radius: 6px; font-size: 12px; }
  .preview-label { color: var(--text-dim); margin-right: 6px; }
  .preview-text { font-family: 'SF Mono', monospace; color: var(--accent); }
  .slider-label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .slider-label input[type="range"] { width: 100%; accent-color: var(--accent); }
  .save-btn { width: 100%; padding: 8px; background: var(--accent); color: var(--bg); border: none; border-radius: 6px; font-weight: 600; font-size: 13px; cursor: pointer; }
  .save-btn:hover { opacity: 0.9; }
  .theme-picker { display: flex; gap: 10px; }
  .theme-swatch { width: 40px; height: 40px; border-radius: 8px; border: 2px solid transparent; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: border-color 0.2s; }
  .theme-swatch.active { border-color: var(--accent) !important; }
  .swatch-dot { width: 12px; height: 12px; border-radius: 50%; }
</style>
