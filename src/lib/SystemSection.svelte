<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children }: {
    title: string;
    children: Snippet;
  } = $props();
</script>

<section class="section">
  <div class="section-header">
    <span class="section-title">{title}</span>
  </div>
  <div class="section-body">
    {@render children()}
  </div>
</section>

<style>
  .section {
    margin-bottom: 8px;
  }
  .section-header {
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 10px;
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-dim);
  }
  .section-body {
    padding: 0;
  }
</style>
