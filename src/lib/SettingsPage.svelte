<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { UserSettings } from '$lib/types';

  let { settings, onClose, onSave }: {
    settings: UserSettings;
    onClose: () => void;
    onSave: (s: UserSettings) => void;
  } = $props();

  let local = $state(JSON.parse(JSON.stringify(settings)));

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
    const idx = local.tray.items.indexOf(key);
    if (idx >= 0) {
      local.tray.items = local.tray.items.filter((k: string) => k !== key);
    } else {
      local.tray.items = [...local.tray.items, key];
    }
  }

  async function save() {
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
        <label class="tray-item">
          <input
            type="checkbox"
            checked={local.tray.items.includes(item.key)}
            onchange={() => toggleItem(item.key)}
          />
          <span>{item.label}</span>
        </label>
      {/each}
    </div>

    <div class="tray-preview">
      <span class="preview-label">Preview:</span>
      <span class="preview-text">
        {local.tray.items.join(local.tray.separator)}
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
  .tray-item input { accent-color: #34d399; }
  .tray-preview { margin-top: 10px; padding: 8px; background: rgba(255, 255, 255, 0.04); border-radius: 6px; font-size: 12px; }
  .preview-label { opacity: 0.4; margin-right: 6px; }
  .preview-text { font-family: 'SF Mono', monospace; color: #34d399; }
  .slider-label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .slider-label input[type="range"] { width: 100%; accent-color: #34d399; }
  .save-btn { width: 100%; padding: 8px; background: #34d399; color: #0f0f1a; border: none; border-radius: 6px; font-weight: 600; font-size: 13px; cursor: pointer; }
  .save-btn:hover { background: #2ec48a; }
</style>
