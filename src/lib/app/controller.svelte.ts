// The app controller: all state, derived values and actions that used to live in the +page.svelte <script>.
// Views (src/lib/components/**) receive the returned facade as `app` — reading `app.x` stays reactive because
// every state field is exposed through a getter/setter over the underlying rune.
import { onMount, tick, untrack } from "svelte";
import { translate, RTL_LOCALES, type Locale } from "$lib/i18n";
import {
  api,
  describeError,
  isAppError,
  type Auth,
  type CollectionImportReport,
  type Environment,
  type EnvironmentWithProject,
  type EnvironmentImportReport,
  type Folder,
  type GeneratedApiDefinition,
  type FormDataPart,
  type HeaderEntry,
  type LocalWorkspaceImportReport,
  type Project,
  type QueryParam,
  type UrlEncodedItem,
  type RequestFull,
  type RequestDiagnostics,
  type RequestSettings,
  type RequestSummary,
  type RequestSearchResult,
  type SearchField,
  type ResolvedTemplate,
  type ResponseMeta,
  type ResponseSummary,
  type SnippetMode,
  type SnippetTarget,
  type VariableView,
  type ConsoleEvent,
  type ConsoleLevel,
  type GitStatus,
  type GitCommit,
  type WorkspaceGitSettings,
  type LegacyGitSettingsCandidate,
  type WorkspaceImportReport,
  type GitHubUser,
  type GitHubRepoInfo,
  type GitHubCollaborator,
  type CollaboratorRole,
  type AiSettings,
  type AiProviderKind,
  type DiscoveredEndpoint,
  type GeneratedTestsAndDocs,
  type SampleResponse,
  type SourceProjectReport,
  type UpdateAiSettingsInput,
  type SystemDiagnostics,
  type Workspace,
  type ProjectHistoryEntry,
  type ConflictVersions,
  } from "$lib/api";
import type { ScreenId, ThemeMode, PaletteScope, ShortcutId, PaletteItem, RequestTab, RequestDraft, SidebarSearchScope, ProjectSortField, RequestSortField, SortDir, DetailHeaderRow, DetailCookieRow, UrlToken, AutocompleteItem, AutocompleteState } from "./types";

