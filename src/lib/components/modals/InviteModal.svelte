<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheckCircle, iconClose, iconGitBranch, iconXCircle } from "$lib/components/icons.svelte";
  import type { CollaboratorRole } from "$lib/api";
  let { app }: { app: App } = $props();
  const { t, sendInvite } = app;

  const roles: CollaboratorRole[] = ["viewer", "collaborator", "admin"];
  const close = () => (app.showInviteModal = false);
</script>

{#if app.showInviteModal}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) close(); }}
    onkeydown={(e) => { if (e.key === "Escape") close(); }}
    role="dialog"
    aria-modal="true"
    aria-labelledby="invite-title"
    tabindex="0"
  >
    <div class="modal-container invite-modal">
      <div class="modal-header">
        <div class="modal-title-wrap">
          <h3 id="invite-title">{t("invite.title")}</h3>
          <span class="modal-sub">{t("invite.subtitle")}</span>
        </div>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={close}>{@render iconClose()}</button>
      </div>

      <div class="invite-body">
        {#if app.inviteBlocker}
          <div class="invite-blocker" role="status">
            <p>{app.inviteBlocker}</p>
            <button type="button" class="btn-primary" onclick={() => { close(); app.activeScreen = "git"; }}>{t("invite.openGit")}</button>
          </div>
        {:else}
          <div class="invite-repo">
            <span class="invite-repo-icon">{@render iconGitBranch()}</span>
            <span class="invite-repo-label">{t("invite.repo")}</span>
            <code class="invite-repo-name">{app.inviteRepoLabel}</code>
          </div>

          <form
            class="invite-form"
            onsubmit={(e) => { e.preventDefault(); sendInvite(); }}
          >
            <label class="invite-field">
              <span class="invite-field-label">{t("invite.username")}</span>
              <input
                class="request-search-input invite-input"
                bind:value={app.inviteUsername}
                placeholder={t("invite.usernamePlaceholder")}
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
              />
              <span class="invite-hint">{t("invite.emailNote")}</span>
            </label>

            <fieldset class="invite-roles">
              <legend class="invite-field-label">{t("invite.role")}</legend>
              {#each roles as r (r)}
                <label class="invite-role" class:selected={app.inviteRole === r}>
                  <input type="radio" name="invite-role" value={r} bind:group={app.inviteRole} />
                  <span class="invite-role-text">
                    <span class="invite-role-name">{t(`invite.role.${r}`)}</span>
                    <span class="invite-role-desc">{t(`invite.role.${r}.desc`)}</span>
                  </span>
                </label>
              {/each}
            </fieldset>

            {#if app.inviteError}
              <div class="invite-msg invite-msg-error" role="alert">
                <span class="invite-msg-icon">{@render iconXCircle()}</span>
                <span>{app.inviteError}</span>
              </div>
            {/if}
            {#if app.inviteNotice}
              <div class="invite-msg invite-msg-ok" role="status">
                <span class="invite-msg-icon">{@render iconCheckCircle()}</span>
                <span>{app.inviteNotice}</span>
              </div>
            {/if}

            <div class="invite-actions">
              <button type="button" class="btn-cancel" onclick={close}>{t("common.close")}</button>
              <button type="submit" class="btn-send" disabled={app.inviteBusy || !app.inviteUsername.trim()}>
                {app.inviteBusy ? t("invite.sending") : t("invite.send")}
              </button>
            </div>
          </form>

          <div class="invite-people">
            <div class="invite-field-label">{t("invite.people")}</div>
            {#if app.collaboratorsLoading}
              <p class="invite-hint">…</p>
            {:else if app.collaborators.length === 0}
              <p class="invite-hint">{t("invite.noPeople")}</p>
            {:else}
              <ul class="invite-people-list">
                {#each app.collaborators as c (c.login + (c.pending ? ":pending" : ""))}
                  <li class="invite-person">
                    {#if c.avatar_url}<img class="invite-avatar" src={c.avatar_url} alt="" width="22" height="22" />{/if}
                    <span class="invite-person-name">{c.login}</span>
                    <span class="invite-person-role">{c.role}</span>
                    {#if c.pending}<span class="invite-pending-pill">{t("invite.pending")}</span>{/if}
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
