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

  // Popup sections management
  const sectionLabels: Record<string, string> = {
    ai_claude: 'AI Usage (Claude)',
    ai_codex: 'AI Usage (Codex)',
    weekly_chart: 'Weekly Chart',
    compute: 'Compute',
    storage_network: 'Storage & Network',
    hardware: 'Hardware',
    bluetooth: 'Bluetooth',
    history: 'History',
  };

  // Ensure all sections exist in settings (migration for older configs)
  function ensureSections() {
    if (!local.popup) local.popup = { sections: {} };
    if (!local.popup.sections) local.popup.sections = {};
    let maxOrder = -1;
    for (const cfg of Object.values(local.popup.sections) as Array<{visible: boolean; order: number}>) {
      if (cfg.order > maxOrder) maxOrder = cfg.order;
    }
    for (const key of Object.keys(sectionLabels)) {
      if (!local.popup.sections[key]) {
        maxOrder++;
        local.popup.sections[key] = { visible: true, order: maxOrder };
      }
    }
  }
  ensureSections();

  let sortedSections = $derived.by(() => {
    return Object.entries(sectionLabels)
      .map(([key, label]) => ({
        key,
        label,
        visible: local.popup.sections[key]?.visible ?? true,
        order: local.popup.sections[key]?.order ?? 99,
      }))
      .sort((a, b) => a.order - b.order);
  });

  function toggleSection(key: string) {
    local.popup.sections[key] = {
      ...local.popup.sections[key],
      visible: !local.popup.sections[key].visible,
    };
  }

  function moveSection(key: string, direction: -1 | 1) {
    const sorted = [...sortedSections];
    const idx = sorted.findIndex(s => s.key === key);
    const swapIdx = idx + direction;
    if (swapIdx < 0 || swapIdx >= sorted.length) return;
    const thisKey = sorted[idx].key;
    const otherKey = sorted[swapIdx].key;
    const thisOrder = local.popup.sections[thisKey].order;
    const otherOrder = local.popup.sections[otherKey].order;
    local.popup.sections[thisKey] = { ...local.popup.sections[thisKey], order: otherOrder };
    local.popup.sections[otherKey] = { ...local.popup.sections[otherKey], order: thisOrder };
  }

  // Drag & drop reorder (pointer-based, no HTML5 drag API issues)
  let dragIdx: number | null = $state(null);
  let overIdx: number | null = $state(null);

  function reorderSections(fromIdx: number, toIdx: number) {
    if (fromIdx === toIdx) return;
    const sorted = [...sortedSections];
    const item = sorted.splice(fromIdx, 1)[0];
    sorted.splice(toIdx, 0, item);
    const newSections = { ...local.popup.sections };
    for (let i = 0; i < sorted.length; i++) {
      newSections[sorted[i].key] = { ...newSections[sorted[i].key], order: i };
    }
    local.popup.sections = newSections;
  }

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
    <h3>Chart Style</h3>
    <div class="chart-mode-picker">
      <button
        class="mode-btn"
        class:active={local.popup.chart_mode === 'spark'}
        onclick={() => local.popup.chart_mode = 'spark'}
      >
        <svg width="24" height="14" viewBox="0 0 24 14"><path d="M0,12 C4,12 4,4 8,6 C12,8 12,2 16,2 C20,2 20,8 24,10" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
        Sparkline
      </button>
      <button
        class="mode-btn"
        class:active={local.popup.chart_mode === 'bar'}
        onclick={() => local.popup.chart_mode = 'bar'}
      >
        <svg width="24" height="14" viewBox="0 0 24 14"><rect x="0" y="4" width="18" height="6" rx="2" fill="currentColor" opacity="0.7"/><rect x="0" y="4" width="24" height="6" rx="2" fill="none" stroke="currentColor" stroke-width="1"/></svg>
        Progress Bar
      </button>
    </div>
  </div>

  <div class="settings-section">
    <label class="inline-toggle">
      <input type="checkbox" bind:checked={local.popup.show_per_core} />
      <span>Show per-core CPU</span>
    </label>
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
    <h3>Popup Sections</h3>
    <div class="section-list">
      {#each sortedSections as section, i (section.key)}
        <div class="section-row">
          <div class="section-arrows">
            <button class="arrow-btn" disabled={i === 0} onclick={() => reorderSections(i, i - 1)}>▲</button>
            <button class="arrow-btn" disabled={i === sortedSections.length - 1} onclick={() => reorderSections(i, i + 1)}>▼</button>
          </div>
          <label class="section-toggle">
            <input
              type="checkbox"
              checked={section.visible}
              onchange={() => toggleSection(section.key)}
            />
            <span class:dimmed={!section.visible}>{section.label}</span>
          </label>
        </div>
      {/each}
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
  .section-list { display: flex; flex-direction: column; gap: 2px; }
  .section-row { display: flex; align-items: center; gap: 6px; padding: 2px 0; }
  .section-arrows { display: flex; flex-direction: column; gap: 0; }
  .arrow-btn { background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 8px; padding: 0 4px; line-height: 1.2; }
  .arrow-btn:hover:not(:disabled) { color: var(--accent); }
  .arrow-btn:disabled { opacity: 0.2; cursor: default; }
  .section-toggle { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  .section-toggle input { accent-color: var(--accent); }
  .dimmed { opacity: 0.4; }
  .inline-toggle { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  .inline-toggle input { accent-color: var(--accent); }
  .chart-mode-picker { display: flex; gap: 8px; }
  .mode-btn { flex: 1; display: flex; align-items: center; justify-content: center; gap: 6px; padding: 8px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-dim); font-size: 12px; cursor: pointer; transition: all 0.15s; }
  .mode-btn:hover { border-color: var(--text-dim); }
  .mode-btn.active { border-color: var(--accent); color: var(--accent); background: var(--btn-hover); }
</style>
