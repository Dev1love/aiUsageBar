<script lang="ts">
  let { download, upload }: {
    download: number;
    upload: number;
  } = $props();

  function formatSpeed(bytes: number): string {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB/s`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB/s`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
    return `${bytes} B/s`;
  }
</script>

<div class="network-speed">
  <span class="down">↓ {formatSpeed(download)}</span>
  <span class="up">↑ {formatSpeed(upload)}</span>
</div>

<style>
  .network-speed {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    padding: 2px 0;
  }
  .down { color: #34d399; }
  .up { color: #60a5fa; }
</style>