export function createApp() {
  // Svelte action: focuses an element as soon as it mounts (e.g. an inline-rename input that
  // just appeared). Same effect as the `autofocus` attribute, without the a11y lint warning
  // that attribute triggers — this is the accessible-equivalent pattern.
  function focusOnMount(node: HTMLElement) {
    node.focus();
    // Select any placeholder/existing text (e.g. "New Project") so typing replaces it outright —
    // the whole point of dropping straight into rename mode is writing the name in one go.
    if (node instanceof HTMLInputElement) {
      node.select();
    }
  }

  // Workspaces group projects — one level above Project (Workspace -> Project -> Folder/Request).
  // There's always at least one (the backend seeds/protects a 'default' workspace), so
  // activeWorkspaceId only stays null for the instant before the first load resolves.
  let workspaces = $state<Workspace[]>([]);
  let sidebarSection = $state<"collections" | "environments" | "history" | "mocks" | "specs" | "projects">("collections");
  let collectionsAccordionOpen = $state(true);
  let environmentsAccordionOpen = $state(false);
  let documentsAccordionOpen = $state(false);
  let specsAccordionOpen = $state(false);
  let mocksAccordionOpen = $state(false);
  let datasetsAccordionOpen = $state(false);
  let flowsAccordionOpen = $state(false);

  // Customizable sidebar section visibility ("only collection/environment/dataset used, others hidden in a view menu")
  let sidebarSectionsVisible = $state({
    collections: true,
    environments: true,
    datasets: false,
    documents: false,
    specs: false,
    mocks: false,
    flows: false,
  });
  let sidebarViewMenuOpen = $state(false);

  if (typeof localStorage !== "undefined") {
    try {
      const saved = localStorage.getItem("lp_sidebar_sections_visibility");
      if (saved) {
        const parsed = JSON.parse(saved);
        sidebarSectionsVisible = { ...sidebarSectionsVisible, ...parsed };
      }
    } catch {
      // ignore
    }
  }

  function toggleSidebarSectionVisibility(section: keyof typeof sidebarSectionsVisible) {
    sidebarSectionsVisible[section] = !sidebarSectionsVisible[section];
    if (typeof localStorage !== "undefined") {
      try {
        localStorage.setItem("lp_sidebar_sections_visibility", JSON.stringify(sidebarSectionsVisible));
      } catch {}
    }
  }
  let notificationsOpen = $state(false);
  let accountMenuOpen = $state(false);
  let envForkCount = $state(0);
  let sendOptionsOpen = $state(false);
  let saveOptionsOpen = $state(false);
  let showSnippetSettings = $state(false);
  let showInviteModal = $state(false);
  let inviteUsername = $state("");
  let inviteRole = $state<CollaboratorRole>("collaborator");
  let inviteBusy = $state(false);
  let inviteError = $state("");
  let inviteNotice = $state("");
  let collaborators = $state<GitHubCollaborator[]>([]);
  let collaboratorsLoading = $state(false);
  let snippetIndentType = $state<"space" | "tab">("space");
  let snippetTrimTrailing = $state(true);

  const sampleDocuments = [
    { id: "doc-overview", name: "API Reference Overview" },
    { id: "doc-auth", name: "Authentication & Security Guide" },
    { id: "doc-remittance", name: "Remittance & Payment Flow" },
    { id: "doc-changelog", name: "Release Notes & Changelog" },
  ];

  const sampleSpecs = [
    { id: "spec-openapi", name: "openapi.json (OpenAPI 3.1)" },
    { id: "spec-swagger", name: "auth-service.yaml (Swagger 2.0)" },
    { id: "spec-remittance", name: "remittance-v2-schema.json" },
  ];

  const sampleMockServers = [
    { id: "mock-local", name: "Local Mock Server (Port 8080)" },
    { id: "mock-auth-200", name: "200 OK - Success Sample Mock" },
    { id: "mock-auth-401", name: "401 Unauthorized Error Mock" },
    { id: "mock-card-200", name: "200 OK - Card Transaction Response" },
  ];

  const sampleDatasets = [
    { id: "data-users", name: "users-test-data.json" },
    { id: "data-payments", name: "payments-batch.csv" },
  ];

  const sampleFlows = [
    { id: "flow-login", name: "User Login & Token Exchange Flow" },
    { id: "flow-onboard", name: "Card Onboarding & PIN Setup Flow" },
    { id: "flow-topup", name: "Batch Remittance & Poll Flow" },
  ];

  const sampleDocContent = `# API Reference Overview

## Base URL
- Production: \`https://api.example.com/v1\`
- UAT / Staging: \`https://staging.api.example.com\`
- Sandbox: \`https://api-sandbox.network.global\`

## Authentication
All requests must include a valid session token obtained from \`/api/v1/auth/login\` and verified via SMS OTP at \`/api/v1/auth/validate-otp\`.

## Endpoints
1. \`POST /api/v1/auth/login\` - Authenticates mobile number and encrypted PIN.
2. \`POST /api/v1/auth/validate-otp\` - Confirms SMS one-time passcode.
3. \`POST /api/v1/card/setCardCon\` - Updates user debit/credit card parameters.
4. \`POST /api/v1/card/transaction\` - Submits an instant remittance payment order.
5. \`POST /api/v1/wps/topup\` - Performs top-up on registered WPS salary accounts.
`;

  const sampleOpenApiJson = JSON.stringify({
    openapi: "3.1.0",
    info: {
      title: "Ansari Remittance & Payment Gateway API",
      version: "2.4.0",
      description: "High-throughput financial exchange & transaction services."
    },
    servers: [
      { url: "https://staging.api.example.com", description: "UAT Gateway" },
      { url: "https://api-sandbox.network.global", description: "Global Sandbox" }
    ],
    paths: {
      "/api/v1/auth/login": {
        post: {
          summary: "Authenticate user and issue session token",
          responses: { "200": { description: "Successful login response" } }
        }
      },
      "/api/v1/auth/validate-otp": {
        post: {
          summary: "Validate SMS OTP",
          responses: { "200": { description: "Device verification confirmed" } }
        }
      },
      "/api/v1/card/transaction": {
        post: {
          summary: "Initiate Card Remittance",
          responses: { "200": { description: "Transaction completed" } }
        }
      }
    }
  }, null, 2);

  let activeWorkspaceId = $state<string | null>(null);
  let workspacePickerOpen = $state(false);
  async function loadWorkspaces() {
    try {
      workspaces = await api.listWorkspaces();
      if (!activeWorkspaceId || !workspaces.some((w) => w.id === activeWorkspaceId)) {
        const saved = (() => {
          try {
            return localStorage.getItem("lp-active-workspace");
          } catch {
            return null;
          }
        })();
        const restored = saved && workspaces.some((w) => w.id === saved) ? saved : null;
        activeWorkspaceId = restored ?? workspaces[0]?.id ?? null;
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }
  async function selectWorkspace(id: string) {
    if (id === activeWorkspaceId) {
      workspacePickerOpen = false;
      return;
    }
    activeWorkspaceId = id;
    workspacePickerOpen = false;
    try {
      localStorage.setItem("lp-active-workspace", id);
    } catch {
      // workspace choice just won't persist across restarts
    }
    selectedProjectId = null;
    selectedRequest = null;
    openTabs = [];
    tabDrafts.clear();
    await loadProjects();
    await loadGitSettings();
  }
  let renamingWorkspaceId = $state<string | null>(null);
  let renameWorkspaceValue = $state("");
  function startRenameWorkspace(ws: Workspace) {
    renamingWorkspaceId = ws.id;
    renameWorkspaceValue = ws.name;
  }
  async function submitRenameWorkspace(e: Event) {
    e.preventDefault();
    const id = renamingWorkspaceId;
    renamingWorkspaceId = null;
    if (!id) return;
    const name = renameWorkspaceValue.trim();
    if (!name) return;
    try {
      const updated = await api.updateWorkspace(id, name);
      workspaces = workspaces.map((w) => (w.id === id ? updated : w)).sort((a, b) => a.name.localeCompare(b.name));
    } catch (err) {
      errorMessage = describeError(err);
    }
  }
  // Same "create with a placeholder name, then drop straight into inline-rename" pattern as
  // quickCreateProject — except the picker menu has to stay open so the rename form (which
  // lives inside it) is actually visible.
  async function quickCreateWorkspace() {
    try {
      const workspace = await api.createWorkspace(t("workspace.defaultName"));
      workspaces = [...workspaces, workspace].sort((a, b) => a.name.localeCompare(b.name));
      activeWorkspaceId = workspace.id;
      try {
        localStorage.setItem("lp-active-workspace", workspace.id);
      } catch {
        // workspace choice just won't persist across restarts
      }
      selectedProjectId = null;
      selectedRequest = null;
      openTabs = [];
      tabDrafts.clear();
      await loadProjects();
      await loadGitSettings();
      startRenameWorkspace(workspace);
      workspacePickerOpen = true;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  let projects = $state<Project[]>([]);
  let selectedProjectId = $state<string | null>(null);

  // Sidebar tree expansion is independent of which project is "active" (selectedProjectId):
  // several projects can stay expanded/browsable at once, VS Code multi-root style. Only the
  // active project gets the full-featured tree (search/sort/pagination/CRUD, driven by the
  // `requests`/`folders` state below); every other expanded project renders from this
  // read-mostly cache instead — see secondaryFolderNode/secondaryRequestRow and selectProject's
  // "snapshot the outgoing project into the cache" step.
  let expandedProjectIds = $state<Set<string>>(new Set());
  let secondaryProjectCache = $state<Map<string, { requests: RequestSummary[]; folders: Folder[] }>>(new Map());
  let secondaryProjectLoading = $state<Set<string>>(new Set());

  let requests = $state<RequestSummary[]>([]);
  let selectedRequest = $state<RequestFull | null>(null);

  // Flat (non-nested) folders — a request either sits directly under its project or under one
  // folder in that project (see models::Folder on the backend).
  let folders = $state<Folder[]>([]);
  let expandedFolderIds = $state<Set<string>>(new Set(["folder-auth-flow"]));
  let renamingFolderId = $state<string | null>(null);
  let renameFolderValue = $state("");

  // Global — every project's environment picker offers every environment from every project
  // (not just its own), so this loads once and isn't re-scoped per selected project.
  let allEnvironments = $state<EnvironmentWithProject[]>([]);
  async function loadAllEnvironments() {
    try {
      allEnvironments = await api.listAllEnvironments();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }
  // Grouped for display (the picker shows every environment from every project, so a heading
  // per source project keeps a 77+-entry list navigable) — already ordered by the backend query.
  let environmentsByProject = $derived.by(() => {
    const map = new Map<string, EnvironmentWithProject[]>();
    for (const env of allEnvironments) {
      const list = map.get(env.project_name) ?? [];
      list.push(env);
      map.set(env.project_name, list);
    }
    return map;
  });
  // Environments screen search (LP redesign: this screen had no filtering at all, unlike the
  // request sidebar — a gap on the same "large workspace" scale the rest of the app targets).
  let envSearchQuery = $state("");
  let filteredEnvironmentsByProject = $derived.by(() => {
    const q = envSearchQuery.trim().toLocaleLowerCase();
    if (!q) return environmentsByProject;
    const filtered = new Map<string, EnvironmentWithProject[]>();
    for (const [projectName, envs] of environmentsByProject) {
      const matches = envs.filter((e) => e.name.toLocaleLowerCase().includes(q));
      if (matches.length) filtered.set(projectName, matches);
    }
    return filtered;
  });
  let selectedEnvironmentId = $state<string | null>(null);
  // Topbar environment picker — its own open/search state, separate from the Environments
  // screen's envSearchQuery, so typing here doesn't leave a stale filter behind on that screen.
  let envPickerOpen = $state(false);
  let envPickerQuery = $state("");
  let envPickerFilteredByProject = $derived.by(() => {
    const q = envPickerQuery.trim().toLocaleLowerCase();
    if (!q) return environmentsByProject;
    const filtered = new Map<string, EnvironmentWithProject[]>();
    for (const [projectName, envs] of environmentsByProject) {
      const matches = envs.filter((e) => e.name.toLocaleLowerCase().includes(q));
      if (matches.length) filtered.set(projectName, matches);
    }
    return filtered;
  });
  let envPickerFilteredEnvironments = $derived.by(() => {
    const q = envPickerQuery.trim().toLocaleLowerCase();
    if (!q) return allEnvironments;
    return allEnvironments.filter((e) => e.name.toLocaleLowerCase().includes(q));
  });
  function openEnvPicker() {
    envPickerQuery = "";
    envPickerOpen = true;
  }
  function pickEnvironment(id: string | null) {
    selectedEnvironmentId = id;
    envPickerOpen = false;
    loadVariables();
    if (id && !expandedEnvIds.has(id)) {
      toggleEnvExpand(id);
    }
  }
  let renamingEnvironmentId = $state<string | null>(null);
  let renameEnvironmentValue = $state("");
  let urlPreview = $state<ResolvedTemplate | null>(null);

  let projectVariables = $state<VariableView[]>([]);
  let environmentVariables = $state<VariableView[]>([]);
  let envVarSearchQuery = $state("");
  let filteredProjectVariables = $derived.by(() => {
    const q = envVarSearchQuery.trim().toLocaleLowerCase();
    return q ? projectVariables.filter((v) => v.key.toLocaleLowerCase().includes(q)) : projectVariables;
  });
  let filteredEnvironmentVariables = $derived.by(() => {
    const q = envVarSearchQuery.trim().toLocaleLowerCase();
    return q ? environmentVariables.filter((v) => v.key.toLocaleLowerCase().includes(q)) : environmentVariables;
  });
  // Trailing draft rows for the Global/Environment variable tables — same "always one empty
  // row, commits when you're done with it" pattern as Params/Headers, just committing to a
  // real backend variable (createVariable) instead of a client-side array, since each row here
  // is its own persisted entity rather than a field on the currently-open request.
  function emptyVarDraft() {
    return { key: "", value: "", isSecret: false, isLocal: false };
  }
  let newGlobalVarDraft = $state(emptyVarDraft());
  let newEnvVarDraft = $state(emptyVarDraft());
  let revealedSecrets = $state<Record<string, string>>({});
  let requestDiagnostics = $state<RequestDiagnostics | null>(null);
  let copyFeedback = $state("");

  // Draft values typed into the hover-to-add popover on an "Unresolved variable" chip.
  let missingVarDrafts = $state<Record<string, string>>({});

  let renamingProjectId = $state<string | null>(null);
  let renameProjectValue = $state("");

  let renamingRequestId = $state<string | null>(null);
  let renameRequestValue = $state("");
  function startRenameRequest(id: string, currentName: string) {
    renamingRequestId = id;
    renameRequestValue = currentName;
  }
  async function submitRenameRequest() {
    const id = renamingRequestId;
    const value = renameRequestValue.trim();
    renamingRequestId = null;
    if (!id || !value) return;
    try {
      const updated = await api.updateRequest({ id, name: value });
      requests = requests.map((r) => (r.id === updated.id ? { ...r, name: updated.name } : r));
      openTabs = openTabs.map((t) => (t.id === updated.id ? { ...t, name: updated.name } : t));
      if (selectedRequest?.id === updated.id) {
        selectedRequest = { ...selectedRequest, name: updated.name };
        editName = updated.name;
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Editor tab selection (LP-0406)
  let activeEditorTab = $state<"params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "mock" | "code">("params");
  let activeScriptTab = $state<"pre" | "post">("pre");

  // Mirrors the active request while it's being edited; reset whenever a
  // different request is opened (README §5 "editor state" step).
  let editName = $state("");
  let editMethod = $state("GET");
  let editUrl = $state("");
  let editHeaders = $state<HeaderEntry[]>([]);
  let editQueryParams = $state<QueryParam[]>([]);
  let editBody = $state("");
  let editDescription = $state("");
  let editAuthType = $state<Auth["type"]>("none");
  let editAuthBearerToken = $state("");
  let editAuthBasicUsername = $state("");
  let editAuthBasicPassword = $state("");
  let editAuthApiKeyKey = $state("");
  let editAuthApiKeyValue = $state("");
  let editAuthApiKeyLocation = $state<"header" | "query">("header");

  // Scripts (LP-0115, LP-0116)
  let editPreScript = $state("");
  let editPostScript = $state("");

  // Settings (LP-0114)
  let editTimeoutMs = $state<number | null>(null);
  let editFollowRedirects = $state(true);
  let editMaxRedirects = $state(10);
  let editVerifySsl = $state(true);
  let editProxyUrl = $state("");
  let editHttpVersion = $state("");

  // Body format helpers (LP-0110, LP-0111, LP-0112)
  let editBodyType = $state<"raw" | "form-data" | "x-www-form-urlencoded" | "binary" | "graphql">("raw");
  let editGraphqlQuery = $state("");
  let editGraphqlVariables = $state("");
  let editFormDataItems = $state<FormDataPart[]>([]);
  let editUrlEncodedItems = $state<UrlEncodedItem[]>([]);
  let editBinaryFilePath = $state("");

  // Raw-body content type (LP-0113) — not stored separately: it's read from/written to the
  // Content-Type header itself, same as Postman's own raw-type picker under the hood, so
  // switching it doesn't need a new backend column.
  const RAW_CONTENT_TYPES: { id: string; mime: string; label: string }[] = [
    { id: "text", mime: "text/plain", label: "body.rawTypeText" },
    { id: "javascript", mime: "application/javascript", label: "body.rawTypeJavascript" },
    { id: "json", mime: "application/json", label: "body.rawTypeJson" },
    { id: "html", mime: "text/html", label: "body.rawTypeHtml" },
    { id: "xml", mime: "application/xml", label: "body.rawTypeXml" },
  ];
  let rawContentType = $derived.by(() => {
    const header = editHeaders.find((h) => h.key.trim().toLowerCase() === "content-type" && h.enabled);
    const value = header?.value.trim().toLowerCase() ?? "";
    return RAW_CONTENT_TYPES.find((t) => value.startsWith(t.mime))?.id ?? "text";
  });
  function setRawContentType(id: string) {
    const type = RAW_CONTENT_TYPES.find((t) => t.id === id);
    if (!type) return;
    const idx = editHeaders.findIndex((h) => h.key.trim().toLowerCase() === "content-type");
    if (idx >= 0) {
      editHeaders[idx] = { ...editHeaders[idx], value: type.mime, enabled: true };
      editHeaders = [...editHeaders];
    } else {
      editHeaders = withTrailingEmptyRow(
        [
          ...editHeaders.filter((h) => h.key.trim() !== ""),
          { key: "Content-Type", value: type.mime, enabled: true, description: "" },
        ],
        () => ({ key: "", value: "", enabled: true, description: "" }),
      );
    }
    scheduleAutoSave();
  }

  let bodyPrettifyFeedback = $state("");

  /** Simple tag-indenter shared by XML and HTML — inserts a newline between adjacent tags, then
   * indents each line by nesting depth (closing tags dedent before printing, opening tags indent
   * after). Not a full formatter (doesn't special-case comments/CDATA/`<pre>` content), but that
   * matches what "Prettify" buttons in most lightweight tools actually do — good enough for
   * skimming an API body without reformatting its content in a surprising way. */
  const HTML_VOID_TAGS = new Set([
    "br", "hr", "img", "input", "meta", "link", "area", "base", "col", "embed", "source", "track", "wbr",
  ]);

  function indentTags(input: string): string {
    // Break only at tag-to-tag boundaries (a ">" immediately followed by a "<") — mixed content
    // like `<p>Hello <b>world</b>` stays on one line, same as text nodes should.
    const withBreaks = input.replace(/>\s*</g, ">\n<").trim();
    let depth = 0;
    const tagRe = /<(\/?)([a-zA-Z][\w:-]*)\b[^>]*?(\/?)>/g;
    return withBreaks
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => {
        const trimmed = line.trim();
        const startsWithClosing = /^<\//.test(trimmed);
        // A "line" here can carry more than one tag (an opening tag plus a fully inline pair,
        // e.g. the `<b>world</b>` above) — net depth change for the line is opens minus closes
        // minus void/self-closing tags, not just a guess from how the line starts or ends.
        let netDepthChange = 0;
        let match: RegExpExecArray | null;
        tagRe.lastIndex = 0;
        while ((match = tagRe.exec(trimmed))) {
          const isClosing = match[1] === "/";
          const isSelfClosing = match[3] === "/" || HTML_VOID_TAGS.has(match[2].toLowerCase());
          if (isClosing) netDepthChange -= 1;
          else if (!isSelfClosing) netDepthChange += 1;
        }
        const printDepth = startsWithClosing ? Math.max(depth - 1, 0) : depth;
        const indented = "  ".repeat(printDepth) + trimmed;
        depth = Math.max(depth + netDepthChange, 0);
        return indented;
      })
      .join("\n");
  }

  function isWellFormedXml(input: string): boolean {
    try {
      const doc = new DOMParser().parseFromString(input, "application/xml");
      return doc.getElementsByTagName("parsererror").length === 0;
    } catch {
      return false;
    }
  }

  function prettifyBody() {
    if (editBodyType === "graphql") {
      try {
        if (editGraphqlVariables && editGraphqlVariables.trim() !== "") {
          editGraphqlVariables = JSON.stringify(JSON.parse(editGraphqlVariables), null, 2);
        }
        bodyPrettifyFeedback = "";
        scheduleAutoSave();
      } catch {
        bodyPrettifyFeedback = "Invalid JSON in Variables";
        setTimeout(() => (bodyPrettifyFeedback = ""), 2500);
      }
      return;
    }
    if (rawContentType === "json") {
      try {
        editBody = JSON.stringify(JSON.parse(editBody), null, 2);
        bodyPrettifyFeedback = "";
        scheduleAutoSave();
      } catch {
        bodyPrettifyFeedback = t("body.prettifyInvalidJson");
        setTimeout(() => (bodyPrettifyFeedback = ""), 2500);
      }
      return;
    }
    if (rawContentType === "xml") {
      if (!isWellFormedXml(editBody)) {
        bodyPrettifyFeedback = t("body.prettifyInvalidXml");
        setTimeout(() => (bodyPrettifyFeedback = ""), 2500);
        return;
      }
      editBody = indentTags(editBody);
      bodyPrettifyFeedback = "";
      scheduleAutoSave();
      return;
    }
    if (rawContentType === "html") {
      editBody = indentTags(editBody);
      bodyPrettifyFeedback = "";
      scheduleAutoSave();
      return;
    }
    // No Content-Type header set yet (or it doesn't match a known type) — rather than make the
    // user go set one before they can even format their body, sniff the body text itself: valid
    // JSON is by far the common case, well-formed XML/HTML the next. Detecting it also sets the
    // matching Content-Type header, so the raw-type dropdown catches up instead of staying "Text".
    try {
      editBody = JSON.stringify(JSON.parse(editBody), null, 2);
      setRawContentType("json");
      bodyPrettifyFeedback = "";
      return;
    } catch {}
    if (isWellFormedXml(editBody)) {
      editBody = indentTags(editBody);
      setRawContentType(editBody.trimStart().startsWith("<!") || editBody.trimStart().startsWith("<html") ? "html" : "xml");
      bodyPrettifyFeedback = "";
      return;
    }
    bodyPrettifyFeedback = t("body.prettifyUnrecognized");
    setTimeout(() => (bodyPrettifyFeedback = ""), 2500);
  }

  /** Reads the stored `body` string into the editor's per-type state. Mirrors the shapes Rust's
   * `RequestBody` (tagged `type`) and the frontend's own GraphQL editor (untagged
   * `{query, variables}`) can produce — anything else is treated as plain "raw" text, which
   * covers JSON API bodies, AI-generated bodies, and cURL/Postman-imported raw bodies alike. */
  function parseBodyForEditing(body: string | null | undefined): {
    bodyType: typeof editBodyType;
    rawBody: string;
    graphqlQuery: string;
    graphqlVariables: string;
    formDataItems: FormDataPart[];
    urlEncodedItems: UrlEncodedItem[];
    binaryFilePath: string;
  } {
    const empty = { bodyType: "raw" as const, rawBody: body ?? "", graphqlQuery: "", graphqlVariables: "", formDataItems: [], urlEncodedItems: [], binaryFilePath: "" };
    if (!body || !body.trim()) return empty;
    let parsed: unknown;
    try {
      parsed = JSON.parse(body);
    } catch {
      return empty;
    }
    if (typeof parsed !== "object" || parsed === null) return empty;
    const obj = parsed as Record<string, unknown>;

    if (obj.type === "form_data" && Array.isArray(obj.items)) {
      return { ...empty, bodyType: "form-data", formDataItems: obj.items as FormDataPart[] };
    }
    if (obj.type === "url_encoded" && Array.isArray(obj.items)) {
      return { ...empty, bodyType: "x-www-form-urlencoded", urlEncodedItems: obj.items as UrlEncodedItem[] };
    }
    if (obj.type === "binary") {
      return { ...empty, bodyType: "binary", binaryFilePath: (obj.file_path as string | null) ?? "" };
    }
    if (obj.type === "raw" && typeof obj.data === "string") {
      return { ...empty, bodyType: "raw", rawBody: obj.data };
    }
    if (obj.type === "graphql" || (typeof obj.query === "string" && !("type" in obj))) {
      const query = typeof obj.query === "string" ? obj.query : "";
      const variables = obj.variables !== undefined ? JSON.stringify(obj.variables, null, 2) : "";
      return { ...empty, bodyType: "graphql", graphqlQuery: query, graphqlVariables: variables };
    }
    return empty;
  }

  /** Inverse of `parseBodyForEditing` — builds the exact string the backend's canonical
   * request model expects for the current `editBodyType`. Raw/GraphQL are sent as plain text
   * (unchanged, already-working behavior); form-data/urlencoded/binary are wrapped in the
   * tagged shape `canonical_request::resolve_body` parses into structured, correctly-encoded
   * bodies (real multipart with a boundary, real percent-encoding, etc.) instead of raw text. */
  function serializeBodyForStorage(): string {
    switch (editBodyType) {
      case "form-data":
        return JSON.stringify({ type: "form_data", items: withoutEmptyKeyRows(editFormDataItems) });
      case "x-www-form-urlencoded":
        return JSON.stringify({ type: "url_encoded", items: withoutEmptyKeyRows(editUrlEncodedItems) });
      case "binary":
        return JSON.stringify({ type: "binary", file_path: editBinaryFilePath || null });
      case "raw":
      case "graphql":
      default:
        return editBody;
    }
  }

  // Auto-expanding rows (Params/Headers/Form Data/URL Encoded): the table always keeps exactly
  // one empty trailing row, so typing a key into it is the "add row" action — no separate
  // Add-row button needed. Rows with an empty key are stripped again before saving.
  function withTrailingEmptyRow<T extends { key: string }>(items: T[], makeEmpty: () => T): T[] {
    const last = items[items.length - 1];
    if (!last || last.key.trim() !== "") {
      return [...items, makeEmpty()];
    }
    return items;
  }
  function withoutEmptyKeyRows<T extends { key: string }>(items: T[]): T[] {
    return items.filter((item) => item.key.trim() !== "");
  }

  function growFormDataItems() {
    editFormDataItems = withTrailingEmptyRow(editFormDataItems, () => ({ key: "", value: "", enabled: true, is_file: false, file_path: null }));
  }
  function removeFormDataItem(index: number) {
    editFormDataItems = withTrailingEmptyRow(editFormDataItems.filter((_, i) => i !== index), () => ({ key: "", value: "", enabled: true, is_file: false, file_path: null }));
  }
  function growUrlEncodedItems() {
    editUrlEncodedItems = withTrailingEmptyRow(editUrlEncodedItems, () => ({ key: "", value: "", enabled: true }));
  }
  function removeUrlEncodedItem(index: number) {
    editUrlEncodedItems = withTrailingEmptyRow(editUrlEncodedItems.filter((_, i) => i !== index), () => ({ key: "", value: "", enabled: true }));
  }

  // cURL importer (LP-0608, LP-0609)
  let curlImportText = $state("");
  let curlImportName = $state("");
  let curlImportError = $state("");

  // Import dialog — a modal, not a screen: importing is something you do to a project or
  // workspace in passing, not a destination you navigate to and linger on.
  let showImportDialog = $state(false);
  let importActiveTab = $state<"collection" | "environment" | "curl" | "localWorkspace">("collection");

  // Local Postman "local files" workspace import (a directory tree, not a single JSON blob).
  let localWorkspacePathInput = $state("");
  let localWorkspaceImportLoading = $state(false);
  let localWorkspaceImportError = $state("");
  let localWorkspaceImportReport = $state<LocalWorkspaceImportReport | null>(null);

  async function importLocalWorkspaceAction() {
    if (!localWorkspacePathInput.trim() || !activeWorkspaceId) return;
    localWorkspaceImportLoading = true;
    localWorkspaceImportError = "";
    localWorkspaceImportReport = null;
    try {
      const report = await api.importLocalPostmanWorkspace(localWorkspacePathInput.trim(), activeWorkspaceId);
      localWorkspaceImportReport = report;
      // Same reasoning as the single-collection import path (importPostmanCollectionAction):
      // create_request/create_folder auto-assign sort_order sequentially, so it already matches
      // this importer's own creation order (alphabetical-by-directory-name — the closest thing
      // to "source order" this on-disk YAML format carries, per project_ids' doc comment on the
      // backend). Without defaulting each newly-created project to Custom order, the sidebar's
      // default "Name" sort would immediately re-alphabetize every one of them, and with dozens
      // or hundreds of projects created in one import, switching each by hand isn't realistic.
      for (const projectId of report.project_ids) {
        try {
          localStorage.setItem(
            `lp-request-sort-${projectId}`,
            JSON.stringify({ field: "custom", dir: "asc" }),
          );
        } catch {}
      }
      await loadProjects();
      await loadAllEnvironments();
      await syncNewProjectIntoWorkspaceRepo();
    } catch (err) {
      localWorkspaceImportError = describeError(err);
    } finally {
      localWorkspaceImportLoading = false;
    }
  }

  // Postman compatibility import/export (LP-0501 - LP-0507, LP-0212)
  let collectionImportText = $state("");
  let collectionImportTarget = $state<"new" | "current">("new");
  let collectionImportLoading = $state(false);
  let collectionImportError = $state("");
  let collectionImportReport = $state<CollectionImportReport | null>(null);

  let environmentImportText = $state("");
  let environmentImportLoading = $state(false);
  let environmentImportError = $state("");
  let environmentImportReport = $state<EnvironmentImportReport | null>(null);

  let exportFeedback = $state("");

  async function exportGlobalVariables() {
    try {
      const data = {
        name: "Globals",
        values: projectVariables.map((v) => ({ key: v.key, value: v.value, enabled: v.enabled })),
        _postman_variable_scope: "globals",
      };
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `globals-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(a.href);
      exportFeedback = "Globals exported!";
      setTimeout(() => (exportFeedback = ""), 2000);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  let snippetMode = $state<SnippetMode>("placeholder");
  // Windows CMD curl by default — this app's dev/target environment is Windows; anything else
  // is one click away in the same dropdown.
  let snippetTarget = $state<SnippetTarget>("bash");
  let snippet = $state("");
  let snippetError = $state("");
  let snippetLoading = $state(false);

  let errorMessage = $state("");
  let loadingRequests = $state(false);

  let sending = $state(false);
  let sendMenuOpen = $state(false);
  // A user-initiated cancel is not a failure — it gets a neutral, self-clearing notice next to
  // Send/Cancel instead of the red error banner every other failed send produces.
  let sendCancelledNotice = $state("");
  let sendCancelledNoticeTimer: ReturnType<typeof setTimeout> | null = null;
  let activeResponse = $state<ResponseMeta | null>(null);
  let activeResponseBody = $state("");
  let activeResponseTruncated = $state(false);
  let responseHistory = $state<ResponseSummary[]>([]);

  // Response Pretty / Raw viewer (LP-0403)
  let responseViewMode = $state<"pretty" | "raw" | "preview">("pretty");
  // Preview is only offered for responses that actually look like a renderable HTML document —
  // Content-Type wins when present, a doctype/html-tag sniff covers servers that mislabel it.
  let responseBodyIsHtml = $derived.by(() => {
    if (!activeResponseBody) return false;
    const contentType = activeResponse?.headers?.find((h) => h.key.toLowerCase() === "content-type")?.value.toLowerCase() ?? "";
    if (contentType.includes("text/html")) return true;
    if (contentType) return false;
    return /^\s*<(!doctype html|html)/i.test(activeResponseBody);
  });
  let prettyResponseBody = $derived.by(() => {
    if (!activeResponseBody) return "";
    if (responseViewMode === "raw") return activeResponseBody;
    try {
      const parsed = JSON.parse(activeResponseBody);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return activeResponseBody;
    }
  });

  function escapeHtml(s: string): string {
    return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
  }

  // Matches exactly the tokens JSON.stringify's own output can contain. Safe to run only on
  // that output (never on arbitrary/raw response text): well-formed JSON syntax cannot place a
  // literal <, >, &, or quote character anywhere except inside a string literal, so escaping the
  // captured string tokens below covers every unsafe character the input could contain.
  const JSON_TOKEN_RE = /"(?:\\.|[^"\\])*"(\s*:)?|\btrue\b|\bfalse\b|\bnull\b|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g;

  function highlightJson(source: string): string {
    return source.replace(JSON_TOKEN_RE, (match, colon: string | undefined) => {
      if (match[0] === '"') {
        const str = colon ? match.slice(0, match.length - colon.length) : match;
        const cls = colon ? "json-key" : "json-string";
        return `<span class="${cls}">${escapeHtml(str)}</span>${colon ?? ""}`;
      }
      if (match === "true" || match === "false") return `<span class="json-boolean">${match}</span>`;
      if (match === "null") return `<span class="json-null">${match}</span>`;
      return `<span class="json-number">${match}</span>`;
    });
  }

  let responseBodyIsJson = $derived.by(() => {
    if (!activeResponseBody || responseViewMode === "raw") return false;
    try {
      JSON.parse(activeResponseBody);
      return true;
    } catch {
      return false;
    }
  });

  let highlightedResponseBody = $derived.by(() => {
    if (!responseBodyIsJson) return "";
    return highlightJson(prettyResponseBody);
  });

  // Response area sub-tabs (Body / Headers / Cookies / Tests) — replaces the old flat stacked layout.
  let responseSubTab = $state<"body" | "headers" | "cookies" | "tests" | "history">("body");

  function formatByteSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Real test-assertion results for the active request's most recent send, sourced from the
  // console event log (script_engine already emits pass/fail per pm.test() call there) — not
  // a separate/fake data source.
  let activeResponseTests = $derived.by(() => {
    if (!selectedRequest) return [];
    const evts = consoleEvents.filter(
      (e) => e.event_type === "test_assertion" && e.request_id === selectedRequest!.id,
    );
    if (!evts.length) return [];
    const latestCorrelationId = evts[evts.length - 1].correlation_id;
    return evts
      .filter((e) => e.correlation_id === latestCorrelationId)
      .map((e) => ({
        name: (e.details?.name as string | undefined) ?? e.message,
        passed: Boolean(e.details?.passed),
        error: (e.details?.error as string | null | undefined) ?? null,
      }));
  });

  // Resizable sidebar (drag handle between the project tree and the main workspace). Its default
  // width scales with the window instead of staying pinned at one small fixed value — otherwise
  // a maximized/fullscreen or ultrawide window leaves project and request names truncated behind
  // the same narrow default a small laptop window would use. Once the user drags the handle
  // themselves, their choice sticks and window resizes stop overriding it.
  function adaptiveSidebarWidth(): number {
    return Math.min(420, Math.max(260, Math.round(window.innerWidth * 0.18)));
  }
  let sidebarWidth = $state(300);
  let sidebarResizing = $state(false);
  let sidebarManuallyResized = false;

  function startSidebarResize(e: MouseEvent) {
    e.preventDefault();
    sidebarResizing = true;
    sidebarManuallyResized = true;
    const onMove = (ev: MouseEvent) => {
      sidebarWidth = Math.min(600, Math.max(200, ev.clientX));
    };
    const onUp = () => {
      sidebarResizing = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // Right sidebar (code snippet / request info) — was permanently fixed at 340px with no way to
  // narrow it, which crowds out the request editor's own tabs/body-mode row badly on anything
  // short of a very wide window (both end up needing their own horizontal scrollbar). Resizable
  // the same way the left sidebar and response pane already are, just anchored to the right edge
  // so dragging measures from window width instead of from 0.
  let rightSidebarWidth = $state(340);
  let rightSidebarResizing = $state(false);

  function startRightSidebarResize(e: MouseEvent) {
    e.preventDefault();
    rightSidebarResizing = true;
    const onMove = (ev: MouseEvent) => {
      rightSidebarWidth = Math.min(640, Math.max(260, window.innerWidth - ev.clientX));
    };
    const onUp = () => {
      rightSidebarResizing = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // Resizable response pane (drag handle between the request editor and the docked response
  // panel — same pattern as the sidebar handle above). Defaults to roughly 42% of the window's
  // height so the response gets real room on first launch instead of a small fixed guess, same
  // reasoning as adaptiveSidebarWidth.
  function adaptiveResponsePaneHeight(): number {
    return Math.min(560, Math.max(220, Math.round(window.innerHeight * 0.42)));
  }
  let responsePaneHeight = $state(320);
  let responsePaneResizing = $state(false);
  let responsePaneManuallyResized = false;
  // Minimized down to a thin status bar (not the same as a small resized height — collapse
  // remembers the full height underneath and restores it exactly on expand).
  let responsePaneCollapsed = $state(false);
  let responsePaneMaximized = $state(false);
  function setResponsePaneCollapsed(value: boolean) {
    responsePaneCollapsed = value;
    if (value) responsePaneMaximized = false;
    try {
      localStorage.setItem("lp-response-pane-collapsed", String(value));
    } catch {
      // collapsed state just won't persist across restarts
    }
  }

  async function handleWindowMinimize() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().minimize();
    } catch {
      // Browser fallback
    }
  }

  async function handleWindowMaximize() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().toggleMaximize();
    } catch {
      if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen().catch(() => {});
      } else {
        document.exitFullscreen().catch(() => {});
      }
    }
  }

  async function handleWindowClose() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().close();
    } catch {
      window.close();
    }
  }

  function startResponsePaneResize(e: MouseEvent) {
    e.preventDefault();
    responsePaneResizing = true;
    responsePaneManuallyResized = true;
    const startY = e.clientY;
    const startHeight = responsePaneHeight;
    document.body.style.userSelect = "none";
    document.body.style.cursor = "ns-resize";
    const onMove = (ev: MouseEvent) => {
      const delta = startY - ev.clientY;
      responsePaneHeight = Math.min(window.innerHeight - 160, Math.max(100, startHeight + delta));
    };
    const onUp = () => {
      responsePaneResizing = false;
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // Resizable console drawer (drag handle along its top edge) — same pattern as the response
  // pane above, since it's the same job (a docked panel you drag taller to read more of).
  let consoleHeight = $state(260);
  let consoleResizing = $state(false);

  function startConsoleResize(e: MouseEvent) {
    e.preventDefault();
    consoleResizing = true;
    const startY = e.clientY;
    const startHeight = consoleHeight;
    const onMove = (ev: MouseEvent) => {
      consoleHeight = Math.min(window.innerHeight - 160, Math.max(120, startHeight - (ev.clientY - startY)));
    };
    const onUp = () => {
      consoleResizing = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // Screen navigation shell — a real left-rail switcher between full-page screens. Each
  // screen reuses the exact same state/functions the old modal-based UI used; nothing here
  // introduces a second source of truth for projects/requests/environments/git/etc.
  
  let activeScreen = $state<ScreenId>("workspace");

  // Back / Forward move through the screens visited in-app. (The webview's own history would
  // step out of the SPA entirely — landing on a blank page — since screens never push entries.)
  let navStack = $state<ScreenId[]>(["workspace"]);
  let navIndex = $state(0);
  let navRestoring = false;
  $effect(() => {
    const screen = activeScreen;
    untrack(() => {
      if (navRestoring) {
        navRestoring = false;
        return;
      }
      if (navStack[navIndex] === screen) return;
      navStack = [...navStack.slice(Math.max(0, navIndex - 48), navIndex + 1), screen];
      navIndex = navStack.length - 1;
    });
  });
  // Globals / Environments / History are project-scoped. Landing on one with no project selected
  // used to leave a near-empty "pick a project" page (and Globals silently ignored "Add variable"),
  // so fall back to the first project instead.
  $effect(() => {
    const screen = activeScreen;
    if ((screen === "globals" || screen === "environments" || screen === "history") && !selectedProjectId && projects.length > 0) {
      const fallback = projects[0].id;
      untrack(() => { void selectProject(fallback); });
    }
  });
  let canGoBack = $derived(navIndex > 0);
  let canGoForward = $derived(navIndex < navStack.length - 1);
  function navigateHistory(direction: -1 | 1) {
    const next = navIndex + direction;
    if (next < 0 || next >= navStack.length) return;
    navRestoring = true;
    navIndex = next;
    activeScreen = navStack[next];
  }
  // `label` stores an i18n key (see SHORTCUT_DEFS above for why) — resolve via t(s.label).
  const SCREENS: { id: ScreenId; label: string }[] = [
    { id: "workspace", label: "rail.workspace" },
    { id: "environments", label: "rail.environments" },
    { id: "git", label: "rail.git" },
    { id: "launcher", label: "rail.launcher" },
    { id: "history", label: "rail.history" },
    { id: "settings", label: "rail.settings" },
  ];

  // Sidebar show/hide, for the Workspace screen's project/request explorer. Persisted —
  // collapsed/expanded is a deliberate choice that should stick until the user changes it
  // again, not reset back to a default every launch.
  let sidebarVisible = $state(true);
  // Icon-only by default (VS Code/IntelliJ activity-bar convention) — the labeled 220px rail
  // was previously the default, costing real width for a 7-item nav a user checks in at a
  // glance. Each rail button already carries a `title` tooltip, so nothing is lost by
  // collapsing; a user who expands it gets that choice remembered via lp-rail-visible below.
  let screensRailVisible = $state(false);
  let rightSidebarVisible = $state(true);
  let utilityRailVisible = $state(true);

  function setSidebarVisible(v: boolean) {
    sidebarVisible = v;
    try { localStorage.setItem("lp-sidebar-visible", String(v)); } catch {}
  }
  function setScreensRailVisible(v: boolean) {
    screensRailVisible = v;
    try { localStorage.setItem("lp-rail-visible", String(v)); } catch {}
  }
  function setRightSidebarVisible(v: boolean) {
    rightSidebarVisible = v;
    try { localStorage.setItem("lp-right-sidebar-visible", String(v)); } catch {}
  }
  function setUtilityRailVisible(v: boolean) {
    utilityRailVisible = v;
    try { localStorage.setItem("lp-utility-rail-visible", String(v)); } catch {}
  }
  // The full-detail response view (stat sidebar, tests, bigger body) used to be its own rail
  // screen, but it's meaningless without Workspace's request context — it's now an expand
  // mode reached from the inline response panel instead of a peer top-level destination.
  let responseExpanded = $state(false);

  // Real, persisted theme toggle — light is this design system's own default; dark, terminal and
  // blueprint are genuine alternate palettes (see the matching [data-theme="..."] blocks), not
  // static mockups. `label` is an i18n key, same convention as SCREENS above.
  
  const THEME_OPTIONS: { id: ThemeMode; label: string }[] = [
    { id: "light", label: "settings.light" },
    { id: "dark", label: "settings.dark" },
    { id: "terminal", label: "theme.terminal" },
    { id: "blueprint", label: "theme.blueprint" },
  ];
  let themeMode = $state<ThemeMode>("dark");

  // Real, user-adjustable auto-sync interval (Settings screen) — previously a hardcoded 60000ms
  // literal with no way to change it. Persisted so it survives a restart.
  let autoSyncIntervalMs = $state(60000);
  function setAutoSyncIntervalMs(ms: number) {
    autoSyncIntervalMs = ms;
    try {
      localStorage.setItem("lp-auto-sync-interval-ms", String(ms));
    } catch {
      // interval just won't persist across restarts
    }
  }
  function setThemeMode(mode: ThemeMode) {
    themeMode = mode;
    try {
      localStorage.setItem("lp-theme", mode);
    } catch {
      // localStorage can throw in a locked-down webview profile — theme just won't persist.
    }
  }

  // Accent color override, independent of the theme — `null` means "use the active theme's own
  // built-in accent" (terracotta / lifted terracotta / teal / blue, per [data-theme]). Presets
  // are drawn from the app's existing --method-* hues so every option is already a color proven
  // legible elsewhere in the UI, not invented fresh.
  const ACCENT_PRESETS: { id: string; hex: string }[] = [
    { id: "terracotta", hex: "#c1603f" },
    { id: "teal", hex: "#0f7d8a" },
    { id: "blue", hex: "#1c5fa8" },
    { id: "violet", hex: "#6b3fa0" },
    { id: "rose", hex: "#d1174a" },
    { id: "amber", hex: "#9a6b00" },
    { id: "green", hex: "#1a7f37" },
    { id: "slate", hex: "#57606a" },
  ];
  let accentColor = $state<string | null>(null);
  function setAccentColor(hex: string | null) {
    accentColor = hex;
    try {
      if (hex) localStorage.setItem("lp-accent-color", hex);
      else localStorage.removeItem("lp-accent-color");
    } catch {
      // localStorage can throw in a locked-down webview profile — choice just won't persist.
    }
  }
  /** WCAG relative-luminance pick between the app's own light-cream and dark-ink text colors —
   * covers any custom accent a user picks, not just the theme's own pre-tuned contrast pairs. */
  function contrastTextFor(hex: string): string {
    const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
    if (!m) return "#fff9f0";
    const n = parseInt(m[1], 16);
    const lin = (v: number) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
    const r = lin(((n >> 16) & 255) / 255);
    const g = lin(((n >> 8) & 255) / 255);
    const b = lin((n & 255) / 255);
    const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    return luminance > 0.4 ? "#201a14" : "#fff9f0";
  }
  /** Inline custom-property overrides for `.app-shell`, layered on top of whatever the active
   * theme already sets — only present once the user picks a color away from the theme default. */
  function accentStyleOverride(hex: string | null): string {
    if (!hex) return "";
    const contrast = contrastTextFor(hex);
    const hover = `color-mix(in srgb, ${hex} 80%, black)`;
    return (
      `--color-accent: ${hex}; --color-accent-hover: ${hover}; --color-accent-contrast: ${contrast}; ` +
      `--color-primary: ${hex}; --color-primary-hover: ${hover}; --color-primary-contrast: ${contrast}; ` +
      `--color-focus: ${hex};`
    );
  }

  // Main surface-color (background/panel) presets — a level beyond just the accent, but
  // deliberately scoped to the light/dark Softline pair only: terminal and blueprint are
  // complete, fixed aesthetic packages the same way method colors are fixed, not something a
  // tint knob should reach into. Each preset is a hand-picked full set (not derived at runtime
  // from one hex) so contrast against the theme's own unchanged text color stays safe — every
  // value sits at roughly the same lightness as the token it replaces, only the hue shifts.
  const SURFACE_TINTS: {
    id: string;
    light: { bg: string; bgSecondary: string; bgTertiary: string; panelBg: string; sidebarBg: string; border: string };
    dark: { bg: string; bgSecondary: string; bgTertiary: string; panelBg: string; sidebarBg: string; border: string };
  }[] = [
    {
      id: "coolGray",
      light: { bg: "#eef1f4", bgSecondary: "#f7f9fb", bgTertiary: "#e4e9ee", panelBg: "#f7f9fb", sidebarBg: "#f7f9fb", border: "#dbe2e8" },
      dark: { bg: "#14171c", bgSecondary: "#1b1f26", bgTertiary: "#232830", panelBg: "#1b1f26", sidebarBg: "#14171c", border: "#2c323b" },
    },
    {
      id: "warmSand",
      light: { bg: "#faf3e8", bgSecondary: "#fffaf1", bgTertiary: "#f3e6d0", panelBg: "#fffaf1", sidebarBg: "#fffaf1", border: "#ecdcc0" },
      dark: { bg: "#211a10", bgSecondary: "#2b2116", bgTertiary: "#35291b", panelBg: "#2b2116", sidebarBg: "#211a10", border: "#453626" },
    },
    {
      id: "rose",
      light: { bg: "#fdf1f0", bgSecondary: "#fff8f7", bgTertiary: "#fbe3e1", panelBg: "#fff8f7", sidebarBg: "#fff8f7", border: "#f2d4d1" },
      dark: { bg: "#201314", bgSecondary: "#2b1a1c", bgTertiary: "#362124", panelBg: "#2b1a1c", sidebarBg: "#201314", border: "#45282b" },
    },
    {
      id: "sage",
      light: { bg: "#f1f6ee", bgSecondary: "#f9fcf7", bgTertiary: "#e5edde", panelBg: "#f9fcf7", sidebarBg: "#f9fcf7", border: "#d7e4cd" },
      dark: { bg: "#161c13", bgSecondary: "#1e261a", bgTertiary: "#26301b", panelBg: "#1e261a", sidebarBg: "#161c13", border: "#34412c" },
    },
  ];
  let surfaceTint = $state<string | null>(null);
  function setSurfaceTint(id: string | null) {
    surfaceTint = id;
    try {
      if (id) localStorage.setItem("lp-surface-tint", id);
      else localStorage.removeItem("lp-surface-tint");
    } catch {
      // localStorage can throw in a locked-down webview profile — choice just won't persist.
    }
  }
  let surfaceTintAvailable = $derived(themeMode === "light" || themeMode === "dark");
  function isCustomSurfaceTint(id: string | null): id is string {
    return !!id && id.startsWith("#");
  }
  /** A custom color has no hand-picked full set, so the rest of the surfaces are derived from
   * it via color-mix — following the same lighten/darken direction the built-in light and dark
   * tokens already use for their own secondary/tertiary steps (dark surfaces get lighter as they
   * layer up; light surfaces get a touch darker/more tinted), rather than one formula for both. */
  function customSurfaceStyleOverride(hex: string, mode: "light" | "dark"): string {
    if (mode === "dark") {
      return (
        `--color-bg: ${hex}; --color-bg-secondary: color-mix(in srgb, ${hex} 85%, white); ` +
        `--color-bg-tertiary: color-mix(in srgb, ${hex} 75%, white); --color-panel-bg: color-mix(in srgb, ${hex} 85%, white); ` +
        `--color-sidebar-bg: ${hex}; --color-border: color-mix(in srgb, ${hex} 60%, white);`
      );
    }
    return (
      `--color-bg: ${hex}; --color-bg-secondary: color-mix(in srgb, ${hex} 90%, white); ` +
      `--color-bg-tertiary: color-mix(in srgb, ${hex} 85%, black); --color-panel-bg: color-mix(in srgb, ${hex} 90%, white); ` +
      `--color-sidebar-bg: color-mix(in srgb, ${hex} 90%, white); --color-border: color-mix(in srgb, ${hex} 70%, black);`
    );
  }
  function surfaceStyleOverride(id: string | null, mode: ThemeMode): string {
    if (!id || (mode !== "light" && mode !== "dark")) return "";
    if (isCustomSurfaceTint(id)) return customSurfaceStyleOverride(id, mode);
    const tint = SURFACE_TINTS.find((t) => t.id === id);
    if (!tint) return "";
    const v = mode === "dark" ? tint.dark : tint.light;
    return (
      `--color-bg: ${v.bg}; --color-bg-secondary: ${v.bgSecondary}; --color-bg-tertiary: ${v.bgTertiary}; ` +
      `--color-panel-bg: ${v.panelBg}; --color-sidebar-bg: ${v.sidebarBg}; --color-border: ${v.border};`
    );
  }

  // Advanced appearance (LP-1405): font family and text color, one layer further than the
  // accent/tint controls above — same override-on-top-of-the-active-theme approach, same
  // localStorage persistence, same "empty = theme's own default, untouched" convention.
  const THEME_FONT_OPTIONS: { id: string; label: string; family: string }[] = [
    { id: "newsreader", label: "Newsreader", family: '"Newsreader", Georgia, serif' },
    { id: "work-sans", label: "Work Sans", family: '"Work Sans", system-ui, sans-serif' },
    { id: "space-grotesk", label: "Space Grotesk", family: '"Space Grotesk", system-ui, sans-serif' },
    { id: "jetbrains-mono", label: "JetBrains Mono", family: '"JetBrains Mono", ui-monospace, monospace' },
    { id: "space-mono", label: "Space Mono", family: '"Space Mono", ui-monospace, monospace' },
  ];
  let headingFontOverride = $state<string | null>(null);
  let bodyFontOverride = $state<string | null>(null);
  function setHeadingFontOverride(id: string | null) {
    headingFontOverride = id;
    try {
      if (id) localStorage.setItem("lp-heading-font", id);
      else localStorage.removeItem("lp-heading-font");
    } catch {
      // localStorage can throw in a locked-down webview profile — choice just won't persist.
    }
  }
  function setBodyFontOverride(id: string | null) {
    bodyFontOverride = id;
    try {
      if (id) localStorage.setItem("lp-body-font", id);
      else localStorage.removeItem("lp-body-font");
    } catch {
      // localStorage can throw in a locked-down webview profile — choice just won't persist.
    }
  }
  function fontStyleOverride(headingId: string | null, bodyId: string | null): string {
    const heading = THEME_FONT_OPTIONS.find((f) => f.id === headingId);
    const body = THEME_FONT_OPTIONS.find((f) => f.id === bodyId);
    let css = "";
    if (heading) css += `--font-heading: ${heading.family}; `;
    if (body) css += `--font-sans: ${body.family}; `;
    return css;
  }

  let textColorOverride = $state<string | null>(null);
  function setTextColorOverride(hex: string | null) {
    textColorOverride = hex;
    try {
      if (hex) localStorage.setItem("lp-text-color", hex);
      else localStorage.removeItem("lp-text-color");
    } catch {
      // localStorage can throw in a locked-down webview profile — choice just won't persist.
    }
  }
  /** Secondary/tertiary text steps are derived by mixing toward transparent (not toward a fixed
   * black/white) — the same trick `--color-bg-hover` already uses elsewhere in this file — so
   * they dim consistently against whatever background (including a custom surface tint) the
   * text actually sits on, rather than assuming a light or dark ground. */
  function textColorStyleOverride(hex: string | null): string {
    if (!hex) return "";
    return (
      `--color-text: ${hex}; ` +
      `--color-text-secondary: color-mix(in srgb, ${hex} 75%, transparent); ` +
      `--color-text-tertiary: color-mix(in srgb, ${hex} 55%, transparent);`
    );
  }
  /** Live WCAG contrast readout for the custom text-color picker — reuses the exact relative
   * luminance formula `contrastTextFor` above already implements, just returning the ratio
   * instead of picking a side, so the UI can warn before the user saves an unreadable color. */
  function contrastRatio(hexA: string, hexB: string): number | null {
    const parse = (h: string) => {
      const m = /^#?([0-9a-f]{6})$/i.exec(h.trim());
      if (!m) return null;
      const n = parseInt(m[1], 16);
      const lin = (v: number) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
      return 0.2126 * lin(((n >> 16) & 255) / 255) + 0.7152 * lin(((n >> 8) & 255) / 255) + 0.0722 * lin((n & 255) / 255);
    };
    const la = parse(hexA);
    const lb = parse(hexB);
    if (la === null || lb === null) return null;
    const [hi, lo] = la > lb ? [la, lb] : [lb, la];
    return (hi + 0.05) / (lo + 0.05);
  }
  /** The background the custom text color would actually sit on right now — reads the live
   * computed value rather than re-deriving it from theme/tint state, since a tint can itself be
   * a hand-picked preset or an arbitrary custom color. */
  function currentComputedBg(): string | null {
    const shell = document.querySelector(".app-shell");
    if (!shell) return null;
    const v = getComputedStyle(shell).getPropertyValue("--color-bg").trim();
    return /^#[0-9a-f]{6}$/i.test(v) ? v : null;
  }
  let textColorContrastWarning = $derived.by(() => {
    if (!textColorOverride) return "";
    const bg = currentComputedBg();
    if (!bg) return "";
    const ratio = contrastRatio(textColorOverride, bg);
    if (ratio === null || ratio >= 4.5) return "";
    return t("settings.textColorLowContrast", { ratio: ratio.toFixed(1) });
  });

  // i18n: locale drives both the string dictionary (t()) and the document's real text
  // direction — Arabic runs right-to-left, and dir="rtl" on <html> is what makes flexbox's
  // "row" axis (used throughout this stylesheet) actually mirror instead of just the text.
  let locale = $state<Locale>("en");
  function t(key: string, params?: Record<string, string | number>): string {
    return translate(locale, key, params);
  }
  function applyLocale(l: Locale) {
    try {
      document.documentElement.lang = l;
      document.documentElement.dir = RTL_LOCALES.includes(l) ? "rtl" : "ltr";
    } catch {
      // SSR-safe no-op; onMount always runs in the browser so this only matters for symmetry.
    }
  }
  function setLocale(l: Locale) {
    locale = l;
    applyLocale(l);
    try {
      localStorage.setItem("lp-locale", l);
    } catch {
      // won't persist across restarts — not fatal.
    }
  }

  // Icon & text size: every size in this stylesheet is in rem, which is relative to the root
  // <html> element's font-size specifically (not the nearest ancestor's) — so scaling that one
  // root value scales every rem-sized icon and text label app-wide with no per-element changes.
  let uiScale = $state(100);
  function setUiScale(pct: number) {
    uiScale = pct;
    try {
      document.documentElement.style.fontSize = `${pct}%`;
      localStorage.setItem("lp-ui-scale", String(pct));
    } catch {
      // won't persist across restarts, and this one load won't be scaled — not fatal.
    }
  }

  // Real system diagnostics (RSS via sysinfo, SQLite/WAL size, entity counts) for the
  // resource-budget widget — never a placeholder number.
  let systemDiagnostics = $state<SystemDiagnostics | null>(null);
  async function refreshSystemDiagnostics() {
    try {
      systemDiagnostics = await api.getSystemDiagnostics();
    } catch {
      // Diagnostics are informational — a failure here shouldn't surface as an app error.
    }
  }

  // Real per-project request counts for the Launcher screen (one query for every project).
  let projectRequestCounts = $state<Record<string, number>>({});
  async function refreshProjectRequestCounts() {
    try {
      projectRequestCounts = await api.getProjectRequestCounts();
    } catch {
      // Informational only.
    }
  }

  // Real project-wide History screen state.
  let projectHistory = $state<ProjectHistoryEntry[]>([]);
  let historyLoading = $state(false);
  let historySearchQuery = $state("");
  let historyShowFailuresOnly = $state(false);
  async function refreshProjectHistory() {
    historyLoading = true;
    try {
      const pid = selectedProjectId || (projects[0]?.id ?? "proj-default");
      const list = await api.listProjectHistory(pid, 200);
      projectHistory = Array.isArray(list) ? list : [];
    } catch (err) {
      errorMessage = describeError(err);
      projectHistory = [];
    } finally {
      historyLoading = false;
    }
  }
  async function clearHistory() {
    projectHistory = [];
    try {
      const pid = selectedProjectId || (projects[0]?.id ?? "proj-default");
      await api.clearProjectHistory(pid);
    } catch (err) {
      // ignore
    }
  }

  // Favorite environments (global / workspace-level for all projects)
  let favoriteEnvIds = $state<string[]>((() => {
    try {
      const saved = localStorage.getItem("lp-global-fav-envs");
      if (saved) return JSON.parse(saved);
      // Fallback/migrate from legacy per-project storage:
      const legacy = localStorage.getItem("lp-project-fav-envs");
      if (legacy) {
        const parsed = JSON.parse(legacy);
        const set = new Set<string>();
        Object.values(parsed).forEach((arr: any) => {
          if (Array.isArray(arr)) arr.forEach((id) => set.add(id));
        });
        return Array.from(set);
      }
      return [];
    } catch {
      return [];
    }
  })());

  function isEnvFavorite(envId: string): boolean {
    return favoriteEnvIds.includes(envId);
  }

  function toggleEnvFavorite(envId: string) {
    const current = [...favoriteEnvIds];
    const idx = current.indexOf(envId);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(envId);
    }
    favoriteEnvIds = current;
    try {
      localStorage.setItem("lp-global-fav-envs", JSON.stringify(favoriteEnvIds));
    } catch {
      // ignore
    }
  }

  let favoriteEnvs = $derived.by(() => {
    return allEnvironments.filter((e) => isEnvFavorite(e.id));
  });

  // Sidebar environment variable expansion ("see key values at the left side")
  let expandedEnvIds = $state<Set<string>>(new Set());
  let envVariablesCache = $state<Map<string, VariableView[]>>(new Map());

  // Environments Sidebar Search (All / Name / Key / Values)
  let envSidebarSearchQuery = $state("");
  let envSidebarSearchScope = $state<"all" | "name" | "key" | "values">("all");
  let envVariablesLoading = $state(false);

  async function ensureAllEnvVariablesLoaded() {
    if (allEnvironments.length === 0) return;
    const missing = allEnvironments.filter((e) => !envVariablesCache.has(e.id));
    if (missing.length === 0) return;
    envVariablesLoading = true;
    try {
      const results = await Promise.all(
        missing.map(async (env) => {
          try {
            const vars = await api.listVariablesForScope("environment", env.id);
            return { id: env.id, vars };
          } catch {
            return { id: env.id, vars: [] };
          }
        })
      );
      for (const res of results) {
        envVariablesCache.set(res.id, res.vars);
      }
      envVariablesCache = new Map(envVariablesCache);
    } finally {
      envVariablesLoading = false;
    }
  }

  let filteredSidebarEnvironments = $derived.by(() => {
    const q = envSidebarSearchQuery.trim().toLowerCase();
    if (!q) return allEnvironments;

    return allEnvironments.filter((env) => {
      const nameMatches = env.name.toLowerCase().includes(q);
      if (envSidebarSearchScope === "name") {
        return nameMatches;
      }

      const vars = envVariablesCache.get(env.id) || [];
      const keyMatches = vars.some((v) => v.key.toLowerCase().includes(q));
      if (envSidebarSearchScope === "key") {
        return keyMatches;
      }

      const valMatches = vars.some((v) => (v.value || "").toLowerCase().includes(q));
      if (envSidebarSearchScope === "values") {
        return valMatches;
      }

      // "all"
      return nameMatches || keyMatches || valMatches;
    });
  });

  let filteredSidebarFavoriteEnvs = $derived.by(() => {
    return filteredSidebarEnvironments.filter((e) => isEnvFavorite(e.id));
  });

  async function toggleEnvExpand(envId: string) {
    const next = new Set(expandedEnvIds);
    if (next.has(envId)) {
      next.delete(envId);
    } else {
      next.add(envId);
      if (!envVariablesCache.has(envId)) {
        try {
          const vars = await api.listVariablesForScope("environment", envId);
          envVariablesCache.set(envId, vars);
          envVariablesCache = new Map(envVariablesCache);
        } catch {
          // ignore
        }
      }
    }
    expandedEnvIds = next;
  }

  let filteredEnvironments = $derived.by(() => {
    const q = envSearchQuery.trim().toLowerCase();
    if (!q) return allEnvironments;
    return allEnvironments.filter((e) => e.name.toLowerCase().includes(q));
  });

  let activeWorkspace = $derived.by(() => {
    return workspaces.find((w) => w.id === activeWorkspaceId) ?? workspaces[0] ?? null;
  });

  function formatRelativeTime(iso: string): string {
    try {
      const diff = Date.now() - new Date(iso).getTime();
      if (diff < 60000) return "Just now";
      if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
      if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }

  // Copies an environment (all its variables, secrets included) into a new one named "<name> Copy".
  // Returns the new environment so callers can select/open it; null if anything failed (the error
  // is already surfaced through errorMessage).
  async function duplicateEnvironment(env: Environment | EnvironmentWithProject, name?: string): Promise<Environment | null> {
    try {
      const copy = await api.createEnvironment(env.project_id, name ?? `${env.name} Copy`);
      const vars = await api.listVariablesForScope("environment", env.id);
      for (const v of vars) {
        await api.createVariable({
          scope: "environment",
          environment_id: copy.id,
          project_id: v.project_id ?? env.project_id,
          key: v.key,
          value: v.value,
          enabled: v.enabled,
          is_secret: v.is_secret,
          is_local: v.is_local,
          description: v.description ?? null,
        });
      }
      await loadAllEnvironments();
      return copy;
    } catch (err) {
      errorMessage = describeError(err);
      return null;
    }
  }

  function openGlobalsTab() {
    loadVariables();
    activeScreen = "globals";
  }

  let filteredProjectHistory = $derived.by(() => {
    const q = historySearchQuery.trim().toLocaleLowerCase();
    return projectHistory.filter((h) => {
      if (historyShowFailuresOnly && h.status < 400) return false;
      if (!q) return true;
      return (
        h.request_name.toLocaleLowerCase().includes(q) ||
        h.method.toLocaleLowerCase().includes(q) ||
        h.url.toLocaleLowerCase().includes(q) ||
        String(h.status).includes(q)
      );
    });
  });

  // Command palette — real fuzzy-ish search over actual projects/requests, real actions only
  // (open the project, open the request). No fabricated "commands".
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteInputEl = $state<HTMLInputElement | null>(null);
  // Scope filter (LP: palette project results were easy to lose in a flood of API matches) —
  // lets the user isolate projects-only or APIs-only instead of always seeing everything mixed.
  
  let paletteScope = $state<PaletteScope>("all");
  // Field filter — which request fields count as a match. Name+URL mirrors the original
  // behavior; Body is opt-in since it only works through the backend (see api.ts) and can be
  // noisier (payload text matches are less obviously "this is the request I meant").
  let paletteFields = $state<Record<SearchField, boolean>>({ name: true, url: true, body: false });
  function togglePaletteField(f: SearchField) {
    paletteFields = { ...paletteFields, [f]: !paletteFields[f] };
  }
  function activeSearchFields(fields: Record<SearchField, boolean>): SearchField[] {
    return (["name", "url", "body"] as SearchField[]).filter((f) => fields[f]);
  }
  function openPalette() {
    paletteOpen = true;
    paletteQuery = "";
    paletteScope = "all";
    paletteFields = { name: true, url: true, body: false };
  }
  function closePalette() {
    paletteOpen = false;
  }

  // Keyboard shortcuts — each one maps to a real, already-existing action (nothing fabricated
  // for the sake of having a shortcuts list). Individually toggleable from Settings, persisted
  // to localStorage the same way theme/auto-sync-interval already are.
  
  // `label` stores an i18n key, not literal text — SHORTCUT_DEFS is a plain const (evaluated
  // once), so resolving the string at definition time would freeze it in whatever locale was
  // active then. Resolve it at render time instead: t(def.label).
  const SHORTCUT_DEFS: { id: ShortcutId; label: string; keys: string }[] = [
    { id: "commandPalette", label: "shortcut.commandPalette", keys: "Ctrl/⌘ K" },
    { id: "sendRequest", label: "shortcut.sendRequest", keys: "Ctrl/⌘ Enter" },
    { id: "saveRequest", label: "shortcut.saveRequest", keys: "Ctrl/⌘ S" },
    { id: "newRequest", label: "shortcut.newRequest", keys: "Ctrl/⌘ N" },
    { id: "nextTab", label: "shortcut.nextTab", keys: "Ctrl/⌘ Tab" },
    { id: "prevTab", label: "shortcut.prevTab", keys: "Ctrl/⌘ Shift Tab" },
    { id: "closeTab", label: "shortcut.closeTab", keys: "Ctrl/⌘ W" },
  ];
  let shortcutsEnabled = $state<Record<ShortcutId, boolean>>({
    commandPalette: true,
    sendRequest: true,
    saveRequest: true,
    newRequest: true,
    nextTab: true,
    prevTab: true,
    closeTab: true,
  });
  function setShortcutEnabled(id: ShortcutId, enabled: boolean) {
    shortcutsEnabled = { ...shortcutsEnabled, [id]: enabled };
    try {
      localStorage.setItem("lp-shortcuts-enabled", JSON.stringify(shortcutsEnabled));
    } catch {
      // shortcut prefs just won't persist across restarts
    }
  }
  $effect(() => {
    if (paletteOpen) paletteInputEl?.focus();
  });
  // Cross-project search (LP-1404): the palette's own project/request loop below only ever
  // covers the CURRENTLY open project, since that's all `requests` holds — everything else in
  // the workspace has to come from the backend. Debounced so switching-workspace-wide search
  // doesn't fire a query per keystroke; a token guards against a slow, stale response overwriting
  // a newer one that already landed.
  let paletteCrossProjectResults = $state<RequestSearchResult[]>([]);
  let paletteCrossProjectToken = 0;
  $effect(() => {
    const q = paletteQuery.trim();
    const fields = activeSearchFields(paletteFields);
    if (!paletteOpen || q.length < 2 || !activeWorkspaceId || fields.length === 0) {
      paletteCrossProjectResults = [];
      return;
    }
    const token = ++paletteCrossProjectToken;
    const workspaceId = activeWorkspaceId;
    const timer = setTimeout(async () => {
      try {
        const results = await api.searchRequestsInWorkspace(workspaceId, q, fields);
        if (token === paletteCrossProjectToken) paletteCrossProjectResults = results;
      } catch {
        // The palette's in-project results still work regardless — this is a pure enhancement.
      }
    }, 150);
    return () => clearTimeout(timer);
  });

  
  let paletteItems = $derived.by((): PaletteItem[] => {
    const q = paletteQuery.trim().toLocaleLowerCase();
    const items: PaletteItem[] = [];
    if (paletteScope !== "apis") {
      for (const p of projects) {
        if (!q || p.name.toLocaleLowerCase().includes(q)) {
          items.push({
            kind: "project",
            label: p.name,
            hint: "project",
            onSelect: () => {
              selectProject(p.id);
              activeScreen = "workspace";
              closePalette();
            },
          });
        }
      }
    }
    const localRequestIds = new Set<string>();
    if (paletteScope !== "projects") {
      const activeProjectName = projects.find((p) => p.id === selectedProjectId)?.name;
      if (selectedProjectId) {
        for (const r of requests) {
          // Body isn't loaded into this local list (see api.ts) — a body-only match still
          // surfaces below, via paletteCrossProjectResults, which searches straight against SQLite.
          if (
            !q ||
            (paletteFields.name && r.name.toLocaleLowerCase().includes(q)) ||
            (paletteFields.url && r.url.toLocaleLowerCase().includes(q))
          ) {
            localRequestIds.add(r.id);
            items.push({
              kind: "request",
              label: r.name,
              method: r.method,
              hint: r.url,
              project: activeProjectName,
              onSelect: () => {
                openRequest(r.id);
                activeScreen = "workspace";
                closePalette();
              },
            });
          }
        }
      }
      for (const r of paletteCrossProjectResults) {
        if (localRequestIds.has(r.id)) continue; // already listed above, from the active project
        items.push({
          kind: "request",
          label: r.name,
          method: r.method,
          hint: r.url,
          project: r.project_name,
          onSelect: async () => {
            await selectProject(r.project_id);
            openRequest(r.id);
            activeScreen = "workspace";
            closePalette();
          },
        });
      }
    }
    return items.slice(0, 30);
  });

  let aiConfigured = $state(false);
  let showAiPanel = $state(false);
  let rightPanel = $state<"code" | "variables" | "info" | "ai" | "comments" | null>(null);
  let aiActiveTab = $state<"generate" | "source" | "settings">("generate");
  let aiPrompt = $state("");
  let aiGenerating = $state(false);
  let aiPreview = $state<GeneratedApiDefinition | null>(null);

  // Project context toggles (LP-0807, LP-0808, LP-0809)
  let aiIncludeExistingRequests = $state(true);
  let aiIncludeVariables = $state(true);

  // AI Settings (LP-0803, LP-0822 — multi-provider)
  let aiSettings = $state<AiSettings | null>(null);
  let aiProviderInput = $state<AiProviderKind>("anthropic");
  let aiApiKeyInput = $state("");
  let aiModelInput = $state("claude-sonnet-5");
  let aiBaseUrlInput = $state("");

  const AI_PROVIDERS: { id: AiProviderKind; labelKey: string }[] = [
    { id: "anthropic", labelKey: "ai.providerAnthropic" },
    { id: "openai", labelKey: "ai.providerOpenAi" },
    { id: "google", labelKey: "ai.providerGoogle" },
    { id: "custom", labelKey: "ai.providerCustom" },
  ];

  // Suggestions only — the model field is free text, never a locked list, so a new model
  // release never requires an app update to become selectable.
  const AI_MODEL_SUGGESTIONS: Record<AiProviderKind, string[]> = {
    anthropic: ["claude-opus-5", "claude-sonnet-5", "claude-haiku-4-5-20251001"],
    openai: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
    google: ["gemini-1.5-pro", "gemini-1.5-flash"],
    custom: [],
  };

  const AI_API_KEY_PLACEHOLDER: Record<AiProviderKind, string> = {
    anthropic: "sk-ant-api03-...",
    openai: "sk-...",
    google: "AIza...",
    custom: "",
  };

  function aiProviderDefaultModel(provider: AiProviderKind): string {
    return AI_MODEL_SUGGESTIONS[provider][0] ?? "";
  }

  function onAiProviderChange() {
    aiModelInput = aiProviderDefaultModel(aiProviderInput);
    aiBaseUrlInput = "";
  }
  let aiShowKey = $state(false);
  let aiTesting = $state(false);
  let aiTestFeedback = $state("");
  let aiTestError = $state("");
  let aiSettingsFeedback = $state("");

  // Source Project Analyzer state (LP-0810 - LP-0821)
  let sourceDirectoryInput = $state("");
  let sourceScanning = $state(false);
  let sourceReport = $state<SourceProjectReport | null>(null);
  let sourceFilter = $state("");
  let filteredSourceEndpoints = $derived.by(() => {
    if (!sourceReport) return [];
    const q = sourceFilter.trim().toLocaleLowerCase();
    if (!q) return sourceReport.endpoints;
    return sourceReport.endpoints.filter(
      (e) =>
        e.path.toLocaleLowerCase().includes(q) ||
        e.method.toLocaleLowerCase().includes(q) ||
        e.source_file.toLocaleLowerCase().includes(q),
    );
  });
  let sourceActionFeedback = $state("");

  // Request-level AI generation state (LP-0806, LP-0819)
  let generatingSample = $state(false);
  let sampleFeedback = $state("");
  // Keyed by request id so the project tree can show/lazy-load any request's saved samples,
  // not just the one currently open in the editor (which reads its own entry via `sampleResponses` below).
  let sampleResponsesByRequestId = $state<Map<string, SampleResponse[]>>(new Map());
  let sampleResponses = $derived(selectedRequest ? (sampleResponsesByRequestId.get(selectedRequest.id) ?? []) : []);
  let generatingTestsDocs = $state(false);
  let testsDocsFeedback = $state("");

  // Tree nesting: expand a request row to lazy-load and show its saved sample responses.
  let expandedTreeRequestIds = $state<Set<string>>(new Set());
  let renamingSampleResponseId = $state<string | null>(null);
  let renameSampleResponseValue = $state("");

  // Tab management (LP-0407, LP-0408, LP-0411)
  

  

  let openTabs = $state<RequestTab[]>([]);
  let activeTabId = $state<string | null>(null);
  let drawerVarSearch = $state("");
  let globalsSearch = $state("");
  let drawerFilteredVariables = $derived.by(() => {
    const q = drawerVarSearch.trim().toLowerCase();
    const envVars = (environmentVariables || []).map((v) => ({ ...v, scopeTag: "E" as const }));
    const globVars = (projectVariables || []).map((v) => ({ ...v, scopeTag: "G" as const }));
    const combined = [...envVars, ...globVars];
    if (!q) return combined;
    return combined.filter((v) => v.key.toLowerCase().includes(q) || v.value.toLowerCase().includes(q));
  });
  const tabDrafts = new Map<string, RequestDraft>();

  $effect(() => {
    if (selectedProjectId) {
      try {
        localStorage.setItem(`lp-open-tabs-${selectedProjectId}`, JSON.stringify(openTabs));
        if (selectedRequest) {
          localStorage.setItem(`lp-selected-request-${selectedProjectId}`, selectedRequest.id);
        } else {
          localStorage.removeItem(`lp-selected-request-${selectedProjectId}`);
        }
      } catch {}
    }
  });

  // Project search — filters the sidebar's project list by name. Scope mirrors the command
  // palette's All/Projects/APIs toggle: "projects" is the original tree-filter behavior,
  // "apis" swaps the tree for a flat cross-workspace request list (same shape as the palette's
  // cross-project results), "all" shows both.
  
  let projectSearchQuery = $state("");
  let projectSearchScope = $state<SidebarSearchScope>("all");
  // Same Name/URL/Body field filter as the palette (see paletteFields/activeSearchFields above).
  let sidebarSearchFields = $state<Record<SearchField, boolean>>({ name: true, url: true, body: false });
  function toggleSidebarSearchField(f: SearchField) {
    sidebarSearchFields = { ...sidebarSearchFields, [f]: !sidebarSearchFields[f] };
  }

  let sidebarApiResults = $state<RequestSearchResult[]>([]);
  let sidebarApiSearchToken = 0;
  $effect(() => {
    const q = projectSearchQuery.trim();
    const fields = activeSearchFields(sidebarSearchFields);
    if (projectSearchScope === "projects" || q.length < 2 || !activeWorkspaceId || fields.length === 0) {
      sidebarApiResults = [];
      return;
    }
    const token = ++sidebarApiSearchToken;
    const workspaceId = activeWorkspaceId;
    const timer = setTimeout(async () => {
      try {
        const results = await api.searchRequestsInWorkspace(workspaceId, q, fields);
        if (token === sidebarApiSearchToken) sidebarApiResults = results;
      } catch {
        // The project tree filter still works regardless — this is a pure enhancement.
      }
    }, 150);
    return () => clearTimeout(timer);
  });

  // Advanced sort: projects list + requests/folders within a project. Persisted so the chosen
  // order survives a reload, same as the other small UI prefs (uiScale, locale, theme).
  
  
  
  let projectSortField = $state<ProjectSortField>("name");
  let projectSortDir = $state<SortDir>("asc");
  let requestSortField = $state<RequestSortField>("custom");
  let requestSortDir = $state<SortDir>("asc");

  function setProjectSortField(field: ProjectSortField) {
    projectSortField = field;
    try { localStorage.setItem("lp-project-sort", JSON.stringify({ field: projectSortField, dir: projectSortDir })); } catch {}
  }
  function toggleProjectSortDir() {
    projectSortDir = projectSortDir === "asc" ? "desc" : "asc";
    try { localStorage.setItem("lp-project-sort", JSON.stringify({ field: projectSortField, dir: projectSortDir })); } catch {}
  }
  function setRequestSortField(field: RequestSortField) {
    requestSortField = field;
    if (!selectedProjectId) return;
    try { localStorage.setItem(`lp-request-sort-${selectedProjectId}`, JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
  }
  function toggleRequestSortDir() {
    requestSortDir = requestSortDir === "asc" ? "desc" : "asc";
    if (!selectedProjectId) return;
    try { localStorage.setItem(`lp-request-sort-${selectedProjectId}`, JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
  }

  // A field already selected toggles direction on click; a different field switches to it (ascending).
  function pickProjectSortField(field: ProjectSortField) {
    if (projectSortField === field) toggleProjectSortDir();
    else setProjectSortField(field);
  }
  function pickRequestSortField(field: RequestSortField) {
    if (requestSortField === field) toggleRequestSortDir();
    else setRequestSortField(field);
  }

  // Drag-and-drop reordering — only meaningful (and only enabled in the markup) once
  // projectSortField === "custom"; dragging under Name/Created/Updated would just get
  // silently undone by the next re-sort, so it's disabled rather than working-then-reverting.
  let draggedProjectId = $state<string | null>(null);
  function onProjectDragStart(id: string) {
    if (projectSortField !== "custom") return;
    draggedProjectId = id;
  }
  async function onProjectDrop(targetId: string) {
    const draggedId = draggedProjectId;
    draggedProjectId = null;
    if (!draggedId || draggedId === targetId || projectSortField !== "custom" || !activeWorkspaceId) return;

    const ids = sortProjectList(projects).map((p) => p.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousProjects = projects;
    projects = projects.map((p) => (orderById.has(p.id) ? { ...p, sort_order: orderById.get(p.id)! } : p));

    try {
      await api.reorderProjects(activeWorkspaceId, ids);
    } catch (err) {
      projects = previousProjects;
      errorMessage = describeError(err);
    }
  }

  // Same pattern as the project list's drag-and-drop (Task 9), but scoped per sibling group:
  // a request only reorders among requests in the same folder (or same project root); a
  // folder only reorders among folders under the same parent (or same project root).
  let draggedRequestId = $state<string | null>(null);
  function onRequestDragStart(id: string) {
    if (requestSortField !== "custom") return;
    draggedRequestId = id;
  }
  async function onRequestDrop(targetId: string, folderId: string | null) {
    const draggedId = draggedRequestId;
    draggedRequestId = null;
    if (!draggedId || draggedId === targetId || requestSortField !== "custom" || !selectedProjectId) return;

    const siblings = folderId ? (requestsByFolderId.get(folderId) ?? []) : rootRequests;
    const ids = siblings.map((r) => r.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousRequests = requests;
    requests = requests.map((r) => (orderById.has(r.id) ? { ...r, sort_order: orderById.get(r.id)! } : r));

    try {
      await api.reorderRequests(selectedProjectId, folderId, ids);
    } catch (err) {
      requests = previousRequests;
      errorMessage = describeError(err);
    }
  }

  let draggedFolderId = $state<string | null>(null);
  function onFolderDragStart(id: string) {
    if (requestSortField !== "custom") return;
    draggedFolderId = id;
  }
  async function onFolderDrop(targetId: string, parentFolderId: string | null) {
    const draggedId = draggedFolderId;
    draggedFolderId = null;
    if (!draggedId || draggedId === targetId || requestSortField !== "custom" || !selectedProjectId) return;

    const key = parentFolderId ?? ROOT_FOLDER_KEY;
    const siblings = foldersByParentId.get(key) ?? [];
    const ids = siblings.map((f) => f.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousFolders = folders;
    folders = folders.map((f) => (orderById.has(f.id) ? { ...f, sort_order: orderById.get(f.id)! } : f));

    try {
      await api.reorderFolders(selectedProjectId, parentFolderId, ids);
    } catch (err) {
      folders = previousFolders;
      errorMessage = describeError(err);
    }
  }

  const PROJECT_SORT_FIELDS: { field: ProjectSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "created", label: "sidebar.sortByCreated" },
    { field: "updated", label: "sidebar.sortByUpdated" },
    { field: "custom", label: "sidebar.sortByCustom" },
  ];
  const REQUEST_SORT_FIELDS: { field: RequestSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "method", label: "sidebar.sortByMethod" },
    { field: "created", label: "sidebar.sortByCreated" },
    { field: "updated", label: "sidebar.sortByUpdated" },
    { field: "custom", label: "sidebar.sortByCustom" },
  ];
  let projectSortMenuOpen = $state(false);
  let requestSortMenuOpen = $state(false);

  // Only one project's "more actions" overflow menu is open at a time.
  let openProjectMenuId = $state<string | null>(null);

  // These small popover menus (env picker, workspace picker, sort menus, project overflow menu)
  // already close on outside-click via their dropdown-backdrop, but had no Escape handling —
  // a window-level listener beats per-element keydown here since it works regardless of what
  // inside the menu currently has focus, and avoids putting keydown handlers (and the a11y-role
  // ceremony that comes with them) on plain wrapper divs.
  $effect(() => {
    if (!envPickerOpen && !workspacePickerOpen && !projectSortMenuOpen && !requestSortMenuOpen && !openProjectMenuId && !sendMenuOpen) return;
    const onKeydown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      envPickerOpen = false;
      workspacePickerOpen = false;
      projectSortMenuOpen = false;
      requestSortMenuOpen = false;
      openProjectMenuId = null;
      sendMenuOpen = false;
    };
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });

  function sortProjectList(list: Project[]): Project[] {
    const dir = projectSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (projectSortField === "custom") return a.sort_order - b.sort_order;
      if (projectSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (projectSortField === "created") return a.created_at.localeCompare(b.created_at) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }
  function sortRequestList(list: RequestSummary[]): RequestSummary[] {
    const dir = requestSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (requestSortField === "custom") return a.sort_order - b.sort_order;
      if (requestSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (requestSortField === "method") return a.method.localeCompare(b.method) * dir;
      if (requestSortField === "created") return a.created_at.localeCompare(b.created_at) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }

  let filteredProjects = $derived.by(() => {
    const q = projectSearchQuery.trim().toLocaleLowerCase();
    const base = q ? projects.filter((p) => p.name.toLocaleLowerCase().includes(q)) : projects;
    return sortProjectList(base);
  });

  // Request search & windowing (LP-0409, LP-0410)
  let requestSearchQuery = $state("");
  let filteredRequests = $derived.by(() => {
    const q = requestSearchQuery.trim().toLocaleLowerCase();
    const base = q
      ? requests.filter(
          (r) =>
            r.name.toLocaleLowerCase().includes(q) ||
            r.method.toLocaleLowerCase().includes(q) ||
            r.url.toLocaleLowerCase().includes(q),
        )
      : requests;
    return sortRequestList(base);
  });
  let requestPageSize = $state(50);
  let requestPage = $state(0);
  let visibleRequests = $derived.by(() => {
    if (filteredRequests.length <= 100) return filteredRequests;
    const start = requestPage * requestPageSize;
    return filteredRequests.slice(start, start + requestPageSize);
  });
  let totalRequestPages = $derived(Math.ceil(filteredRequests.length / requestPageSize));

  // Folder-grouped tree (used when not actively searching — a search flattens across folders,
  // same as it already flattens everything else).
  let rootRequests = $derived(sortRequestList(requests.filter((r) => !r.folder_id)));
  let requestsByFolderId = $derived.by(() => {
    const map = new Map<string, RequestSummary[]>();
    for (const r of requests) {
      if (r.folder_id) {
        const list = map.get(r.folder_id) ?? [];
        list.push(r);
        map.set(r.folder_id, list);
      }
    }
    for (const [key, list] of map) {
      map.set(key, sortRequestList(list));
    }
    return map;
  });

  // Folders don't have a "method" field, so that sort criterion falls back to name for them.
  // Grouped by parent so the sidebar can render them as a tree — folders can nest inside other
  // folders now, not just sit directly under the project (see models::Folder on the backend).
  const ROOT_FOLDER_KEY = "";
  let foldersByParentId = $derived.by(() => {
    const dir = requestSortDir === "asc" ? 1 : -1;
    const sorted = [...folders].sort((a, b) => {
      if (requestSortField === "custom") return a.sort_order - b.sort_order;
      if (requestSortField === "updated") return a.updated_at.localeCompare(b.updated_at) * dir;
      return a.name.localeCompare(b.name) * dir;
    });
    const map = new Map<string, Folder[]>();
    for (const f of sorted) {
      const key = f.parent_folder_id ?? ROOT_FOLDER_KEY;
      const list = map.get(key) ?? [];
      list.push(f);
      map.set(key, list);
    }
    return map;
  });
  let rootFolders = $derived(foldersByParentId.get(ROOT_FOLDER_KEY) ?? []);

  // The request bar's breadcrumb (see `.breadcrumb-row`) shows Project › ...folders... › Request,
  // matching Postman's own path — without the folder segment(s), a request nested several levels
  // deep looked identical to one sitting at the project root. Walks parent_folder_id up from the
  // selected request's folder to the project root, root-first for display order.
  let selectedRequestFolderChain = $derived.by(() => {
    if (!selectedRequest?.folder_id) return [];
    const byId = new Map(folders.map((f) => [f.id, f]));
    const chain: string[] = [];
    let current = byId.get(selectedRequest.folder_id);
    // A cap, not an expected depth — guards against an accidental parent_folder_id cycle turning
    // this into an infinite loop instead of just a wrong (but harmless) breadcrumb.
    for (let i = 0; current && i < 100; i++) {
      chain.unshift(current.name);
      current = current.parent_folder_id ? byId.get(current.parent_folder_id) : undefined;
    }
    return chain;
  });

  // Expand/collapse every folder in the current project's tree at once.
  function toggleExpandAllFolders() {
    if (expandedFolderIds.size < folders.length) {
      expandedFolderIds = new Set(folders.map((f) => f.id));
    } else {
      expandedFolderIds = new Set();
    }
  }

  // Developer Console state (LP-0412 - LP-0421)
  let showConsole = $state(false);
  let consoleEvents = $state<ConsoleEvent[]>([]);
  let consoleLevelFilter = $state<string>("all");
  let consoleActiveRequestOnly = $state(false);
  let consoleSearchFilter = $state("");
  let expandedEventIds = $state<Set<string>>(new Set());

  let filteredConsoleEvents = $derived.by(() => {
    return consoleEvents.filter((e) => {
      if (consoleLevelFilter !== "all" && e.level !== consoleLevelFilter) {
        return false;
      }
      if (consoleActiveRequestOnly && selectedRequest && e.request_id !== selectedRequest.id) {
        return false;
      }
      if (consoleSearchFilter.trim()) {
        const q = consoleSearchFilter.toLocaleLowerCase();
        const inMsg = e.message.toLocaleLowerCase().includes(q);
        const inType = e.event_type.toLocaleLowerCase().includes(q);
        const inCid = e.correlation_id.toLocaleLowerCase().includes(q);
        const inDetails = e.details ? JSON.stringify(e.details).toLocaleLowerCase().includes(q) : false;
        if (!inMsg && !inType && !inCid && !inDetails) return false;
      }
      return true;
    });
  });

  let consoleErrorCount = $derived(
    consoleEvents.filter((e) => e.level === "error").length,
  );
  let consoleWarnCount = $derived(
    consoleEvents.filter((e) => e.level === "warn").length,
  );

  // Structured Developer Console detail rendering (LP-1402) — every field rendered below is a
  // real field execution.rs's c.log(...) calls actually attach to that event_type (see
  // execution.rs's request_start/response_received/cookie_injected/request_error/test_assertion
  // sites); anything not recognized still falls through to the raw-JSON view further down so no
  // detail is ever silently dropped.
  let rawDetailsVisible = $state<Set<string>>(new Set());
  function toggleRawDetails(id: string) {
    const next = new Set(rawDetailsVisible);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    rawDetailsVisible = next;
  }

  
  function asHeaderRows(value: unknown): DetailHeaderRow[] {
    if (!Array.isArray(value)) return [];
    return value
      .filter((h): h is Record<string, unknown> => typeof h === "object" && h !== null)
      .map((h) => ({
        key: typeof h.key === "string" ? h.key : "",
        value: typeof h.value === "string" ? h.value : "",
        enabled: typeof h.enabled === "boolean" ? h.enabled : undefined,
      }));
  }

  
  function asCookieRows(value: unknown): DetailCookieRow[] {
    if (!Array.isArray(value)) return [];
    return value
      .filter((c): c is Record<string, unknown> => typeof c === "object" && c !== null)
      .map((c) => ({
        name: typeof c.name === "string" ? c.name : "",
        value: typeof c.value === "string" ? c.value : "",
        domain: typeof c.domain === "string" ? c.domain : "",
        path: typeof c.path === "string" ? c.path : "",
        http_only: Boolean(c.http_only),
        secure: Boolean(c.secure),
      }));
  }

  function detailStr(details: Record<string, unknown> | null | undefined, field: string): string | null {
    const v = details?.[field];
    return v === undefined || v === null ? null : String(v);
  }

  // Git & Collaboration state (LP-0701 - LP-0713)
  let showDiffModal = $state(false);
  let showHistoryModal = $state(false);
  let gitActiveTab = $state<"sync" | "conflicts" | "github" | "projectfile">("sync");

  let gitSettings = $state<WorkspaceGitSettings | null>(null);
  // Leftover per-project git configs (from before Git sync was workspace-level), offered as
  // options when this workspace has no settings of its own yet — never applied automatically.
  let legacyGitCandidates = $state<LegacyGitSettingsCandidate[]>([]);
  let gitStatus = $state<GitStatus | null>(null);
  let gitHistory = $state<GitCommit[]>([]);
  let gitDiffContent = $state("");
  let gitCommitMessage = $state("");
  let gitLoading = $state(false);
  let gitStatusLoading = $state(false);
  let gitActionFeedback = $state("");
  let gitActionError = $state("");

  // Real 3-way conflict view (LP-0711 follow-up) — base/local/remote content read straight
  // from Git's index stages, not a fabricated diff.
  let selectedConflictFile = $state<string | null>(null);
  let conflictVersions = $state<ConflictVersions | null>(null);
  let conflictVersionsLoading = $state(false);
  async function loadConflictVersions(file: string) {
    if (!gitRepoPathInput.trim()) return;
    selectedConflictFile = file;
    conflictVersionsLoading = true;
    try {
      conflictVersions = await api.gitGetConflictVersions(gitRepoPathInput.trim(), file);
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      conflictVersionsLoading = false;
    }
  }

  let gitRepoPathInput = $state("");
  let gitRemoteUrlInput = $state("");
  let gitBranchInput = $state("main");
  let gitAutoSyncInput = $state(false);
  let githubTokenInput = $state("");
  let githubShowToken = $state(false);
  let githubUser = $state<GitHubUser | null>(null);
  let githubRepoInfo = $state<GitHubRepoInfo | null>(null);
  let githubValidating = $state(false);

  let projectFileJson = $state("");
  let projectFileMaskSecrets = $state(true);
  let projectFileStatus = $state("");

  let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

  // Guards the hydration effect below so it only re-populates the editor from `selectedRequest`
  // when the SELECTION actually changes to a different request. `selectedRequest` is also
  // reassigned in place for the same request (autosave's `selectedRequest = updated`, a rename's
  // optimistic `{...selectedRequest, name}`) — without this guard, each of those would re-run
  // the hydration and clobber whatever the user had typed into some other field since the last
  // hydration, since it always reads from the (possibly slightly stale) object being assigned.
  let hydratedRequestId: string | null = null;

  function hydrateRequestFields(req: RequestFull) {
    editName = req.name;
    editMethod = req.method;
    editUrl = req.url;
    editHeaders = withTrailingEmptyRow(req.headers.map((h) => ({ ...h })), () => ({ key: "", value: "", enabled: true, description: "" }));
    editQueryParams = withTrailingEmptyRow(req.query_params.map((p) => ({ ...p })), () => ({ key: "", value: "", enabled: true }));
    const parsedBody = parseBodyForEditing(req.body);
    editBodyType = parsedBody.bodyType;
    editBody = parsedBody.rawBody;
    editGraphqlQuery = parsedBody.graphqlQuery;
    editGraphqlVariables = parsedBody.graphqlVariables;
    editFormDataItems = withTrailingEmptyRow(parsedBody.formDataItems, () => ({ key: "", value: "", enabled: true, is_file: false, file_path: null }));
    editUrlEncodedItems = withTrailingEmptyRow(parsedBody.urlEncodedItems, () => ({ key: "", value: "", enabled: true }));
    editBinaryFilePath = parsedBody.binaryFilePath;
    editDescription = req.description ?? "";

    // A request with a body someone actually filled in opens straight to Body — better
    // than always landing on Params and making the user go find it every time.
    const bodyHasContent =
      (parsedBody.bodyType === "raw" && parsedBody.rawBody.trim().length > 0) ||
      (parsedBody.bodyType === "graphql" && parsedBody.graphqlQuery.trim().length > 0) ||
      (parsedBody.bodyType === "form-data" && parsedBody.formDataItems.some((i) => i.key.trim())) ||
      (parsedBody.bodyType === "x-www-form-urlencoded" && parsedBody.urlEncodedItems.some((i) => i.key.trim())) ||
      (parsedBody.bodyType === "binary" && parsedBody.binaryFilePath.trim().length > 0);
    activeEditorTab = bodyHasContent ? "body" : "params";

    const auth = req.auth;
    editAuthType = auth.type;
    editAuthBearerToken = auth.type === "bearer" ? auth.token : "";
    editAuthBasicUsername = auth.type === "basic" ? auth.username : "";
    editAuthBasicPassword = auth.type === "basic" ? auth.password : "";
    editAuthApiKeyKey = auth.type === "api_key" ? auth.key : "";
    editAuthApiKeyValue = auth.type === "api_key" ? auth.value : "";
    editAuthApiKeyLocation = auth.type === "api_key" ? auth.location : "header";

    // Scripts
    editPreScript = req.pre_request_script ?? "";
    editPostScript = req.post_request_script ?? "";

    // Settings
    const settings = req.settings;
    editTimeoutMs = settings?.timeout_ms ?? null;
    editFollowRedirects = settings?.follow_redirects ?? true;
    editMaxRedirects = settings?.max_redirects ?? 10;
    editVerifySsl = settings?.verify_ssl ?? true;
    editProxyUrl = settings?.proxy_url ?? "";
    editHttpVersion = settings?.http_version ?? "";

    snippet = "";
    snippetError = "";
    autoSaveStatus = "saved";
    curlDetectedFeedback = "";
  }

  $effect(() => {
    if (selectedRequest && selectedRequest.id !== hydratedRequestId) {
      hydratedRequestId = selectedRequest.id;
      if (!tabDrafts.has(selectedRequest.id)) {
        hydrateRequestFields(selectedRequest);
      }
    } else if (!selectedRequest) {
      // Nothing open — clear the guard so reopening this same request later (e.g. after
      // closing its tab) is treated as a fresh selection and hydrates from the server again.
      hydratedRequestId = null;
    }
  });

  function buildAuthFromEditFields(): Auth {
    switch (editAuthType) {
      case "bearer":
        return { type: "bearer", token: editAuthBearerToken };
      case "basic":
        return { type: "basic", username: editAuthBasicUsername, password: editAuthBasicPassword };
      case "api_key":
        return { type: "api_key", key: editAuthApiKeyKey, value: editAuthApiKeyValue, location: editAuthApiKeyLocation };
      default:
        return { type: "none" };
    }
  }

  let requestContextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    request?: any | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    request: null,
  });

  function openRequestContextMenu(e: MouseEvent, req?: any) {
    e.preventDefault();
    e.stopPropagation();
    const x = Math.min(e.clientX, typeof window !== "undefined" ? window.innerWidth - 260 : 600);
    const y = Math.min(e.clientY, typeof window !== "undefined" ? window.innerHeight - 520 : 400);
    requestContextMenu = {
      visible: true,
      x: Math.max(10, x),
      y: Math.max(10, y),
      request: req || selectedRequest,
    };
  }

  function closeRequestContextMenu() {
    requestContextMenu = { visible: false, x: 0, y: 0, request: null };
  }

  let tabContextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    tab: RequestTab | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    tab: null,
  });

  function openTabContextMenu(e: MouseEvent, tab?: RequestTab | null) {
    e.preventDefault();
    e.stopPropagation();
    closeRequestContextMenu();
    const x = Math.min(e.clientX, typeof window !== "undefined" ? window.innerWidth - 240 : 600);
    const y = Math.min(e.clientY, typeof window !== "undefined" ? window.innerHeight - 340 : 400);
    tabContextMenu = {
      visible: true,
      x: Math.max(10, x),
      y: Math.max(10, y),
      tab: tab ?? currentTab,
    };
  }

  function closeTabContextMenu() {
    tabContextMenu = { visible: false, x: 0, y: 0, tab: null };
  }

  async function handleTabMenuNewRequest() {
    closeTabContextMenu();
    await quickCreateRequest(selectedProjectId ?? projects[0]?.id);
  }

  async function handleTabMenuDuplicateTab() {
    const target = tabContextMenu.tab ?? currentTab;
    closeTabContextMenu();
    if (!target) return;
    if (!target.tabType || target.tabType === "request") {
      await duplicateRequest(target.id);
    } else if (target.tabType === "env") {
      const env = allEnvironments.find((e) => e.id === (target.envId ?? target.id));
      if (env) {
        await quickCreateEnvironment(`${env.name} (Copy)`);
      }
    } else {
      const newId = `${target.id}-copy-${Date.now()}`;
      openTabs = [...openTabs, { ...target, id: newId, name: `${target.name} (Copy)` }];
      activeTabId = newId;
    }
  }

  function handleTabMenuCloseTab() {
    const target = tabContextMenu.tab ?? currentTab;
    closeTabContextMenu();
    if (target) {
      closeTabAction(target.id);
    }
  }

  function handleTabMenuForceCloseTab() {
    const target = tabContextMenu.tab ?? currentTab;
    closeTabContextMenu();
    if (target) {
      closeTab(target.id, true);
    }
  }

  function handleTabMenuCloseOtherTabs() {
    const target = tabContextMenu.tab ?? currentTab;
    closeTabContextMenu();
    if (!target) return;
    const others = openTabs.filter((t) => t.id !== target.id);
    for (const t of others) {
      tabDrafts.delete(t.id);
    }
    openTabs = openTabs.filter((t) => t.id === target.id);
    selectTab(target);
  }

  function handleTabMenuCloseAllTabs() {
    closeTabContextMenu();
    const hasDirty = openTabs.some((t) => isTabDirty(t.id));
    if (hasDirty) {
      showConfirm(t("common.confirm"), "You have unsaved changes in one or more tabs. Close all without saving?", () => {
        handleTabMenuForceCloseAllTabs();
      });
    } else {
      handleTabMenuForceCloseAllTabs();
    }
  }

  function handleTabMenuForceCloseAllTabs() {
    closeTabContextMenu();
    for (const t of openTabs) {
      tabDrafts.delete(t.id);
    }
    openTabs = [];
    activeTabId = null;
    selectedRequest = null;
  }

  async function handleTabMenuRevealInSidebar() {
    const target = tabContextMenu.tab ?? currentTab;
    closeTabContextMenu();
    if (!target) return;

    sidebarVisible = true;
    if (target.tabType === "env") {
      environmentsAccordionOpen = true;
      sidebarSection = "environments";
      await tick();
      const el = document.querySelector(`.sidebar-env-item-row.active, [title="${target.name}"]`);
      el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
      return;
    }

    const reqSummary = requests.find((r) => r.id === target.id);
    if (reqSummary) {
      if (reqSummary.project_id && reqSummary.project_id !== selectedProjectId) {
        await selectProject(reqSummary.project_id);
      }
      if (reqSummary.folder_id) {
        expandedFolderIds = new Set([...expandedFolderIds, reqSummary.folder_id]);
      }
      await tick();
      const el = document.querySelector(`.request-item.active, [data-req-id="${target.id}"]`) ||
                 Array.from(document.querySelectorAll('.request-item')).find(item => item.textContent?.includes(target.name));
      if (el) {
        el.scrollIntoView({ block: "nearest", behavior: "smooth" });
        (el as HTMLElement).classList.add('reveal-pulse');
        setTimeout(() => (el as HTMLElement).classList.remove('reveal-pulse'), 1500);
      }
    }
  }

  // Replaces {{name}} with the value from the selected environment (which overrides globals).
  // Secret variables are masked by the backend, and unknown names are left as-is — so "Resolved"
  // output never invents or leaks values.
  function substituteKnownVariables(text: string): string {
    const lookup = new Map<string, string>();
    for (const v of projectVariables) if (v.enabled && !v.is_secret) lookup.set(v.key, v.value);
    for (const v of environmentVariables) if (v.enabled && !v.is_secret) lookup.set(v.key, v.value);
    return text.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (whole, name: string) => lookup.get(name) ?? whole);
  }

  function getEffectiveRequestData(req?: any) {
    const isCurrent = !req || req.id === selectedRequest?.id;
    const method = isCurrent ? editMethod : (req.method || "GET");
    const rawUrl: string = isCurrent ? editUrl : (req.url ?? "");
    const url = snippetMode === "resolved" ? substituteKnownVariables(rawUrl) : rawUrl;
    let body = "";
    if (isCurrent) {
      body = editBody || "";
    } else if (req.body) {
      body = typeof req.body === "string" ? req.body : JSON.stringify(req.body, null, 2);
    }
    const headers = isCurrent ? editHeaders.filter((h: any) => h.enabled && h.key.trim()) : [];
    return { method, url, body, headers };
  }

  function generateCurlBash(req?: any): string {
    const { method, url, body, headers } = getEffectiveRequestData(req);
    const lines = [
      `curl --location${method !== "GET" ? ` --request ${method}` : ""} '${url}'`
    ];
    if (headers.length > 0) {
      for (const h of headers) {
        lines.push(`--header '${h.key}: ${h.value}'`);
      }
    } else {
      lines.push(`--header 'Content-Type: application/json'`);
      lines.push(`--header 'Accept: application/json'`);
    }
    if (body && method !== "GET" && method !== "HEAD") {
      lines.push(`--data-raw '${body}'`);
    }
    return lines.join(" \\\n");
  }

  function generateCurlCmd(req?: any): string {
    const { method, url, body, headers } = getEffectiveRequestData(req);
    const lines = [
      `curl --location${method !== "GET" ? ` --request ${method}` : ""} "${url}"`
    ];
    if (headers.length > 0) {
      for (const h of headers) {
        lines.push(`--header "${h.key}: ${h.value}"`);
      }
    } else {
      lines.push(`--header "Content-Type: application/json"`);
      lines.push(`--header "Accept: application/json"`);
    }
    if (body && method !== "GET" && method !== "HEAD") {
      const escaped = body.replace(/"/g, '\\"');
      lines.push(`--data-raw "${escaped}"`);
    }
    return lines.join(" ^\n");
  }

  function generateCurlPowerShell(req?: any): string {
    const { method, url, body, headers } = getEffectiveRequestData(req);
    let hStr = "";
    if (headers.length > 0) {
      hStr = headers.map((h: any) => `  "${h.key}" = "${h.value}"`).join("\n");
    } else {
      hStr = '  "Content-Type" = "application/json"\n  "Accept" = "application/json"';
    }
    const bodyPart = body ? `$body = @'\n${body}\n'@\n` : "";
    const bodyArg = body ? ` -Body $body` : "";
    return `$headers = @{\n${hStr}\n}\n${bodyPart}$response = Invoke-RestMethod -Uri "${url}" -Method ${method} -Headers $headers${bodyArg}`;
  }

  function generateFetch(req?: any): string {
    const { method, url, body } = getEffectiveRequestData(req);
    const bodyPart = body && method !== "GET" ? `\n  body: ${JSON.stringify(body)},` : "";
    return `const myHeaders = new Headers();\nmyHeaders.append("Content-Type", "application/json");\n\nconst raw = JSON.stringify(${body || "{}"});\n\nconst requestOptions = {\n  method: "${method}",\n  headers: myHeaders,${bodyPart}\n  redirect: "follow"\n};\n\nfetch("${url}", requestOptions)\n  .then((response) => response.text())\n  .then((result) => console.log(result))\n  .catch((error) => console.error(error));`;
  }

  function generateNodeFetch(req?: any): string {
    const { method, url, body } = getEffectiveRequestData(req);
    const bodyPart = body && method !== "GET" ? `,\n  body: JSON.stringify(${body || "{}"})` : "";
    return `const fetch = (...args) => import('node-fetch').then(({default: fetch}) => fetch(...args));\n\nconst response = await fetch("${url}", {\n  method: "${method}",\n  headers: {\n    "Content-Type": "application/json"\n  }${bodyPart}\n});\nconst data = await response.json();\nconsole.log(data);`;
  }

  function generatePythonRequests(req?: any): string {
    const { method, url, body } = getEffectiveRequestData(req);
    const payloadPart = body ? `payload = json.dumps(${body})\n` : "payload = {}\n";
    return `import requests\nimport json\n\nurl = "${url}"\n${payloadPart}headers = {\n  'Content-Type': 'application/json'\n}\n\nresponse = requests.request("${method}", url, headers=headers, data=payload)\nprint(response.text)`;
  }

  function generatePreload(req?: any): string {
    const { url } = getEffectiveRequestData(req);
    return `<link rel="preload" href="${url}" as="fetch" crossorigin="anonymous">`;
  }

  function generateHar(req?: any): string {
    const { method, url, body } = getEffectiveRequestData(req);
    const harObj = {
      log: {
        version: "1.2",
        creator: { name: "Postman", version: "11.0.0" },
        entries: [{
          startedDateTime: new Date().toISOString(),
          request: {
            method,
            url,
            httpVersion: "HTTP/1.1",
            headers: [{ name: "Content-Type", value: "application/json" }],
            postData: body ? { mimeType: "application/json", text: body } : undefined
          }
        }]
      }
    };
    return JSON.stringify(harObj, null, 2);
  }

  function getAllRequestsList() {
    if (requests && requests.length > 0) return requests;
    if (openTabs && openTabs.length > 0) return openTabs.filter((t: any) => t.tabType !== "env" && t.tabType !== "doc");
    if (selectedRequest) return [selectedRequest];
    return [];
  }

  async function copyContextUrl(req?: any) {
    closeRequestContextMenu();
    const { url } = getEffectiveRequestData(req);
    await copyTextToClipboard(url);
  }
  async function copyContextAsCurlBash(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateCurlBash(req));
  }
  async function copyContextAsCurlCmd(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateCurlCmd(req));
  }
  async function copyContextAsCurlPowerShell(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateCurlPowerShell(req));
  }
  async function copyContextAsFetch(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateFetch(req));
  }
  async function copyContextAsNodeFetch(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateNodeFetch(req));
  }
  async function copyContextAsPythonRequests(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generatePythonRequests(req));
  }
  async function copyContextAsPreload(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generatePreload(req));
  }
  async function copyContextAsHar(req?: any) {
    closeRequestContextMenu();
    await copyTextToClipboard(generateHar(req));
  }

  async function copyAllUrlsAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const urls = list.map(r => getEffectiveRequestData(r).url).join("\n");
    await copyTextToClipboard(urls);
  }
  async function copyAllAsCurlBashAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const all = list.map(r => generateCurlBash(r)).join("\n\n");
    await copyTextToClipboard(all);
  }
  async function copyAllAsCurlCmdAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const all = list.map(r => generateCurlCmd(r)).join("\n\n");
    await copyTextToClipboard(all);
  }
  async function copyAllAsCurlPowerShellAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const all = list.map(r => generateCurlPowerShell(r)).join("\n\n");
    await copyTextToClipboard(all);
  }
  async function copyAllAsFetchAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const all = list.map(r => generateFetch(r)).join("\n\n");
    await copyTextToClipboard(all);
  }
  async function copyAllAsNodeFetchAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const all = list.map(r => generateNodeFetch(r)).join("\n\n");
    await copyTextToClipboard(all);
  }
  async function copyAllAsHarAction() {
    closeRequestContextMenu();
    const list = getAllRequestsList();
    const entries = list.map(r => {
      const data = getEffectiveRequestData(r);
      return {
        startedDateTime: new Date().toISOString(),
        request: {
          method: data.method,
          url: data.url,
          httpVersion: "HTTP/1.1",
          headers: [{ name: "Content-Type", value: "application/json" }],
          postData: data.body ? { mimeType: "application/json", text: data.body } : undefined
        }
      };
    });
    const harObj = {
      log: {
        version: "1.2",
        creator: { name: "Postman", version: "11.0.0" },
        entries
      }
    };
    await copyTextToClipboard(JSON.stringify(harObj, null, 2));
  }

  function highlightSnippetCode(raw: string): string {
    if (!raw) return "";
    let html = raw
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");

    const strings: string[] = [];
    html = html.replace(/('(?:[^'\\]|\\.)*'|"(?:[^"\\]|\\.)*")/g, (match) => {
      const idx = strings.length;
      strings.push(match);
      return `__SNIP_STR_${idx}__`;
    });

    html = html
      .replace(/(^|\s)(--[a-zA-Z0-9_-]+|-[a-zA-Z0-9]+)/g, '$1<span class="tok-flag">$2</span>')
      .replace(/\b(curl|Invoke-RestMethod|fetch|requests\.request|const|let|var|import|from)\b/g, '<span class="tok-keyword">$1</span>')
      .replace(/(\\\n|\^\n)/g, '<span class="tok-escape">$1</span>');

    html = html.replace(/__SNIP_STR_(\d+)__/g, (_, idx) => {
      return `<span class="tok-string">${strings[Number(idx)]}</span>`;
    });

    return html;
  }

  async function handleVisualizeResponse() {
    sendMenuOpen = false;
    responseSubTab = "tests";
    await sendCurrentRequest();
  }
  function handleWriteTests() {
    sendMenuOpen = false;
    activeEditorTab = "scripts";
  }
  function handleDebugRequest() {
    sendMenuOpen = false;
    showConsole = true;
    refreshConsoleEvents();
  }
  function handleDownloadResponse() {
    sendMenuOpen = false;
    sendAndDownload();
  }

  async function copyAsCurl() {
    if (!selectedRequest) return;
    snippetError = "";
    snippetLoading = true;
    try {
      if (snippetTarget === "bash") {
        snippet = generateCurlBash(selectedRequest);
      } else if (snippetTarget === "windows_cmd") {
        snippet = generateCurlCmd(selectedRequest);
      } else if (snippetTarget === "power_shell") {
        snippet = generateCurlPowerShell(selectedRequest);
      } else if (snippetTarget === "java_script_fetch") {
        snippet = generateFetch(selectedRequest);
      } else if (snippetTarget === "node_fetch") {
        snippet = generateNodeFetch(selectedRequest);
      } else if (snippetTarget === "python_requests") {
        snippet = generatePythonRequests(selectedRequest);
      } else if (snippetTarget === "preload") {
        snippet = generatePreload(selectedRequest);
      } else if (snippetTarget === "har") {
        snippet = generateHar(selectedRequest);
      } else {
        snippet = await api.generateCurlSnippet(selectedRequest.id, selectedEnvironmentId, snippetMode, snippetTarget);
      }
    } catch (err) {
      snippetError = describeError(err);
    } finally {
      snippetLoading = false;
    }
  }

  // Regenerates automatically — the Code Snippet panel has no "Generate" button; it just always
  // shows the snippet for whatever's currently selected (target/mode/request/environment).
  // Code is the right sidebar's default view (rightPanel starts null, not "code" — see the
  // template's `{:else}` fallback), so this fires whenever the panel is visible and isn't Info,
  // not just when it's explicitly "code" — but never while the sidebar itself is hidden, since
  // a hidden panel isn't "in use" (matches the app's lazy-everything rule: no request costs
  // CPU/network for a view the user isn't looking at).
  $effect(() => {
    const visible = rightSidebarVisible;
    const panel = rightPanel;
    const req = selectedRequest;
    const target = snippetTarget;
    const mode = snippetMode;
    const envId = selectedEnvironmentId;
    if (visible && panel !== "info" && req) {
      copyAsCurl();
    }
  });

  async function copySnippetToClipboard() {
    if (!snippet) return;
    try {
      await navigator.clipboard.writeText(snippet);
      textCopiedNotice = "Copied to clipboard!";
      setTimeout(() => (textCopiedNotice = ""), 2000);
    } catch (err) {
      snippetError = describeError(err);
    }
  }
  let textCopiedNotice = $state("");
  async function copyTextToClipboard(text: string) {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      textCopiedNotice = "Copied to clipboard!";
      setTimeout(() => (textCopiedNotice = ""), 2000);
    } catch {}
  }

  // Live preview of what {{vars}} in the URL resolve to for the currently selected
  // environment — same resolver code path the HTTP engine will use to build the real
  // request (README §14 "Stored Request -> ... -> Final HTTP Request").
  async function refreshUrlPreview() {
    const projectId = selectedProjectId;
    const requestId = selectedRequest?.id ?? null;
    const template = editUrl;
    if (!projectId || !template) {
      urlPreview = null;
      return;
    }
    try {
      urlPreview = await api.resolvePreview(projectId, selectedEnvironmentId, requestId, template);
    } catch {
      urlPreview = null;
    }
  }

  $effect(() => {
    // Re-run whenever any of these change — reading them here (rather than inside
    // refreshUrlPreview) is what makes them tracked dependencies of this effect.
    void selectedProjectId;
    void selectedRequest?.id;
    void selectedEnvironmentId;
    void editUrl;
    refreshUrlPreview();
  });

  // Splits the raw URL template into plain-text runs and {{variable}} references, so the URL
  // bar can render each variable as its own highlighted, hoverable token directly in place —
  // matching where the user is actually looking, instead of only in the warning banner below.
  
  function parseUrlTokens(url: string): UrlToken[] {
    const tokens: UrlToken[] = [];
    const re = /\{\{([^{}]+)\}\}/g;
    let lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = re.exec(url))) {
      if (match.index > lastIndex) tokens.push({ type: "text", text: url.slice(lastIndex, match.index) });
      tokens.push({ type: "var", name: match[1].trim(), raw: match[0] });
      lastIndex = match.index + match[0].length;
    }
    if (lastIndex < url.length) tokens.push({ type: "text", text: url.slice(lastIndex) });
    return tokens;
  }
  let urlTokens = $derived(parseUrlTokens(editUrl));

  // Keeps the highlight overlay's horizontal scroll glued to the real (invisible-text) input
  // underneath it, so long URLs that scroll internally don't desync the two layers.
  function syncUrlOverlayScroll(e: Event) {
    const input = e.target as HTMLInputElement;
    const overlay = input.previousElementSibling as HTMLElement | null;
    if (overlay) overlay.scrollLeft = input.scrollLeft;
  }

  onMount(() => {
    loadWorkspaces().then(() => Promise.all([loadProjects(), loadGitSettings()]));
    loadAllEnvironments();
    loadAiSettings();
    api.isAiConfigured().then((configured) => (aiConfigured = configured)).catch((err) => console.error("Failed to check AI configuration", err));
    refreshConsoleEvents();
    refreshSystemDiagnostics();
    try {
      const saved = localStorage.getItem("lp-theme");
      if (saved === "dark" || saved === "light" || saved === "terminal" || saved === "blueprint") themeMode = saved; else themeMode = "dark";
      const savedAccent = localStorage.getItem("lp-accent-color");
      if (savedAccent && /^#[0-9a-f]{6}$/i.test(savedAccent)) accentColor = savedAccent;
      const savedTint = localStorage.getItem("lp-surface-tint");
      if (savedTint && (SURFACE_TINTS.some((t) => t.id === savedTint) || /^#[0-9a-f]{6}$/i.test(savedTint))) {
        surfaceTint = savedTint;
      }
      const savedHeadingFont = localStorage.getItem("lp-heading-font");
      if (savedHeadingFont && THEME_FONT_OPTIONS.some((f) => f.id === savedHeadingFont)) headingFontOverride = savedHeadingFont;
      const savedBodyFont = localStorage.getItem("lp-body-font");
      if (savedBodyFont && THEME_FONT_OPTIONS.some((f) => f.id === savedBodyFont)) bodyFontOverride = savedBodyFont;
      const savedTextColor = localStorage.getItem("lp-text-color");
      if (savedTextColor && /^#[0-9a-f]{6}$/i.test(savedTextColor)) textColorOverride = savedTextColor;
      const savedCollapsed = localStorage.getItem("lp-response-pane-collapsed");
      if (savedCollapsed === "true" || savedCollapsed === "false") responsePaneCollapsed = savedCollapsed === "true";
      const savedInterval = localStorage.getItem("lp-auto-sync-interval-ms");
      if (savedInterval && Number(savedInterval) > 0) autoSyncIntervalMs = Number(savedInterval);
      const savedShortcuts = localStorage.getItem("lp-shortcuts-enabled");
      if (savedShortcuts) {
        shortcutsEnabled = { ...shortcutsEnabled, ...JSON.parse(savedShortcuts) };
      }
      const savedScale = localStorage.getItem("lp-ui-scale");
      if (savedScale && Number(savedScale) > 0) {
        uiScale = Number(savedScale);
        document.documentElement.style.fontSize = `${uiScale}%`;
      }
      const savedLocale = localStorage.getItem("lp-locale");
      if (savedLocale === "en" || savedLocale === "ar") {
        locale = savedLocale;
      }
      applyLocale(locale);
      const savedProjectSort = localStorage.getItem("lp-project-sort");
      if (savedProjectSort) {
        const parsed = JSON.parse(savedProjectSort);
        if (parsed.field === "name" || parsed.field === "created" || parsed.field === "updated" || parsed.field === "custom") projectSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") projectSortDir = parsed.dir;
      }
      const savedSidebarVisible = localStorage.getItem("lp-sidebar-visible");
      if (savedSidebarVisible === "true" || savedSidebarVisible === "false") sidebarVisible = savedSidebarVisible === "true";
      const savedRailVisible = localStorage.getItem("lp-rail-visible");
      if (savedRailVisible === "true" || savedRailVisible === "false") screensRailVisible = savedRailVisible === "true";
      const savedRightSidebarVisible = localStorage.getItem("lp-right-sidebar-visible");
      if (savedRightSidebarVisible === "true" || savedRightSidebarVisible === "false") rightSidebarVisible = savedRightSidebarVisible === "true";
      const savedUtilityRailVisible = localStorage.getItem("lp-utility-rail-visible");
      if (savedUtilityRailVisible === "true" || savedUtilityRailVisible === "false") utilityRailVisible = savedUtilityRailVisible === "true";
    } catch {
      // ignore — settings just stay at their defaults
    }

    sidebarWidth = adaptiveSidebarWidth();
    responsePaneHeight = adaptiveResponsePaneHeight();
    const onWindowResize = () => {
      if (!sidebarManuallyResized) sidebarWidth = adaptiveSidebarWidth();
      if (!responsePaneManuallyResized) responsePaneHeight = adaptiveResponsePaneHeight();
    };
    window.addEventListener("resize", onWindowResize);

    const onGlobalKeydown = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      const key = e.key.toLowerCase();
      if (key === "k" && shortcutsEnabled.commandPalette) {
        e.preventDefault();
        openPalette();
      } else if (e.key === "Enter" && shortcutsEnabled.sendRequest) {
        if (selectedRequest && !sending) {
          e.preventDefault();
          sendCurrentRequest();
        }
      } else if (key === "s" && shortcutsEnabled.saveRequest) {
        if (selectedRequest) {
          e.preventDefault();
          saveRequest();
        }
      } else if (key === "n" && shortcutsEnabled.newRequest) {
        if (selectedProjectId) {
          e.preventDefault();
          quickCreateRequest(selectedProjectId);
        }
      } else if (key === "t") {
        e.preventDefault();
        quickCreateRequest(selectedProjectId ?? projects[0]?.id);
      } else if (key === "tab" && shortcutsEnabled.nextTab && !e.shiftKey) {
        e.preventDefault();
        cycleTab(1);
      } else if (key === "tab" && shortcutsEnabled.prevTab && e.shiftKey) {
        e.preventDefault();
        cycleTab(-1);
      } else if (key === "w" && shortcutsEnabled.closeTab) {
        e.preventDefault();
        const tabToClose = activeTabId || selectedRequest?.id;
        if (tabToClose) {
          if (e.altKey) {
            closeTab(tabToClose, true);
          } else {
            closeTabAction(tabToClose);
          }
        }
      }
    };
    window.addEventListener("keydown", onGlobalKeydown);
    return () => {
      window.removeEventListener("resize", onWindowResize);
      window.removeEventListener("keydown", onGlobalKeydown);
    };
  });

  // Keep each screen's real data fresh only while it's actually visible, without polling when
  // nobody can see it.
  $effect(() => {
    if (activeScreen === "workspace" || activeScreen === "settings") {
      refreshSystemDiagnostics();
    }
    if (activeScreen === "launcher" || activeScreen === "workspace") {
      refreshProjectRequestCounts();
    }
    if (activeScreen === "history") {
      refreshProjectHistory();
    }
    if (activeScreen !== "workspace") {
      responseExpanded = false;
    }
  });

  function areHeadersEqual(edit: typeof editHeaders, orig: HeaderEntry[] | undefined | null): boolean {
    const clean = withoutEmptyKeyRows(edit);
    const origList = orig || [];
    if (clean.length !== origList.length) return false;
    for (let i = 0; i < clean.length; i++) {
      const a = clean[i];
      const b = origList[i];
      if (!b) return false;
      if (a.key !== b.key || a.value !== b.value || a.enabled !== b.enabled) return false;
      if ((a.description ?? "") !== (b.description ?? "")) return false;
    }
    return true;
  }

  function areQueryParamsEqual(edit: typeof editQueryParams, orig: QueryParam[] | undefined | null): boolean {
    const clean = withoutEmptyKeyRows(edit);
    const origList = orig || [];
    if (clean.length !== origList.length) return false;
    for (let i = 0; i < clean.length; i++) {
      const a = clean[i];
      const b = origList[i];
      if (!b) return false;
      if (a.key !== b.key || a.value !== b.value || a.enabled !== b.enabled) return false;
    }
    return true;
  }

  function isAuthEqual(orig: Auth | undefined | null): boolean {
    if (!orig) return editAuthType === "none";
    if (editAuthType !== orig.type) return false;
    const o = orig as any;
    if (editAuthType === "bearer") {
      return (editAuthBearerToken || "") === (o.token || "");
    }
    if (editAuthType === "basic") {
      return (editAuthBasicUsername || "") === (o.username || "") && (editAuthBasicPassword || "") === (o.password || "");
    }
    if (editAuthType === "api_key") {
      return (editAuthApiKeyKey || "") === (o.key || "") && (editAuthApiKeyValue || "") === (o.value || "") && (editAuthApiKeyLocation || "header") === (o.location || "header");
    }
    return true;
  }

  function isBodyEqual(origBody: string | null | undefined): boolean {
    const orig = (origBody ?? "").trim();
    const current = serializeBodyForStorage().trim();
    return orig === current;
  }

  function areSettingsEqual(settings: RequestSettings | null | undefined): boolean {
    if (!settings) {
      if (editTimeoutMs !== null) return false;
      if (editFollowRedirects !== true) return false;
      if (editMaxRedirects !== 10) return false;
      if (editVerifySsl !== true) return false;
      if (editProxyUrl.trim() !== "") return false;
      if (editHttpVersion.trim() !== "") return false;
      return true;
    }
    if (editTimeoutMs !== (settings.timeout_ms ?? null)) return false;
    if (editFollowRedirects !== (settings.follow_redirects ?? true)) return false;
    if (editMaxRedirects !== (settings.max_redirects ?? 10)) return false;
    if (editVerifySsl !== (settings.verify_ssl ?? true)) return false;
    if (editProxyUrl.trim() !== (settings.proxy_url ?? "")) return false;
    if (editHttpVersion.trim() !== (settings.http_version ?? "")) return false;
    return true;
  }

  function isCurrentRequestDirty(): boolean {
    if (!selectedRequest) return false;
    if (editName !== selectedRequest.name) return true;
    if (editMethod !== selectedRequest.method) return true;
    if (editUrl !== selectedRequest.url) return true;
    if (editDescription !== (selectedRequest.description ?? "")) return true;
    if (editPreScript !== (selectedRequest.pre_request_script ?? "")) return true;
    if (editPostScript !== (selectedRequest.post_request_script ?? "")) return true;
    if (!isBodyEqual(selectedRequest.body)) return true;
    if (!areHeadersEqual(editHeaders, selectedRequest.headers)) return true;
    if (!areQueryParamsEqual(editQueryParams, selectedRequest.query_params)) return true;
    if (!isAuthEqual(selectedRequest.auth)) return true;
    if (!areSettingsEqual(selectedRequest.settings)) return true;
    return false;
  }

  function saveCurrentDraft() {
    if (!selectedRequest) return;
    if (!isCurrentRequestDirty()) {
      tabDrafts.delete(selectedRequest.id);
      return;
    }
    const draft: RequestDraft = {
      editName,
      editMethod,
      editUrl,
      editHeaders: editHeaders.map((h) => ({ ...h })),
      editQueryParams: editQueryParams.map((p) => ({ ...p })),
      editBody,
      editDescription,
      editAuthType,
      editAuthBearerToken,
      editAuthBasicUsername,
      editAuthBasicPassword,
      editAuthApiKeyKey,
      editAuthApiKeyValue,
      editAuthApiKeyLocation,
      editPreScript,
      editPostScript,
      editTimeoutMs,
      editFollowRedirects,
      editMaxRedirects,
      editVerifySsl,
      editProxyUrl,
      editHttpVersion,
      editBodyType,
      editGraphqlQuery,
      editGraphqlVariables,
      editFormDataItems: editFormDataItems.map((i) => ({ ...i })),
      editUrlEncodedItems: editUrlEncodedItems.map((i) => ({ ...i })),
      editBinaryFilePath,
      activeEditorTab,
      activeResponse,
      activeResponseBody,
      activeResponseTruncated,
    };
    tabDrafts.set(selectedRequest.id, draft);
    // Bounded cache (LP-0411): evict oldest entry if exceeding 50 entries
    if (tabDrafts.size > 50) {
      const oldestKey = tabDrafts.keys().next().value;
      if (oldestKey && oldestKey !== selectedRequest.id) {
        tabDrafts.delete(oldestKey);
      }
    }
  }

  function restoreDraft(draft: RequestDraft) {
    editName = draft.editName;
    editMethod = draft.editMethod;
    editUrl = draft.editUrl;
    editHeaders = draft.editHeaders.map((h) => ({ ...h }));
    editQueryParams = draft.editQueryParams.map((p) => ({ ...p }));
    editBody = draft.editBody;
    editDescription = draft.editDescription;
    editAuthType = draft.editAuthType;
    editAuthBearerToken = draft.editAuthBearerToken;
    editAuthBasicUsername = draft.editAuthBasicUsername;
    editAuthBasicPassword = draft.editAuthBasicPassword;
    editAuthApiKeyKey = draft.editAuthApiKeyKey;
    editAuthApiKeyValue = draft.editAuthApiKeyValue;
    editAuthApiKeyLocation = draft.editAuthApiKeyLocation;
    editPreScript = draft.editPreScript;
    editPostScript = draft.editPostScript;
    editTimeoutMs = draft.editTimeoutMs;
    editFollowRedirects = draft.editFollowRedirects;
    editMaxRedirects = draft.editMaxRedirects;
    editVerifySsl = draft.editVerifySsl;
    editProxyUrl = draft.editProxyUrl;
    editHttpVersion = draft.editHttpVersion;
    editBodyType = draft.editBodyType;
    editGraphqlQuery = draft.editGraphqlQuery;
    editGraphqlVariables = draft.editGraphqlVariables;
    editFormDataItems = draft.editFormDataItems.map((i) => ({ ...i }));
    editUrlEncodedItems = draft.editUrlEncodedItems.map((i) => ({ ...i }));
    editBinaryFilePath = draft.editBinaryFilePath;
    activeEditorTab = draft.activeEditorTab;
    activeResponse = draft.activeResponse;
    activeResponseBody = draft.activeResponseBody;
    activeResponseTruncated = draft.activeResponseTruncated;
    if (isCurrentRequestDirty()) {
      scheduleAutoSave();
    } else if (selectedRequest) {
      tabDrafts.delete(selectedRequest.id);
    }
  }

  function isTabDirty(tabId: string): boolean {
    const tab = openTabs.find((t) => t.id === tabId);
    if (tab && tab.tabType && tab.tabType !== "request") {
      return false;
    }
    if (tabId === selectedRequest?.id) {
      return isCurrentRequestDirty();
    }
    return tabDrafts.has(tabId);
  }

  // Ctrl/Cmd+Tab / Ctrl/Cmd+Shift+Tab — cycles through open tabs in their current order,
  // wrapping around at either end. A no-op with 0-1 tabs open.
  function cycleTab(direction: 1 | -1) {
    if (openTabs.length < 2 || !selectedRequest) return;
    const idx = openTabs.findIndex((t) => t.id === selectedRequest!.id);
    if (idx === -1) return;
    const nextIdx = (idx + direction + openTabs.length) % openTabs.length;
    openRequest(openTabs[nextIdx].id);
  }

  function closeTabAction(id: string) {
    if (isTabDirty(id)) {
      showConfirm(t("common.confirm"), t("error.confirmCloseDirtyTab") || "You have unsaved changes. Are you sure you want to close without saving?", () => {
        // discard changes
        if (id === selectedRequest?.id && autoSaveTimer) {
          clearTimeout(autoSaveTimer);
          autoSaveTimer = null;
          autoSaveStatus = "saved";
        }
        tabDrafts.delete(id);
        closeTab(id, true);
      });
    } else {
      closeTab(id, false);
    }
  }

  function openEnvironmentTab(env: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === env.id)) {
      openTabs = [
        ...openTabs,
        {
          id: env.id,
          name: env.name,
          method: "ENV",
          tabType: "env",
          envId: env.id,
        },
      ];
    }
    activeTabId = env.id;
    selectedEnvironmentId = env.id;
    loadVariables();
    activeScreen = "workspace";
    if (!expandedEnvIds.has(env.id)) {
      toggleEnvExpand(env.id);
    }
  }

  function openDocumentTab(doc: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === doc.id)) {
      openTabs = [
        ...openTabs,
        {
          id: doc.id,
          name: doc.name,
          method: "DOC",
          tabType: "doc",
        },
      ];
    }
    activeTabId = doc.id;
    activeScreen = "workspace";
  }

  function openSpecTab(spec: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === spec.id)) {
      openTabs = [
        ...openTabs,
        {
          id: spec.id,
          name: spec.name,
          method: "SPEC",
          tabType: "spec",
        },
      ];
    }
    activeTabId = spec.id;
    activeScreen = "workspace";
  }

  function openMockTab(mockItem: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === mockItem.id)) {
      openTabs = [
        ...openTabs,
        {
          id: mockItem.id,
          name: mockItem.name,
          method: "MOCK",
          tabType: "mock",
        },
      ];
    }
    activeTabId = mockItem.id;
    activeScreen = "workspace";
  }

  function openDatasetTab(dataset: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === dataset.id)) {
      openTabs = [
        ...openTabs,
        {
          id: dataset.id,
          name: dataset.name,
          method: "DATA",
          tabType: "dataset",
        },
      ];
    }
    activeTabId = dataset.id;
    activeScreen = "workspace";
  }

  function openFlowTab(flow: { id: string; name: string }) {
    if (!openTabs.some((t) => t.id === flow.id)) {
      openTabs = [
        ...openTabs,
        {
          id: flow.id,
          name: flow.name,
          method: "FLOW",
          tabType: "flow",
        },
      ];
    }
    activeTabId = flow.id;
    activeScreen = "workspace";
  }

  function selectTab(tab: RequestTab) {
    activeTabId = tab.id;
    if (tab.tabType && tab.tabType !== "request") {
      activeScreen = "workspace";
      if (tab.tabType === "env") {
        selectedEnvironmentId = tab.envId ?? tab.id;
        loadVariables();
      }
      return;
    }
    openRequest(tab.id);
  }

  async function forkCurrentEnvironment() {
    const current = allEnvironments.find((e) => e.id === selectedEnvironmentId);
    if (!current) return;
    const copy = await duplicateEnvironment(current, `${current.name} (Fork ${envForkCount + 1})`);
    if (!copy) return;
    envForkCount++;
    openEnvironmentTab(copy);
    exportFeedback = `Created fork "${copy.name}" with ${environmentVariables.length} variable(s)`;
    setTimeout(() => { exportFeedback = ""; }, 3000);
  }

  function shareCurrentEnvironment() {
    const shareable = environmentVariables.filter((v) => v.enabled && !v.is_secret);
    const skipped = environmentVariables.filter((v) => v.is_secret).length;
    const vars = Object.fromEntries(shareable.map((v) => [v.key, v.value]));
    copyTextToClipboard(JSON.stringify(vars, null, 2));
    exportFeedback = skipped
      ? `Copied ${shareable.length} variable(s) as JSON (${skipped} secret value(s) left out)`
      : `Copied ${shareable.length} variable(s) as JSON`;
    setTimeout(() => { exportFeedback = ""; }, 3000);
  }

  function handleAddGlobalVariableClick() {
    const defaultKey = `variable_${projectVariables.length + 1}`;
    projectVariables = [
      ...projectVariables,
      {
        id: `var-new-${Date.now()}`,
        key: defaultKey,
        value: "",
        enabled: true,
        is_secret: false,
        is_local: false,
        scope: "global",
        environment_id: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ];
    exportFeedback = `Added new global variable "${defaultKey}"`;
    setTimeout(() => { exportFeedback = ""; }, 2500);
  }

  function toggleUtilityRail() {
    const next = !utilityRailVisible;
    setUtilityRailVisible(next);
    if (!next) {
      setRightSidebarVisible(false);
      rightPanel = null;
    }
  }

  let currentTab = $derived.by(() => {
    if (activeTabId) {
      const found = openTabs.find((t) => t.id === activeTabId);
      if (found) return found;
    }
    const req = selectedRequest;
    if (req) {
      const found = openTabs.find((t) => t.id === req.id);
      if (found) return found;
    }
    return openTabs[0] ?? null;
  });

  async function closeTab(id: string, skipSave: boolean = false) {
    tabDrafts.delete(id);
    const idx = openTabs.findIndex((t) => t.id === id);
    if (idx === -1) return;
    const wasActive = (activeTabId === id) || (selectedRequest?.id === id);
    openTabs = openTabs.filter((t) => t.id !== id);

    if (wasActive) {
      if (openTabs.length > 0) {
        const nextTab = openTabs[Math.max(0, idx - 1)];
        selectTab(nextTab);
      } else {
        activeTabId = null;
        if (!skipSave) await saveRequest();
        selectedRequest = null;
        activeResponse = null;
        activeResponseBody = "";
        responseHistory = [];
      }
    }
  }

  async function refreshConsoleEvents() {
    try {
      consoleEvents = await api.getConsoleEvents(200);
    } catch (err) {
      console.error("Failed to load console events:", err);
    }
  }

  async function clearConsole() {
    try {
      await api.clearConsoleEvents();
      consoleEvents = [];
      expandedEventIds.clear();
    } catch (err) {
      console.error("Failed to clear console:", err);
    }
  }

  async function copyConsoleLog() {
    try {
      const lines = filteredConsoleEvents.map(
        (e) =>
          `[${e.timestamp}] [${e.level.toUpperCase()}] [${e.event_type}] [#${e.correlation_id.slice(0, 8)}] ${e.message}`,
      );
      await navigator.clipboard.writeText(lines.join("\n"));
      copyFeedback = t("copy.consoleLogCopied");
      setTimeout(() => (copyFeedback = ""), 2000);
    } catch (err) {
      console.error("Failed to copy console log:", err);
    }
  }

  async function exportConsoleJson() {
    try {
      const jsonStr = await api.exportConsoleEvents();
      const blob = new Blob([jsonStr], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `light-postman-console-${new Date().toISOString().replace(/[:.]/g, "-")}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Failed to export console events:", err);
    }
  }

  function toggleEventExpanded(eventId: string) {
    const updated = new Set(expandedEventIds);
    if (updated.has(eventId)) {
      updated.delete(eventId);
    } else {
      updated.add(eventId);
    }
    expandedEventIds = updated;
  }

  async function copyEventDetails(evt: ConsoleEvent) {
    try {
      await navigator.clipboard.writeText(JSON.stringify(evt, null, 2));
      copyFeedback = t("copy.eventJsonCopied");
      setTimeout(() => (copyFeedback = ""), 2000);
    } catch (err) {
      console.error("Failed to copy event details:", err);
    }
  }

  function formatConsoleTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toTimeString().split(" ")[0] + "." + String(d.getMilliseconds()).padStart(3, "0");
    } catch {
      return iso;
    }
  }

  async function loadProjects() {
    if (!activeWorkspaceId) return;
    try {
      projects = await api.listProjects(activeWorkspaceId);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function loadAiSettings() {
    try {
      const s = await api.getAiSettings();
      aiSettings = s;
      aiConfigured = s.is_configured;
      aiProviderInput = s.provider;
      aiApiKeyInput = s.api_key ?? "";
      aiModelInput = s.model || aiProviderDefaultModel(s.provider);
      aiBaseUrlInput = s.base_url ?? "";
    } catch (err) {
      console.error("Failed to load AI settings", err);
    }
  }

  async function saveAiSettingsAction() {
    aiTestError = "";
    aiTestFeedback = "";
    aiSettingsFeedback = "";
    if (aiProviderInput === "custom" && !aiBaseUrlInput.trim()) {
      aiTestError = t("ai.customRequiresBaseUrl");
      return;
    }
    try {
      await api.saveAiSettings({
        provider: aiProviderInput,
        api_key: aiApiKeyInput.trim() || null,
        model: aiModelInput.trim() || null,
        base_url: aiBaseUrlInput.trim() || null,
      });
      await loadAiSettings();
      aiSettingsFeedback = t("ai.settingsSaved");
      setTimeout(() => (aiSettingsFeedback = ""), 3000);
    } catch (err) {
      aiTestError = describeError(err);
    }
  }

  async function testAiConnectionAction() {
    aiTesting = true;
    aiTestFeedback = "";
    aiTestError = "";
    try {
      const res = await api.testAiConnection();
      aiTestFeedback = res;
      aiConfigured = true;
    } catch (err) {
      aiTestError = `AI Connection Failed: ${describeError(err)}`;
    } finally {
      aiTesting = false;
    }
  }

  async function generateWithAi(event: Event) {
    event.preventDefault();
    if (!aiPrompt.trim() || aiGenerating) return;
    aiGenerating = true;
    aiPreview = null;
    try {
      if (selectedProjectId) {
        aiPreview = await api.generateApiWithProjectContext(
          selectedProjectId,
          aiPrompt.trim(),
          aiIncludeExistingRequests,
          aiIncludeVariables,
        );
      } else {
        aiPreview = await api.generateApiWithAi(aiPrompt.trim());
      }
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      aiGenerating = false;
    }
  }

  async function saveCurrentResponse() {
    if (!selectedRequest || !activeResponse || !activeResponseBody) return;
    const defaultName = `Example - ${activeResponse.status}`;
    try {
      const created = await api.createSampleResponse({
        request_id: selectedRequest.id,
        name: defaultName,
        status: activeResponse.status,
        status_text: activeResponse.status_text || "",
        headers: activeResponse.headers ?? [],
        body: activeResponseBody,
        content_type: activeResponse.content_type || null
      });
      await loadSampleResponses(selectedRequest.id);

      // Jump to the Mock tab so the new sample is visible, and drop straight into renaming it.
      activeEditorTab = "mock";
      startRenameSampleResponse(created);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function loadSampleResponses(requestId: string) {
    try {
      const list = await api.listSampleResponses(requestId);
      const next = new Map(sampleResponsesByRequestId);
      next.set(requestId, list);
      sampleResponsesByRequestId = next;
    } catch (err) {
      console.error("Failed to load sample responses", err);
    }
  }

  function deleteSampleResponseAction(requestId: string, id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteSampleResponse"), async () => {
      try {
      await api.deleteSampleResponse(id);
      await loadSampleResponses(requestId);
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  function startRenameSampleResponse(sr: SampleResponse) {
    renamingSampleResponseId = sr.id;
    renameSampleResponseValue = sr.name;
  }

  async function submitRenameSampleResponse(requestId: string) {
    if (!renamingSampleResponseId) return;
    const id = renamingSampleResponseId;
    const name = renameSampleResponseValue.trim();
    renamingSampleResponseId = null;
    if (!name) return;
    try {
      await api.updateSampleResponse({ id, name });
      await loadSampleResponses(requestId);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Expanding a request row in the tree lazy-loads its samples the first time — same
  // lazy-loading discipline as the rest of the app (README §4/§20).
  async function toggleTreeRequestExpanded(requestId: string) {
    const next = new Set(expandedTreeRequestIds);
    if (next.has(requestId)) {
      next.delete(requestId);
    } else {
      next.add(requestId);
      if (!sampleResponsesByRequestId.has(requestId)) {
        await loadSampleResponses(requestId);
      }
    }
    expandedTreeRequestIds = next;
  }

  async function generateSampleResponseWithAiAction() {
    if (!selectedRequest) return;
    generatingSample = true;
    sampleFeedback = "";
    try {
      const sample = await api.generateSampleResponseWithAi(selectedRequest.id);
      await loadSampleResponses(selectedRequest.id);
      sampleFeedback = `Sample response (${sample.status}) generated!`;
      setTimeout(() => (sampleFeedback = ""), 3000);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      generatingSample = false;
    }
  }

  async function generateTestsAndDocsWithAiAction(target: "tests" | "docs") {
    if (!selectedRequest) return;
    generatingTestsDocs = true;
    testsDocsFeedback = "";
    try {
      const res = await api.generateTestsAndDocsWithAi(selectedRequest.id);
      if (target === "tests") {
        editPostScript = editPostScript
          ? `${editPostScript}\n\n${res.tests_script}`
          : res.tests_script;
        testsDocsFeedback = "Generated test assertions added to post-request script!";
      } else {
        editDescription = editDescription
          ? `${editDescription}\n\n${res.documentation}`
          : res.documentation;
        testsDocsFeedback = "Generated documentation added!";
      }
      setTimeout(() => (testsDocsFeedback = ""), 3000);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      generatingTestsDocs = false;
    }
  }

  async function scanSourceProjectAction() {
    const dir = sourceDirectoryInput.trim();
    if (!dir) {
      sourceActionFeedback = "Please enter a source directory path.";
      return;
    }
    sourceScanning = true;
    sourceActionFeedback = "";
    try {
      const report = await api.scanSourceProject(dir);
      sourceReport = report;
      if (selectedProjectId) {
        await api.setProjectSourceDirectory(selectedProjectId, dir, report.frameworks.join(", ") || null);
      }
      sourceActionFeedback = `Scanned ${report.scanned_files_count} files. Discovered ${report.endpoints.length} API endpoints.`;
    } catch (err) {
      sourceActionFeedback = describeError(err);
    } finally {
      sourceScanning = false;
    }
  }

  async function importDiscoveredEndpointAction(endpoint: DiscoveredEndpoint) {
    if (!selectedProjectId) return;
    try {
      const summary = await api.importDiscoveredEndpoint(selectedProjectId, endpoint);
      requests = [summary, ...requests];
      await openRequest(summary.id);
      sourceActionFeedback = `Imported ${endpoint.method} ${endpoint.path} into project!`;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function loadSourceAssociation(projectId: string) {
    try {
      const dir = await api.getProjectSourceDirectory(projectId);
      if (dir) {
        sourceDirectoryInput = dir;
      } else {
        sourceDirectoryInput = "";
        sourceReport = null;
      }
    } catch {
      // ignore
    }
  }

  // Explicit approval step (LP-0805): nothing from the AI preview is persisted until the
  // user clicks this — it goes through the exact same create_request validation as a
  // manually-typed request.
  async function addAiPreviewToProject() {
    if (!aiPreview || !selectedProjectId) return;
    try {
      const request = await api.createRequest({
        project_id: selectedProjectId,
        name: aiPreview.name,
        method: aiPreview.method,
        url: aiPreview.url,
        headers: aiPreview.headers,
        query_params: aiPreview.query_params,
        auth: { type: "none" },
        body: aiPreview.body,
      });
      requests = [
        { id: request.id, project_id: request.project_id, folder_id: request.folder_id, name: request.name, method: request.method, url: request.url, sort_order: request.sort_order, created_at: request.created_at, updated_at: request.updated_at },
        ...requests,
      ];
      openTabs = [
        ...openTabs,
        { id: request.id, name: request.name, method: request.method },
      ];
      selectedRequest = request;
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = [];
      aiPreview = null;
      aiPrompt = "";
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // "+" with no upfront text field: create with a placeholder name, then drop straight into
  // the same inline-rename UI used for renaming an existing project, so the user types the
  // real name in place instead of in a separate form first.
  // Best-effort, silent: if this workspace already has a git repo configured, a newly created
  // (or imported) project should show up in it without the user having to remember to hit
  // "Save" — same repo-wide write `saveWorkspaceToRepoAction` uses, just not user-triggered.
  // Failures are logged, not surfaced — the project itself was still created successfully.
  async function syncNewProjectIntoWorkspaceRepo() {
    if (!activeWorkspaceId || !gitSettings?.repo_path) return;
    try {
      await api.saveWorkspaceToRepo(activeWorkspaceId, gitSettings.repo_path, false);
    } catch (err) {
      console.error("Failed to add the new project to the workspace's git repo:", err);
    }
  }

  async function quickCreateProject() {
    if (!activeWorkspaceId) return;
    try {
      const project = await api.createProject("New Project", activeWorkspaceId);
      projects = [project, ...projects];
      await selectProject(project.id);
      startRenameProject(project);
      await syncNewProjectIntoWorkspaceRepo();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Only ever loads request *metadata* for the selected project — bodies/headers
  // stay on disk until a specific request tab is opened (README §4/§20).
  // Clicking the already-open project again closes it (collapses its request list back) instead
  // of just reselecting the same project — a real toggle, not a no-op re-fetch.
  async function toggleProjectSelection(id: string) {
    if (selectedProjectId === id) {
      // Same flush as openRequest/closeTab — collapsing the project must not silently drop a
      // pending debounced edit on whatever request was open.
      await saveRequest();
      selectedProjectId = null;
      selectedRequest = null;
      selectedEnvironmentId = null;
      openTabs = [];
      tabDrafts.clear();
      requests = [];
      folders = [];
      setRightSidebarVisible(false);
      rightPanel = null;
      const nextExpanded = new Set(expandedProjectIds);
      nextExpanded.delete(id);
      expandedProjectIds = nextExpanded;
      const nextCache = new Map(secondaryProjectCache);
      nextCache.delete(id);
      secondaryProjectCache = nextCache;
    } else {
      selectProject(id);
    }
  }

  // Expand/collapse a project's tree in the sidebar without making it the active project — lets
  // you peek into (or tuck away) another project while keeping your current one primary.
  async function toggleProjectExpand(id: string) {
    if (expandedProjectIds.has(id)) {
      const next = new Set(expandedProjectIds);
      next.delete(id);
      expandedProjectIds = next;
      return;
    }
    expandedProjectIds = new Set(expandedProjectIds).add(id);
    if (id === selectedProjectId || secondaryProjectCache.has(id)) return;
    secondaryProjectLoading = new Set(secondaryProjectLoading).add(id);
    try {
      const [reqs, flds] = await Promise.all([api.listRequests(id), api.listFolders(id)]);
      const next = new Map(secondaryProjectCache);
      next.set(id, { requests: reqs, folders: flds });
      secondaryProjectCache = next;
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      const next = new Set(secondaryProjectLoading);
      next.delete(id);
      secondaryProjectLoading = next;
    }
  }

  // A request opened from a secondary (expanded-but-not-active) project's tree first promotes
  // that project to active — same machinery selecting it from the launcher/palette already uses.
  async function openSecondaryRequest(projectId: string, requestId: string) {
    await selectProject(projectId);
    openRequest(requestId);
  }

  async function selectProject(id: string) {
    await saveRequest();

    // Snapshot the project we're leaving into the secondary cache so its tree stays visible
    // (and reasonably fresh) in the sidebar instead of vanishing the moment it stops being
    // primary — this is what lets several projects stay open/browsable at once.
    if (selectedProjectId && selectedProjectId !== id) {
      const next = new Map(secondaryProjectCache);
      next.set(selectedProjectId, { requests, folders });
      secondaryProjectCache = next;
    }
    if (!expandedProjectIds.has(id)) {
      expandedProjectIds = new Set(expandedProjectIds).add(id);
    }
    if (secondaryProjectCache.has(id)) {
      const next = new Map(secondaryProjectCache);
      next.delete(id);
      secondaryProjectCache = next;
    }

    selectedProjectId = id;
    selectedRequest = null;
    selectedEnvironmentId = null;
    openTabs = [];
    tabDrafts.clear();
    try {
      const savedTabs = localStorage.getItem(`lp-open-tabs-${id}`);
      if (savedTabs) openTabs = JSON.parse(savedTabs);
    } catch {}
    requestSortField = "custom";
    requestSortDir = "asc";
    try {
      const savedRequestSort = localStorage.getItem(`lp-request-sort-${id}`);
      if (savedRequestSort) {
        const parsed = JSON.parse(savedRequestSort);
        if (parsed.field === "name" || parsed.field === "method" || parsed.field === "created" || parsed.field === "updated" || parsed.field === "custom") requestSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") requestSortDir = parsed.dir;
      }
    } catch {}
    // The Code Snippet panel is generated from whatever request was open — with none open
    // anymore (fresh project, no tabs yet), it has nothing to show, so close it rather than
    // leaving an empty panel visible until the user notices and closes it themselves.
    setRightSidebarVisible(false);
    rightPanel = null;
    loadingRequests = true;
    try {
      requests = await api.listRequests(id);
      folders = await api.listFolders(id);
      try {
        const savedReqId = localStorage.getItem(`lp-selected-request-${id}`);
        if (savedReqId && requests.some(r => r.id === savedReqId)) {
          openRequest(savedReqId).catch(console.error);
        }
      } catch {}
      // Auto-select the project's preferred environment, if it set one and that environment
      // still exists (it may have been deleted since — the backend already clears the
      // reference then, but the frontend's stale `projects` entry might not have refreshed yet).
      const defaultEnvId = projects.find((p) => p.id === id)?.default_environment_id;
      if (defaultEnvId && allEnvironments.some((e) => e.id === defaultEnvId)) {
        selectedEnvironmentId = defaultEnvId;
      }
      await loadVariables();
      await loadSourceAssociation(id);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      loadingRequests = false;
    }
  }

  // Marks (or clears) the currently-selected environment as this project's default — the one
  // auto-selected the next time the project is opened.
  async function toggleDefaultEnvironment() {
    if (!selectedProjectId) return;
    const project = projects.find((p) => p.id === selectedProjectId);
    if (!project) return;
    try {
      const isCurrentlyDefault = project.default_environment_id === selectedEnvironmentId;
      const updated = isCurrentlyDefault
        ? await api.updateProject({ id: selectedProjectId, clear_default_environment_id: true })
        : await api.updateProject({ id: selectedProjectId, default_environment_id: selectedEnvironmentId ?? undefined, clear_default_environment_id: !selectedEnvironmentId });
      projects = projects.map((p) => (p.id === updated.id ? updated : p));
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // "+ Add new environment…" sticky option in the env select, and the "+" in the Environments
  // screen sidebar: create with a placeholder name, then drop straight into inline-rename —
  // same pattern as quickCreateProject/quickCreateRequest, no persistent text field needed.
  async function quickCreateEnvironment(customName?: unknown) {
    const pid = selectedProjectId ?? (projects[0]?.id ?? "");
    if (!pid) return;
    const name = typeof customName === "string" && customName.trim() ? customName.trim() : "New Environment";
    try {
      const env = await api.createEnvironment(pid, name);
      await loadAllEnvironments();
      selectedEnvironmentId = env.id;
      await loadVariables();
      if (typeof customName !== "string") {
        startRenameEnvironment(env);
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function startRenameEnvironment(env: Environment | EnvironmentWithProject) {
    renamingEnvironmentId = env.id;
    renameEnvironmentValue = env.name;
  }

  async function submitRenameEnvironment() {
    const id = renamingEnvironmentId;
    const value = renameEnvironmentValue.trim();
    renamingEnvironmentId = null;
    if (!id || !value) return;
    try {
      await api.updateEnvironment({ id, name: value });
      await loadAllEnvironments();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function deleteEnvironmentAction(id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteEnvironment"), async () => {
    try {
      await api.deleteEnvironment(id);
      await loadAllEnvironments();
      if (selectedEnvironmentId === id) {
        selectedEnvironmentId = null;
        await loadVariables();
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  // "+" next to a project: create a request with sensible defaults (no upfront method/name/URL
  // form) and drop straight into inline-rename, same pattern as quickCreateProject. Works from
  // any project row, not just the currently-selected one — the sidebar only renders a project's
  // request tree while it's selected, so a non-selected project must be selected first or the
  // new request would be created correctly on the backend but never appear anywhere in the UI.
  async function quickCreateRequest(projectId: string, folderId: string | null = null) {
    try {
      if (selectedProjectId !== projectId) {
        await selectProject(projectId);
      }
      const request = await api.createRequest({
        project_id: projectId,
        folder_id: folderId,
        name: "New Request",
        method: "GET",
        url: "",
        headers: [],
        query_params: [],
        auth: { type: "none" },
        body: null,
      });
      requests = [
        {
          id: request.id,
          project_id: request.project_id,
          folder_id: request.folder_id,
          name: request.name,
          method: request.method,
          url: request.url,
          sort_order: request.sort_order,
          created_at: request.created_at,
          updated_at: request.updated_at,
        },
        ...requests,
      ];
      if (folderId) expandedFolderIds = new Set([...expandedFolderIds, folderId]);
      openTabs = [...openTabs, { id: request.id, name: request.name, method: request.method }];
      selectedRequest = request;
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = [];
      startRenameRequest(request.id, request.name);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // "+" for folder next to a project (or a folder's own "add subfolder" action) — creates the
  // folder, then drops straight into inline-rename, same pattern as
  // quickCreateRequest/quickCreateProject.
  async function quickCreateFolder(projectId: string, parentFolderId: string | null = null) {
    try {
      if (selectedProjectId !== projectId) {
        await selectProject(projectId);
      }
      const folder = await api.createFolder({ project_id: projectId, name: "New Folder", parent_folder_id: parentFolderId });
      folders = [...folders, folder];
      const nextExpanded = new Set([...expandedFolderIds, folder.id]);
      if (parentFolderId) nextExpanded.add(parentFolderId);
      expandedFolderIds = nextExpanded;
      startRenameFolder(folder);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function startRenameFolder(folder: Folder) {
    renamingFolderId = folder.id;
    renameFolderValue = folder.name;
  }

  async function submitRenameFolder() {
    const id = renamingFolderId;
    const value = renameFolderValue.trim();
    renamingFolderId = null;
    if (!id || !value) return;
    try {
      const updated = await api.updateFolder({ id, name: value });
      folders = folders.map((f) => (f.id === updated.id ? updated : f));
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Ungroups the folder's requests back to the project root and promotes any child folders up
  // to the deleted folder's own parent, instead of deleting either — matches the backend's own
  // delete_folder semantics (see folder_store.rs).
  function deleteFolderAction(id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteFolder"), async () => {
    try {
      await api.deleteFolder(id);
      const deleted = folders.find((f) => f.id === id);
      const promotedParentId = deleted?.parent_folder_id ?? null;
      folders = folders
        .filter((f) => f.id !== id)
        .map((f) => (f.parent_folder_id === id ? { ...f, parent_folder_id: promotedParentId } : f));
      requests = requests.map((r) => (r.folder_id === id ? { ...r, folder_id: null } : r));
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  function toggleFolderExpanded(id: string) {
    const next = new Set(expandedFolderIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedFolderIds = next;
  }

  // Hydrate the full request only when the user actually opens it.
  async function openRequest(id: string) {
    activeTabId = id;
    // The sidebar is visible on every screen, so picking a request must always bring the editor back.
    activeScreen = "workspace";
    if (selectedRequest?.id === id) return;
    if (selectedRequest) {
      if (autoSaveTimer) {
        clearTimeout(autoSaveTimer);
        autoSaveTimer = null;
        await saveRequest();
      }
      if (isCurrentRequestDirty()) {
        saveCurrentDraft();
      } else {
        tabDrafts.delete(selectedRequest.id);
      }
    }

    // Ensure tab exists in openTabs (LP-0407)
    const reqSummary = requests.find((r) => r.id === id);
    if (!openTabs.some((t) => t.id === id)) {
      openTabs = [
        ...openTabs,
        {
          id,
          name: reqSummary?.name ?? "Request",
          method: reqSummary?.method ?? "GET",
        },
      ];
    }

    try {
      selectedRequest = await api.getRequest(id);
      hydratedRequestId = selectedRequest.id;
      openTabs = openTabs.map((t) =>
        t.id === id ? { ...t, name: selectedRequest!.name, method: selectedRequest!.method } : t
      );
      if (tabDrafts.has(id)) {
        restoreDraft(tabDrafts.get(id)!);
      } else {
        hydrateRequestFields(selectedRequest);
        activeResponse = null;
        activeResponseBody = "";
      }
      activeScreen = "workspace";
      responseHistory = await api.listResponseSummaries(id);
      if (responseHistory.length > 0) {
        await openHistoryResponse(responseHistory[0].id);
      }
      await loadSampleResponses(id);
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function sendCurrentRequest() {
    if (!selectedRequest || sending) return;
    // Send always executes the persisted request, so unsaved edits (URL, headers, body, params,
    // auth, ...) must be flushed first — otherwise Send would silently fire the last-saved
    // version while the editor shows something different. saveRequest() is a cheap diff-based
    // no-op when nothing changed, so it's safe to call unconditionally.
    await saveRequest();
    if (!selectedRequest) return;
    const requestId = selectedRequest.id;
    sending = true;
    try {
      const meta = await api.sendRequest(requestId, selectedEnvironmentId);
      activeResponse = meta;
      responseSubTab = "body";
      responsePaneCollapsed = false;
      if (responsePaneHeight < 180) responsePaneHeight = 320;
      const body = await api.getResponseBody(meta.id);
      activeResponseBody = body.text;
      activeResponseTruncated = body.truncated;
      responseHistory = await api.listResponseSummaries(requestId);
    } catch (err) {
      if (isAppError(err) && err.kind === "Cancelled") {
        if (sendCancelledNoticeTimer) clearTimeout(sendCancelledNoticeTimer);
        sendCancelledNotice = describeError(err);
        sendCancelledNoticeTimer = setTimeout(() => (sendCancelledNotice = ""), 2500);
      } else {
        errorMessage = describeError(err);
      }
    } finally {
      sending = false;
      await refreshConsoleEvents();
    }
  }

  // Postman's "Send and Download" — same request, but the response body goes straight to a
  // file-save dialog instead of just filling the response viewer. Reuses sendCurrentRequest's own
  // success/error handling (including the save-before-send flush) and only downloads when it
  // actually produced a response — a failed or cancelled send has nothing worth saving.
  async function sendAndDownload() {
    sendMenuOpen = false;
    await sendCurrentRequest();
    if (activeResponse && activeResponseBody) downloadResponseBody();
  }

  async function cancelCurrentSend() {
    if (!selectedRequest) return;
    try {
      await api.cancelSend(selectedRequest.id);
      await refreshConsoleEvents();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function openHistoryResponse(id: string) {
    try {
      activeResponse = await api.getResponse(id);
      responseSubTab = "body";
      const body = await api.getResponseBody(id);
      activeResponseBody = body.text;
      activeResponseTruncated = body.truncated;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Clicking a saved sample response in the tree used to just open its parent request (showing
  // whatever that request's last *real* response was, if any) — this actually loads the sample's
  // own captured status/headers/body into the response viewer, same as clicking a history entry.
  async function openSampleResponse(requestId: string, sr: SampleResponse) {
    await openRequest(requestId);
    activeResponse = {
      id: sr.id,
      request_id: sr.request_id,
      status: sr.status,
      status_text: sr.status_text,
      duration_ms: 0,
      body_size: (sr.body ?? "").length,
      created_at: sr.created_at,
      headers: sr.headers,
      content_type: sr.content_type,
      cookies: [],
    };
    activeResponseBody = sr.body ?? "";
    activeResponseTruncated = false;
    responseSubTab = "body";
  }

  function startRenameProject(project: Project) {
    renamingProjectId = project.id;
    renameProjectValue = project.name;
  }

  async function submitRenameProject(event: Event) {
    event.preventDefault();
    if (!renamingProjectId || !renameProjectValue.trim()) return;
    try {
      const updated = await api.updateProject({
        id: renamingProjectId,
        name: renameProjectValue.trim(),
      });
      projects = projects.map((p) => (p.id === updated.id ? updated : p));
      renamingProjectId = null;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function deleteProject(id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteProject"), async () => {
    try {
      await api.deleteProject(id);
      projects = projects.filter((p) => p.id !== id);
      await loadAllEnvironments(); // deleting a project cascades its environments too
      if (selectedProjectId === id) {
        selectedProjectId = null;
        requests = [];
        selectedRequest = null;
      }
      if (expandedProjectIds.has(id)) {
        const next = new Set(expandedProjectIds);
        next.delete(id);
        expandedProjectIds = next;
      }
      if (secondaryProjectCache.has(id)) {
        const next = new Map(secondaryProjectCache);
        next.delete(id);
        secondaryProjectCache = next;
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  // Autosave: every editable field schedules a debounced save instead of requiring an explicit
  // "Save" click. sendCurrentRequest() also flushes synchronously before sending, so a send never
  // races an unsaved edit even if the debounce hasn't fired yet.
  let autoSaveStatus = $state<"saved" | "unsaved" | "saving" | "error">("saved");
  let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
  function scheduleAutoSave() {
    if (!selectedRequest) return;
    autoSaveStatus = "unsaved";
    if (autoSaveTimer) clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(() => {
      autoSaveTimer = null;
      saveRequest();
    }, 700);
  }

  let curlDetectedFeedback = $state("");

  function looksLikeCurlCommand(text: string): boolean {
    return /^\s*curl(\.exe)?\s/i.test(text);
  }

  // Pasting a full `curl ...` command into the URL field parses it with the same importer the
  // dedicated Import screen uses, then fills in method/URL/headers/auth/body in place — instead
  // of requiring a trip to Import just because the user had a curl snippet handy.
  async function handleUrlPaste(event: ClipboardEvent) {
    const text = event.clipboardData?.getData("text") ?? "";
    if (!looksLikeCurlCommand(text) || !selectedRequest) return;
    event.preventDefault();
    curlDetectedFeedback = "Detecting curl command…";
    try {
      const parsed = await api.importCurl(text.trim());
      editMethod = parsed.method;
      editUrl = parsed.url;
      editHeaders = withTrailingEmptyRow(parsed.headers.map((h) => ({ ...h })), () => ({ key: "", value: "", enabled: true, description: "" }));
      editQueryParams = withTrailingEmptyRow(parsed.query_params.map((p) => ({ ...p })), () => ({ key: "", value: "", enabled: true }));
      const auth = parsed.auth;
      editAuthType = auth.type;
      editAuthBearerToken = auth.type === "bearer" ? auth.token : "";
      editAuthBasicUsername = auth.type === "basic" ? auth.username : "";
      editAuthBasicPassword = auth.type === "basic" ? auth.password : "";
      editAuthApiKeyKey = auth.type === "api_key" ? auth.key : "";
      editAuthApiKeyValue = auth.type === "api_key" ? auth.value : "";
      editAuthApiKeyLocation = auth.type === "api_key" ? auth.location : "header";
      const parsedBody = parseBodyForEditing(parsed.body);
      editBodyType = parsedBody.bodyType;
      editBody = parsedBody.rawBody;
      editGraphqlQuery = parsedBody.graphqlQuery;
      editGraphqlVariables = parsedBody.graphqlVariables;
      editFormDataItems = withTrailingEmptyRow(parsedBody.formDataItems, () => ({ key: "", value: "", enabled: true, is_file: false, file_path: null }));
      editUrlEncodedItems = withTrailingEmptyRow(parsedBody.urlEncodedItems, () => ({ key: "", value: "", enabled: true }));
      editBinaryFilePath = parsedBody.binaryFilePath;
      // Only apply the two settings curl's own flags (-k, -x) can actually express — leave
      // timeout/redirects/HTTP-version alone, since curl parsing has no information about them
      // and resetting them would silently discard whatever the user already had configured.
      if (parsed.settings.verify_ssl !== undefined && parsed.settings.verify_ssl !== null) {
        editVerifySsl = parsed.settings.verify_ssl;
      }
      if (parsed.settings.proxy_url) {
        editProxyUrl = parsed.settings.proxy_url;
      }
      curlDetectedFeedback = "Detected a curl command — filled in method, headers, auth, and body.";
      scheduleAutoSave();
    } catch (err) {
      curlDetectedFeedback = "";
      errorMessage = describeError(err);
    }
  }

  // Only sends fields that actually differ from the hydrated request —
  // the backend preserves anything omitted, but there's no reason to send it either.
  async function saveRequest() {
    if (!selectedRequest) return;
    if (autoSaveTimer) {
      clearTimeout(autoSaveTimer);
      autoSaveTimer = null;
    }
    const original = selectedRequest;
    autoSaveStatus = "saving";
    try {
      const auth = buildAuthFromEditFields();
      const currentSettings: RequestSettings = {
        timeout_ms: editTimeoutMs,
        follow_redirects: editFollowRedirects,
        max_redirects: editMaxRedirects,
        verify_ssl: editVerifySsl,
        proxy_url: editProxyUrl.trim() ? editProxyUrl.trim() : null,
        http_version: editHttpVersion.trim() ? editHttpVersion.trim() : null,
      };

      const updated = await api.updateRequest({
        id: original.id,
        ...(editName !== original.name ? { name: editName } : {}),
        ...(editMethod !== original.method ? { method: editMethod } : {}),
        ...(editUrl !== original.url ? { url: editUrl } : {}),
        ...(() => {
          const cleanQueryParams = withoutEmptyKeyRows(editQueryParams);
          return JSON.stringify(cleanQueryParams) !== JSON.stringify(original.query_params)
            ? { query_params: cleanQueryParams }
            : {};
        })(),
        ...(() => {
          const cleanHeaders = withoutEmptyKeyRows(editHeaders);
          return JSON.stringify(cleanHeaders) !== JSON.stringify(original.headers)
            ? { headers: cleanHeaders }
            : {};
        })(),
        ...(JSON.stringify(auth) !== JSON.stringify(original.auth) ? { auth } : {}),
        ...(() => {
          const serialized = serializeBodyForStorage();
          if (serialized !== (original.body ?? "")) {
            return serialized.trim().length === 0
              ? { clear_body: true }
              : { body: serialized };
          }
          return {};
        })(),
        ...(() => {
          if (editDescription !== (original.description ?? "")) {
            return editDescription.trim().length === 0
              ? { clear_description: true }
              : { description: editDescription };
          }
          return {};
        })(),
        ...(() => {
          if (editPreScript !== (original.pre_request_script ?? "")) {
            return editPreScript.trim().length === 0
              ? { clear_pre_request_script: true }
              : { pre_request_script: editPreScript };
          }
          return {};
        })(),
        ...(() => {
          if (editPostScript !== (original.post_request_script ?? "")) {
            return editPostScript.trim().length === 0
              ? { clear_post_request_script: true }
              : { post_request_script: editPostScript };
          }
          return {};
        })(),
        settings: currentSettings,
      });
      selectedRequest = updated;
      await refreshDiagnostics();
      openTabs = openTabs.map((t) =>
        t.id === updated.id
          ? { ...t, name: updated.name, method: updated.method }
          : t,
      );
      tabDrafts.delete(updated.id);
      requests = requests.map((r) =>
        r.id === updated.id
          ? { id: updated.id, project_id: updated.project_id, folder_id: updated.folder_id, name: updated.name, method: updated.method, url: updated.url, sort_order: updated.sort_order, created_at: updated.created_at, updated_at: updated.updated_at }
          : r,
      );
      autoSaveStatus = "saved";
    } catch (err) {
      autoSaveStatus = "error";
      errorMessage = describeError(err);
    }
  }

  async function importCurlCommand(event: Event) {
    event.preventDefault();
    if (!curlImportText.trim() || !selectedProjectId) return;
    curlImportError = "";
    try {
      const parsed = await api.importCurl(curlImportText.trim());
      let reqName = curlImportName.trim();
      if (!reqName) {
        try {
          const u = new URL(parsed.url);
          reqName = `${parsed.method} ${u.pathname || "/"}`;
        } catch {
          reqName = `${parsed.method} ${parsed.url}`;
        }
      }
      const request = await api.createRequest({
        project_id: selectedProjectId,
        name: reqName,
        method: parsed.method,
        url: parsed.url,
        headers: parsed.headers,
        query_params: parsed.query_params,
        auth: parsed.auth,
        body: parsed.body,
        settings: parsed.settings,
      });
      requests = [
        {
          id: request.id,
          project_id: request.project_id,
          folder_id: request.folder_id,
          name: request.name,
          method: request.method,
          url: request.url,
          sort_order: request.sort_order,
          created_at: request.created_at,
          updated_at: request.updated_at,
        },
        ...requests,
      ];
      openTabs = [
        ...openTabs,
        { id: request.id, name: request.name, method: request.method },
      ];
      selectedRequest = request;
      curlImportText = "";
      curlImportName = "";
    } catch (err) {
      curlImportError = describeError(err);
    }
  }

  function handleCollectionFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const reader = new FileReader();
      reader.onload = (e) => {
        collectionImportText = (e.target?.result as string) || "";
      };
      reader.readAsText(input.files[0]);
    }
  }

  async function importPostmanCollectionAction() {
    if (!collectionImportText.trim() || !activeWorkspaceId) return;
    collectionImportLoading = true;
    collectionImportError = "";
    collectionImportReport = null;
    try {
      const targetId = collectionImportTarget === "current" ? selectedProjectId : null;
      const report = await api.importPostmanCollection(collectionImportText.trim(), targetId, activeWorkspaceId);
      collectionImportReport = report;
      await loadProjects();
      if (!selectedProjectId || collectionImportTarget === "new") {
        // A fresh import already lands its requests in sort_order matching the source
        // collection's order (create_request auto-assigns sequentially — see
        // docs/superpowers/specs/2026-09-15-custom-order-design.md). Without this, the
        // sidebar's default "Name" sort would immediately re-alphabetize them, hiding that
        // order. Written before selectProject() so its per-project load (Task 7) picks
        // this up instead of falling back to the "name"/"asc" default.
        try {
          localStorage.setItem(
            `lp-request-sort-${report.project_id}`,
            JSON.stringify({ field: "custom", dir: "asc" }),
          );
        } catch {}
        await selectProject(report.project_id);
      } else {
        await selectProject(selectedProjectId);
      }
      collectionImportText = "";
      await syncNewProjectIntoWorkspaceRepo();
    } catch (err) {
      collectionImportError = describeError(err);
    } finally {
      collectionImportLoading = false;
    }
  }

  async function exportPostmanCollectionAction(projectId?: string) {
    const targetId = projectId ?? selectedProjectId;
    if (!targetId) return;
    try {
      const json = await api.exportPostmanCollection(targetId);
      const proj = projects.find((p) => p.id === targetId);
      const safeName = (proj?.name || "collection").replace(/[^a-z0-9_-]/gi, "_");
      downloadFile(json, `${safeName}.postman_collection.json`, "application/json");
      exportFeedback = "Exported Postman Collection!";
      setTimeout(() => (exportFeedback = ""), 3000);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function handleEnvironmentFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const reader = new FileReader();
      reader.onload = (e) => {
        environmentImportText = (e.target?.result as string) || "";
      };
      reader.readAsText(input.files[0]);
    }
  }

  async function importPostmanEnvironmentAction() {
    if (!environmentImportText.trim() || !selectedProjectId) return;
    environmentImportLoading = true;
    environmentImportError = "";
    environmentImportReport = null;
    try {
      const report = await api.importPostmanEnvironment(environmentImportText.trim(), selectedProjectId);
      environmentImportReport = report;
      await loadAllEnvironments();
      selectedEnvironmentId = report.environment_id;
      await loadVariables();
      environmentImportText = "";
    } catch (err) {
      environmentImportError = describeError(err);
    } finally {
      environmentImportLoading = false;
    }
  }

  async function exportPostmanEnvironmentAction() {
    if (!selectedEnvironmentId) return;
    try {
      const json = await api.exportPostmanEnvironment(selectedEnvironmentId);
      const env = allEnvironments.find((e) => e.id === selectedEnvironmentId);
      const safeName = (env?.name || "environment").replace(/[^a-z0-9_-]/gi, "_");
      downloadFile(json, `${safeName}.postman_environment.json`, "application/json");
      exportFeedback = "Exported Postman Environment!";
      setTimeout(() => (exportFeedback = ""), 3000);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function downloadFile(content: string, filename: string, type: string) {
    const blob = new Blob([content], { type: `${type};charset=utf-8` });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function growHeaders() {
    editHeaders = withTrailingEmptyRow(editHeaders, () => ({ key: "", value: "", enabled: true, description: "" }));
  }

  function removeHeader(index: number) {
    editHeaders = withTrailingEmptyRow(editHeaders.filter((_, i) => i !== index), () => ({ key: "", value: "", enabled: true, description: "" }));
  }

  function growQueryParams() {
    editQueryParams = withTrailingEmptyRow(editQueryParams, () => ({ key: "", value: "", enabled: true }));
  }

  function removeQueryParam(index: number) {
    editQueryParams = withTrailingEmptyRow(editQueryParams.filter((_, i) => i !== index), () => ({ key: "", value: "", enabled: true }));
  }

  async function loadVariables() {
    try {
      if (selectedProjectId) {
        projectVariables = await api.listVariablesForScope("global", selectedProjectId);
      } else {
        projectVariables = [];
      }
      const envIdToLoad = (currentTab?.tabType === "env" ? (currentTab.envId ?? currentTab.id) : selectedEnvironmentId);
      if (envIdToLoad) {
        environmentVariables = await api.listVariablesForScope("environment", envIdToLoad);
      } else {
        environmentVariables = [];
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  $effect(() => {
    const tab = currentTab;
    if (tab?.tabType === "env") {
      const envId = tab.envId ?? tab.id;
      if (selectedEnvironmentId !== envId) {
        selectedEnvironmentId = envId;
      }
      loadVariables();
    }
  });

  async function commitNewGlobalVar() {
    const key = newGlobalVarDraft.key.trim();
    if (!selectedProjectId || !key) return;
    try {
      await api.createVariable({
        scope: "global",
        project_id: selectedProjectId,
        key,
        value: newGlobalVarDraft.value,
        is_secret: newGlobalVarDraft.isSecret,
        is_local: newGlobalVarDraft.isLocal,
        enabled: true,
      });
      newGlobalVarDraft = emptyVarDraft();
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function commitNewEnvVar() {
    const key = newEnvVarDraft.key.trim();
    const envId = (currentTab?.tabType === "env" ? (currentTab.envId ?? currentTab.id) : selectedEnvironmentId);
    if (!envId || !key) return;
    try {
      await api.createVariable({
        scope: "environment",
        environment_id: envId,
        key,
        value: newEnvVarDraft.value,
        is_secret: newEnvVarDraft.isSecret,
        is_local: newEnvVarDraft.isLocal,
        enabled: true,
      });
      newEnvVarDraft = emptyVarDraft();
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function toggleVariableEnabled(v: VariableView) {
    try {
      await api.updateVariable({ id: v.id, enabled: !v.enabled });
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function toggleVariableSecret(v: VariableView) {
    try {
      await api.updateVariable({ id: v.id, is_secret: !v.is_secret });
      await loadVariables();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function toggleVariableLocal(v: VariableView) {
    try {
      await api.updateVariable({ id: v.id, is_local: !v.is_local });
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function updateVariableValue(v: VariableView, value: string) {
    if (v.value === value) return;
    try {
      await api.updateVariable({ id: v.id, value });
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function updateVariableKey(v: VariableView, key: string) {
    if (v.key === key) return;
    try {
      await api.updateVariable({ id: v.id, key });
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function refreshDiagnostics() {
    if (!selectedRequest) {
      requestDiagnostics = null;
      return;
    }
    try {
      requestDiagnostics = await api.diagnoseRequest(selectedRequest.id, selectedEnvironmentId);
    } catch {
      requestDiagnostics = null;
    }
  }

  // Fills in an unresolved {{variable}} straight from the "Unresolved variables" warning —
  // no trip to the Environments screen needed. Lands in the active environment if one is
  // selected (matching what actually resolved it), otherwise the project's global scope.
  // Every diagnostic that depends on variables (the warning banner, tab badges, URL preview)
  // is re-fetched right after, so the fix is reflected everywhere immediately.
  async function addMissingVariable(name: string) {
    if (!selectedProjectId) return;
    const value = (missingVarDrafts[name] ?? "").trim();
    if (!value) return;
    try {
      if (selectedEnvironmentId) {
        await api.createVariable({
          scope: "environment",
          environment_id: selectedEnvironmentId,
          key: name,
          value,
          is_secret: false,
          is_local: false,
          enabled: true,
        });
      } else {
        await api.createVariable({
          scope: "global",
          project_id: selectedProjectId,
          key: name,
          value,
          is_secret: false,
          is_local: false,
          enabled: true,
        });
      }
      delete missingVarDrafts[name];
      if (missingVarHover?.name === name) missingVarHover = null;
      await loadVariables();
      await refreshDiagnostics();
      await refreshUrlPreview();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Shared hover-to-add-value popover for missing variables, used both by the "Unresolved
  // variables" warning banner and by {{tokens}} highlighted inline in the URL bar. Positioned
  // via JS (not CSS :hover) and rendered as a top-level portal — a plain CSS :hover popover
  // nested inside the URL bar gets clipped, since the bar also needs overflow-x clipping for
  // long URLs and CSS doesn't allow "clip X, don't clip Y" (a non-"visible" axis paired with
  // "visible" silently becomes "auto", which still clips).
  //
  // Extended (LP-1201) to work for ALREADY-resolved variables too, not just missing ones — same
  // popover, same position/hide logic, just prefilled with the live value and wired to update
  // instead of create. Only searches the two scopes the request-editing surface already knows
  // about (environment, then global) — the same scopes addMissingVariable can create into, so
  // this doesn't reach further than what was already editable elsewhere.
  let missingVarHover = $state<{
    name: string;
    top: number;
    left: number;
    placeAbove?: boolean;
  } | null>(null);
  let hoveredVarRect = $state<{
    top: number;
    bottom: number;
    left: number;
    right: number;
  } | null>(null);
  let missingVarHoverHideTimer: ReturnType<typeof setTimeout> | null = null;
  let popoverInputFocused = $state(false);
  let popoverSaveSuccess = $state(false);
  let hoveredVarElement: HTMLElement | null = null;

  let measureCanvas: HTMLCanvasElement | null = null;
  let measureCtx: CanvasRenderingContext2D | null = null;

  function getMeasureContext(): CanvasRenderingContext2D | null {
    if (!measureCtx && typeof document !== "undefined") {
      measureCanvas = document.createElement("canvas");
      measureCtx = measureCanvas.getContext("2d");
    }
    return measureCtx;
  }

  function findResolvedVariable(name: string): VariableView | undefined {
    return (
      environmentVariables.find((v) => v.key === name) ??
      projectVariables.find((v) => v.key === name)
    );
  }

  function showVarPopover(
    name: string,
    target: HTMLElement | { top: number; bottom: number; left: number; right: number }
  ) {
    if (missingVarDrafts[name] === undefined) {
      const resolved = findResolvedVariable(name);
      if (resolved) missingVarDrafts[name] = resolved.value;
    }
    showMissingVarPopover(name, target);
  }

  async function saveVariableFromPopover(name: string) {
    const resolved = findResolvedVariable(name);
    if (!resolved) {
      await addMissingVariable(name);
      popoverSaveSuccess = true;
      setTimeout(() => {
        popoverSaveSuccess = false;
        missingVarHover = null;
        hoveredVarRect = null;
      }, 600);
      return;
    }
    const value = missingVarDrafts[name] ?? "";
    try {
      await api.updateVariable({ id: resolved.id, value });
      popoverSaveSuccess = true;
      await loadVariables();
      await refreshDiagnostics();
      await refreshUrlPreview();
      if (rightSidebarVisible && rightPanel !== "info" && selectedRequest) {
        copyAsCurl();
      }
      setTimeout(() => {
        popoverSaveSuccess = false;
        missingVarHover = null;
        hoveredVarRect = null;
      }, 600);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function showMissingVarPopover(
    name: string,
    target: HTMLElement | { top: number; bottom: number; left: number; right: number }
  ) {
    if (missingVarHoverHideTimer) {
      clearTimeout(missingVarHoverHideTimer);
      missingVarHoverHideTimer = null;
    }
    const isElem = typeof (target as HTMLElement).getBoundingClientRect === "function";
    const r = isElem ? (target as HTMLElement).getBoundingClientRect() : (target as { top: number; bottom: number; left: number; right?: number });
    const popoverWidth = 320;
    const popoverHeight = 175;

    let left = Math.round(r.left);
    if (typeof window !== "undefined") {
      if (left + popoverWidth > window.innerWidth - 16) {
        left = Math.max(16, window.innerWidth - popoverWidth - 16);
      }
      if (left < 16) left = 16;
    }

    let top = Math.round(r.bottom + 6);
    let placeAbove = false;
    if (typeof window !== "undefined") {
      if (r.bottom + popoverHeight > window.innerHeight - 16 && r.top - popoverHeight > 16) {
        top = Math.round(r.top - popoverHeight - 6);
        placeAbove = true;
      }
    }

    hoveredVarRect = {
      top: r.top,
      bottom: r.bottom,
      left: r.left,
      right: r.right !== undefined ? r.right : r.left + 60,
    };
    missingVarHover = { name, top, left, placeAbove };
  }

  function scheduleHideMissingVarPopover(delay = 250) {
    if (popoverInputFocused) return;
    if (missingVarHoverHideTimer) clearTimeout(missingVarHoverHideTimer);
    missingVarHoverHideTimer = setTimeout(() => {
      if (!popoverInputFocused) {
        missingVarHover = null;
        hoveredVarRect = null;
        if (hoveredVarElement) {
          hoveredVarElement.style.cursor = "";
          hoveredVarElement = null;
        }
      }
      missingVarHoverHideTimer = null;
    }, delay);
  }

  function cancelHideMissingVarPopover() {
    if (missingVarHoverHideTimer) {
      clearTimeout(missingVarHoverHideTimer);
      missingVarHoverHideTimer = null;
    }
  }

  function detectVarInMonospaceTextarea(
    textarea: HTMLTextAreaElement,
    clientX: number,
    clientY: number
  ): { name: string; rect: { top: number; bottom: number; left: number; right: number } } | null {
    const text = textarea.value;
    if (!text || !text.includes("{{")) return null;

    const rect = textarea.getBoundingClientRect();
    if (
      clientX < rect.left ||
      clientX > rect.right ||
      clientY < rect.top ||
      clientY > rect.bottom
    ) {
      return null;
    }

    const style = window.getComputedStyle(textarea);
    const paddingTop = parseFloat(style.paddingTop) || 0;
    const paddingLeft = parseFloat(style.paddingLeft) || 0;
    const borderTop = parseFloat(style.borderTopWidth) || 0;
    const borderLeft = parseFloat(style.borderLeftWidth) || 0;
    let lineHeight = parseFloat(style.lineHeight);
    if (isNaN(lineHeight) || lineHeight <= 0) {
      lineHeight = (parseFloat(style.fontSize) || 12) * 1.5;
    }

    const ctx = getMeasureContext();
    if (ctx) {
      ctx.font = `${style.fontWeight || "normal"} ${style.fontSize || "12px"} ${style.fontFamily || "monospace"}`;
    }

    const relX = clientX - rect.left - borderLeft - paddingLeft + textarea.scrollLeft;
    const relY = clientY - rect.top - borderTop - paddingTop + textarea.scrollTop;

    if (relY < 0 || relX < 0) return null;

    const lineIdx = Math.floor(relY / lineHeight);
    const lines = text.split("\n");
    if (lineIdx < 0 || lineIdx >= lines.length) return null;

    const line = lines[lineIdx];
    if (!line.includes("{{")) return null;

    const regex = /\{\{([^}]+)\}\}/g;
    let match: RegExpExecArray | null;

    while ((match = regex.exec(line)) !== null) {
      const varName = match[1].trim();
      let startX = 0;
      let endX = 0;

      if (ctx) {
        startX = ctx.measureText(line.slice(0, match.index)).width;
        endX = startX + ctx.measureText(match[0]).width;
      } else {
        startX = match.index * 7.2;
        endX = (match.index + match[0].length) * 7.2;
      }

      if (relX >= startX - 4 && relX <= endX + 4) {
        const tokenScreenLeft = rect.left + borderLeft + paddingLeft + startX - textarea.scrollLeft;
        const tokenScreenTop = rect.top + borderTop + paddingTop + lineIdx * lineHeight - textarea.scrollTop;
        return {
          name: varName,
          rect: {
            top: tokenScreenTop,
            bottom: tokenScreenTop + lineHeight,
            left: tokenScreenLeft,
            right: tokenScreenLeft + (endX - startX),
          },
        };
      }
    }
    return null;
  }

  function detectVarInInput(
    input: HTMLInputElement,
    clientX: number,
    clientY: number
  ): { name: string; rect: { top: number; bottom: number; left: number; right: number } } | null {
    const val = input.value;
    if (!val || !val.includes("{{")) return null;

    const rect = input.getBoundingClientRect();
    if (
      clientX < rect.left ||
      clientX > rect.right ||
      clientY < rect.top ||
      clientY > rect.bottom
    ) {
      return null;
    }

    const style = window.getComputedStyle(input);
    const paddingLeft = parseFloat(style.paddingLeft) || 0;
    const borderLeft = parseFloat(style.borderLeftWidth) || 0;

    const ctx = getMeasureContext();
    if (ctx) {
      ctx.font = `${style.fontWeight || "normal"} ${style.fontSize || "12px"} ${style.fontFamily || "sans-serif"}`;
    }

    const relX = clientX - rect.left - borderLeft - paddingLeft + input.scrollLeft;
    const regex = /\{\{([^}]+)\}\}/g;
    let match: RegExpExecArray | null;

    while ((match = regex.exec(val)) !== null) {
      const varName = match[1].trim();
      let startX = 0;
      let endX = 0;

      if (ctx) {
        startX = ctx.measureText(val.slice(0, match.index)).width;
        endX = startX + ctx.measureText(match[0]).width;
      } else {
        startX = match.index * 7.5;
        endX = (match.index + match[0].length) * 7.5;
      }

      if (relX >= startX - 4 && relX <= endX + 4) {
        const tokenScreenLeft = rect.left + borderLeft + paddingLeft + startX - input.scrollLeft;
        return {
          name: varName,
          rect: {
            top: rect.top,
            bottom: rect.bottom,
            left: tokenScreenLeft,
            right: tokenScreenLeft + (endX - startX),
          },
        };
      }
    }
    return null;
  }

  function handleBodyMouseMove(e: MouseEvent) {
    const textarea = e.currentTarget as HTMLTextAreaElement;
    if (!textarea || !editBody || !editBody.includes("{{")) {
      if (hoveredVarElement === textarea) {
        textarea.style.cursor = "";
        hoveredVarElement = null;
        scheduleHideMissingVarPopover();
      }
      return;
    }

    const hit = detectVarInMonospaceTextarea(textarea, e.clientX, e.clientY);
    if (hit) {
      textarea.style.cursor = "pointer";
      hoveredVarElement = textarea;
      showVarPopover(hit.name, hit.rect);
    } else {
      if (hoveredVarElement === textarea) {
        textarea.style.cursor = "";
        hoveredVarElement = null;
        scheduleHideMissingVarPopover();
      }
    }
  }

  function handleBodyMouseLeave(e: MouseEvent) {
    const textarea = e.currentTarget as HTMLTextAreaElement;
    if (textarea) {
      textarea.style.cursor = "";
      if (hoveredVarElement === textarea) hoveredVarElement = null;
    }
    scheduleHideMissingVarPopover();
  }

  function handleGenericInputMouseMove(e: MouseEvent) {
    const input = e.currentTarget as HTMLInputElement;
    if (!input || !input.value || !input.value.includes("{{")) {
      if (hoveredVarElement === input) {
        input.style.cursor = "";
        hoveredVarElement = null;
        scheduleHideMissingVarPopover();
      }
      return;
    }

    const hit = detectVarInInput(input, e.clientX, e.clientY);
    if (hit) {
      input.style.cursor = "pointer";
      hoveredVarElement = input;
      showVarPopover(hit.name, hit.rect);
    } else {
      if (hoveredVarElement === input) {
        input.style.cursor = "";
        hoveredVarElement = null;
        scheduleHideMissingVarPopover();
      }
    }
  }

  function handleGenericInputMouseLeave(e: MouseEvent) {
    const input = e.currentTarget as HTMLInputElement;
    if (input) {
      input.style.cursor = "";
      if (hoveredVarElement === input) hoveredVarElement = null;
    }
    scheduleHideMissingVarPopover();
  }

  function handlePaneMouseMoveDelegated(e: MouseEvent) {
    const target = e.target as HTMLElement | null;
    if (!target) return;
    if (target.closest(".postman-var-popover") || target.closest(".missing-var-popover-portal")) return;

    if (target instanceof HTMLInputElement) {
      if (target.value && target.value.includes("{{")) {
        const hit = detectVarInInput(target, e.clientX, e.clientY);
        if (hit) {
          target.style.cursor = "pointer";
          hoveredVarElement = target;
          showVarPopover(hit.name, hit.rect);
          return;
        }
      }
    } else if (target instanceof HTMLTextAreaElement) {
      if (target.value && target.value.includes("{{")) {
        const hit = detectVarInMonospaceTextarea(target, e.clientX, e.clientY);
        if (hit) {
          target.style.cursor = "pointer";
          hoveredVarElement = target;
          showVarPopover(hit.name, hit.rect);
          return;
        }
      }
    }

    if (hoveredVarElement) {
      hoveredVarElement.style.cursor = "";
      hoveredVarElement = null;
      scheduleHideMissingVarPopover();
    }
  }

  // Inline autocomplete (LP-1401) for two contexts that share one floating dropdown:
  // "var" mode suggests {{variable}} names while inside an unclosed {{ in any text field,
  // "pm" mode suggests pm.*/console.* API members while typing in a pre/post-request script
  // textarea. Attached generically via oninput/onkeydown so no per-field wiring is needed beyond
  // passing a `setValue` closure that writes back into that field's own bound state.
  
  
  let autocomplete = $state<AutocompleteState | null>(null);

  // Mirrors the textarea/input's text into an offscreen div with identical font metrics so we can
  // read where a given character index actually lands on screen — there is no DOM API that maps a
  // string index to pixel coordinates directly, so the standard workaround is to lay out the same
  // text a second time and measure it.
  const CARET_MIRROR_PROPERTIES = [
    "boxSizing", "width", "height", "overflowX", "overflowY",
    "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth", "borderStyle",
    "paddingTop", "paddingRight", "paddingBottom", "paddingLeft",
    "fontStyle", "fontVariant", "fontWeight", "fontStretch", "fontSize", "fontFamily",
    "lineHeight", "textAlign", "textTransform", "textIndent", "letterSpacing", "wordSpacing",
    "tabSize", "whiteSpace", "wordBreak",
  ] as const;

  function caretScreenPosition(el: HTMLInputElement | HTMLTextAreaElement, index: number): { top: number; left: number; height: number } {
    const isInput = el.tagName === "INPUT";
    const style = getComputedStyle(el);
    const div = document.createElement("div");
    const divStyle = div.style as CSSStyleDeclaration & Record<string, string>;
    const computedStyle = style as CSSStyleDeclaration & Record<string, string>;
    for (const prop of CARET_MIRROR_PROPERTIES) {
      divStyle[prop] = computedStyle[prop];
    }
    div.style.position = "absolute";
    div.style.visibility = "hidden";
    div.style.whiteSpace = isInput ? "pre" : "pre-wrap";
    div.style.wordWrap = "break-word";
    document.body.appendChild(div);
    const before = el.value.slice(0, index);
    div.textContent = isInput ? before.replace(/ /g, " ") : before;
    const marker = document.createElement("span");
    marker.textContent = el.value.slice(index) || ".";
    div.appendChild(marker);
    const rect = el.getBoundingClientRect();
    const lineHeight = parseInt(style.lineHeight, 10) || 18;
    const top = rect.top + marker.offsetTop - el.scrollTop;
    const left = rect.left + marker.offsetLeft - el.scrollLeft;
    document.body.removeChild(div);
    return { top, left, height: lineHeight };
  }

  function variableAutocompleteQuery(text: string, caret: number): { query: string; start: number } | null {
    const upto = text.slice(0, caret);
    const openIdx = upto.lastIndexOf("{{");
    if (openIdx === -1) return null;
    const closeIdx = upto.indexOf("}}", openIdx);
    if (closeIdx !== -1) return null;
    const inner = upto.slice(openIdx + 2);
    if (/[{}\s]/.test(inner)) return null;
    return { query: inner, start: openIdx + 2 };
  }

  function variableSuggestionItems(query: string): AutocompleteItem[] {
    const q = query.toLowerCase();
    const seen = new Set<string>();
    const items: AutocompleteItem[] = [];
    for (const v of environmentVariables) {
      if (seen.has(v.key) || (q && !v.key.toLowerCase().includes(q))) continue;
      seen.add(v.key);
      items.push({ insertText: v.key, label: v.key, detail: t("var.scopeEnvironment") });
    }
    for (const v of projectVariables) {
      if (seen.has(v.key) || (q && !v.key.toLowerCase().includes(q))) continue;
      seen.add(v.key);
      items.push({ insertText: v.key, label: v.key, detail: t("var.scopeGlobal") });
    }
    return items.slice(0, 20);
  }

  function scriptAutocompleteQuery(text: string, caret: number): { query: string; start: number } | null {
    const upto = text.slice(0, caret);
    const match = /[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*$/.exec(upto);
    if (!match) return null;
    const word = match[0];
    if (word.length < 2) return null;
    const prefixes = ["pm", "console", "CryptoJS"];
    if (!prefixes.some((p) => word.startsWith(p) || p.startsWith(word))) {
      return null;
    }
    return { query: word, start: caret - word.length };
  }

  function pmApiSuggestions(includeResponse: boolean): AutocompleteItem[] {
    const base: { insertText: string; label: string; detailKey: string }[] = [
      { insertText: "pm.environment.get(", label: "pm.environment.get(key)", detailKey: "autocomplete.pmEnvGet" },
      { insertText: "pm.environment.set(", label: "pm.environment.set(key, value)", detailKey: "autocomplete.pmEnvSet" },
      { insertText: "pm.environment.unset(", label: "pm.environment.unset(key)", detailKey: "autocomplete.pmEnvUnset" },
      { insertText: "pm.environment.has(", label: "pm.environment.has(key)", detailKey: "autocomplete.pmEnvHas" },
      { insertText: "pm.variables.get(", label: "pm.variables.get(key)", detailKey: "autocomplete.pmVarGet" },
      { insertText: "pm.variables.set(", label: "pm.variables.set(key, value)", detailKey: "autocomplete.pmVarSet" },
      { insertText: "pm.test(", label: "pm.test(name, fn)", detailKey: "autocomplete.pmTest" },
      { insertText: "console.log(", label: "console.log(...)", detailKey: "autocomplete.consoleLog" },
      { insertText: "console.info(", label: "console.info(...)", detailKey: "autocomplete.consoleInfo" },
      { insertText: "console.warn(", label: "console.warn(...)", detailKey: "autocomplete.consoleWarn" },
      { insertText: "console.error(", label: "console.error(...)", detailKey: "autocomplete.consoleError" },
    ];
    const responseOnly: { insertText: string; label: string; detailKey: string }[] = [
      { insertText: "pm.response.code", label: "pm.response.code", detailKey: "autocomplete.pmResCode" },
      { insertText: "pm.response.status", label: "pm.response.status", detailKey: "autocomplete.pmResStatus" },
      { insertText: "pm.response.statusText", label: "pm.response.statusText", detailKey: "autocomplete.pmResStatusText" },
      { insertText: "pm.response.headers", label: "pm.response.headers", detailKey: "autocomplete.pmResHeaders" },
      { insertText: "pm.response.text()", label: "pm.response.text()", detailKey: "autocomplete.pmResText" },
      { insertText: "pm.response.json()", label: "pm.response.json()", detailKey: "autocomplete.pmResJson" },
      { insertText: "pm.response.to.have.status(", label: "pm.response.to.have.status(code)", detailKey: "autocomplete.pmResToHaveStatus" },
      { insertText: "pm.response.to.have.header(", label: "pm.response.to.have.header(key, value?)", detailKey: "autocomplete.pmResToHaveHeader" },
    ];
    // pm.request only exists in pre-request scripts (LP-1406) — the request hasn't been sent
    // yet in that context, so it's the mirror image of pm.response being post-request-only.
    const requestOnly: { insertText: string; label: string; detailKey: string }[] = [
      { insertText: "pm.request.headers.add(", label: "pm.request.headers.add({key, value})", detailKey: "autocomplete.pmReqHeadersAdd" },
      { insertText: "pm.request.headers.upsert(", label: "pm.request.headers.upsert({key, value})", detailKey: "autocomplete.pmReqHeadersUpsert" },
      { insertText: "pm.request.headers.remove(", label: "pm.request.headers.remove(key)", detailKey: "autocomplete.pmReqHeadersRemove" },
      { insertText: "pm.request.headers.has(", label: "pm.request.headers.has(key)", detailKey: "autocomplete.pmReqHeadersHas" },
      { insertText: "pm.request.headers.get(", label: "pm.request.headers.get(key)", detailKey: "autocomplete.pmReqHeadersGet" },
      { insertText: "pm.request.body.toString()", label: "pm.request.body.toString()", detailKey: "autocomplete.pmReqBodyToString" },
      { insertText: "pm.request.body.update(", label: "pm.request.body.update(newBody)", detailKey: "autocomplete.pmReqBodyUpdate" },
      { insertText: "pm.request.url.toString()", label: "pm.request.url.toString()", detailKey: "autocomplete.pmReqUrl" },
      { insertText: "pm.request.method", label: "pm.request.method", detailKey: "autocomplete.pmReqMethod" },
      { insertText: "CryptoJS.HmacSHA256(", label: "CryptoJS.HmacSHA256(message, key)", detailKey: "autocomplete.cryptoHmacSha256" },
      { insertText: "CryptoJS.HmacSHA1(", label: "CryptoJS.HmacSHA1(message, key)", detailKey: "autocomplete.cryptoHmacSha1" },
      { insertText: "CryptoJS.SHA256(", label: "CryptoJS.SHA256(message)", detailKey: "autocomplete.cryptoSha256" },
      { insertText: "CryptoJS.SHA1(", label: "CryptoJS.SHA1(message)", detailKey: "autocomplete.cryptoSha1" },
      { insertText: "CryptoJS.enc.Base64.stringify(", label: "CryptoJS.enc.Base64.stringify(wordArray)", detailKey: "autocomplete.cryptoBase64Stringify" },
      { insertText: "CryptoJS.enc.Hex.stringify(", label: "CryptoJS.enc.Hex.stringify(wordArray)", detailKey: "autocomplete.cryptoHexStringify" },
    ];
    const all = includeResponse ? [...base, ...responseOnly] : [...base, ...requestOnly];
    return all.map((s) => ({ insertText: s.insertText, label: s.label, detail: t(s.detailKey) }));
  }

  function pmSuggestionItems(query: string, includeResponse: boolean): AutocompleteItem[] {
    const q = query.toLowerCase();
    return pmApiSuggestions(includeResponse)
      .filter((s) => s.insertText.toLowerCase().startsWith(q) || s.label.toLowerCase().startsWith(q))
      .slice(0, 20);
  }

  // A curated set of headers actually useful when testing an API — not the full IANA registry,
  // which would mostly just add noise (browser-only headers like Sec-Fetch-*, response-only
  // headers like ETag, etc. don't belong in a request's own header list).
  const STANDARD_REQUEST_HEADERS = [
    "Accept", "Accept-Charset", "Accept-Encoding", "Accept-Language",
    "Authorization", "Cache-Control", "Connection", "Content-Disposition",
    "Content-Length", "Content-Type", "Cookie", "DNT", "Expect", "Forwarded",
    "Host", "If-Match", "If-Modified-Since", "If-None-Match", "If-Unmodified-Since",
    "Origin", "Pragma", "Range", "Referer", "TE", "User-Agent",
    "Upgrade-Insecure-Requests", "Warning", "X-Api-Key", "X-CSRF-Token",
    "X-Forwarded-For", "X-Forwarded-Host", "X-Forwarded-Proto", "X-Request-ID",
    "X-Requested-With",
  ];

  function headerNameQuery(text: string): { query: string; start: number } | null {
    if (!text) return null;
    return { query: text, start: 0 };
  }

  function headerSuggestionItems(query: string): AutocompleteItem[] {
    const q = query.toLowerCase();
    return STANDARD_REQUEST_HEADERS.filter((h) => h.toLowerCase().startsWith(q) && h.toLowerCase() !== q)
      .slice(0, 20)
      .map((h) => ({ insertText: h, label: h }));
  }

  function updateAutocompleteFor(
    el: HTMLInputElement | HTMLTextAreaElement,
    mode: "var" | "pm" | "header",
    includeResponse: boolean,
    setValue: (value: string) => void,
  ) {
    const caret = el.selectionStart ?? el.value.length;
    const hit =
      mode === "var"
        ? variableAutocompleteQuery(el.value, caret)
        : mode === "pm"
          ? scriptAutocompleteQuery(el.value, caret)
          : headerNameQuery(el.value);
    if (!hit) {
      autocomplete = null;
      return;
    }
    const items =
      mode === "var"
        ? variableSuggestionItems(hit.query)
        : mode === "pm"
          ? pmSuggestionItems(hit.query, includeResponse)
          : headerSuggestionItems(hit.query);
    if (!items.length) {
      autocomplete = null;
      return;
    }
    // Header-name mode always replaces the whole field, regardless of caret position.
    const replaceEnd = mode === "header" ? el.value.length : caret;
    const pos = caretScreenPosition(el, hit.start);
    autocomplete = {
      mode,
      items,
      activeIndex: 0,
      top: pos.top + pos.height + 2,
      left: pos.left,
      targetEl: el,
      replaceStart: hit.start,
      replaceEnd,
      setValue,
    };
  }

  function handleAutocompleteKeydown(e: KeyboardEvent) {
    if (!autocomplete) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      autocomplete = { ...autocomplete, activeIndex: (autocomplete.activeIndex + 1) % autocomplete.items.length };
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      autocomplete = { ...autocomplete, activeIndex: (autocomplete.activeIndex - 1 + autocomplete.items.length) % autocomplete.items.length };
    } else if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      applyAutocompleteItem(autocomplete.items[autocomplete.activeIndex]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      autocomplete = null;
    }
  }

  function applyAutocompleteItem(item: AutocompleteItem) {
    const ac = autocomplete;
    if (!ac) return;
    const { targetEl, replaceStart, replaceEnd, mode, setValue } = ac;
    const value = targetEl.value;
    const before = value.slice(0, replaceStart);
    const after = value.slice(replaceEnd);
    const hasClosing = mode === "var" && after.startsWith("}}");
    const insertText = mode === "var" && !hasClosing ? item.insertText + "}}" : item.insertText;
    const newValue = before + insertText + after;
    const newCaret = before.length + insertText.length + (hasClosing ? 2 : 0);
    setValue(newValue);
    autocomplete = null;
    tick().then(() => {
      targetEl.focus();
      targetEl.setSelectionRange(newCaret, newCaret);
    });
  }

  function hideAutocompleteSoon() {
    setTimeout(() => {
      autocomplete = null;
    }, 120);
  }

  function deleteVariable(id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteVariable"), async () => {
    try {
      await api.deleteVariable(id);
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  async function revealSecret(id: string) {
    try {
      const val = await api.revealVariableValue(id);
      revealedSecrets = { ...revealedSecrets, [id]: val };
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function copyResponseBody() {
    if (!activeResponseBody) return;
    try {
      await navigator.clipboard.writeText(activeResponseBody);
      copyFeedback = t("copy.copied");
      setTimeout(() => (copyFeedback = ""), 2000);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function downloadResponseBody() {
    if (!activeResponseBody || !activeResponse) return;
    const blob = new Blob([activeResponseBody], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `response-${activeResponse.status}-${activeResponse.id.slice(0, 8)}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function deleteRequest(id: string) {
    showConfirm(t("common.confirm"), t("error.confirmDeleteRequest"), async () => {
    try {
      await api.deleteRequest(id);
      requests = requests.filter((r) => r.id !== id);
      closeTab(id);
    } catch (err) {
      errorMessage = describeError(err);
    }
    });
  }

  // Full copy — headers, params, auth, body, description and scripts all carry over, not just
  // method/URL, so "Duplicate" is a real starting point for a variant request, not a blank one.
  async function duplicateRequest(id: string) {
    try {
      const source = await api.getRequest(id);
      const created = await api.createRequest({
        project_id: source.project_id,
        folder_id: source.folder_id,
        name: t("sidebar.copyOf", { name: source.name }),
        method: source.method,
        url: source.url,
        headers: source.headers,
        query_params: source.query_params,
        auth: source.auth,
        body: source.body,
        description: source.description,
        settings: source.settings,
        pre_request_script: source.pre_request_script,
        post_request_script: source.post_request_script,
      });
      requests = [
        {
          id: created.id,
          project_id: created.project_id,
          folder_id: created.folder_id,
          name: created.name,
          method: created.method,
          url: created.url,
          sort_order: created.sort_order,
          created_at: created.created_at,
          updated_at: created.updated_at,
        },
        ...requests,
      ];
      await openRequest(created.id);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // --- Git & Collaboration Functions (LP-0701 - LP-0713) ---
  // ---- Workspace invites -------------------------------------------------------------------
  // "Invite" adds a GitHub user to the workspace's sync repository (viewer = read-only,
  // collaborator = read + write). The repo and token come from the Git panel's saved settings.
  let inviteRepoLabel = $derived.by(() => {
    const m = /github\.com[/:]([^/]+)\/([^/]+?)(?:\.git)?\/?$/i.exec(gitSettings?.remote_url ?? "");
    return m ? `${m[1]}/${m[2]}` : "";
  });
  // Why an invite can't be sent right now (null = ready to send).
  let inviteBlocker = $derived.by((): string | null => {
    if (!activeWorkspaceId) return t("invite.noWorkspace");
    if (!gitSettings?.remote_url?.trim()) return t("invite.noRemote");
    if (!inviteRepoLabel) return t("invite.notGithub");
    if (!gitSettings.github_token?.trim()) return t("invite.noToken");
    return null;
  });

  async function refreshCollaborators() {
    collaborators = [];
    if (inviteBlocker || !activeWorkspaceId) return;
    collaboratorsLoading = true;
    try {
      collaborators = await api.listWorkspaceCollaborators(activeWorkspaceId);
    } catch (err) {
      inviteError = describeError(err);
    } finally {
      collaboratorsLoading = false;
    }
  }

  async function openInviteModal() {
    inviteError = "";
    inviteNotice = "";
    inviteUsername = "";
    showInviteModal = true;
    await loadGitSettings();
    await refreshCollaborators();
  }

  async function sendInvite() {
    if (!activeWorkspaceId || inviteBusy) return;
    inviteError = "";
    inviteNotice = "";
    const username = inviteUsername.trim();
    if (!username) {
      inviteError = t("invite.usernameRequired");
      return;
    }
    inviteBusy = true;
    try {
      const result = await api.inviteWorkspaceCollaborator(activeWorkspaceId, username, inviteRole);
      const role = t(`invite.role.${inviteRole}.short`);
      inviteNotice = t(result.status === "invited" ? "invite.sentPending" : "invite.updated", { user: result.username, role });
      inviteUsername = "";
      await refreshCollaborators();
    } catch (err) {
      inviteError = describeError(err);
    } finally {
      inviteBusy = false;
    }
  }

  // ---- Notifications -----------------------------------------------------------------------
  // Derived from real app state (Git sync, console, AI setup) instead of static placeholder rows.
  type AppNotification = { id: string; level: "info" | "warn" | "error"; title: string; detail: string; actionLabel?: string; action?: () => void };
  let notifications = $derived.by((): AppNotification[] => {
    const out: AppNotification[] = [];
    if (gitStatus?.has_conflicts) {
      out.push({ id: "git-conflict", level: "error", title: t("notifications.gitConflict"), detail: t("notifications.gitConflictDetail", { count: gitStatus.conflict_files.length }), actionLabel: t("notifications.openGit"), action: () => { activeScreen = "git"; } });
    } else if (gitStatus?.status_kind === "behind" || (gitStatus?.behind ?? 0) > 0) {
      out.push({ id: "git-behind", level: "warn", title: t("notifications.gitBehind"), detail: t("notifications.gitBehindDetail", { count: gitStatus?.behind ?? 0 }), actionLabel: t("notifications.openGit"), action: () => { activeScreen = "git"; } });
    } else if ((gitStatus?.ahead ?? 0) > 0 || gitStatus?.status_kind === "modified") {
      out.push({ id: "git-ahead", level: "info", title: t("notifications.gitAhead"), detail: t("notifications.gitAheadDetail"), actionLabel: t("notifications.openGit"), action: () => { activeScreen = "git"; } });
    }
    if (consoleErrorCount > 0) {
      out.push({ id: "console-errors", level: "error", title: t("notifications.consoleErrors", { count: consoleErrorCount }), detail: t("notifications.consoleErrorsDetail"), actionLabel: t("notifications.openConsole"), action: () => { showConsole = true; refreshConsoleEvents(); } });
    }
    if (!aiConfigured) {
      out.push({ id: "ai-setup", level: "info", title: t("notifications.aiSetup"), detail: t("notifications.aiSetupDetail"), actionLabel: t("topbar.setupAi"), action: () => { showAiPanel = true; } });
    }
    return out;
  });

  async function loadGitSettings() {
    if (!activeWorkspaceId) return;
    try {
      const s = await api.getWorkspaceGitSettings(activeWorkspaceId);
      gitSettings = s;
      if (s) {
        gitRepoPathInput = s.repo_path ?? "";
        gitRemoteUrlInput = s.remote_url ?? "";
        gitBranchInput = s.branch || "main";
        gitAutoSyncInput = s.auto_sync;
        githubTokenInput = s.github_token ?? "";
        if (s.repo_path) {
          await refreshGitStatus(s.repo_path);
        } else {
          gitStatus = null;
        }
        if (s.github_token) {
          verifyGitHubTokenAction(false);
        }
      } else {
        gitRepoPathInput = "";
        gitRemoteUrlInput = "";
        gitBranchInput = "main";
        gitAutoSyncInput = false;
        githubTokenInput = "";
        gitStatus = null;
        githubUser = null;
        githubRepoInfo = null;
      }
      // No workspace-level settings yet — offer any leftover per-project configs as options
      // rather than guessing; the user picks one (or dismisses and configures from scratch).
      legacyGitCandidates = s ? [] : await api.findLegacyGitSettingsForWorkspace(activeWorkspaceId);
    } catch (err) {
      console.error("Failed to load git settings:", err);
    }
  }

  async function refreshGitStatus(path?: string) {
    const dir = path ?? gitRepoPathInput.trim();
    if (!dir) return;
    gitStatusLoading = true;
    try {
      const status = await api.getGitStatus(dir);
      gitStatus = status;
      if (status.has_conflicts && status.conflict_files.length > 0) {
        gitActiveTab = "conflicts";
      }
    } catch (err) {
      gitStatus = null;
      console.error("Failed to refresh git status:", err);
    } finally {
      gitStatusLoading = false;
    }
  }

  async function saveGitSettingsAction() {
    if (!activeWorkspaceId) return;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      const settings: WorkspaceGitSettings = {
        workspace_id: activeWorkspaceId,
        repo_path: gitRepoPathInput.trim() || null,
        remote_url: gitRemoteUrlInput.trim() || null,
        branch: gitBranchInput.trim() || "main",
        auto_sync: gitAutoSyncInput,
        github_token: githubTokenInput.trim() || null,
        last_sync_at: gitSettings?.last_sync_at ?? null,
      };
      await api.saveWorkspaceGitSettings(settings);
      gitSettings = settings;
      legacyGitCandidates = [];
      gitActionFeedback = "Settings saved successfully.";
      if (settings.repo_path) {
        await refreshGitStatus(settings.repo_path);
      }
    } catch (err) {
      gitActionError = describeError(err);
    }
  }

  // The user picked one of the offered legacy per-project configs — fill the form with it and
  // save it as this workspace's own settings (a real save, not just a preview: they explicitly
  // chose it from the options).
  async function adoptLegacyGitSettings(candidate: LegacyGitSettingsCandidate) {
    gitRepoPathInput = candidate.settings.repo_path ?? "";
    gitRemoteUrlInput = candidate.settings.remote_url ?? "";
    gitBranchInput = candidate.settings.branch || "main";
    gitAutoSyncInput = candidate.settings.auto_sync;
    githubTokenInput = candidate.settings.github_token ?? "";
    await saveGitSettingsAction();
  }

  function dismissLegacyGitCandidates() {
    legacyGitCandidates = [];
  }

  async function initializeGitRepoAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir) {
      gitActionError = "Please specify a local repository directory.";
      return;
    }
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.gitInitRepository(dir);
      await saveGitSettingsAction();
      if (activeWorkspaceId) {
        await api.saveWorkspaceToRepo(activeWorkspaceId, dir, false);
      }
      await refreshGitStatus(dir);
      gitActionFeedback = "Git repository initialized successfully with .gitignore and every project's canonical file.";
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function saveWorkspaceToRepoAction() {
    if (!activeWorkspaceId || !gitRepoPathInput.trim()) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      const paths = await api.saveWorkspaceToRepo(activeWorkspaceId, gitRepoPathInput.trim(), false);
      await refreshGitStatus();
      gitActionFeedback = `Saved ${paths.length} project file${paths.length === 1 ? "" : "s"} to the repo.`;
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function commitAndPushAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir || !activeWorkspaceId) return;
    const msg = gitCommitMessage.trim() || "Update workspace from Light Postman";
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.saveWorkspaceToRepo(activeWorkspaceId, dir, false);
      const commitHash = await api.gitCommitChanges(dir, msg);
      gitCommitMessage = "";
      let pushed = false;
      if (gitRemoteUrlInput.trim()) {
        try {
          await api.gitPushRepository(dir, "origin", gitBranchInput.trim() || "main");
          pushed = true;
        } catch (pushErr) {
          gitActionFeedback = `Committed (${commitHash.slice(0, 7)}), but remote push failed: ${describeError(pushErr)}`;
          await refreshGitStatus();
          return;
        }
      }
      const now = new Date().toISOString();
      if (gitSettings) {
        gitSettings.last_sync_at = now;
        await api.saveWorkspaceGitSettings(gitSettings);
      }
      gitActionFeedback = pushed
        ? `Committed (${commitHash.slice(0, 7)}) and pushed to remote.`
        : `Committed (${commitHash.slice(0, 7)}) to local git repo.`;
      await refreshGitStatus();
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  function describeWorkspaceImportReport(report: WorkspaceImportReport): string {
    const parts = [];
    if (report.updated_projects.length) parts.push(`${report.updated_projects.length} updated`);
    if (report.created_projects.length) parts.push(`${report.created_projects.length} created`);
    if (!parts.length) parts.push("nothing to import");
    let msg = `Pulled from remote — ${parts.join(", ")}.`;
    if (report.warnings.length) msg += ` ${report.warnings.length} warning(s): ${report.warnings.join("; ")}`;
    return msg;
  }

  async function pullRepositoryAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir || !activeWorkspaceId) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.gitPullRepository(dir, "origin", gitBranchInput.trim() || "main");
      const report = await api.loadWorkspaceFromRepo(activeWorkspaceId, dir);
      await loadProjects();
      gitActionFeedback = describeWorkspaceImportReport(report);
      await refreshGitStatus();
    } catch (err) {
      gitActionError = describeError(err);
      await refreshGitStatus();
    } finally {
      gitLoading = false;
    }
  }

  async function viewDiffAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir) return;
    gitLoading = true;
    gitActionError = "";
    try {
      gitDiffContent = await api.gitGetDiff(dir);
      showDiffModal = true;
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function viewHistoryAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir) return;
    gitLoading = true;
    gitActionError = "";
    try {
      gitHistory = await api.gitGetLog(dir, 25);
      showHistoryModal = true;
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function resolveConflictAction(file: string, choice: string) {
    const dir = gitRepoPathInput.trim();
    if (!dir || !activeWorkspaceId) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.gitResolveConflict(dir, file, choice);
      gitActionFeedback = `Resolved conflict on '${file}' with choice '${choice}'.`;
      if (choice === "theirs" && file.includes("light-postman.json")) {
        await api.loadWorkspaceFromRepo(activeWorkspaceId, dir);
        await loadProjects();
      }
      await refreshGitStatus();
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function verifyGitHubTokenAction(showFeedback = true) {
    const token = githubTokenInput.trim();
    if (!token) return;
    githubValidating = true;
    if (showFeedback) {
      gitActionError = "";
      gitActionFeedback = "";
    }
    try {
      githubUser = await api.verifyGitHubToken(token);
      if (showFeedback) {
        gitActionFeedback = `Authenticated as @${githubUser.login}`;
      }
    } catch (err) {
      githubUser = null;
      if (showFeedback) {
        gitActionError = `GitHub token verification failed: ${describeError(err)}`;
      }
    } finally {
      githubValidating = false;
    }
  }

  async function checkGitHubRepoAction() {
    const token = githubTokenInput.trim();
    const url = gitRemoteUrlInput.trim();
    if (!token || !url) {
      gitActionError = "GitHub Token and Remote URL are required to verify repository permissions.";
      return;
    }
    githubValidating = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      const match = url.match(/github\.com[:/]([^/]+)\/([^/.]+)(?:\.git)?/);
      if (!match) {
        throw new Error("Could not parse owner/repo from Remote URL. Expected format: https://github.com/owner/repo.git");
      }
      const [, owner, repo] = match;
      githubRepoInfo = await api.getGitHubRepoInfo(token, owner, repo);
      gitActionFeedback = `Repository permissions verified for ${githubRepoInfo.full_name}.`;
    } catch (err) {
      githubRepoInfo = null;
      gitActionError = `Repository permission check failed: ${describeError(err)}`;
    } finally {
      githubValidating = false;
    }
  }

  async function exportProjectFileAction() {
    if (!selectedProjectId) return;
    try {
      projectFileJson = await api.exportProjectFile(selectedProjectId, !projectFileMaskSecrets);
      projectFileStatus = "Canonical project file generated.";
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function importProjectFileAction() {
    if (!projectFileJson.trim() || !activeWorkspaceId) return;
    try {
      const proj = await api.importProjectFile(projectFileJson, selectedProjectId, activeWorkspaceId);
      projectFileStatus = `Project '${proj.name}' imported successfully.`;
      await selectProject(proj.id);
      await syncNewProjectIntoWorkspaceRepo();
    } catch (err) {
      projectFileStatus = `Import failed: ${describeError(err)}`;
    }
  }

  function downloadProjectFile() {
    if (!projectFileJson || !selectedProjectId) return;
    const blob = new Blob([projectFileJson], { type: "application/json;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "light-postman.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  $effect(() => {
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }
    if (gitAutoSyncInput && gitRepoPathInput.trim() && activeWorkspaceId) {
      autoSyncTimer = setInterval(async () => {
        try {
          const status = await api.getGitStatus(gitRepoPathInput.trim());
          gitStatus = status;
          if (status.status_kind === "behind") {
            await pullRepositoryAction();
          } else if (status.status_kind === "ahead" || status.status_kind === "modified") {
            await commitAndPushAction();
          }
        } catch {
          // ignore auto-sync errors
        }
      }, autoSyncIntervalMs);
    }
    return () => {
      if (autoSyncTimer) {
        clearInterval(autoSyncTimer);
        autoSyncTimer = null;
      }
    };
  });

  let confirmDialog = $state<{
    show: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
  }>({
    show: false,
    title: "",
    message: "",
    onConfirm: () => {}
  });

  function showConfirm(title: string, message: string, onConfirm: () => void) {
    confirmDialog = { show: true, title, message, onConfirm };
  }

  return {
    sendAndDownload,
    duplicateEnvironment,
    exportPostmanEnvironmentAction,
    toggleDefaultEnvironment,
    navigateHistory,
    openInviteModal,
    sendInvite,
    refreshCollaborators,
    ACCENT_PRESETS,
    AI_API_KEY_PLACEHOLDER,
    AI_MODEL_SUGGESTIONS,
    AI_PROVIDERS,
    PROJECT_SORT_FIELDS,
    RAW_CONTENT_TYPES,
    REQUEST_SORT_FIELDS,
    SHORTCUT_DEFS,
    SURFACE_TINTS,
    THEME_FONT_OPTIONS,
    THEME_OPTIONS,
    accentStyleOverride,
    addAiPreviewToProject,
    adoptLegacyGitSettings,
    applyAutocompleteItem,
    asCookieRows,
    asHeaderRows,
    cancelCurrentSend,
    cancelHideMissingVarPopover,
    checkGitHubRepoAction,
    clearConsole,
    clearHistory,
    closePalette,
    closeRequestContextMenu,
    closeTabAction,
    closeTabContextMenu,
    commitAndPushAction,
    commitNewEnvVar,
    commitNewGlobalVar,
    copyAllAsCurlBashAction,
    copyAllAsCurlCmdAction,
    copyAllAsCurlPowerShellAction,
    copyAllAsFetchAction,
    copyAllAsHarAction,
    copyAllAsNodeFetchAction,
    copyAllUrlsAction,
    copyConsoleLog,
    copyContextAsCurlBash,
    copyContextAsCurlCmd,
    copyContextAsCurlPowerShell,
    copyContextAsFetch,
    copyContextAsHar,
    copyContextAsNodeFetch,
    copyContextAsPreload,
    copyContextAsPythonRequests,
    copyContextUrl,
    copyEventDetails,
    copyResponseBody,
    copySnippetToClipboard,
    copyTextToClipboard,
    deleteEnvironmentAction,
    deleteFolderAction,
    deleteProject,
    deleteRequest,
    deleteSampleResponseAction,
    deleteVariable,
    detailStr,
    dismissLegacyGitCandidates,
    downloadFile,
    downloadProjectFile,
    downloadResponseBody,
    duplicateRequest,
    ensureAllEnvVariablesLoaded,
    exportConsoleJson,
    exportGlobalVariables,
    exportPostmanCollectionAction,
    exportProjectFileAction,
    findResolvedVariable,
    focusOnMount,
    fontStyleOverride,
    forkCurrentEnvironment,
    formatByteSize,
    formatConsoleTime,
    formatRelativeTime,
    generateSampleResponseWithAiAction,
    generateTestsAndDocsWithAiAction,
    generateWithAi,
    growFormDataItems,
    growHeaders,
    growQueryParams,
    growUrlEncodedItems,
    handleAddGlobalVariableClick,
    handleAutocompleteKeydown,
    handleBodyMouseLeave,
    handleBodyMouseMove,
    handleCollectionFileUpload,
    handleDebugRequest,
    handleDownloadResponse,
    handleEnvironmentFileUpload,
    handleGenericInputMouseLeave,
    handleGenericInputMouseMove,
    handlePaneMouseMoveDelegated,
    handleTabMenuCloseAllTabs,
    handleTabMenuCloseOtherTabs,
    handleTabMenuCloseTab,
    handleTabMenuDuplicateTab,
    handleTabMenuForceCloseAllTabs,
    handleTabMenuForceCloseTab,
    handleTabMenuNewRequest,
    handleTabMenuRevealInSidebar,
    handleUrlPaste,
    handleVisualizeResponse,
    handleWindowClose,
    handleWindowMaximize,
    handleWindowMinimize,
    handleWriteTests,
    hideAutocompleteSoon,
    highlightSnippetCode,
    importCurlCommand,
    importDiscoveredEndpointAction,
    importLocalWorkspaceAction,
    importPostmanCollectionAction,
    importPostmanEnvironmentAction,
    importProjectFileAction,
    initializeGitRepoAction,
    isCustomSurfaceTint,
    isEnvFavorite,
    isTabDirty,
    loadConflictVersions,
    loadVariables,
    onAiProviderChange,
    onFolderDragStart,
    onFolderDrop,
    onProjectDragStart,
    onProjectDrop,
    onRequestDragStart,
    onRequestDrop,
    openDatasetTab,
    openDocumentTab,
    openEnvironmentTab,
    openFlowTab,
    openHistoryResponse,
    openMockTab,
    openPalette,
    openRequest,
    openRequestContextMenu,
    openSampleResponse,
    openSecondaryRequest,
    openSpecTab,
    openTabContextMenu,
    pickEnvironment,
    pickProjectSortField,
    pickRequestSortField,
    prettifyBody,
    pullRepositoryAction,
    quickCreateEnvironment,
    quickCreateFolder,
    quickCreateProject,
    quickCreateRequest,
    quickCreateWorkspace,
    refreshConsoleEvents,
    refreshGitStatus,
    refreshProjectHistory,
    removeFormDataItem,
    removeHeader,
    removeQueryParam,
    removeUrlEncodedItem,
    resolveConflictAction,
    revealSecret,
    sampleDatasets,
    sampleDocContent,
    sampleDocuments,
    sampleFlows,
    sampleMockServers,
    sampleOpenApiJson,
    sampleSpecs,
    saveAiSettingsAction,
    saveCurrentResponse,
    saveGitSettingsAction,
    saveRequest,
    saveVariableFromPopover,
    saveWorkspaceToRepoAction,
    scanSourceProjectAction,
    scheduleAutoSave,
    scheduleHideMissingVarPopover,
    selectProject,
    selectTab,
    selectWorkspace,
    sendCurrentRequest,
    setAccentColor,
    setAutoSyncIntervalMs,
    setBodyFontOverride,
    setHeadingFontOverride,
    setLocale,
    setRawContentType,
    setResponsePaneCollapsed,
    setRightSidebarVisible,
    setShortcutEnabled,
    setSidebarVisible,
    setSurfaceTint,
    setTextColorOverride,
    setThemeMode,
    setUiScale,
    shareCurrentEnvironment,
    showMissingVarPopover,
    showVarPopover,
    sortRequestList,
    startConsoleResize,
    startRenameEnvironment,
    startRenameFolder,
    startRenameProject,
    startRenameRequest,
    startRenameSampleResponse,
    startRenameWorkspace,
    startResponsePaneResize,
    startRightSidebarResize,
    startSidebarResize,
    submitRenameEnvironment,
    submitRenameFolder,
    submitRenameProject,
    submitRenameRequest,
    submitRenameSampleResponse,
    submitRenameWorkspace,
    surfaceStyleOverride,
    syncUrlOverlayScroll,
    t,
    testAiConnectionAction,
    textColorStyleOverride,
    toggleEnvExpand,
    toggleEnvFavorite,
    toggleEventExpanded,
    toggleExpandAllFolders,
    toggleFolderExpanded,
    togglePaletteField,
    toggleProjectExpand,
    toggleProjectSelection,
    toggleRawDetails,
    toggleSidebarSearchField,
    toggleSidebarSectionVisibility,
    toggleTreeRequestExpanded,
    toggleUtilityRail,
    toggleVariableEnabled,
    toggleVariableLocal,
    toggleVariableSecret,
    updateAutocompleteFor,
    updateVariableKey,
    updateVariableValue,
    verifyGitHubTokenAction,
    viewDiffAction,
    viewHistoryAction,
    withoutEmptyKeyRows,
    get accentColor() { return accentColor; },
    set accentColor(v: typeof accentColor) { accentColor = v; },
    get accountMenuOpen() { return accountMenuOpen; },
    set accountMenuOpen(v: typeof accountMenuOpen) { accountMenuOpen = v; },
    get activeEditorTab() { return activeEditorTab; },
    set activeEditorTab(v: typeof activeEditorTab) { activeEditorTab = v; },
    get activeResponse() { return activeResponse; },
    set activeResponse(v: typeof activeResponse) { activeResponse = v; },
    get activeResponseBody() { return activeResponseBody; },
    set activeResponseBody(v: typeof activeResponseBody) { activeResponseBody = v; },
    get activeResponseTests() { return activeResponseTests; },
    get activeResponseTruncated() { return activeResponseTruncated; },
    set activeResponseTruncated(v: typeof activeResponseTruncated) { activeResponseTruncated = v; },
    get activeScreen() { return activeScreen; },
    set activeScreen(v: typeof activeScreen) { activeScreen = v; },
    get activeScriptTab() { return activeScriptTab; },
    set activeScriptTab(v: typeof activeScriptTab) { activeScriptTab = v; },
    get activeTabId() { return activeTabId; },
    set activeTabId(v: typeof activeTabId) { activeTabId = v; },
    get activeWorkspaceId() { return activeWorkspaceId; },
    set activeWorkspaceId(v: typeof activeWorkspaceId) { activeWorkspaceId = v; },
    get aiActiveTab() { return aiActiveTab; },
    set aiActiveTab(v: typeof aiActiveTab) { aiActiveTab = v; },
    get aiApiKeyInput() { return aiApiKeyInput; },
    set aiApiKeyInput(v: typeof aiApiKeyInput) { aiApiKeyInput = v; },
    get aiBaseUrlInput() { return aiBaseUrlInput; },
    set aiBaseUrlInput(v: typeof aiBaseUrlInput) { aiBaseUrlInput = v; },
    get aiConfigured() { return aiConfigured; },
    set aiConfigured(v: typeof aiConfigured) { aiConfigured = v; },
    get aiGenerating() { return aiGenerating; },
    set aiGenerating(v: typeof aiGenerating) { aiGenerating = v; },
    get aiIncludeExistingRequests() { return aiIncludeExistingRequests; },
    set aiIncludeExistingRequests(v: typeof aiIncludeExistingRequests) { aiIncludeExistingRequests = v; },
    get aiIncludeVariables() { return aiIncludeVariables; },
    set aiIncludeVariables(v: typeof aiIncludeVariables) { aiIncludeVariables = v; },
    get aiModelInput() { return aiModelInput; },
    set aiModelInput(v: typeof aiModelInput) { aiModelInput = v; },
    get aiPreview() { return aiPreview; },
    set aiPreview(v: typeof aiPreview) { aiPreview = v; },
    get aiPrompt() { return aiPrompt; },
    set aiPrompt(v: typeof aiPrompt) { aiPrompt = v; },
    get aiProviderInput() { return aiProviderInput; },
    set aiProviderInput(v: typeof aiProviderInput) { aiProviderInput = v; },
    get aiSettings() { return aiSettings; },
    set aiSettings(v: typeof aiSettings) { aiSettings = v; },
    get aiSettingsFeedback() { return aiSettingsFeedback; },
    set aiSettingsFeedback(v: typeof aiSettingsFeedback) { aiSettingsFeedback = v; },
    get aiShowKey() { return aiShowKey; },
    set aiShowKey(v: typeof aiShowKey) { aiShowKey = v; },
    get aiTestError() { return aiTestError; },
    set aiTestError(v: typeof aiTestError) { aiTestError = v; },
    get aiTestFeedback() { return aiTestFeedback; },
    set aiTestFeedback(v: typeof aiTestFeedback) { aiTestFeedback = v; },
    get aiTesting() { return aiTesting; },
    set aiTesting(v: typeof aiTesting) { aiTesting = v; },
    get allEnvironments() { return allEnvironments; },
    set allEnvironments(v: typeof allEnvironments) { allEnvironments = v; },
    get autoSaveStatus() { return autoSaveStatus; },
    set autoSaveStatus(v: typeof autoSaveStatus) { autoSaveStatus = v; },
    get autoSyncIntervalMs() { return autoSyncIntervalMs; },
    set autoSyncIntervalMs(v: typeof autoSyncIntervalMs) { autoSyncIntervalMs = v; },
    get autocomplete() { return autocomplete; },
    set autocomplete(v: typeof autocomplete) { autocomplete = v; },
    get bodyFontOverride() { return bodyFontOverride; },
    set bodyFontOverride(v: typeof bodyFontOverride) { bodyFontOverride = v; },
    get bodyPrettifyFeedback() { return bodyPrettifyFeedback; },
    set bodyPrettifyFeedback(v: typeof bodyPrettifyFeedback) { bodyPrettifyFeedback = v; },
    get collectionImportError() { return collectionImportError; },
    set collectionImportError(v: typeof collectionImportError) { collectionImportError = v; },
    get collectionImportLoading() { return collectionImportLoading; },
    set collectionImportLoading(v: typeof collectionImportLoading) { collectionImportLoading = v; },
    get collectionImportReport() { return collectionImportReport; },
    set collectionImportReport(v: typeof collectionImportReport) { collectionImportReport = v; },
    get collectionImportTarget() { return collectionImportTarget; },
    set collectionImportTarget(v: typeof collectionImportTarget) { collectionImportTarget = v; },
    get collectionImportText() { return collectionImportText; },
    set collectionImportText(v: typeof collectionImportText) { collectionImportText = v; },
    get collectionsAccordionOpen() { return collectionsAccordionOpen; },
    set collectionsAccordionOpen(v: typeof collectionsAccordionOpen) { collectionsAccordionOpen = v; },
    get confirmDialog() { return confirmDialog; },
    set confirmDialog(v: typeof confirmDialog) { confirmDialog = v; },
    get conflictVersions() { return conflictVersions; },
    set conflictVersions(v: typeof conflictVersions) { conflictVersions = v; },
    get conflictVersionsLoading() { return conflictVersionsLoading; },
    set conflictVersionsLoading(v: typeof conflictVersionsLoading) { conflictVersionsLoading = v; },
    get consoleActiveRequestOnly() { return consoleActiveRequestOnly; },
    set consoleActiveRequestOnly(v: typeof consoleActiveRequestOnly) { consoleActiveRequestOnly = v; },
    get consoleErrorCount() { return consoleErrorCount; },
    get consoleHeight() { return consoleHeight; },
    set consoleHeight(v: typeof consoleHeight) { consoleHeight = v; },
    get consoleLevelFilter() { return consoleLevelFilter; },
    set consoleLevelFilter(v: typeof consoleLevelFilter) { consoleLevelFilter = v; },
    get consoleResizing() { return consoleResizing; },
    set consoleResizing(v: typeof consoleResizing) { consoleResizing = v; },
    get consoleSearchFilter() { return consoleSearchFilter; },
    set consoleSearchFilter(v: typeof consoleSearchFilter) { consoleSearchFilter = v; },
    get consoleWarnCount() { return consoleWarnCount; },
    get copyFeedback() { return copyFeedback; },
    set copyFeedback(v: typeof copyFeedback) { copyFeedback = v; },
    get curlDetectedFeedback() { return curlDetectedFeedback; },
    set curlDetectedFeedback(v: typeof curlDetectedFeedback) { curlDetectedFeedback = v; },
    get curlImportError() { return curlImportError; },
    set curlImportError(v: typeof curlImportError) { curlImportError = v; },
    get curlImportName() { return curlImportName; },
    set curlImportName(v: typeof curlImportName) { curlImportName = v; },
    get curlImportText() { return curlImportText; },
    set curlImportText(v: typeof curlImportText) { curlImportText = v; },
    get currentTab() { return currentTab; },
    get datasetsAccordionOpen() { return datasetsAccordionOpen; },
    set datasetsAccordionOpen(v: typeof datasetsAccordionOpen) { datasetsAccordionOpen = v; },
    get documentsAccordionOpen() { return documentsAccordionOpen; },
    set documentsAccordionOpen(v: typeof documentsAccordionOpen) { documentsAccordionOpen = v; },
    get drawerFilteredVariables() { return drawerFilteredVariables; },
    get drawerVarSearch() { return drawerVarSearch; },
    set drawerVarSearch(v: typeof drawerVarSearch) { drawerVarSearch = v; },
    get editAuthApiKeyKey() { return editAuthApiKeyKey; },
    set editAuthApiKeyKey(v: typeof editAuthApiKeyKey) { editAuthApiKeyKey = v; },
    get editAuthApiKeyLocation() { return editAuthApiKeyLocation; },
    set editAuthApiKeyLocation(v: typeof editAuthApiKeyLocation) { editAuthApiKeyLocation = v; },
    get editAuthApiKeyValue() { return editAuthApiKeyValue; },
    set editAuthApiKeyValue(v: typeof editAuthApiKeyValue) { editAuthApiKeyValue = v; },
    get editAuthBasicPassword() { return editAuthBasicPassword; },
    set editAuthBasicPassword(v: typeof editAuthBasicPassword) { editAuthBasicPassword = v; },
    get editAuthBasicUsername() { return editAuthBasicUsername; },
    set editAuthBasicUsername(v: typeof editAuthBasicUsername) { editAuthBasicUsername = v; },
    get editAuthBearerToken() { return editAuthBearerToken; },
    set editAuthBearerToken(v: typeof editAuthBearerToken) { editAuthBearerToken = v; },
    get editAuthType() { return editAuthType; },
    set editAuthType(v: typeof editAuthType) { editAuthType = v; },
    get editBinaryFilePath() { return editBinaryFilePath; },
    set editBinaryFilePath(v: typeof editBinaryFilePath) { editBinaryFilePath = v; },
    get editBody() { return editBody; },
    set editBody(v: typeof editBody) { editBody = v; },
    get editBodyType() { return editBodyType; },
    set editBodyType(v: typeof editBodyType) { editBodyType = v; },
    get editDescription() { return editDescription; },
    set editDescription(v: typeof editDescription) { editDescription = v; },
    get editFollowRedirects() { return editFollowRedirects; },
    set editFollowRedirects(v: typeof editFollowRedirects) { editFollowRedirects = v; },
    get editFormDataItems() { return editFormDataItems; },
    set editFormDataItems(v: typeof editFormDataItems) { editFormDataItems = v; },
    get editGraphqlQuery() { return editGraphqlQuery; },
    set editGraphqlQuery(v: typeof editGraphqlQuery) { editGraphqlQuery = v; },
    get editGraphqlVariables() { return editGraphqlVariables; },
    set editGraphqlVariables(v: typeof editGraphqlVariables) { editGraphqlVariables = v; },
    get editHeaders() { return editHeaders; },
    set editHeaders(v: typeof editHeaders) { editHeaders = v; },
    get editHttpVersion() { return editHttpVersion; },
    set editHttpVersion(v: typeof editHttpVersion) { editHttpVersion = v; },
    get editMaxRedirects() { return editMaxRedirects; },
    set editMaxRedirects(v: typeof editMaxRedirects) { editMaxRedirects = v; },
    get editMethod() { return editMethod; },
    set editMethod(v: typeof editMethod) { editMethod = v; },
    get editPostScript() { return editPostScript; },
    set editPostScript(v: typeof editPostScript) { editPostScript = v; },
    get editPreScript() { return editPreScript; },
    set editPreScript(v: typeof editPreScript) { editPreScript = v; },
    get editProxyUrl() { return editProxyUrl; },
    set editProxyUrl(v: typeof editProxyUrl) { editProxyUrl = v; },
    get editQueryParams() { return editQueryParams; },
    set editQueryParams(v: typeof editQueryParams) { editQueryParams = v; },
    get editTimeoutMs() { return editTimeoutMs; },
    set editTimeoutMs(v: typeof editTimeoutMs) { editTimeoutMs = v; },
    get editUrl() { return editUrl; },
    set editUrl(v: typeof editUrl) { editUrl = v; },
    get editUrlEncodedItems() { return editUrlEncodedItems; },
    set editUrlEncodedItems(v: typeof editUrlEncodedItems) { editUrlEncodedItems = v; },
    get editVerifySsl() { return editVerifySsl; },
    set editVerifySsl(v: typeof editVerifySsl) { editVerifySsl = v; },
    get envForkCount() { return envForkCount; },
    set envForkCount(v: typeof envForkCount) { envForkCount = v; },
    get envPickerFilteredEnvironments() { return envPickerFilteredEnvironments; },
    get envPickerOpen() { return envPickerOpen; },
    set envPickerOpen(v: typeof envPickerOpen) { envPickerOpen = v; },
    get envPickerQuery() { return envPickerQuery; },
    set envPickerQuery(v: typeof envPickerQuery) { envPickerQuery = v; },
    get envSearchQuery() { return envSearchQuery; },
    set envSearchQuery(v: typeof envSearchQuery) { envSearchQuery = v; },
    get envSidebarSearchQuery() { return envSidebarSearchQuery; },
    set envSidebarSearchQuery(v: typeof envSidebarSearchQuery) { envSidebarSearchQuery = v; },
    get envSidebarSearchScope() { return envSidebarSearchScope; },
    set envSidebarSearchScope(v: typeof envSidebarSearchScope) { envSidebarSearchScope = v; },
    get envVarSearchQuery() { return envVarSearchQuery; },
    set envVarSearchQuery(v: typeof envVarSearchQuery) { envVarSearchQuery = v; },
    get envVariablesCache() { return envVariablesCache; },
    set envVariablesCache(v: typeof envVariablesCache) { envVariablesCache = v; },
    get environmentImportError() { return environmentImportError; },
    set environmentImportError(v: typeof environmentImportError) { environmentImportError = v; },
    get environmentImportLoading() { return environmentImportLoading; },
    set environmentImportLoading(v: typeof environmentImportLoading) { environmentImportLoading = v; },
    get environmentImportReport() { return environmentImportReport; },
    set environmentImportReport(v: typeof environmentImportReport) { environmentImportReport = v; },
    get environmentImportText() { return environmentImportText; },
    set environmentImportText(v: typeof environmentImportText) { environmentImportText = v; },
    get environmentVariables() { return environmentVariables; },
    set environmentVariables(v: typeof environmentVariables) { environmentVariables = v; },
    get environmentsAccordionOpen() { return environmentsAccordionOpen; },
    set environmentsAccordionOpen(v: typeof environmentsAccordionOpen) { environmentsAccordionOpen = v; },
    get errorMessage() { return errorMessage; },
    set errorMessage(v: typeof errorMessage) { errorMessage = v; },
    get expandedEnvIds() { return expandedEnvIds; },
    set expandedEnvIds(v: typeof expandedEnvIds) { expandedEnvIds = v; },
    get expandedEventIds() { return expandedEventIds; },
    set expandedEventIds(v: typeof expandedEventIds) { expandedEventIds = v; },
    get expandedFolderIds() { return expandedFolderIds; },
    set expandedFolderIds(v: typeof expandedFolderIds) { expandedFolderIds = v; },
    get expandedProjectIds() { return expandedProjectIds; },
    set expandedProjectIds(v: typeof expandedProjectIds) { expandedProjectIds = v; },
    get expandedTreeRequestIds() { return expandedTreeRequestIds; },
    set expandedTreeRequestIds(v: typeof expandedTreeRequestIds) { expandedTreeRequestIds = v; },
    get exportFeedback() { return exportFeedback; },
    set exportFeedback(v: typeof exportFeedback) { exportFeedback = v; },
    get favoriteEnvs() { return favoriteEnvs; },
    get filteredConsoleEvents() { return filteredConsoleEvents; },
    get filteredEnvironmentVariables() { return filteredEnvironmentVariables; },
    get filteredEnvironmentsByProject() { return filteredEnvironmentsByProject; },
    get filteredProjectHistory() { return filteredProjectHistory; },
    get filteredProjectVariables() { return filteredProjectVariables; },
    get filteredProjects() { return filteredProjects; },
    get filteredRequests() { return filteredRequests; },
    get filteredSidebarEnvironments() { return filteredSidebarEnvironments; },
    get filteredSidebarFavoriteEnvs() { return filteredSidebarFavoriteEnvs; },
    get filteredSourceEndpoints() { return filteredSourceEndpoints; },
    get flowsAccordionOpen() { return flowsAccordionOpen; },
    set flowsAccordionOpen(v: typeof flowsAccordionOpen) { flowsAccordionOpen = v; },
    get folders() { return folders; },
    set folders(v: typeof folders) { folders = v; },
    get foldersByParentId() { return foldersByParentId; },
    get generatingSample() { return generatingSample; },
    set generatingSample(v: typeof generatingSample) { generatingSample = v; },
    get generatingTestsDocs() { return generatingTestsDocs; },
    set generatingTestsDocs(v: typeof generatingTestsDocs) { generatingTestsDocs = v; },
    get gitActionError() { return gitActionError; },
    set gitActionError(v: typeof gitActionError) { gitActionError = v; },
    get gitActionFeedback() { return gitActionFeedback; },
    set gitActionFeedback(v: typeof gitActionFeedback) { gitActionFeedback = v; },
    get gitActiveTab() { return gitActiveTab; },
    set gitActiveTab(v: typeof gitActiveTab) { gitActiveTab = v; },
    get gitAutoSyncInput() { return gitAutoSyncInput; },
    set gitAutoSyncInput(v: typeof gitAutoSyncInput) { gitAutoSyncInput = v; },
    get gitCommitMessage() { return gitCommitMessage; },
    set gitCommitMessage(v: typeof gitCommitMessage) { gitCommitMessage = v; },
    get gitDiffContent() { return gitDiffContent; },
    set gitDiffContent(v: typeof gitDiffContent) { gitDiffContent = v; },
    get gitHistory() { return gitHistory; },
    set gitHistory(v: typeof gitHistory) { gitHistory = v; },
    get gitLoading() { return gitLoading; },
    set gitLoading(v: typeof gitLoading) { gitLoading = v; },
    get gitRemoteUrlInput() { return gitRemoteUrlInput; },
    set gitRemoteUrlInput(v: typeof gitRemoteUrlInput) { gitRemoteUrlInput = v; },
    get gitRepoPathInput() { return gitRepoPathInput; },
    set gitRepoPathInput(v: typeof gitRepoPathInput) { gitRepoPathInput = v; },
    get gitSettings() { return gitSettings; },
    set gitSettings(v: typeof gitSettings) { gitSettings = v; },
    get gitStatus() { return gitStatus; },
    set gitStatus(v: typeof gitStatus) { gitStatus = v; },
    get gitStatusLoading() { return gitStatusLoading; },
    set gitStatusLoading(v: typeof gitStatusLoading) { gitStatusLoading = v; },
    get githubRepoInfo() { return githubRepoInfo; },
    set githubRepoInfo(v: typeof githubRepoInfo) { githubRepoInfo = v; },
    get githubShowToken() { return githubShowToken; },
    set githubShowToken(v: typeof githubShowToken) { githubShowToken = v; },
    get githubTokenInput() { return githubTokenInput; },
    set githubTokenInput(v: typeof githubTokenInput) { githubTokenInput = v; },
    get githubUser() { return githubUser; },
    set githubUser(v: typeof githubUser) { githubUser = v; },
    get githubValidating() { return githubValidating; },
    set githubValidating(v: typeof githubValidating) { githubValidating = v; },
    get globalsSearch() { return globalsSearch; },
    set globalsSearch(v: typeof globalsSearch) { globalsSearch = v; },
    get headingFontOverride() { return headingFontOverride; },
    set headingFontOverride(v: typeof headingFontOverride) { headingFontOverride = v; },
    get highlightedResponseBody() { return highlightedResponseBody; },
    get historyLoading() { return historyLoading; },
    set historyLoading(v: typeof historyLoading) { historyLoading = v; },
    get historySearchQuery() { return historySearchQuery; },
    set historySearchQuery(v: typeof historySearchQuery) { historySearchQuery = v; },
    get historyShowFailuresOnly() { return historyShowFailuresOnly; },
    set historyShowFailuresOnly(v: typeof historyShowFailuresOnly) { historyShowFailuresOnly = v; },
    get hoveredVarRect() { return hoveredVarRect; },
    set hoveredVarRect(v: typeof hoveredVarRect) { hoveredVarRect = v; },
    get importActiveTab() { return importActiveTab; },
    set importActiveTab(v: typeof importActiveTab) { importActiveTab = v; },
    get inviteUsername() { return inviteUsername; },
    set inviteUsername(v: typeof inviteUsername) { inviteUsername = v; },
    get inviteBusy() { return inviteBusy; },
    get inviteError() { return inviteError; },
    set inviteError(v: typeof inviteError) { inviteError = v; },
    get inviteNotice() { return inviteNotice; },
    get inviteBlocker() { return inviteBlocker; },
    get inviteRepoLabel() { return inviteRepoLabel; },
    get collaborators() { return collaborators; },
    get collaboratorsLoading() { return collaboratorsLoading; },
    get canGoBack() { return canGoBack; },
    get canGoForward() { return canGoForward; },
    get notifications() { return notifications; },
    get inviteRole() { return inviteRole; },
    set inviteRole(v: typeof inviteRole) { inviteRole = v; },
    get legacyGitCandidates() { return legacyGitCandidates; },
    set legacyGitCandidates(v: typeof legacyGitCandidates) { legacyGitCandidates = v; },
    get loadingRequests() { return loadingRequests; },
    set loadingRequests(v: typeof loadingRequests) { loadingRequests = v; },
    get localWorkspaceImportError() { return localWorkspaceImportError; },
    set localWorkspaceImportError(v: typeof localWorkspaceImportError) { localWorkspaceImportError = v; },
    get localWorkspaceImportLoading() { return localWorkspaceImportLoading; },
    set localWorkspaceImportLoading(v: typeof localWorkspaceImportLoading) { localWorkspaceImportLoading = v; },
    get localWorkspaceImportReport() { return localWorkspaceImportReport; },
    set localWorkspaceImportReport(v: typeof localWorkspaceImportReport) { localWorkspaceImportReport = v; },
    get localWorkspacePathInput() { return localWorkspacePathInput; },
    set localWorkspacePathInput(v: typeof localWorkspacePathInput) { localWorkspacePathInput = v; },
    get locale() { return locale; },
    set locale(v: typeof locale) { locale = v; },
    get missingVarDrafts() { return missingVarDrafts; },
    set missingVarDrafts(v: typeof missingVarDrafts) { missingVarDrafts = v; },
    get missingVarHover() { return missingVarHover; },
    set missingVarHover(v: typeof missingVarHover) { missingVarHover = v; },
    get mocksAccordionOpen() { return mocksAccordionOpen; },
    set mocksAccordionOpen(v: typeof mocksAccordionOpen) { mocksAccordionOpen = v; },
    get newEnvVarDraft() { return newEnvVarDraft; },
    set newEnvVarDraft(v: typeof newEnvVarDraft) { newEnvVarDraft = v; },
    get newGlobalVarDraft() { return newGlobalVarDraft; },
    set newGlobalVarDraft(v: typeof newGlobalVarDraft) { newGlobalVarDraft = v; },
    get notificationsOpen() { return notificationsOpen; },
    set notificationsOpen(v: typeof notificationsOpen) { notificationsOpen = v; },
    get openProjectMenuId() { return openProjectMenuId; },
    set openProjectMenuId(v: typeof openProjectMenuId) { openProjectMenuId = v; },
    get openTabs() { return openTabs; },
    set openTabs(v: typeof openTabs) { openTabs = v; },
    get paletteFields() { return paletteFields; },
    set paletteFields(v: typeof paletteFields) { paletteFields = v; },
    get paletteInputEl() { return paletteInputEl; },
    set paletteInputEl(v: typeof paletteInputEl) { paletteInputEl = v; },
    get paletteItems() { return paletteItems; },
    get paletteOpen() { return paletteOpen; },
    set paletteOpen(v: typeof paletteOpen) { paletteOpen = v; },
    get paletteQuery() { return paletteQuery; },
    set paletteQuery(v: typeof paletteQuery) { paletteQuery = v; },
    get paletteScope() { return paletteScope; },
    set paletteScope(v: typeof paletteScope) { paletteScope = v; },
    get popoverInputFocused() { return popoverInputFocused; },
    set popoverInputFocused(v: typeof popoverInputFocused) { popoverInputFocused = v; },
    get popoverSaveSuccess() { return popoverSaveSuccess; },
    set popoverSaveSuccess(v: typeof popoverSaveSuccess) { popoverSaveSuccess = v; },
    get prettyResponseBody() { return prettyResponseBody; },
    get projectFileJson() { return projectFileJson; },
    set projectFileJson(v: typeof projectFileJson) { projectFileJson = v; },
    get projectFileMaskSecrets() { return projectFileMaskSecrets; },
    set projectFileMaskSecrets(v: typeof projectFileMaskSecrets) { projectFileMaskSecrets = v; },
    get projectFileStatus() { return projectFileStatus; },
    set projectFileStatus(v: typeof projectFileStatus) { projectFileStatus = v; },
    get projectHistory() { return projectHistory; },
    set projectHistory(v: typeof projectHistory) { projectHistory = v; },
    get projectRequestCounts() { return projectRequestCounts; },
    set projectRequestCounts(v: typeof projectRequestCounts) { projectRequestCounts = v; },
    get projectSearchQuery() { return projectSearchQuery; },
    set projectSearchQuery(v: typeof projectSearchQuery) { projectSearchQuery = v; },
    get projectSearchScope() { return projectSearchScope; },
    set projectSearchScope(v: typeof projectSearchScope) { projectSearchScope = v; },
    get projectSortDir() { return projectSortDir; },
    set projectSortDir(v: typeof projectSortDir) { projectSortDir = v; },
    get projectSortField() { return projectSortField; },
    set projectSortField(v: typeof projectSortField) { projectSortField = v; },
    get projectSortMenuOpen() { return projectSortMenuOpen; },
    set projectSortMenuOpen(v: typeof projectSortMenuOpen) { projectSortMenuOpen = v; },
    get projectVariables() { return projectVariables; },
    set projectVariables(v: typeof projectVariables) { projectVariables = v; },
    get projects() { return projects; },
    set projects(v: typeof projects) { projects = v; },
    get rawContentType() { return rawContentType; },
    get rawDetailsVisible() { return rawDetailsVisible; },
    set rawDetailsVisible(v: typeof rawDetailsVisible) { rawDetailsVisible = v; },
    get renameEnvironmentValue() { return renameEnvironmentValue; },
    set renameEnvironmentValue(v: typeof renameEnvironmentValue) { renameEnvironmentValue = v; },
    get renameFolderValue() { return renameFolderValue; },
    set renameFolderValue(v: typeof renameFolderValue) { renameFolderValue = v; },
    get renameProjectValue() { return renameProjectValue; },
    set renameProjectValue(v: typeof renameProjectValue) { renameProjectValue = v; },
    get renameRequestValue() { return renameRequestValue; },
    set renameRequestValue(v: typeof renameRequestValue) { renameRequestValue = v; },
    get renameSampleResponseValue() { return renameSampleResponseValue; },
    set renameSampleResponseValue(v: typeof renameSampleResponseValue) { renameSampleResponseValue = v; },
    get renameWorkspaceValue() { return renameWorkspaceValue; },
    set renameWorkspaceValue(v: typeof renameWorkspaceValue) { renameWorkspaceValue = v; },
    get renamingEnvironmentId() { return renamingEnvironmentId; },
    set renamingEnvironmentId(v: typeof renamingEnvironmentId) { renamingEnvironmentId = v; },
    get renamingFolderId() { return renamingFolderId; },
    set renamingFolderId(v: typeof renamingFolderId) { renamingFolderId = v; },
    get renamingProjectId() { return renamingProjectId; },
    set renamingProjectId(v: typeof renamingProjectId) { renamingProjectId = v; },
    get renamingRequestId() { return renamingRequestId; },
    set renamingRequestId(v: typeof renamingRequestId) { renamingRequestId = v; },
    get renamingSampleResponseId() { return renamingSampleResponseId; },
    set renamingSampleResponseId(v: typeof renamingSampleResponseId) { renamingSampleResponseId = v; },
    get renamingWorkspaceId() { return renamingWorkspaceId; },
    set renamingWorkspaceId(v: typeof renamingWorkspaceId) { renamingWorkspaceId = v; },
    get requestContextMenu() { return requestContextMenu; },
    set requestContextMenu(v: typeof requestContextMenu) { requestContextMenu = v; },
    get requestDiagnostics() { return requestDiagnostics; },
    set requestDiagnostics(v: typeof requestDiagnostics) { requestDiagnostics = v; },
    get requestPage() { return requestPage; },
    set requestPage(v: typeof requestPage) { requestPage = v; },
    get requestSearchQuery() { return requestSearchQuery; },
    set requestSearchQuery(v: typeof requestSearchQuery) { requestSearchQuery = v; },
    get requestSortDir() { return requestSortDir; },
    set requestSortDir(v: typeof requestSortDir) { requestSortDir = v; },
    get requestSortField() { return requestSortField; },
    set requestSortField(v: typeof requestSortField) { requestSortField = v; },
    get requestSortMenuOpen() { return requestSortMenuOpen; },
    set requestSortMenuOpen(v: typeof requestSortMenuOpen) { requestSortMenuOpen = v; },
    get requests() { return requests; },
    set requests(v: typeof requests) { requests = v; },
    get requestsByFolderId() { return requestsByFolderId; },
    get responseBodyIsHtml() { return responseBodyIsHtml; },
    get responseBodyIsJson() { return responseBodyIsJson; },
    get responseExpanded() { return responseExpanded; },
    set responseExpanded(v: typeof responseExpanded) { responseExpanded = v; },
    get responseHistory() { return responseHistory; },
    set responseHistory(v: typeof responseHistory) { responseHistory = v; },
    get responsePaneCollapsed() { return responsePaneCollapsed; },
    set responsePaneCollapsed(v: typeof responsePaneCollapsed) { responsePaneCollapsed = v; },
    get responsePaneHeight() { return responsePaneHeight; },
    set responsePaneHeight(v: typeof responsePaneHeight) { responsePaneHeight = v; },
    get responsePaneManuallyResized() { return responsePaneManuallyResized; },
    set responsePaneManuallyResized(v: typeof responsePaneManuallyResized) { responsePaneManuallyResized = v; },
    get responsePaneMaximized() { return responsePaneMaximized; },
    set responsePaneMaximized(v: typeof responsePaneMaximized) { responsePaneMaximized = v; },
    get responsePaneResizing() { return responsePaneResizing; },
    set responsePaneResizing(v: typeof responsePaneResizing) { responsePaneResizing = v; },
    get responseSubTab() { return responseSubTab; },
    set responseSubTab(v: typeof responseSubTab) { responseSubTab = v; },
    get responseViewMode() { return responseViewMode; },
    set responseViewMode(v: typeof responseViewMode) { responseViewMode = v; },
    get revealedSecrets() { return revealedSecrets; },
    set revealedSecrets(v: typeof revealedSecrets) { revealedSecrets = v; },
    get rightPanel() { return rightPanel; },
    set rightPanel(v: typeof rightPanel) { rightPanel = v; },
    get rightSidebarResizing() { return rightSidebarResizing; },
    set rightSidebarResizing(v: typeof rightSidebarResizing) { rightSidebarResizing = v; },
    get rightSidebarVisible() { return rightSidebarVisible; },
    set rightSidebarVisible(v: typeof rightSidebarVisible) { rightSidebarVisible = v; },
    get rightSidebarWidth() { return rightSidebarWidth; },
    set rightSidebarWidth(v: typeof rightSidebarWidth) { rightSidebarWidth = v; },
    get rootFolders() { return rootFolders; },
    get rootRequests() { return rootRequests; },
    get sampleFeedback() { return sampleFeedback; },
    set sampleFeedback(v: typeof sampleFeedback) { sampleFeedback = v; },
    get sampleResponses() { return sampleResponses; },
    get sampleResponsesByRequestId() { return sampleResponsesByRequestId; },
    set sampleResponsesByRequestId(v: typeof sampleResponsesByRequestId) { sampleResponsesByRequestId = v; },
    get secondaryProjectCache() { return secondaryProjectCache; },
    set secondaryProjectCache(v: typeof secondaryProjectCache) { secondaryProjectCache = v; },
    get secondaryProjectLoading() { return secondaryProjectLoading; },
    set secondaryProjectLoading(v: typeof secondaryProjectLoading) { secondaryProjectLoading = v; },
    get selectedConflictFile() { return selectedConflictFile; },
    set selectedConflictFile(v: typeof selectedConflictFile) { selectedConflictFile = v; },
    get selectedEnvironmentId() { return selectedEnvironmentId; },
    set selectedEnvironmentId(v: typeof selectedEnvironmentId) { selectedEnvironmentId = v; },
    get selectedProjectId() { return selectedProjectId; },
    set selectedProjectId(v: typeof selectedProjectId) { selectedProjectId = v; },
    get selectedRequest() { return selectedRequest; },
    set selectedRequest(v: typeof selectedRequest) { selectedRequest = v; },
    get selectedRequestFolderChain() { return selectedRequestFolderChain; },
    get sendCancelledNotice() { return sendCancelledNotice; },
    set sendCancelledNotice(v: typeof sendCancelledNotice) { sendCancelledNotice = v; },
    get sendMenuOpen() { return sendMenuOpen; },
    set sendMenuOpen(v: typeof sendMenuOpen) { sendMenuOpen = v; },
    get sending() { return sending; },
    set sending(v: typeof sending) { sending = v; },
    get shortcutsEnabled() { return shortcutsEnabled; },
    set shortcutsEnabled(v: typeof shortcutsEnabled) { shortcutsEnabled = v; },
    get showAiPanel() { return showAiPanel; },
    set showAiPanel(v: typeof showAiPanel) { showAiPanel = v; },
    get showConsole() { return showConsole; },
    set showConsole(v: typeof showConsole) { showConsole = v; },
    get showDiffModal() { return showDiffModal; },
    set showDiffModal(v: typeof showDiffModal) { showDiffModal = v; },
    get showHistoryModal() { return showHistoryModal; },
    set showHistoryModal(v: typeof showHistoryModal) { showHistoryModal = v; },
    get showImportDialog() { return showImportDialog; },
    set showImportDialog(v: typeof showImportDialog) { showImportDialog = v; },
    get showInviteModal() { return showInviteModal; },
    set showInviteModal(v: typeof showInviteModal) { showInviteModal = v; },
    get showSnippetSettings() { return showSnippetSettings; },
    set showSnippetSettings(v: typeof showSnippetSettings) { showSnippetSettings = v; },
    get sidebarApiResults() { return sidebarApiResults; },
    set sidebarApiResults(v: typeof sidebarApiResults) { sidebarApiResults = v; },
    get sidebarManuallyResized() { return sidebarManuallyResized; },
    set sidebarManuallyResized(v: typeof sidebarManuallyResized) { sidebarManuallyResized = v; },
    get sidebarResizing() { return sidebarResizing; },
    set sidebarResizing(v: typeof sidebarResizing) { sidebarResizing = v; },
    get sidebarSearchFields() { return sidebarSearchFields; },
    set sidebarSearchFields(v: typeof sidebarSearchFields) { sidebarSearchFields = v; },
    get sidebarSection() { return sidebarSection; },
    set sidebarSection(v: typeof sidebarSection) { sidebarSection = v; },
    get sidebarSectionsVisible() { return sidebarSectionsVisible; },
    set sidebarSectionsVisible(v: typeof sidebarSectionsVisible) { sidebarSectionsVisible = v; },
    get sidebarVisible() { return sidebarVisible; },
    set sidebarVisible(v: typeof sidebarVisible) { sidebarVisible = v; },
    get sidebarWidth() { return sidebarWidth; },
    set sidebarWidth(v: typeof sidebarWidth) { sidebarWidth = v; },
    get snippet() { return snippet; },
    set snippet(v: typeof snippet) { snippet = v; },
    get snippetError() { return snippetError; },
    set snippetError(v: typeof snippetError) { snippetError = v; },
    get snippetIndentType() { return snippetIndentType; },
    set snippetIndentType(v: typeof snippetIndentType) { snippetIndentType = v; },
    get snippetLoading() { return snippetLoading; },
    set snippetLoading(v: typeof snippetLoading) { snippetLoading = v; },
    get snippetMode() { return snippetMode; },
    set snippetMode(v: typeof snippetMode) { snippetMode = v; },
    get snippetTarget() { return snippetTarget; },
    set snippetTarget(v: typeof snippetTarget) { snippetTarget = v; },
    get snippetTrimTrailing() { return snippetTrimTrailing; },
    set snippetTrimTrailing(v: typeof snippetTrimTrailing) { snippetTrimTrailing = v; },
    get sourceActionFeedback() { return sourceActionFeedback; },
    set sourceActionFeedback(v: typeof sourceActionFeedback) { sourceActionFeedback = v; },
    get sourceDirectoryInput() { return sourceDirectoryInput; },
    set sourceDirectoryInput(v: typeof sourceDirectoryInput) { sourceDirectoryInput = v; },
    get sourceFilter() { return sourceFilter; },
    set sourceFilter(v: typeof sourceFilter) { sourceFilter = v; },
    get sourceReport() { return sourceReport; },
    set sourceReport(v: typeof sourceReport) { sourceReport = v; },
    get sourceScanning() { return sourceScanning; },
    set sourceScanning(v: typeof sourceScanning) { sourceScanning = v; },
    get specsAccordionOpen() { return specsAccordionOpen; },
    set specsAccordionOpen(v: typeof specsAccordionOpen) { specsAccordionOpen = v; },
    get surfaceTint() { return surfaceTint; },
    set surfaceTint(v: typeof surfaceTint) { surfaceTint = v; },
    get surfaceTintAvailable() { return surfaceTintAvailable; },
    get systemDiagnostics() { return systemDiagnostics; },
    set systemDiagnostics(v: typeof systemDiagnostics) { systemDiagnostics = v; },
    get tabContextMenu() { return tabContextMenu; },
    set tabContextMenu(v: typeof tabContextMenu) { tabContextMenu = v; },
    get testsDocsFeedback() { return testsDocsFeedback; },
    set testsDocsFeedback(v: typeof testsDocsFeedback) { testsDocsFeedback = v; },
    get textColorContrastWarning() { return textColorContrastWarning; },
    get textColorOverride() { return textColorOverride; },
    set textColorOverride(v: typeof textColorOverride) { textColorOverride = v; },
    get textCopiedNotice() { return textCopiedNotice; },
    set textCopiedNotice(v: typeof textCopiedNotice) { textCopiedNotice = v; },
    get themeMode() { return themeMode; },
    set themeMode(v: typeof themeMode) { themeMode = v; },
    get totalRequestPages() { return totalRequestPages; },
    get uiScale() { return uiScale; },
    set uiScale(v: typeof uiScale) { uiScale = v; },
    get urlPreview() { return urlPreview; },
    set urlPreview(v: typeof urlPreview) { urlPreview = v; },
    get urlTokens() { return urlTokens; },
    get utilityRailVisible() { return utilityRailVisible; },
    set utilityRailVisible(v: typeof utilityRailVisible) { utilityRailVisible = v; },
    get visibleRequests() { return visibleRequests; },
    get workspacePickerOpen() { return workspacePickerOpen; },
    set workspacePickerOpen(v: typeof workspacePickerOpen) { workspacePickerOpen = v; },
    get workspaces() { return workspaces; },
    set workspaces(v: typeof workspaces) { workspaces = v; },
  };
}

export type App = ReturnType<typeof createApp>;
