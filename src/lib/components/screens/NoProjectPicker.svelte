<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconFolder } from "$lib/components/icons.svelte";
  let { app, screenName }: { app: App; screenName: string } = $props();
  const { selectProject, t } = app;
</script>

<section class="screen-page">
  <div class="screen-page-header">
    <span class="screen-kicker">{screenName}</span>
    <h1 class="screen-title">{t("env.pickProject")}</h1>
  </div>
  {#if app.projects.length}
    <div class="screen-page-body">
      <select
        class="project-picker-select"
        value=""
        onchange={(e) => {
          const id = (e.target as HTMLSelectElement).value;
          if (id) selectProject(id);
        }}
      >
        <option value="" disabled>{t("env.chooseProject")}</option>
        {#each app.projects as p (p.id)}
          <option value={p.id}>{p.name}</option>
        {/each}
      </select>
    </div>
  {:else}
    <div class="screen-empty">
      <div class="empty-icon">{@render iconFolder()}</div>
      <p>{t("env.noProjectsYet")}</p>
    </div>
  {/if}
</section>
