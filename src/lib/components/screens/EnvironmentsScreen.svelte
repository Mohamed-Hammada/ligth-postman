<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheck, iconClose, iconEdit, iconEye, iconGlobe, iconLock, iconMonitor, iconTrash } from "$lib/components/icons.svelte";
  import NoProjectPicker from "$lib/components/screens/NoProjectPicker.svelte";
  import ProjectSwitcher from "$lib/components/screens/ProjectSwitcher.svelte";
  let { app }: { app: App } = $props();
  const { commitNewEnvVar, commitNewGlobalVar, deleteEnvironmentAction, deleteVariable, focusOnMount, loadVariables, quickCreateEnvironment, revealSecret, startRenameEnvironment, submitRenameEnvironment, t, toggleVariableEnabled, toggleVariableLocal, toggleVariableSecret } = app;
</script>

{#if !app.selectedProjectId}
  <NoProjectPicker {app} screenName={t("rail.environments")} />
{:else}
  <div class="env-screen">
    <aside class="env-screen-side">
      <div class="env-screen-side-header">
        <span class="screen-kicker">{t("env.title")}</span>
        <button type="button" class="icon-btn" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
      </div>
      <div class="env-screen-project-row">
        <span class="env-screen-project-label">{t("env.project")}</span>
        <ProjectSwitcher {app} />
      </div>
      <div class="request-search-box">
        <input
          type="search"
          placeholder={t("env.searchEnvironments")}
          bind:value={app.envSearchQuery}
          class="request-search-input"
        />
      </div>
      <div class="env-screen-list">
        {#if !app.envSearchQuery}
          <button
            type="button"
            class="env-screen-item"
            class:active={!app.selectedEnvironmentId}
            onclick={() => { app.selectedEnvironmentId = null; loadVariables(); }}
          >
            {t("env.noEnvironment")}
          </button>
        {/if}
        {#each app.filteredEnvironmentsByProject as [projectName, envs] (projectName)}
          <div class="env-screen-group-label">{projectName}</div>
          {#each envs as env (env.id)}
            {#if app.renamingEnvironmentId === env.id}
              <form class="inline-form env-screen-rename-form" onsubmit={submitRenameEnvironment}>
                <input bind:value={app.renameEnvironmentValue} use:focusOnMount onblur={submitRenameEnvironment} />
                <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingEnvironmentId = null)}>{@render iconClose()}</button>
              </form>
            {:else}
              <div class="env-screen-item-row" class:active={app.selectedEnvironmentId === env.id}>
                <button
                  type="button"
                  class="env-screen-item"
                  onclick={() => { app.selectedEnvironmentId = env.id; loadVariables(); }}
                  ondblclick={() => startRenameEnvironment(env)}
                >
                  {env.name}
                </button>
                <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameEnvironment(env)}>{@render iconEdit()}</button>
                <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.delete")} onclick={() => deleteEnvironmentAction(env.id)}>{@render iconTrash()}</button>
              </div>
            {/if}
          {/each}
        {:else}
          {#if app.envSearchQuery}
            <p class="screen-empty-inline">{t("palette.noMatches")}</p>
          {/if}
        {/each}
      </div>
    </aside>

    <section class="screen-page">
      <div class="screen-page-header">
        <span class="screen-kicker">{t("env.editing")}</span>
        <h1 class="screen-title">{app.selectedEnvironmentId ? (app.allEnvironments.find((e) => e.id === app.selectedEnvironmentId)?.name ?? t("env.fallbackName")) : t("env.globalAll")}</h1>
      </div>

      <div class="screen-page-body">
        <div class="request-search-box">
          <input
            type="search"
            placeholder={t("env.searchVariables")}
            bind:value={app.envVarSearchQuery}
            class="request-search-input"
          />
          {#if app.envVarSearchQuery}
            <span class="request-count-badge">{app.filteredProjectVariables.length + app.filteredEnvironmentVariables.length}/{app.projectVariables.length + app.environmentVariables.length}</span>
          {/if}
        </div>
        <h4>{t("env.globalVariables", { count: app.filteredProjectVariables.length })}</h4>
        <div class="params-table">
          {#each app.filteredProjectVariables as v (v.id)}
            <div class="params-row">
              <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title={t("params.enabled")} />
              <span class="var-key">{v.key}</span>
              <span class="var-val">{app.revealedSecrets[v.id] ?? v.value}</span>
              {#if v.is_local}
                <span class="badge badge-local" title={t("env.localBadgeTitle")}>{t("env.localBadge")}</span>
              {/if}
              {#if v.is_secret}
                <span class="badge">{t("env.secretBadge")}</span>
                {#if !app.revealedSecrets[v.id]}
                  <button type="button" class="icon-btn" title={t("env.reveal")} onclick={() => revealSecret(v.id)}>{@render iconEye()}</button>
                {/if}
              {/if}
              <button type="button" class="icon-btn" title={v.is_local ? t("env.makeShared") : t("env.makeLocalOnly")} onclick={() => toggleVariableLocal(v)}>{#if v.is_local}{@render iconMonitor()}{:else}{@render iconGlobe()}{/if}</button>
              <button type="button" class="icon-btn" title={t("env.toggleSecret")} onclick={() => toggleVariableSecret(v)}>{@render iconLock()}</button>
              <button type="button" class="icon-btn" title={t("sidebar.delete")} onclick={() => deleteVariable(v.id)}>{@render iconTrash()}</button>
            </div>
          {/each}
          <div
            class="params-row"
            onfocusout={(e) => {
              const row = e.currentTarget as HTMLElement;
              if (!e.relatedTarget || !row.contains(e.relatedTarget as Node)) commitNewGlobalVar();
            }}
          >
            <span class="params-row-spacer"></span>
            <input
              placeholder={t("params.key")}
              bind:value={app.newGlobalVarDraft.key}
              onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewGlobalVar(); } }}
            />
            <input
              placeholder={t("params.value")}
              bind:value={app.newGlobalVarDraft.value}
              onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewGlobalVar(); } }}
            />
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={app.newGlobalVarDraft.isSecret} /> {t("env.secret")}
            </label>
            <label class="checkbox-label" title={t("env.localHint")}>
              <input type="checkbox" bind:checked={app.newGlobalVarDraft.isLocal} /> {t("env.local")}
            </label>
          </div>
        </div>

        {#if app.selectedEnvironmentId}
          <h4>{t("env.environmentVariables", { count: app.filteredEnvironmentVariables.length })}</h4>
          <div class="params-table">
            {#each app.filteredEnvironmentVariables as v (v.id)}
              <div class="params-row">
                <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title={t("params.enabled")} />
                <span class="var-key">{v.key}</span>
                <span class="var-val">{app.revealedSecrets[v.id] ?? v.value}</span>
                {#if v.is_local}
                  <span class="badge badge-local" title={t("env.localBadgeTitle")}>{t("env.localBadge")}</span>
                {/if}
                {#if v.is_secret}
                  <span class="badge">{t("env.secretBadge")}</span>
                  {#if !app.revealedSecrets[v.id]}
                    <button type="button" class="icon-btn" title={t("env.reveal")} onclick={() => revealSecret(v.id)}>{@render iconEye()}</button>
                  {/if}
                {/if}
                <button type="button" class="icon-btn" title={v.is_local ? t("env.makeShared") : t("env.makeLocalOnly")} onclick={() => toggleVariableLocal(v)}>{#if v.is_local}{@render iconMonitor()}{:else}{@render iconGlobe()}{/if}</button>
                <button type="button" class="icon-btn" title={t("env.toggleSecret")} onclick={() => toggleVariableSecret(v)}>{@render iconLock()}</button>
                <button type="button" class="icon-btn" title={t("sidebar.delete")} onclick={() => deleteVariable(v.id)}>{@render iconTrash()}</button>
              </div>
            {/each}
            <div
              class="params-row"
              onfocusout={(e) => {
                const row = e.currentTarget as HTMLElement;
                if (!e.relatedTarget || !row.contains(e.relatedTarget as Node)) commitNewEnvVar();
              }}
            >
              <span class="params-row-spacer"></span>
              <input
                placeholder={t("params.key")}
                bind:value={app.newEnvVarDraft.key}
                onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
              />
              <input
                placeholder={t("params.value")}
                bind:value={app.newEnvVarDraft.value}
                onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
              />
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={app.newEnvVarDraft.isSecret} /> {t("env.secret")}
              </label>
              <label class="checkbox-label" title={t("env.localHint")}>
                <input type="checkbox" bind:checked={app.newEnvVarDraft.isLocal} /> {t("env.local")}
              </label>
            </div>
          </div>
        {/if}
      </div>
    </section>
  </div>
{/if}
