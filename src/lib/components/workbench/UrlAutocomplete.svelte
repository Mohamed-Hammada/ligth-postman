<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { applyAutocompleteItem } = app;
</script>

{#if app.autocomplete}
  <ul
    class="autocomplete-portal"
    role="listbox"
    style="top: {app.autocomplete.top}px; left: {app.autocomplete.left}px;"
  >
    {#each app.autocomplete.items as item, i (item.insertText)}
      <li>
        <button
          type="button"
          class="autocomplete-item"
          class:active={i === app.autocomplete.activeIndex}
          onmousedown={(e) => { e.preventDefault(); applyAutocompleteItem(item); }}
        >
          <span class="autocomplete-label">{item.label}</span>
          {#if item.detail}<span class="autocomplete-detail">{item.detail}</span>{/if}
        </button>
      </li>
    {/each}
  </ul>
{/if}
