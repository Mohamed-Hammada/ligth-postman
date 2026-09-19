<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { selectProject } = app;
</script>

<select
  class="project-picker-select project-picker-select-sm"
  value={app.selectedProjectId ?? ""}
  onchange={(e) => {
    const id = (e.target as HTMLSelectElement).value;
    if (id) selectProject(id);
  }}
>
  {#each app.projects as p (p.id)}
    <option value={p.id}>{p.name}</option>
  {/each}
</select>
