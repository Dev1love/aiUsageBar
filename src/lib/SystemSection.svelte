<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children, collapsible = true }: {
    title: string;
    children: Snippet;
    collapsible?: boolean;
  } = $props();

  let collapsed = $state(false);
</script>

<section class="section">
  {#if collapsible}
    <button class="section-header clickable" onclick={() => collapsed = !collapsed}>
      <span class="section-title">{title}</span>
      <span class="chevron">{collapsed ? '▸' : '▾'}</span>
    </button>
  {:else}
    <div class="section-header">
      <span class="section-title">{title}</span>
    </div>
  {/if}
  {#if !collapsed}
    <div class="section-body">
      {@render children()}
    </div>
  {/if}
</section>

<style>
  .section {
    margin-bottom: 8px;
    contain: layout style;
  }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 10px;
    user-select: none;
    width: 100%;
    background: none;
    border-top: none;
    border-left: none;
    border-right: none;
    color: inherit;
    font: inherit;
    padding-left: 0;
    padding-right: 0;
    padding-top: 0;
  }
  .section-header.clickable {
    cursor: pointer;
  }
  .section-header.clickable:hover .section-title {
    color: var(--text);
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-dim);
    transition: color 0.15s;
  }
  .chevron {
    font-size: 10px;
    color: var(--text-dim);
    transition: color 0.15s;
  }
  .section-body {
    padding: 0;
  }
</style>
