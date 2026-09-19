<script lang="ts">
  import { iconSearch, iconBell, iconHome, iconUser, iconArrowLeft, iconArrowRight, iconCube, iconClose, iconCheck, iconCheckCircle, iconXCircle, iconTrash, iconEdit, iconWarning, iconInfo, iconMoreVertical, iconMenu, iconFolder, iconFolderOpen, iconFolderPlus, iconGlobe, iconImport, iconEye, iconLock, iconGitBranch, iconMonitor, iconGrid, iconLayout, iconSidebar, iconClock, iconSettings, iconSparkle, iconInboxEmpty, iconFileText, iconSave, iconStar, iconChevronRight, iconChevronLeft, iconChevronDown, iconChevronUp, iconExpandAll, iconCollapseAll, iconCopy, iconExpandDiagonal, iconCompressDiagonal, iconMinus, iconArrowUp, iconArrowDown, railScreenIcon } from "$lib/components/icons.svelte";
  import { onMount, tick } from "svelte";
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
    datasets: true,
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
  let inviteEmail = $state("");
  let inviteRole = $state("editor");
  let showUpgradeModal = $state(false);
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
- Production: \`https://api.alansari.ae/v1\`
- UAT / Staging: \`https://aaeuat.alansari.ae:19443/FintechGateway\`
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
      { url: "https://aaeuat.alansari.ae:19443/FintechGateway", description: "UAT Gateway" },
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
  type ScreenId = "workspace" | "environments" | "globals" | "git" | "launcher" | "history" | "settings";
  let activeScreen = $state<ScreenId>("workspace");
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
  type ThemeMode = "light" | "dark" | "terminal" | "blueprint";
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

  async function duplicateEnvironment(env: Environment) {
    try {
      const newEnv = await api.createEnvironment({
        project_id: env.project_id,
        name: `${env.name} Copy`
      });
      allEnvironments = [...allEnvironments, newEnv];
    } catch (err) {
      errorMessage = describeError(err);
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
  type PaletteScope = "all" | "projects" | "apis";
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
  type ShortcutId = "commandPalette" | "sendRequest" | "saveRequest" | "newRequest" | "nextTab" | "prevTab" | "closeTab";
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

  type PaletteItem = { kind: "project" | "request"; label: string; method?: string; hint: string; project?: string; onSelect: () => void };
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
  interface RequestTab {
    id: string;
    name: string;
    method: string;
    tabType?: "request" | "env" | "doc" | "spec" | "mock" | "dataset" | "flow";
    envId?: string;
    data?: any;
  }

  interface RequestDraft {
    editName: string;
    editMethod: string;
    editUrl: string;
    editHeaders: HeaderEntry[];
    editQueryParams: QueryParam[];
    editBody: string;
    editDescription: string;
    editAuthType: Auth["type"];
    editAuthBearerToken: string;
    editAuthBasicUsername: string;
    editAuthBasicPassword: string;
    editAuthApiKeyKey: string;
    editAuthApiKeyValue: string;
    editAuthApiKeyLocation: "header" | "query";
    editPreScript: string;
    editPostScript: string;
    editTimeoutMs: number | null;
    editFollowRedirects: boolean;
    editMaxRedirects: number;
    editVerifySsl: boolean;
    editProxyUrl: string;
    editHttpVersion: string;
    editBodyType: "raw" | "form-data" | "x-www-form-urlencoded" | "binary" | "graphql";
    editGraphqlQuery: string;
    editGraphqlVariables: string;
    editFormDataItems: FormDataPart[];
    editUrlEncodedItems: UrlEncodedItem[];
    editBinaryFilePath: string;
    activeEditorTab: "params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "mock" | "code";
    activeResponse: ResponseMeta | null;
    activeResponseBody: string;
    activeResponseTruncated: boolean;
  }

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
  type SidebarSearchScope = "all" | "projects" | "apis";
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
  type ProjectSortField = "name" | "created" | "updated" | "custom";
  type RequestSortField = "name" | "method" | "created" | "updated" | "custom";
  type SortDir = "asc" | "desc";
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

  interface DetailHeaderRow {
    key: string;
    value: string;
    enabled?: boolean;
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

  interface DetailCookieRow {
    name: string;
    value: string;
    domain: string;
    path: string;
    http_only: boolean;
    secure: boolean;
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

  function getEffectiveRequestData(req?: any) {
    const isCurrent = !req || req.id === selectedRequest?.id;
    const method = isCurrent ? editMethod : (req.method || "GET");
    const rawUrl = isCurrent ? editUrl : (req.url || "https://aaeuat.alansari.ae:19443/FintechGateway/api/v1/auth/login");
    const url = (snippetMode === "resolved" && rawUrl.includes("{{host}}"))
      ? rawUrl.replace("{{host}}", "https://uat.alansari.ae")
      : (rawUrl || "https://aaeuat.alansari.ae:19443/FintechGateway/api/v1/auth/login");
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

  async function handleGetSuccessfulResponse() {
    sendMenuOpen = false;
    await sendCurrentRequest();
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
  function handleExploreApiCapabilities() {
    sendMenuOpen = false;
    sidebarSection = "specs";
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
  type UrlToken = { type: "text"; text: string } | { type: "var"; name: string; raw: string };
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

  function areQueryParamsEqual(edit: typeof editQueryParams, orig: QueryParamEntry[] | undefined | null): boolean {
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
    const name = current ? `${current.name} (Fork ${envForkCount + 1})` : "New Environment Fork";
    await quickCreateEnvironment(name);
    envForkCount++;
    exportFeedback = `Created fork "${name}"`;
    setTimeout(() => { exportFeedback = ""; }, 3000);
  }

  function shareCurrentEnvironment() {
    const vars = environmentVariables.reduce((acc, v) => ({ ...acc, [v.key]: v.value }), {});
    copyTextToClipboard(JSON.stringify(vars, null, 2));
    exportFeedback = "Environment variables JSON copied to clipboard!";
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
  interface AutocompleteItem {
    insertText: string;
    label: string;
    detail?: string;
  }
  interface AutocompleteState {
    mode: "var" | "pm" | "header";
    items: AutocompleteItem[];
    activeIndex: number;
    top: number;
    left: number;
    targetEl: HTMLInputElement | HTMLTextAreaElement;
    replaceStart: number;
    replaceEnd: number;
    setValue: (value: string) => void;
  }
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
</script>

{#if confirmDialog.show}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) confirmDialog.show = false; }}
    onkeydown={(e) => { if (e.key === "Escape") confirmDialog.show = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container">
      <div class="modal-header">
        <h3>{confirmDialog.title}</h3>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (confirmDialog.show = false)}>{@render iconClose()}</button>
      </div>
      <div class="modal-body">
        <p>{confirmDialog.message}</p>
      </div>
      <div class="modal-footer">
        <button type="button" onclick={() => (confirmDialog.show = false)}>{t("request.cancel")}</button>
        <button type="button" class="btn-primary" onclick={() => { confirmDialog.show = false; confirmDialog.onConfirm(); }}>{t("common.confirm")}</button>
      </div>
    </div>
  </div>
{/if}

<div
  class="app-shell"
  data-theme={themeMode}
  style="{surfaceStyleOverride(surfaceTint, themeMode)} {accentStyleOverride(accentColor)} {fontStyleOverride(headingFontOverride, bodyFontOverride)} {textColorStyleOverride(textColorOverride)}"
>
  <header class="topbar">
    <div class="topbar-left">
      <div class="topbar-nav-arrows">
        <button type="button" class="nav-arrow-btn" title="Back" onclick={() => { if (typeof window !== "undefined") window.history.back(); }}>{@render iconArrowLeft()}</button>
        <button type="button" class="nav-arrow-btn" title="Forward" onclick={() => { if (typeof window !== "undefined") window.history.forward(); }}>{@render iconArrowRight()}</button>
        <button type="button" class="nav-arrow-btn" title="Home" onclick={() => (activeScreen = "workspace")}>{@render iconHome()}</button>
      </div>
      <div class="menu-wrap topbar-workspace-wrap">
        <button
          type="button"
          class="topbar-workspace-btn"
          onclick={() => (workspacePickerOpen = !workspacePickerOpen)}
        >
          <span class="workspace-avatar-icon">{@render iconUser()}</span>
          <span class="workspace-name">{workspaces.find((w) => w.id === activeWorkspaceId)?.name ?? t("workspace.defaultName")}</span>
          {@render iconChevronDown()}
        </button>
        {#if workspacePickerOpen}
          <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (workspacePickerOpen = false)}></button>
          <div class="dropdown-menu workspace-picker-menu">
            {#each workspaces as ws (ws.id)}
              {#if renamingWorkspaceId === ws.id}
                <form class="inline-form" onsubmit={submitRenameWorkspace}>
                  <input bind:value={renameWorkspaceValue} use:focusOnMount onblur={submitRenameWorkspace} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingWorkspaceId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                <div class="dropdown-menu-item workspace-picker-item" class:active={ws.id === activeWorkspaceId}>
                  <button type="button" class="workspace-picker-item-btn" onclick={() => selectWorkspace(ws.id)}>{ws.name}</button>
                  <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameWorkspace(ws)}>{@render iconEdit()}</button>
                </div>
              {/if}
            {/each}
            <button type="button" class="dropdown-menu-item" onclick={quickCreateWorkspace}>+ {t("workspace.newWorkspace")}</button>
          </div>
        {/if}
      </div>
    </div>
    <div class="topbar-center">
      <button type="button" class="topbar-search-pill" title={t("topbar.searchPlaceholder")} onclick={openPalette}>
        {@render iconSearch()}
        <span class="topbar-search-text">Search</span>
        <span class="topbar-search-kbd">&#8984;K</span>
      </button>
    </div>
    <div class="topbar-right">
      <button type="button" class="topbar-btn topbar-btn-subtle" onclick={() => (showInviteModal = true)}>Invite</button>
      <button type="button" class="topbar-btn topbar-btn-upgrade" onclick={() => (showUpgradeModal = true)}>Upgrade</button>
      <button
        type="button"
        class="topbar-btn topbar-btn-ai"
        title={aiConfigured ? t("topbar.askAiTitle") : t("topbar.setupAiTitle")}
        onclick={() => (showAiPanel = true)}
      >
        {@render iconSparkle()} {aiConfigured ? t("topbar.askAi") : t("topbar.setupAi")}
      </button>
      <button
        type="button"
        class="topbar-btn"
        title={t("topbar.importTitle")}
        onclick={() => { importActiveTab = "collection"; collectionImportReport = null; collectionImportError = ""; showImportDialog = true; }}
      >
        {@render iconImport()} {t("topbar.import")}
      </button>
      <div class="menu-wrap">
        <button type="button" class="topbar-icon-btn" title="Notifications" onclick={() => (notificationsOpen = !notificationsOpen)}>{@render iconBell()}</button>
        {#if notificationsOpen}
          <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (notificationsOpen = false)}></button>
          <div class="dropdown-menu notifications-menu" style="right: 0; min-width: 280px; padding: 8px 0;">
            <div style="padding: 6px 14px; font-weight: 600; font-size: 11px; color: #888; border-bottom: 1px solid #333;">NOTIFICATIONS</div>
            <div class="notification-item" style="padding: 10px 14px; border-bottom: 1px solid #282828; font-size: 12px;">
              <div style="font-weight: 600; color: #e6e6e6;">Workspace Synced</div>
              <div style="color: #999; font-size: 11px; margin-top: 2px;">Default workspace is up to date.</div>
            </div>
            <div class="notification-item" style="padding: 10px 14px; font-size: 12px;">
              <div style="font-weight: 600; color: #e6e6e6;">Welcome to Postman</div>
              <div style="color: #999; font-size: 11px; margin-top: 2px;">Start creating requests or import a collection.</div>
            </div>
          </div>
        {/if}
      </div>
      <button
        type="button"
        class="topbar-icon-btn"
        title={t("settings.title")}
        onclick={() => (activeScreen = activeScreen === "settings" ? "workspace" : "settings")}
      >
        {@render iconSettings()}
      </button>
      <div class="menu-wrap" style="z-index: 60;">
        <button
          type="button"
          class="topbar-avatar"
          title="Account (Developer)"
          onclick={() => (accountMenuOpen = !accountMenuOpen)}
          aria-label="Account menu"
          aria-expanded={accountMenuOpen}
        >
          <span>D</span>
        </button>
        {#if accountMenuOpen}
          <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (accountMenuOpen = false)}></button>
          <div class="dropdown-menu account-menu" style="right: 0; min-width: 200px; padding: 6px 0;">
            <div style="padding: 8px 14px; border-bottom: 1px solid #333;">
              <div style="font-weight: 600; font-size: 12px; color: #fff;">Developer</div>
              <div style="font-size: 11px; color: #888;">dev@postman.local</div>
            </div>
            <button type="button" class="dropdown-menu-item" onclick={() => { accountMenuOpen = false; activeScreen = "settings"; }}>Preferences</button>
            <button type="button" class="dropdown-menu-item" onclick={() => { accountMenuOpen = false; exportFeedback = "Active profile is local."; setTimeout(() => { exportFeedback = ""; }, 2500); }}>Account Details</button>
          </div>
        {/if}
      </div>
      <div class="window-controls">
        <button type="button" class="win-ctrl-btn" title="Minimize" onclick={handleWindowMinimize}>&minus;</button>
        <button type="button" class="win-ctrl-btn" title="Maximize" onclick={handleWindowMaximize}>&#9634;</button>
        <button type="button" class="win-ctrl-btn win-ctrl-close" title="Close" onclick={handleWindowClose}>&#10005;</button>
      </div>
    </div>
  </header>
  

  {#snippet noProjectPicker(screenName: string)}
    <section class="screen-page">
      <div class="screen-page-header">
        <span class="screen-kicker">{screenName}</span>
        <h1 class="screen-title">{t("env.pickProject")}</h1>
      </div>
      {#if projects.length}
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
            {#each projects as p (p.id)}
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
  {/snippet}

  {#snippet projectSwitcher()}
    <select
      class="project-picker-select project-picker-select-sm"
      value={selectedProjectId ?? ""}
      onchange={(e) => {
        const id = (e.target as HTMLSelectElement).value;
        if (id) selectProject(id);
      }}
    >
      {#each projects as p (p.id)}
        <option value={p.id}>{p.name}</option>
      {/each}
    </select>
  {/snippet}

  {#snippet requestRow(req: RequestSummary)}
    {@const sampleCount = sampleResponsesByRequestId.get(req.id)?.length}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <li
      class="request-item-wrapper"
      draggable={requestSortField === "custom"}
      ondragstart={(e) => { e.stopPropagation(); onRequestDragStart(req.id); }}
      ondragover={(e) => { e.stopPropagation(); if (requestSortField === "custom") e.preventDefault(); }}
      ondrop={(e) => { e.stopPropagation(); e.preventDefault(); onRequestDrop(req.id, req.folder_id); }}
    >
      <div class="request-item" class:active={req.id === selectedRequest?.id} oncontextmenu={(e) => openRequestContextMenu(e, req)}>
        {#if renamingRequestId === req.id && req.id !== selectedRequest?.id}
          <form class="inline-form" onsubmit={submitRenameRequest}>
            <input bind:value={renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
            <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
            <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingRequestId = null)}>{@render iconClose()}</button>
          </form>
        {:else}
          <button
            type="button"
            class="tree-expand-btn"
            title={expandedTreeRequestIds.has(req.id) ? t("sidebar.collapseSamples") : t("sidebar.expandSamples")}
            onclick={() => toggleTreeRequestExpanded(req.id)}
          >{#if expandedTreeRequestIds.has(req.id)}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
          <button type="button" class="request-link" onclick={() => openRequest(req.id)} ondblclick={() => startRenameRequest(req.id, req.name)}>
            <span class="sidebar-method method-{req.method.toLowerCase()}">{req.method}</span>
            <span class="request-name">{req.name}</span>
            {#if sampleCount}<span class="tab-badge">{sampleCount}</span>{/if}
          </button>
          <button class="icon-btn icon-btn-ghost" title={t("sidebar.duplicate")} onclick={() => duplicateRequest(req.id)}>{@render iconCopy()}</button>
          <button class="icon-btn icon-btn-ghost" title={t("sidebar.delete")} onclick={() => deleteRequest(req.id)}>{@render iconTrash()}</button>
        {/if}
      </div>
      {#if expandedTreeRequestIds.has(req.id)}
        <ul class="sample-tree-list">
          {#each sampleResponsesByRequestId.get(req.id) ?? [] as sr (sr.id)}
            <li class="sample-tree-item">
              {#if renamingSampleResponseId === sr.id}
                <form class="inline-form" onsubmit={(e) => { e.preventDefault(); submitRenameSampleResponse(req.id); }}>
                  <input bind:value={renameSampleResponseValue} use:focusOnMount onblur={() => submitRenameSampleResponse(req.id)} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingSampleResponseId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                <button
                  type="button"
                  class="sample-tree-link"
                  title={t("sample.badge")}
                  onclick={() => openSampleResponse(req.id, sr)}
                  ondblclick={() => startRenameSampleResponse(sr)}
                >
                  <span class="status-chip" class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>{sr.status}</span>
                  <span class="sample-tree-name">{sr.name}</span>
                </button>
                <button class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameSampleResponse(sr)}>{@render iconEdit()}</button>
                <button class="icon-btn icon-btn-ghost" title={t("sample.delete")} onclick={() => deleteSampleResponseAction(req.id, sr.id)}>{@render iconTrash()}</button>
              {/if}
            </li>
          {:else}
            <li class="empty">{t("sidebar.noSamplesYet")}</li>
          {/each}
        </ul>
      {/if}
    </li>
  {/snippet}

  <!-- Recursive: a folder can contain child folders (unlimited depth), each independently
       collapsible/expandable via the same expandedFolderIds set as its parent — clicking a
       folder toggles it open/closed regardless of how deep it's nested. -->
  {#snippet folderNode(project: Project, folder: Folder)}
    {@const isExpanded = expandedFolderIds.has(folder.id)}
    {@const childFolders = foldersByParentId.get(folder.id) ?? []}
    {@const childRequests = requestsByFolderId.get(folder.id) ?? []}
    <div class="folder-node">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="folder-row"
        draggable={requestSortField === "custom"}
        ondragstart={(e) => { e.stopPropagation(); onFolderDragStart(folder.id); }}
        ondragover={(e) => { e.stopPropagation(); if (requestSortField === "custom") e.preventDefault(); }}
        ondrop={(e) => { e.stopPropagation(); e.preventDefault(); onFolderDrop(folder.id, folder.parent_folder_id); }}
      >
        {#if renamingFolderId === folder.id}
          <form class="inline-form" onsubmit={submitRenameFolder}>
            <input bind:value={renameFolderValue} use:focusOnMount onblur={submitRenameFolder} />
            <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
            <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingFolderId = null)}>{@render iconClose()}</button>
          </form>
        {:else}
          <button
            type="button"
            class="tree-expand-btn"
            title={isExpanded ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
            onclick={() => toggleFolderExpanded(folder.id)}
          >{#if isExpanded}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
          <button type="button" class="folder-link" onclick={() => toggleFolderExpanded(folder.id)} ondblclick={() => startRenameFolder(folder)}>
            <span class="folder-icon">{#if isExpanded}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
            <span class="project-name">{folder.name}</span>
            {#if childRequests.length}<span class="request-count-badge">{childRequests.length}</span>{/if}
          </button>
          <div class="project-row-actions">
            <button class="icon-btn" title={t("sidebar.addSubfolder")} onclick={() => quickCreateFolder(project.id, folder.id)}>{@render iconFolderPlus()}</button>
            <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id, folder.id)}>+</button>
            <button class="icon-btn" title={t("sidebar.rename")} onclick={() => startRenameFolder(folder)}>{@render iconEdit()}</button>
            <button class="icon-btn" title={t("sidebar.deleteFolder")} onclick={() => deleteFolderAction(folder.id)}>{@render iconTrash()}</button>
          </div>
        {/if}
      </div>
      {#if isExpanded}
        <div class="folder-children">
          {#each childFolders as child (child.id)}
            {@render folderNode(project, child)}
          {/each}
          <ul class="request-list">
            {#each childRequests as req (req.id)}
              {@render requestRow(req)}
            {:else}
              {#if !childFolders.length}
                <li class="empty">{t("sidebar.noRequestsInFolder")}</li>
              {/if}
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/snippet}

  <!-- Lightweight, read-mostly counterpart to folderNode/requestRow for a project that's
       expanded in the sidebar but not the active one — browse and open, no rename/delete/create
       (open a request here promotes its project to active, which gets you the full tree). -->
  {#snippet secondaryFolderNode(projectId: string, folder: Folder, cache: { requests: RequestSummary[]; folders: Folder[] })}
    {@const isExpanded = expandedFolderIds.has(folder.id)}
    {@const childFolders = cache.folders.filter((f) => f.parent_folder_id === folder.id)}
    {@const childRequests = sortRequestList(cache.requests.filter((r) => r.folder_id === folder.id))}
    <div class="folder-node">
      <div class="folder-row">
        <button
          type="button"
          class="tree-expand-btn"
          title={isExpanded ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
          onclick={() => toggleFolderExpanded(folder.id)}
        >{#if isExpanded}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
        <button type="button" class="folder-link" onclick={() => toggleFolderExpanded(folder.id)}>
          <span class="folder-icon">{#if isExpanded}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
          <span class="project-name">{folder.name}</span>
          {#if childRequests.length}<span class="request-count-badge">{childRequests.length}</span>{/if}
        </button>
      </div>
      {#if isExpanded}
        <div class="folder-children">
          {#each childFolders as child (child.id)}
            {@render secondaryFolderNode(projectId, child, cache)}
          {/each}
          <ul class="request-list">
            {#each childRequests as req (req.id)}
              {@render secondaryRequestRow(projectId, req)}
            {:else}
              {#if !childFolders.length}
                <li class="empty">{t("sidebar.noRequestsInFolder")}</li>
              {/if}
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/snippet}

  {#snippet secondaryRequestRow(projectId: string, req: RequestSummary)}
    <li class="request-item-wrapper">
      <div class="request-item" oncontextmenu={(e) => openRequestContextMenu(e, req)}>
        <button type="button" class="request-link" onclick={() => openSecondaryRequest(projectId, req.id)}>
          <span class="sidebar-method method-{req.method.toLowerCase()}">{req.method}</span>
          <span class="request-name">{req.name}</span>
        </button>
      </div>
    </li>
  {/snippet}

  {#snippet expandedResponseView()}
    <section class="screen-page">
      <div class="screen-page-header">
        <span class="screen-kicker">{t("response.title")}</span>
        <div class="screen-title-row">
          {#if selectedRequest}
            <h1 class="screen-title">{selectedRequest.method} {selectedRequest.name}</h1>
          {:else}
            <h1 class="screen-title">{t("response.noRequestOpen")}</h1>
          {/if}
          <button type="button" class="btn-ghost btn-xs" title={t("response.backToWorkspace")} onclick={() => (responseExpanded = false)}>{t("response.backToWorkspace")}</button>
        </div>
      </div>

      {#if !selectedRequest}
        <div class="screen-empty">
          <div class="empty-icon">{@render iconInboxEmpty()}</div>
          <p>{t("response.openFromWorkspace")}</p>
        </div>
      {:else if !activeResponse}
        <div class="screen-empty">
          <div class="empty-icon">{@render iconInboxEmpty()}</div>
          <p>{t("response.sendToSee")}</p>
        </div>
      {:else}
        <div class="response-screen-body">
          <aside class="response-screen-side">
            <div class="response-screen-stat-block">
              <span class="screen-kicker">{t("response.status")}</span>
              <div class="response-screen-status" class:status-ok={activeResponse.status < 400} class:status-err={activeResponse.status >= 400}>
                {activeResponse.status} {activeResponse.status_text}
              </div>
              <div class="response-screen-path">{selectedRequest.method} {selectedRequest.url}</div>
            </div>
            <div class="response-screen-metrics">
              <div class="response-screen-metric">
                <span class="screen-kicker">{t("response.time")}</span>
                <div class="response-screen-metric-value">{activeResponse.duration_ms} ms</div>
              </div>
              <div class="response-screen-metric">
                <span class="screen-kicker">{t("response.size")}</span>
                <div class="response-screen-metric-value">{formatByteSize(activeResponse.body_size)}</div>
              </div>
              <div class="response-screen-metric">
                <span class="screen-kicker">{t("response.heldInRam")}</span>
                <div class="response-screen-metric-value">{formatByteSize(Math.min(activeResponse.body_size, 256 * 1024))}</div>
              </div>
              <div class="response-screen-metric">
                <span class="screen-kicker">{t("response.storage")}</span>
                <div class="response-screen-metric-value">{activeResponse.body_size > 256 * 1024 ? t("response.disk") : t("response.inline")}</div>
              </div>
            </div>
            <div class="response-screen-tests">
              <span class="screen-kicker">{t("response.tests")}</span>
              {#if activeResponseTests.length}
                {#each activeResponseTests as test, i (i)}
                  <div class="response-screen-test-row">
                    <span class:status-ok={test.passed} class:status-err={!test.passed}>{test.passed ? t("response.pass") : t("response.fail")}</span>
                    <span>{test.name}</span>
                  </div>
                {/each}
              {:else}
                <p class="screen-empty-inline">{t("response.noTestsRan")}</p>
              {/if}
            </div>
          </aside>

          <div class="response-screen-main">
            <div class="response-subtabs">
              <button type="button" class="response-subtab" class:active={responseSubTab === "body"} onclick={() => (responseSubTab = "body")}>{t("response.body")}</button>
              <button type="button" class="response-subtab" class:active={responseSubTab === "headers"} onclick={() => (responseSubTab = "headers")}>
                {t("response.headers")} {#if activeResponse.headers?.length}<span class="tab-badge">{activeResponse.headers.length}</span>{/if}
              </button>
              <button type="button" class="response-subtab" class:active={responseSubTab === "cookies"} onclick={() => (responseSubTab = "cookies")}>
                {t("response.cookies")} {#if activeResponse.cookies?.length}<span class="tab-badge">{activeResponse.cookies.length}</span>{/if}
              </button>
              <button type="button" class="response-subtab" class:active={responseSubTab === "history"} onclick={() => (responseSubTab = "history")}>
                {t("response.history")} {#if responseHistory.length}<span class="tab-badge">{responseHistory.length}</span>{/if}
              </button>
              <div class="response-stat-spacer"></div>
              {#if responseSubTab === "body"}
                <button type="button" class="btn-ghost btn-xs" class:active={responseViewMode === "pretty"} onclick={() => (responseViewMode = "pretty")}>{t("response.pretty")}</button>
                <button type="button" class="btn-ghost btn-xs" class:active={responseViewMode === "raw"} onclick={() => (responseViewMode = "raw")}>{t("response.raw")}</button>
                {#if responseBodyIsHtml}
                  <button type="button" class="btn-ghost btn-xs" class:active={responseViewMode === "preview"} onclick={() => (responseViewMode = "preview")}>{t("response.preview")}</button>
                {/if}
              {/if}
              <button type="button" class="btn-ghost btn-xs" onclick={copyResponseBody}>{t("response.copyAction")}</button>
              <button type="button" class="btn-ghost btn-xs" onclick={downloadResponseBody}>{t("response.saveToFile")}</button>
            </div>
            <div class="response-screen-content">
              {#if responseSubTab === "body"}
                {#if responseViewMode === "preview" && responseBodyIsHtml}
                  <iframe class="response-preview-frame" title={t("response.preview")} sandbox="" srcdoc={activeResponseBody}></iframe>
                {:else if responseBodyIsJson}
                  <pre class="body-view screen-body-view">{@html highlightedResponseBody}</pre>
                {:else}
                  <pre class="body-view screen-body-view">{prettyResponseBody}</pre>
                {/if}
                {#if activeResponseTruncated}<p class="hint">{t("response.truncated")}</p>{/if}
              {:else if responseSubTab === "headers"}
                {#if activeResponse.headers?.length}
                  <div class="headers-list">
                    {#each activeResponse.headers as h}
                      <div class="header-line"><strong>{h.key}:</strong> {h.value}</div>
                    {/each}
                  </div>
                {:else}
                  <p class="empty">{t("response.noHeaders")}</p>
                {/if}
              {:else if responseSubTab === "cookies"}
                {#if activeResponse.cookies?.length}
                  <div class="headers-list">
                    {#each activeResponse.cookies as c}
                      <div class="header-line"><strong>{c.name}:</strong> {c.value}</div>
                    {/each}
                  </div>
                {:else}
                  <p class="empty">{t("response.noCookies")}</p>
                {/if}
              {:else if responseSubTab === "history"}
                {#if responseHistory.length}
                  <ul class="response-history-list">
                    {#each responseHistory as r (r.id)}
                      <li>
                        <button type="button" class="response-history-row" onclick={() => openHistoryResponse(r.id)}>
                          <span class="status-chip" class:status-ok={r.status < 400} class:status-err={r.status >= 400}>{r.status}</span>
                          <span class="response-history-duration">{r.duration_ms} ms</span>
                          <span class="response-history-time">{new Date(r.created_at).toLocaleString()}</span>
                        </button>
                      </li>
                    {/each}
                  </ul>
                {:else}
                  <p class="empty">{t("history.empty")}</p>
                {/if}
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </section>
  {/snippet}

  <div class="screen-area">
  {#if activeScreen === "workspace"}
  {#if responseExpanded}
    {@render expandedResponseView()}
  {:else}
  <div class="app">
  

  {#if exportFeedback}
    <div class="success-banner"><span class="banner-message">{@render iconCheckCircle()} {exportFeedback}</span></div>
  {/if}
  {#if errorMessage}
    <div class="error-banner">
      <span class="banner-message">{@render iconXCircle()} {errorMessage}</span>
      <button type="button" class="dismiss-btn" title={t("error.dismiss")} onclick={() => (errorMessage = "")}>{@render iconClose()}</button>
    </div>
  {/if}

  <div class="workspace">
    {#if sidebarVisible}
    <aside class="sidebar" style="width: {sidebarWidth}px">
      <div class="sidebar-top-icons">
        <button type="button" class="sidebar-top-icon-btn" class:active={sidebarSection === "collections"} title="Collections & APIs" onclick={() => { sidebarSection = "collections"; activeScreen = "workspace"; }}>{@render iconCube()}</button>
        <button type="button" class="sidebar-top-icon-btn" class:active={sidebarSection === "environments"} title="Environments" onclick={() => { sidebarSection = "environments"; environmentsAccordionOpen = true; activeScreen = "workspace"; ensureAllEnvVariablesLoaded(); const targetId = selectedEnvironmentId || allEnvironments[0]?.id; if (targetId && !expandedEnvIds.has(targetId)) toggleEnvExpand(targetId); }}>{@render iconGlobe()}</button>
        <button type="button" class="sidebar-top-icon-btn" class:active={sidebarSection === "history"} title="History" onclick={() => { sidebarSection = "history"; activeScreen = "workspace"; refreshProjectHistory(); }}>{@render iconClock()}</button>
        <button type="button" class="sidebar-top-icon-btn" class:active={sidebarSection === "projects"} title="Projects & Workspaces" onclick={() => { sidebarSection = "projects"; activeScreen = "workspace"; }}>{@render iconFolder()}</button>
      </div>

      {#if sidebarSection === "history"}
        <div class="sidebar-filter-bar">
          <div class="sidebar-search-box">
            {@render iconSearch()}
            <input
              type="search"
              placeholder="Search history..."
              bind:value={historySearchQuery}
              class="sidebar-search-input"
            />
            {#if historySearchQuery.trim()}
              <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{filteredProjectHistory.length}/{projectHistory.length}</span>
            {/if}
          </div>
          {#if projectHistory.length > 0}
            <button type="button" class="icon-btn" title="Clear History" onclick={clearHistory}>
              {@render iconTrash()}
            </button>
          {/if}
        </div>

        <div class="sidebar-history-list" style="max-height: calc(100vh - 160px); overflow-y: auto; padding: 4px 0;">
          {#if historyLoading}
            <p class="empty" style="padding: 12px;">{t("history.loading")}</p>
          {:else if filteredProjectHistory.length === 0}
            <div class="sidebar-empty-state" style="padding: 24px 12px; text-align: center;">
              <span style="font-size: 24px; opacity: 0.5;">⏱</span>
              <p class="empty" style="margin: 6px 0 0; font-size: 12px;">
                {projectHistory.length === 0 ? "No requests sent yet" : "No history matches"}
              </p>
              <p style="font-size: 11px; color: #888; margin: 4px 0 0;">
                Send a request to see it in your history
              </p>
            </div>
          {:else}
            {#each filteredProjectHistory as h (h.id)}
              <button
                type="button"
                class="sidebar-history-item"
                onclick={async () => {
                  await openRequest(h.request_id);
                  await openHistoryResponse(h.id);
                  activeScreen = "workspace";
                  responseExpanded = true;
                }}
              >
                <div class="history-item-top">
                  <span class="palette-item-method method-{h.method.toLowerCase()}">{h.method}</span>
                  <span class="status-chip" class:status-ok={h.status < 400} class:status-err={h.status >= 400}>{h.status}</span>
                  <div class="response-stat-spacer"></div>
                  <span class="history-duration">{h.duration_ms} ms</span>
                </div>
                <div class="history-item-url" title={h.url}>
                  {h.url}
                </div>
                <div class="history-item-bottom">
                  <span class="history-req-name">{h.request_name}</span>
                  <div class="response-stat-spacer"></div>
                  <span class="history-time">{formatRelativeTime(h.created_at)}</span>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      {:else if sidebarSection === "environments"}
        <div class="sidebar-filter-bar">
          <div class="sidebar-search-box">
            {@render iconSearch()}
            <input
              type="search"
              placeholder={t("env.searchEnvironments")}
              bind:value={envSidebarSearchQuery}
              class="sidebar-search-input"
            />
            {#if envSidebarSearchQuery.trim()}
              <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{filteredSidebarEnvironments.length}/{allEnvironments.length}</span>
            {/if}
          </div>
          <button type="button" class="icon-btn" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
        </div>

        <!-- Environments search options row: All, Name, Key, Values -->
        <div class="palette-scope-row sidebar-scope-row" style="gap: 4px; padding: 4px 10px 8px;">
          <span style="font-size: 10px; color: #888; text-transform: uppercase; font-weight: 600; margin-right: 2px;">Search in:</span>
          <button
            type="button"
            class="palette-scope-btn"
            class:active={envSidebarSearchScope === "all"}
            onclick={() => (envSidebarSearchScope = "all")}
          >All</button>
          <button
            type="button"
            class="palette-scope-btn"
            class:active={envSidebarSearchScope === "name"}
            onclick={() => (envSidebarSearchScope = "name")}
          >Name</button>
          <button
            type="button"
            class="palette-scope-btn"
            class:active={envSidebarSearchScope === "key"}
            onclick={() => (envSidebarSearchScope = "key")}
          >Key</button>
          <button
            type="button"
            class="palette-scope-btn"
            class:active={envSidebarSearchScope === "values"}
            onclick={() => (envSidebarSearchScope = "values")}
          >Values</button>
        </div>

        <div class="sidebar-env-list" style="max-height: calc(100vh - 170px); overflow-y: auto; padding: 4px 0;">
          <div class="sidebar-accordion-section-title" style="cursor: default;">
            <span>ENVIRONMENTS ({filteredSidebarEnvironments.length})</span>
            <div class="response-stat-spacer"></div>
            <button type="button" class="icon-btn icon-btn-ghost" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
          </div>

          {#if filteredSidebarEnvironments.length === 0}
            <div class="sidebar-empty-state" style="padding: 24px 12px; text-align: center;">
              <span style="font-size: 24px; opacity: 0.5;">🌐</span>
              <p class="empty" style="margin: 6px 0 0; font-size: 12px;">
                {allEnvironments.length === 0 ? "No environments yet" : "No environments match"}
              </p>
              {#if envSidebarSearchQuery.trim()}
                <p style="font-size: 11px; color: #888; margin: 4px 0 0;">
                  No environment matches "{envSidebarSearchQuery}" in {envSidebarSearchScope.toUpperCase()}
                </p>
                <button
                  type="button"
                  class="btn-xs-primary"
                  style="margin-top: 8px;"
                  onclick={() => (envSidebarSearchQuery = "")}
                >Clear Search</button>
              {:else}
                <button
                  type="button"
                  class="btn-xs-primary"
                  style="margin-top: 8px;"
                  onclick={quickCreateEnvironment}
                >+ Create Environment</button>
              {/if}
            </div>
          {:else}
            <!-- Starred / Favorite Environments Section -->
            {#if filteredSidebarFavoriteEnvs.length > 0}
              <div class="sidebar-fav-header">
                <span>★ FAVORITES ({filteredSidebarFavoriteEnvs.length})</span>
              </div>
              {#each filteredSidebarFavoriteEnvs as env (env.id)}
                {@const q = envSidebarSearchQuery.trim().toLowerCase()}
                {@const vars = envVariablesCache.get(env.id) || []}
                {@const hasVarMatch = q !== "" && vars.some(v => v.key.toLowerCase().includes(q) || (v.value || "").toLowerCase().includes(q))}
                {@const isAutoExpanded = expandedEnvIds.has(env.id) || hasVarMatch}
                <div class="sidebar-env-item-row" class:active={selectedEnvironmentId === env.id}>
                  <button
                    type="button"
                    class="env-expand-btn"
                    title={isAutoExpanded ? "Collapse variables" : "Expand variables"}
                    onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
                  >{#if isAutoExpanded}▼{:else}▶{/if}</button>
                  <button
                    type="button"
                    class="sidebar-env-link"
                    onclick={() => {
                      toggleEnvExpand(env.id);
                      openEnvironmentTab(env);
                    }}
                  >
                    <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
                    <span class="env-name-text">{env.name}</span>
                    {#if selectedEnvironmentId === env.id}
                      <span class="env-check-icon" title="Active">✓</span>
                    {/if}
                  </button>
                  <button
                    type="button"
                    class="env-star-btn favorited"
                    title="Remove from favorites"
                    onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                  >★</button>
                </div>
                {#if isAutoExpanded}
                  <div class="sidebar-env-vars-container">
                    {#each vars as v (v.id)}
                      {@const isKeyMatch = q !== "" && (envSidebarSearchScope === "all" || envSidebarSearchScope === "key") && v.key.toLowerCase().includes(q)}
                      {@const isValMatch = q !== "" && (envSidebarSearchScope === "all" || envSidebarSearchScope === "values") && (v.value || "").toLowerCase().includes(q)}
                      <div class="sidebar-env-var-item" class:matched-var={isKeyMatch || isValMatch} title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                        <span class="var-key-text" style={isKeyMatch ? "color: #ff9800; font-weight: 700;" : ""}>{v.key}:</span>
                        <span class="var-val-text" style={isValMatch ? "color: #ffeb3b; font-weight: 700;" : ""}>{v.is_secret ? "••••••••" : (v.value || '""')}</span>
                      </div>
                    {:else}
                      <div class="sidebar-env-var-item empty">No variables</div>
                    {/each}
                  </div>
                {/if}
              {/each}
              <div class="sidebar-fav-divider"></div>
            {/if}

            <!-- All Environments (matching search) -->
            {#each filteredSidebarEnvironments as env (env.id)}
              {@const isFav = isEnvFavorite(env.id)}
              {@const q = envSidebarSearchQuery.trim().toLowerCase()}
              {@const vars = envVariablesCache.get(env.id) || []}
              {@const hasVarMatch = q !== "" && vars.some(v => v.key.toLowerCase().includes(q) || (v.value || "").toLowerCase().includes(q))}
              {@const isAutoExpanded = expandedEnvIds.has(env.id) || hasVarMatch}
              <div class="sidebar-env-item-row" class:active={selectedEnvironmentId === env.id}>
                <button
                  type="button"
                  class="env-expand-btn"
                  title={isAutoExpanded ? "Collapse variables" : "Expand variables"}
                  onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
                >{#if isAutoExpanded}▼{:else}▶{/if}</button>
                <button
                  type="button"
                  class="sidebar-env-link"
                  onclick={() => {
                    toggleEnvExpand(env.id);
                    openEnvironmentTab(env);
                  }}
                >
                  <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
                  <span class="env-name-text">{env.name}</span>
                  {#if selectedEnvironmentId === env.id}
                    <span class="env-check-icon" title="Active">✓</span>
                  {/if}
                </button>
                <button
                  type="button"
                  class="env-star-btn"
                  class:favorited={isFav}
                  title={isFav ? "Remove from favorites" : "Add to favorites"}
                  onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                >{isFav ? "★" : "☆"}</button>
              </div>
              {#if isAutoExpanded}
                <div class="sidebar-env-vars-container">
                  {#each vars as v (v.id)}
                    {@const isKeyMatch = q !== "" && (envSidebarSearchScope === "all" || envSidebarSearchScope === "key") && v.key.toLowerCase().includes(q)}
                    {@const isValMatch = q !== "" && (envSidebarSearchScope === "all" || envSidebarSearchScope === "values") && (v.value || "").toLowerCase().includes(q)}
                    <div class="sidebar-env-var-item" class:matched-var={isKeyMatch || isValMatch} title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                      <span class="var-key-text" style={isKeyMatch ? "color: #ff9800; font-weight: 700;" : ""}>{v.key}:</span>
                      <span class="var-val-text" style={isValMatch ? "color: #ffeb3b; font-weight: 700;" : ""}>{v.is_secret ? "••••••••" : (v.value || '""')}</span>
                    </div>
                  {:else}
                    <div class="sidebar-env-var-item empty">No variables</div>
                  {/each}
                </div>
              {/if}
            {/each}
          {/if}
        </div>
      {:else}
        <div class="sidebar-filter-bar">
          <div class="sidebar-search-box">
            {@render iconSearch()}
            <input
              type="search"
              placeholder={t("sidebar.searchProjects")}
              bind:value={projectSearchQuery}
              class="sidebar-search-input"
            />
            {#if projectSearchQuery.trim()}
              <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{filteredProjects.length}/{projects.length}</span>
            {/if}
          </div>
          <button type="button" class="icon-btn" title={t("sidebar.newProject")} onclick={quickCreateProject}>+</button>
          <div class="menu-wrap">
            <button
              type="button"
              class="icon-btn"
              title={t("sidebar.sortOptions")}
              onclick={() => (projectSortMenuOpen = !projectSortMenuOpen)}
            >{@render iconMoreVertical()}</button>
            {#if projectSortMenuOpen}
              <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (projectSortMenuOpen = false)}></button>
              <div class="dropdown-menu">
                {#each PROJECT_SORT_FIELDS as f (f.field)}
                  <button
                    type="button"
                    class="dropdown-menu-item"
                    class:active={projectSortField === f.field}
                    onclick={() => pickProjectSortField(f.field)}
                  >
                    <span>{t(f.label)}</span>
                    {#if projectSortField === f.field}
                      <span class="sort-dir-indicator">{#if projectSortDir === "asc"}{@render iconArrowUp()}{:else}{@render iconArrowDown()}{/if}</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Search options row: always visible below the search bar -->
        <div class="palette-scope-row sidebar-scope-row">
          <button type="button" class="palette-scope-btn" class:active={projectSearchScope === "all"} onclick={() => (projectSearchScope = "all")}>{t("palette.scopeAll")}</button>
          <button type="button" class="palette-scope-btn" class:active={projectSearchScope === "projects"} onclick={() => (projectSearchScope = "projects")}>{t("palette.scopeProjects")}</button>
          <button type="button" class="palette-scope-btn" class:active={projectSearchScope === "apis"} onclick={() => (projectSearchScope = "apis")}>{t("palette.scopeApis")}</button>
        </div>
        {#if projectSearchScope !== "projects"}
          <div class="palette-scope-row palette-field-row sidebar-scope-row">
            <span class="palette-field-label">{t("palette.fieldsLabel")}</span>
            <button type="button" class="palette-scope-btn" class:active={sidebarSearchFields.name} aria-pressed={sidebarSearchFields.name} onclick={() => toggleSidebarSearchField("name")}>{t("palette.fieldName")}</button>
            <button type="button" class="palette-scope-btn" class:active={sidebarSearchFields.url} aria-pressed={sidebarSearchFields.url} onclick={() => toggleSidebarSearchField("url")}>{t("palette.fieldUrl")}</button>
            <button type="button" class="palette-scope-btn" class:active={sidebarSearchFields.body} aria-pressed={sidebarSearchFields.body} onclick={() => toggleSidebarSearchField("body")}>{t("palette.fieldBody")}</button>
          </div>
        {/if}

        {#if projectSearchScope !== "projects" && projectSearchQuery.trim().length >= 2}
          <ul class="sidebar-api-results">
            {#each sidebarApiResults as r (r.id)}
              <li>
                <button
                  type="button"
                  class="palette-item sidebar-api-result-item"
                  onclick={async () => {
                    await selectProject(r.project_id);
                    openRequest(r.id);
                  }}
                >
                  <span class="palette-item-method">{r.method}</span>
                  <span class="palette-item-label"><span class="palette-item-breadcrumb">{r.project_name} ›</span> {r.name}</span>
                  <div class="response-stat-spacer"></div>
                  <span class="palette-item-hint">{r.url}</span>
                </button>
              </li>
            {:else}
              <li class="empty">{t("palette.noMatches")}</li>
            {/each}
          </ul>
        {/if}

        {#if sidebarSectionsVisible.collections && projectSearchScope !== "apis"}
        <div class="sidebar-accordion-section-title" onclick={() => (collectionsAccordionOpen = !collectionsAccordionOpen)}>
          <span class="accordion-arrow">{#if collectionsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
          <span>COLLECTIONS</span>
          <div class="response-stat-spacer"></div>
          <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.newProject")} onclick={(e) => { e.stopPropagation(); quickCreateProject(); }}>+</button>
        </div>
        {/if}

      {#if sidebarSectionsVisible.collections && collectionsAccordionOpen}
      <div class="project-list">
        {#if projectSearchScope !== "apis"}
        {#each filteredProjects as project (project.id)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="project-node"
            draggable={projectSortField === "custom"}
            class:drag-active={projectSortField === "custom"}
            ondragstart={() => onProjectDragStart(project.id)}
            ondragover={(e) => { if (projectSortField === "custom") e.preventDefault(); }}
            ondrop={(e) => { e.preventDefault(); onProjectDrop(project.id); }}
          >
            <div class="project-row" class:active={project.id === selectedProjectId}>
              {#if renamingProjectId === project.id}
                <form class="inline-form" onsubmit={submitRenameProject}>
                  <input bind:value={renameProjectValue} use:focusOnMount onblur={submitRenameProject} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingProjectId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                {@const isOpen = expandedProjectIds.has(project.id)}
                <button
                  type="button"
                  class="tree-expand-btn"
                  title={isOpen ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
                  onclick={() => toggleProjectExpand(project.id)}
                >{#if isOpen}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
                <button type="button" class="project-link" onclick={() => toggleProjectSelection(project.id)} ondblclick={() => startRenameProject(project)}>
                  <span class="folder-icon">{#if isOpen}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
                  <span class="project-name">{project.name}</span>
                  {#if projectRequestCounts[project.id]}<span class="request-count-badge">{projectRequestCounts[project.id]}</span>{/if}
                </button>
                <div class="project-row-actions" class:force-visible={openProjectMenuId === project.id}>
                  <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id)}>+</button>
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.moreActions")}
                      onclick={() => (openProjectMenuId = openProjectMenuId === project.id ? null : project.id)}
                    >{@render iconMoreVertical()}</button>
                    {#if openProjectMenuId === project.id}
                      <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (openProjectMenuId = null)}></button>
                      <div class="dropdown-menu">
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { openProjectMenuId = null; quickCreateFolder(project.id); }}
                        >{t("sidebar.addFolder")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={async () => {
                            openProjectMenuId = null;
                            await selectProject(project.id);
                            collectionImportTarget = "current";
                            importActiveTab = "collection";
                            collectionImportReport = null;
                            collectionImportError = "";
                            showImportDialog = true;
                          }}
                        >{t("sidebar.importInto")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { openProjectMenuId = null; exportPostmanCollectionAction(project.id); }}
                        >{t("sidebar.exportCollection")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={async () => {
                            openProjectMenuId = null;
                            await selectProject(project.id);
                            await exportProjectFileAction();
                            gitActiveTab = "projectfile";
                            activeScreen = "git";
                          }}
                        >{t("sidebar.exportProjectFile")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { openProjectMenuId = null; startRenameProject(project); }}
                        >{t("sidebar.rename")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { openProjectMenuId = null; deleteProject(project.id); }}
                        >{t("sidebar.delete")}</button>
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            {#if expandedProjectIds.has(project.id) && project.id === selectedProjectId}
              <div class="project-requests">
                <div class="request-search-box">
                  <input
                    type="search"
                    placeholder={t("sidebar.searchRequests")}
                    bind:value={requestSearchQuery}
                    class="request-search-input"
                  />
                  {#if requestSearchQuery}
                    <span class="request-count-badge">{filteredRequests.length}/{requests.length}</span>
                  {/if}
                  {#if !requestSearchQuery && folders.length > 0}
                    <button
                      type="button"
                      class="icon-btn"
                      title={expandedFolderIds.size < folders.length ? t("sidebar.expandAllFolders") : t("sidebar.collapseAllFolders")}
                      onclick={toggleExpandAllFolders}
                    >{#if expandedFolderIds.size < folders.length}{@render iconExpandAll()}{:else}{@render iconCollapseAll()}{/if}</button>
                  {/if}
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.sortOptions")}
                      onclick={() => (requestSortMenuOpen = !requestSortMenuOpen)}
                    >{@render iconMoreVertical()}</button>
                    {#if requestSortMenuOpen}
                      <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (requestSortMenuOpen = false)}></button>
                      <div class="dropdown-menu">
                        {#each REQUEST_SORT_FIELDS as f (f.field)}
                          <button
                            type="button"
                            class="dropdown-menu-item"
                            class:active={requestSortField === f.field}
                            onclick={() => pickRequestSortField(f.field)}
                          >
                            <span>{t(f.label)}</span>
                            {#if requestSortField === f.field}
                              <span class="sort-dir-indicator">{#if requestSortDir === "asc"}{@render iconArrowUp()}{:else}{@render iconArrowDown()}{/if}</span>
                            {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>

                {#if loadingRequests}
                  <p class="hint">{t("rail.loading")}</p>
                {:else if requestSearchQuery}
                  <ul class="request-list">
                    {#each visibleRequests as req (req.id)}
                      {@render requestRow(req)}
                    {:else}
                      <li class="empty">{t("sidebar.noMatchingRequests")}</li>
                    {/each}
                  </ul>

                  {#if totalRequestPages > 1}
                    <div class="request-pagination">
                      <button type="button" title={t("sidebar.prevPage")} disabled={requestPage === 0} onclick={() => (requestPage = Math.max(0, requestPage - 1))}>{@render iconChevronLeft()}</button>
                      <span>{requestPage + 1} / {totalRequestPages}</span>
                      <button type="button" title={t("sidebar.nextPage")} disabled={requestPage >= totalRequestPages - 1} onclick={() => (requestPage = Math.min(totalRequestPages - 1, requestPage + 1))}>{@render iconChevronRight()}</button>
                    </div>
                  {/if}
                {:else}
                  {#each rootFolders as folder (folder.id)}
                    {@render folderNode(project, folder)}
                  {/each}

                  <ul class="request-list">
                    {#each rootRequests as req (req.id)}
                      {@render requestRow(req)}
                    {:else}
                      {#if !folders.length}
                        <li class="empty">{t("sidebar.noRequestsYet")}</li>
                      {/if}
                    {/each}
                  </ul>
                {/if}
              </div>
            {:else if expandedProjectIds.has(project.id) && project.id !== selectedProjectId}
              <div class="project-requests project-requests-secondary">
                {#if secondaryProjectLoading.has(project.id)}
                  <p class="hint">{t("rail.loading")}</p>
                {:else}
                  {@const cache = secondaryProjectCache.get(project.id)}
                  {#if cache}
                    {@const rootFlds = cache.folders.filter((f) => !f.parent_folder_id)}
                    {@const rootReqs = sortRequestList(cache.requests.filter((r) => !r.folder_id))}
                    {#each rootFlds as folder (folder.id)}
                      {@render secondaryFolderNode(project.id, folder, cache)}
                    {/each}
                    <ul class="request-list">
                      {#each rootReqs as req (req.id)}
                        {@render secondaryRequestRow(project.id, req)}
                      {:else}
                        {#if !rootFlds.length}
                          <li class="empty">{t("sidebar.noRequestsYet")}</li>
                        {/if}
                      {/each}
                    </ul>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <p class="empty">{projectSearchQuery ? t("sidebar.noMatchingProjects") : t("sidebar.noProjectsYet")}</p>
        {/each}
        {/if}
      </div>
      {/if}

      {#if sidebarSectionsVisible.environments}
      <div class="sidebar-accordion-section-title" onclick={() => (environmentsAccordionOpen = !environmentsAccordionOpen)}>
        <span class="accordion-arrow">{#if environmentsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>ENVIRONMENTS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New Environment" onclick={(e) => { e.stopPropagation(); quickCreateEnvironment(); }}>+</button>
      </div>
      {#if environmentsAccordionOpen}
        <div class="sidebar-env-list">
          <!-- Starred / Favorite Environments Section -->
          {#if favoriteEnvs.length > 0}
            <div class="sidebar-fav-header">
              <span>★ FAVORITES ({favoriteEnvs.length})</span>
            </div>
            {#each favoriteEnvs as env (env.id)}
              <div class="sidebar-env-item-row" class:active={selectedEnvironmentId === env.id}>
                <button
                  type="button"
                  class="env-expand-btn"
                  title={expandedEnvIds.has(env.id) ? "Collapse variables" : "Expand variables"}
                  onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
                >{#if expandedEnvIds.has(env.id)}▼{:else}▶{/if}</button>
                <button
                  type="button"
                  class="sidebar-env-link"
                  onclick={() => {
                    toggleEnvExpand(env.id);
                    openEnvironmentTab(env);
                  }}
                >
                  <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
                  <span class="env-name-text">{env.name}</span>
                  {#if selectedEnvironmentId === env.id}
                    <span class="env-check-icon" title="Active">✓</span>
                  {/if}
                </button>
                <button
                  type="button"
                  class="env-star-btn favorited"
                  title="Remove from favorites"
                  onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                >★</button>
              </div>
              {#if expandedEnvIds.has(env.id)}
                {@const vars = envVariablesCache.get(env.id) || []}
                <div class="sidebar-env-vars-container">
                  {#each vars as v (v.id)}
                    <div class="sidebar-env-var-item" title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                      <span class="var-key-text">{v.key}:</span>
                      <span class="var-val-text">{v.is_secret ? "••••••••" : (v.value || '""')}</span>
                    </div>
                  {:else}
                    <div class="sidebar-env-var-item empty">No variables</div>
                  {/each}
                </div>
              {/if}
            {/each}
            <div class="sidebar-fav-divider"></div>
          {/if}

          <!-- All Environments -->
          {#each allEnvironments as env (env.id)}
            {@const isFav = isEnvFavorite(env.id)}
            <div class="sidebar-env-item-row" class:active={selectedEnvironmentId === env.id}>
              <button
                type="button"
                class="env-expand-btn"
                title={expandedEnvIds.has(env.id) ? "Collapse variables" : "Expand variables"}
                onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
              >{#if expandedEnvIds.has(env.id)}▼{:else}▶{/if}</button>
              <button
                type="button"
                class="sidebar-env-link"
                onclick={() => {
                  toggleEnvExpand(env.id);
                  openEnvironmentTab(env);
                }}
              >
                <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
                <span class="env-name-text">{env.name}</span>
                {#if selectedEnvironmentId === env.id}
                  <span class="env-check-icon" title="Active">✓</span>
                {/if}
              </button>
              <button
                type="button"
                class="env-star-btn"
                class:favorited={isFav}
                title={isFav ? "Remove from favorites" : "Add to favorites"}
                onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
              >{isFav ? "★" : "☆"}</button>
            </div>
            {#if expandedEnvIds.has(env.id)}
              {@const vars = envVariablesCache.get(env.id) || []}
              <div class="sidebar-env-vars-container">
                {#each vars as v (v.id)}
                  <div class="sidebar-env-var-item" title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                    <span class="var-key-text">{v.key}:</span>
                    <span class="var-val-text">{v.is_secret ? "••••••••" : (v.value || '""')}</span>
                  </div>
                {:else}
                  <div class="sidebar-env-var-item empty">No variables</div>
                {/each}
              </div>
            {/if}
          {/each}
        </div>
      {/if}
      {/if}

      {#if sidebarSectionsVisible.datasets}
      <div class="sidebar-accordion-section-title" onclick={() => (datasetsAccordionOpen = !datasetsAccordionOpen)}>
        <span class="accordion-arrow">{#if datasetsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>DATASETS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New Dataset" onclick={(e) => { e.stopPropagation(); openDatasetTab({ id: 'data-' + Date.now(), name: 'test-dataset.csv' }); }}>+</button>
      </div>
      {#if datasetsAccordionOpen}
        <div class="sidebar-sub-list">
          {#each sampleDatasets as d (d.id)}
            <button type="button" class="sidebar-sub-link" onclick={() => openDatasetTab(d)}>
              <span class="sidebar-sub-icon" style="color: #722ed1;">📊</span>
              <span class="sidebar-sub-name">{d.name}</span>
            </button>
          {/each}
        </div>
      {/if}
      {/if}

      {#if sidebarSectionsVisible.documents}
      <div class="sidebar-accordion-section-title" onclick={() => (documentsAccordionOpen = !documentsAccordionOpen)}>
        <span class="accordion-arrow">{#if documentsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>DOCUMENTS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New Document" onclick={(e) => { e.stopPropagation(); openDocumentTab({ id: 'doc-' + Date.now(), name: 'New Documentation' }); }}>+</button>
      </div>
      {#if documentsAccordionOpen}
        <div class="sidebar-sub-list">
          {#each sampleDocuments as doc (doc.id)}
            <button type="button" class="sidebar-sub-link" onclick={() => openDocumentTab(doc)}>
              <span class="sidebar-sub-icon" style="color: #0cbb52;">📖</span>
              <span class="sidebar-sub-name">{doc.name}</span>
            </button>
          {/each}
        </div>
      {/if}
      {/if}

      {#if sidebarSectionsVisible.specs}
      <div class="sidebar-accordion-section-title" onclick={() => (specsAccordionOpen = !specsAccordionOpen)}>
        <span class="accordion-arrow">{#if specsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>SPECS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New API Spec" onclick={(e) => { e.stopPropagation(); openSpecTab({ id: 'spec-' + Date.now(), name: 'new-spec.yaml' }); }}>+</button>
      </div>
      {#if specsAccordionOpen}
        <div class="sidebar-sub-list">
          {#each sampleSpecs as spec (spec.id)}
            <button type="button" class="sidebar-sub-link" onclick={() => openSpecTab(spec)}>
              <span class="sidebar-sub-icon" style="color: #108ee9;">⚡</span>
              <span class="sidebar-sub-name">{spec.name}</span>
            </button>
          {/each}
        </div>
      {/if}
      {/if}

      {#if sidebarSectionsVisible.mocks}
      <div class="sidebar-accordion-section-title" onclick={() => (mocksAccordionOpen = !mocksAccordionOpen)}>
        <span class="accordion-arrow">{#if mocksAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>MOCKS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New Mock Server" onclick={(e) => { e.stopPropagation(); openMockTab({ id: 'mock-' + Date.now(), name: 'Mock Server ' + (sampleMockServers.length + 1) }); }}>+</button>
      </div>
      {#if mocksAccordionOpen}
        <div class="sidebar-sub-list">
          {#each sampleMockServers as m (m.id)}
            <button type="button" class="sidebar-sub-link" onclick={() => openMockTab(m)}>
              <span class="sidebar-sub-icon" style="color: #fa8c16;">📦</span>
              <span class="sidebar-sub-name">{m.name}</span>
            </button>
          {/each}
        </div>
      {/if}
      {/if}

      {#if sidebarSectionsVisible.flows}
      <div class="sidebar-accordion-section-title" onclick={() => (flowsAccordionOpen = !flowsAccordionOpen)}>
        <span class="accordion-arrow">{#if flowsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
        <span>FLOWS</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="icon-btn icon-btn-ghost" title="New Flow" onclick={(e) => { e.stopPropagation(); openFlowTab({ id: 'flow-' + Date.now(), name: 'New Request Flow' }); }}>+</button>
      </div>
      {#if flowsAccordionOpen}
        <div class="sidebar-sub-list">
          {#each sampleFlows as fl (fl.id)}
            <button type="button" class="sidebar-sub-link" onclick={() => openFlowTab(fl)}>
              <span class="sidebar-sub-icon" style="color: #13c2c2;">🔀</span>
              <span class="sidebar-sub-name">{fl.name}</span>
            </button>
          {/each}
        </div>
      {/if}
      {/if}
      {/if}
    </aside>

    <div
      class="sidebar-resize-handle"
      class:resizing={sidebarResizing}
      onmousedown={startSidebarResize}
      onkeydown={(e) => {
        sidebarManuallyResized = true;
        if (e.key === "ArrowLeft") sidebarWidth = Math.max(200, sidebarWidth - 16);
        else if (e.key === "ArrowRight") sidebarWidth = Math.min(600, sidebarWidth + 16);
      }}
      role="slider"
      aria-orientation="vertical"
      aria-label="Resize sidebar"
      aria-valuenow={sidebarWidth}
      aria-valuemin={200}
      aria-valuemax={600}
      tabindex="0"
    ></div>
    {:else}
    <button type="button" class="sidebar-expand-btn" title={t("sidebar.show")} onclick={() => setSidebarVisible(true)}>{@render iconChevronRight()}</button>
    {/if}

    <main class="main">
      {#if !selectedProjectId && !currentTab && openTabs.length === 0}
        <div class="empty-state">
          <div class="empty-icon">{@render iconFolder()}</div>
          <p>{t("workspace.selectProject")}</p>
        </div>
      {:else if !selectedRequest && !currentTab}
        <div class="empty-state">
          <div class="empty-icon">{@render iconFileText()}</div>
          <p>{t("request.selectPrompt")}</p>
        </div>
      {:else}
        <section class="detail">
          <div class="workbench-header-strip">
            <div class="request-tabs-bar" oncontextmenu={(e) => { if (e.target === e.currentTarget) { e.preventDefault(); openTabContextMenu(e, null); } }}>
              {#each openTabs as tab (tab.id)}
                <button
                  type="button"
                  class="request-tab-pill"
                  class:active={(activeTabId ? tab.id === activeTabId : tab.id === selectedRequest?.id)}
                  class:dirty={isTabDirty(tab.id)}
                  onclick={() => selectTab(tab)}
                  oncontextmenu={(e) => { e.preventDefault(); openTabContextMenu(e, tab); }}
                  onmousedown={(e) => {
                    if (e.button === 1) {
                      e.preventDefault();
                      closeTabAction(tab.id);
                    }
                  }}
                >
                  {#if tab.tabType === "env"}
                    <span class="tab-env-icon" style="color: #ff6c37; font-size: 11px; margin-right: 4px;">📄</span>
                  {:else if tab.tabType === "doc"}
                    <span class="tab-env-icon" style="color: #0cbb52; font-size: 11px; margin-right: 4px;">📖</span>
                  {:else if tab.tabType === "spec"}
                    <span class="tab-env-icon" style="color: #108ee9; font-size: 11px; margin-right: 4px;">⚡</span>
                  {:else if tab.tabType === "mock"}
                    <span class="tab-env-icon" style="color: #fa8c16; font-size: 11px; margin-right: 4px;">📦</span>
                  {:else if tab.tabType === "dataset"}
                    <span class="tab-env-icon" style="color: #722ed1; font-size: 11px; margin-right: 4px;">📊</span>
                  {:else if tab.tabType === "flow"}
                    <span class="tab-env-icon" style="color: #13c2c2; font-size: 11px; margin-right: 4px;">🔀</span>
                  {:else}
                    <span class="tab-method method-{(tab.method || 'GET').toLowerCase()}">{tab.method || 'GET'}</span>
                  {/if}
                  <span class="tab-title">{tab.name}</span>
                  {#if isTabDirty(tab.id)}
                    <span class="tab-unsaved-dot" title={t("tab.unsavedChanges")}></span>
                  {/if}
                  <span
                    class="tab-close-btn"
                    title={t("tab.closeTab")}
                    onclick={(e) => {
                      e.stopPropagation();
                      closeTabAction(tab.id);
                    }}
                  >
                    {@render iconClose()}
                  </span>
                </button>
              {/each}
              <button
                type="button"
                class="tab-add-btn"
                title="New Tab"
                onclick={() => quickCreateRequest(selectedProjectId ?? projects[0]?.id)}
              >+</button>
              {#if openTabs.length > 6}
                <details class="tab-overflow-menu">
                  <summary class="tab-overflow-trigger" title={t("tab.allOpenTabs")}>
                    {@render iconChevronDown()}
                  </summary>
                  <div class="tab-overflow-list">
                    {#each openTabs as tab (tab.id)}
                      <button
                        type="button"
                        class="tab-overflow-item"
                        class:active={tab.id === selectedRequest?.id}
                        onclick={(e) => {
                          openRequest(tab.id);
                          (e.currentTarget as HTMLElement).closest("details")?.removeAttribute("open");
                        }}
                        oncontextmenu={(e) => { e.preventDefault(); openTabContextMenu(e, tab); }}
                      >
                        <span class="tab-method method-{tab.method.toLowerCase()}">{tab.method}</span>
                        <span class="request-name">{tab.name}</span>
                      </button>
                    {/each}
                  </div>
                </details>
              {/if}
            </div>

            <div class="workbench-env-picker">
              <div class="menu-wrap">
                <button
                  type="button"
                  class="env-pill-btn"
                  onclick={() => (envPickerOpen = !envPickerOpen)}
                >
                  <span class="env-pill-dot" class:empty={!selectedEnvironmentId}></span>
                  <span class="env-pill-label">{selectedEnvironmentId ? (allEnvironments.find((e) => e.id === selectedEnvironmentId)?.name ?? selectedEnvironmentId) : t("topbar.noEnvironment")}</span>
                  {@render iconChevronDown()}
                </button>
                {#if envPickerOpen}
                  <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (envPickerOpen = false)}></button>
                  <div class="dropdown-menu env-picker-menu">
                    <input
                      type="search"
                      class="request-search-input env-picker-search"
                      placeholder={t("env.searchEnvironments")}
                      bind:value={envPickerQuery}
                      use:focusOnMount
                    />
                    <button type="button" class="dropdown-menu-item" onclick={() => { envPickerOpen = false; quickCreateEnvironment(); }}>{t("topbar.newEnvironment")}</button>
                    {#if !envPickerQuery}
                      <button type="button" class="dropdown-menu-item" class:active={!selectedEnvironmentId} onclick={() => pickEnvironment(null)}>{t("topbar.noEnvironment")}</button>
                    {/if}
                    <div class="env-picker-list">
                      <!-- Favorites at the top -->
                      {#if favoriteEnvs.length > 0 && !envPickerQuery}
                        <div class="env-screen-group-label" style="display: flex; align-items: center; gap: 4px; color: #f5a623;">
                          <span>★</span>
                          <span>FAVORITES</span>
                        </div>
                        {#each favoriteEnvs as env (env.id)}
                          <div class="env-picker-row" class:active={selectedEnvironmentId === env.id}>
                            <button type="button" class="dropdown-menu-item env-picker-item-btn" class:active={selectedEnvironmentId === env.id} onclick={() => pickEnvironment(env.id)}>
                              <span>{env.name}</span>
                            </button>
                            <button
                              type="button"
                              class="env-star-btn favorited"
                              title="Remove from favorites"
                              onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                            >★</button>
                          </div>
                        {/each}
                        <div class="dropdown-menu-divider" style="margin: 4px 8px;"></div>
                      {/if}

                      <div class="env-screen-group-label">ALL ENVIRONMENTS</div>
                      {#each envPickerFilteredEnvironments as env (env.id)}
                        {@const isFav = isEnvFavorite(env.id)}
                        <div class="env-picker-row" class:active={selectedEnvironmentId === env.id}>
                          <button type="button" class="dropdown-menu-item env-picker-item-btn" class:active={selectedEnvironmentId === env.id} onclick={() => pickEnvironment(env.id)}>
                            <span>{env.name}</span>
                          </button>
                          <button
                            type="button"
                            class="env-star-btn"
                            class:favorited={isFav}
                            title={isFav ? "Remove from favorites" : "Add to favorites"}
                            onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                          >{isFav ? "★" : "☆"}</button>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
              <button
                type="button"
                class="icon-btn"
                title={t("topbar.manageVariables")}
                onclick={() => { loadVariables(); activeScreen = "environments"; }}
              >
                {@render iconEye()}
              </button>
            </div>
          </div>

          {#if currentTab?.tabType === "env"}
            <div class="env-tab-view">
              <div class="env-tab-header">
                <div class="env-tab-title-group">
                  <span class="env-dot-indicator"></span>
                  <h1 class="env-tab-title">{currentTab.name}</h1>
                </div>
                <div class="env-tab-actions">
                  <button type="button" class="btn-env-action" onclick={forkCurrentEnvironment}>
                    <span class="action-icon">&#9901;</span>
                    <span>Fork {envForkCount}</span>
                  </button>
                  <button type="button" class="btn-env-action" onclick={shareCurrentEnvironment}>
                    <span>Share</span>
                  </button>
                  <button type="button" class="icon-btn" title="Copy link" onclick={() => copyTextToClipboard(window.location.href)}>
                    {@render iconCopy()}
                  </button>
                </div>
              </div>

              <div class="env-tab-desc-row">
                <span class="env-tab-desc">Environments are sets of variables that allow you to customize requests for different setups.</span>
              </div>

              <div class="env-tab-search-bar">
                <div class="env-search-wrapper">
                  <span class="env-search-icon">🔍</span>
                  <input
                    type="search"
                    placeholder="Filter variables"
                    class="env-var-filter-input"
                    bind:value={envVarSearchQuery}
                  />
                </div>
              </div>

              <div class="env-variables-table-container">
                <table class="env-variables-table">
                  <thead>
                    <tr>
                      <th class="col-check"><input type="checkbox" checked title="Select all" /></th>
                      <th class="col-key">VARIABLE</th>
                      <th class="col-type">TYPE</th>
                      <th class="col-val col-init">
                        <div class="col-header-cloud">
                          <span>INITIAL VALUE</span>
                          <span class="cloud-icon" title="Cloud sync">☁ ˅</span>
                        </div>
                      </th>
                      <th class="col-val">CURRENT VALUE</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each filteredEnvironmentVariables as v (v.id)}
                      <tr>
                        <td class="col-check">
                          <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} />
                        </td>
                        <td class="col-key font-mono font-bold">
                          <input class="table-cell-input" value={v.key} onblur={(e) => updateVariableKey(v, (e.target as HTMLInputElement).value)} />
                        </td>
                        <td class="col-type">
                          <span class="env-type-pill">{v.is_secret ? "secret" : "default"}</span>
                        </td>
                        <td class="col-val font-mono">
                          <input class="table-cell-input text-secondary" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
                        </td>
                        <td class="col-val font-mono">
                          <input class="table-cell-input" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
                        </td>
                      </tr>
                    {/each}
                    <tr class="add-row">
                      <td class="col-check"><input type="checkbox" disabled /></td>
                      <td class="col-key">
                        <input
                          class="table-cell-input placeholder-row"
                          placeholder="Add a new variable"
                          bind:value={newEnvVarDraft.key}
                          onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
                        />
                      </td>
                      <td class="col-type"><span class="env-type-pill muted">default</span></td>
                      <td class="col-val">
                        <input
                          class="table-cell-input"
                          placeholder=""
                          bind:value={newEnvVarDraft.value}
                          onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
                        />
                      </td>
                      <td class="col-val"></td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          {:else if currentTab?.tabType === "doc"}
            <div class="doc-tab-view">
              <div class="doc-tab-header">
                <div class="doc-tab-title-group">
                  <span class="doc-icon-badge">📖</span>
                  <h1 class="doc-tab-title">{currentTab.name}</h1>
                  <span class="doc-status-badge">Published</span>
                </div>
                <div class="doc-tab-actions">
                  <button type="button" class="btn-env-action" onclick={() => downloadFile(sampleDocContent, `${currentTab.name}.md`, "text/markdown")}>
                    <span>Export Markdown</span>
                  </button>
                  <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard(sampleDocContent)}>
                    {@render iconCopy()}
                    <span>Copy Content</span>
                  </button>
                </div>
              </div>
              <div class="doc-tab-body">
                <div class="doc-preview-card">
                  <div class="doc-markdown-content font-mono">
                    <pre class="doc-content-pre">{sampleDocContent}</pre>
                  </div>
                </div>
              </div>
            </div>
          {:else if currentTab?.tabType === "spec"}
            <div class="spec-tab-view">
              <div class="spec-tab-header">
                <div class="spec-tab-title-group">
                  <span class="spec-icon-badge">⚡</span>
                  <h1 class="spec-tab-title">{currentTab.name}</h1>
                  <span class="spec-version-badge">OpenAPI 3.1.0</span>
                  <span class="spec-valid-badge">✓ Valid</span>
                </div>
                <div class="spec-tab-actions">
                  <button type="button" class="btn-env-action" onclick={() => downloadFile(sampleOpenApiJson, currentTab.name, "application/json")}>
                    <span>Download Spec</span>
                  </button>
                  <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard(sampleOpenApiJson)}>
                    {@render iconCopy()}
                    <span>Copy JSON</span>
                  </button>
                </div>
              </div>
              <div class="spec-tab-body">
                <div class="spec-endpoints-summary">
                  <div class="spec-summary-item"><span class="spec-summary-label">Servers</span><span class="spec-summary-val">2 active</span></div>
                  <div class="spec-summary-item"><span class="spec-summary-label">Paths</span><span class="spec-summary-val">3 endpoints</span></div>
                  <div class="spec-summary-item"><span class="spec-summary-label">Format</span><span class="spec-summary-val">JSON Schema</span></div>
                </div>
                <div class="spec-code-card font-mono">
                  <pre class="spec-code-pre">{sampleOpenApiJson}</pre>
                </div>
              </div>
            </div>
          {:else if currentTab?.tabType === "mock"}
            <div class="mock-tab-view">
              <div class="mock-tab-header">
                <div class="mock-tab-title-group">
                  <span class="mock-icon-badge">📦</span>
                  <h1 class="mock-tab-title">{currentTab.name}</h1>
                  <span class="mock-status-pill online">● Running</span>
                </div>
                <div class="mock-tab-actions">
                  <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard("http://127.0.0.1:8080/mock/v1")}>
                    {@render iconCopy()}
                    <span>Copy Mock URL</span>
                  </button>
                </div>
              </div>
              <div class="mock-tab-body">
                <div class="mock-url-banner">
                  <span class="mock-url-label">Mock Server Endpoint:</span>
                  <code class="mock-url-code">http://127.0.0.1:8080/mock/v1</code>
                  <span class="mock-latency-badge">⚡ 150ms simulated latency</span>
                </div>
                <div class="mock-endpoints-table-wrap">
                  <table class="env-variables-table">
                    <thead>
                      <tr>
                        <th class="col-type">METHOD</th>
                        <th class="col-key">MOCK PATH</th>
                        <th class="col-type">RESPONSE CODE</th>
                        <th class="col-val">SIMULATED PAYLOAD</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <td><span class="tab-method method-post">POST</span></td>
                        <td class="font-mono">/api/v1/auth/login</td>
                        <td><span class="spec-valid-badge">200 OK</span></td>
                        <td class="font-mono text-secondary">&#123; "token": "mock-jwt-token-xyz", "expiresIn": 3600 &#125;</td>
                      </tr>
                      <tr>
                        <td><span class="tab-method method-post">POST</span></td>
                        <td class="font-mono">/api/v1/auth/validate-otp</td>
                        <td><span class="spec-valid-badge">200 OK</span></td>
                        <td class="font-mono text-secondary">&#123; "status": "VERIFIED", "code": "00" &#125;</td>
                      </tr>
                      <tr>
                        <td><span class="tab-method method-post">POST</span></td>
                        <td class="font-mono">/api/v1/card/transaction</td>
                        <td><span class="spec-valid-badge">200 OK</span></td>
                        <td class="font-mono text-secondary">&#123; "transactionId": "TX-9988231", "status": "SUCCESS" &#125;</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          {:else if currentTab?.tabType === "dataset"}
            <div class="dataset-tab-view">
              <div class="dataset-tab-header">
                <div class="dataset-tab-title-group">
                  <span class="dataset-icon-badge">📊</span>
                  <h1 class="dataset-tab-title">{currentTab.name}</h1>
                  <span class="dataset-count-badge">4 Rows • 4 Columns</span>
                </div>
                <div class="dataset-tab-actions">
                  <button type="button" class="btn-env-action" onclick={() => downloadFile("user_id,email,amount,currency\n101,dev1@alansari.ae,500,AED\n102,dev2@alansari.ae,1250,AED\n103,manager@alansari.ae,4500,USD\n104,tester@alansari.ae,250,EUR\n", currentTab.name, "text/csv")}>
                    <span>Export CSV</span>
                  </button>
                </div>
              </div>
              <div class="dataset-tab-body">
                <div class="dataset-table-wrap">
                  <table class="env-variables-table">
                    <thead>
                      <tr>
                        <th class="col-type">#</th>
                        <th class="col-key">user_id</th>
                        <th class="col-key">email</th>
                        <th class="col-key">amount</th>
                        <th class="col-type">currency</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <td>1</td>
                        <td class="font-mono">101</td>
                        <td class="font-mono">dev1@alansari.ae</td>
                        <td class="font-mono">500</td>
                        <td><span class="env-type-pill">AED</span></td>
                      </tr>
                      <tr>
                        <td>2</td>
                        <td class="font-mono">102</td>
                        <td class="font-mono">dev2@alansari.ae</td>
                        <td class="font-mono">1250</td>
                        <td><span class="env-type-pill">AED</span></td>
                      </tr>
                      <tr>
                        <td>3</td>
                        <td class="font-mono">103</td>
                        <td class="font-mono">manager@alansari.ae</td>
                        <td class="font-mono">4500</td>
                        <td><span class="env-type-pill">USD</span></td>
                      </tr>
                      <tr>
                        <td>4</td>
                        <td class="font-mono">104</td>
                        <td class="font-mono">tester@alansari.ae</td>
                        <td class="font-mono">250</td>
                        <td><span class="env-type-pill">EUR</span></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          {:else if currentTab?.tabType === "flow"}
            <div class="flow-tab-view">
              <div class="flow-tab-header">
                <div class="flow-tab-title-group">
                  <span class="flow-icon-badge">🔀</span>
                  <h1 class="flow-tab-title">{currentTab.name}</h1>
                  <span class="flow-steps-badge">4 Blocks</span>
                </div>
                <div class="flow-tab-actions">
                  <button type="button" class="btn-send" onclick={() => { exportFeedback = "Running flow sequence..."; setTimeout(() => { exportFeedback = "Flow completed successfully (4/4 blocks passed)"; setTimeout(() => { exportFeedback = ""; }, 3000); }, 1200); }}>
                    <span>▶ Run Flow</span>
                  </button>
                </div>
              </div>
              <div class="flow-tab-body">
                <div class="flow-canvas">
                  <div class="flow-block">
                    <div class="flow-block-header start">1. Start Trigger</div>
                    <div class="flow-block-content">Manual execution on click</div>
                  </div>
                  <div class="flow-connector">➔</div>
                  <div class="flow-block">
                    <div class="flow-block-header request">2. Send Request</div>
                    <div class="flow-block-content font-mono">POST /api/v1/auth/login</div>
                  </div>
                  <div class="flow-connector">➔</div>
                  <div class="flow-block">
                    <div class="flow-block-header evaluate">3. Extract Token</div>
                    <div class="flow-block-content font-mono">data.sessionToken</div>
                  </div>
                  <div class="flow-connector">➔</div>
                  <div class="flow-block">
                    <div class="flow-block-header request">4. Submit Payment</div>
                    <div class="flow-block-content font-mono">POST /api/v1/card/transaction</div>
                  </div>
                </div>
              </div>
            </div>
          {:else if selectedRequest}
            <div class="breadcrumb-row">
              <div class="breadcrumb-left">
              <span class="http-pill">HTTP</span>
              <div class="breadcrumb-trail">
                <span class="breadcrumb-path">{projects.find((p) => p.id === selectedProjectId)?.name ?? ""}</span>
                {#each selectedRequestFolderChain as folderName (folderName)}
                  <span class="breadcrumb-sep">&rsaquo;</span>
                  <span class="breadcrumb-path">{folderName}</span>
                {/each}
                <span class="breadcrumb-sep">&rsaquo;</span>
                {#if renamingRequestId === selectedRequest.id}
                  <form class="inline-form" onsubmit={submitRenameRequest}>
                    <input bind:value={renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
                    <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                    <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingRequestId = null)}>{@render iconClose()}</button>
                  </form>
                {:else}
                  <button
                    type="button"
                    class="breadcrumb-current"
                    title={t("breadcrumb.renameHint")}
                    onclick={() => startRenameRequest(selectedRequest!.id, selectedRequest!.name)}
                  >
                    {selectedRequest.name}
                  </button>
                {/if}
              </div>
            </div>
            <div class="breadcrumb-right">
              <div class="btn-save-split">
                <button type="button" class="btn-save-main" onclick={() => saveRequest()}>
                  {@render iconSave()} <span>{t("request.saveLabel")}</span>
                </button>
                <button type="button" class="btn-save-caret" onclick={() => saveRequest()}>{@render iconChevronDown()}</button>
              </div>
              <button type="button" class="btn-share" onclick={(e) => openRequestContextMenu(e, selectedRequest)}>
                {@render iconCopy()} <span>Share</span>
              </button>
            </div>
          </div>

          <form class="request-bar" onsubmit={(e) => { e.preventDefault(); saveRequest(); }}>
            <div class="request-bar-row">
              <div class="url-pill">
                <select bind:value={editMethod} class="method-select method-{editMethod.toLowerCase()}" onchange={scheduleAutoSave}>
                  <option>GET</option>
                  <option>POST</option>
                  <option>PUT</option>
                  <option>PATCH</option>
                  <option>DELETE</option>
                  <option>HEAD</option>
                  <option>OPTIONS</option>
                  <option>TRACE</option>
                </select>
                <span class="url-pill-divider"></span>
                <div class="url-input-shell">
                  <div class="url-token-overlay" aria-hidden="true">
                    {#each urlTokens as tok, i (i)}
                      {#if tok.type === "text"}
                        <span class="url-token-text">{tok.text}</span>
                      {:else}
                        {@const missing = requestDiagnostics?.all_missing?.includes(tok.name) ?? false}
                        <span
                          class="url-token-var"
                          class:missing
                          role="presentation"
                          onmouseenter={(e) => showVarPopover(tok.name, e.currentTarget as HTMLElement)}
                          onmouseleave={() => scheduleHideMissingVarPopover()}
                          onclick={(e) => {
                            const shell = (e.currentTarget as HTMLElement).closest(".url-input-shell");
                            const input = shell?.querySelector("input.url-input") as HTMLInputElement | null;
                            if (input) input.focus();
                          }}
                        >
                          {tok.raw}
                        </span>
                      {/if}
                    {/each}
                  </div>
                  <input
                    placeholder={t("request.urlPlaceholder")}
                    bind:value={editUrl}
                    class="url-input url-input-ghost"
                    oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLInputElement, "var", false, (v) => (editUrl = v)); }}
                    onkeydown={handleAutocompleteKeydown}
                    onblur={hideAutocompleteSoon}
                    onpaste={handleUrlPaste}
                    onscroll={syncUrlOverlayScroll}
                    onmousemove={handleGenericInputMouseMove}
                    onmouseleave={handleGenericInputMouseLeave}
                  />
                </div>
              </div>
              <div class="send-action">
                {#if sending}
                  <button type="button" class="btn-cancel" onclick={cancelCurrentSend}>{t("request.cancel")}</button>
                {:else}
                  <div class="btn-send-group">
                    <button type="button" class="btn-send" onclick={sendCurrentRequest}>{t("request.send")}</button>
                    <div class="menu-wrap">
                      <button
                        type="button"
                        class="btn-send-caret"
                        title={t("request.sendOptions")}
                        onclick={() => (sendMenuOpen = !sendMenuOpen)}
                      >{@render iconChevronDown()}</button>
                      {#if sendMenuOpen}
                        <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (sendMenuOpen = false)}></button>
                        <div class="send-dropdown-menu">
                          <button type="button" class="send-menu-item" onclick={handleGetSuccessfulResponse}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="3,2 14,8 3,14"/></svg>
                            <span>Get a successful response</span>
                          </button>
                          <button type="button" class="send-menu-item" onclick={handleVisualizeResponse}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2.5"/></svg>
                            <span>Visualize response</span>
                          </button>
                          <button type="button" class="send-menu-item" onclick={handleWriteTests}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 2h10v12H3z"/><path d="M6 6h4M6 9h4M6 12h2"/></svg>
                            <span>Write tests</span>
                          </button>
                          <button type="button" class="send-menu-item" onclick={handleDebugRequest}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2a3 3 0 0 0-3 3v2h6V5a3 3 0 0 0-3-3zM4 9a4 4 0 0 0 8 0v2a4 4 0 0 1-8 0V9zM2 8h2M12 8h2M3 13l2-1M13 13l-2-1M3 5l2 1M13 5l-2 1"/></svg>
                            <span>Debug request</span>
                          </button>
                          <button type="button" class="send-menu-item" onclick={handleExploreApiCapabilities}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><polygon points="10.5,5.5 9,9 5.5,10.5 7,7"/></svg>
                            <span>Explore API capabilities</span>
                          </button>
                          <div class="send-menu-divider"></div>
                          <button type="button" class="send-menu-item" onclick={handleDownloadResponse}>
                            <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2v9M4 8l4 4 4-4M2 14h12"/></svg>
                            <span>Download response</span>
                          </button>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}
                <button
                  type="button"
                  class="btn-save"
                  class:is-error={autoSaveStatus === "error"}
                  class:is-unsaved={autoSaveStatus === "unsaved"}
                  title={autoSaveStatus === "saving" ? t("request.saving") : autoSaveStatus === "unsaved" ? t("request.unsaved") : autoSaveStatus === "error" ? t("request.saveFailed") : t("request.saved")}
                  onclick={() => saveRequest()}
                >
                  {@render iconSave()}
                  <span>{t("request.saveLabel")}</span>
                  {#if autoSaveStatus === "unsaved" || autoSaveStatus === "error"}<span class="dirty-dot" aria-hidden="true">•</span>{/if}
                </button>
                <button
                  type="button"
                  class="icon-btn btn-code-toggle"
                  class:active={rightSidebarVisible && rightPanel === "code"}
                  title={t("bottom.codeSnippet")}
                  onclick={() => {
                    if (rightSidebarVisible && rightPanel === "code") {
                      setRightSidebarVisible(false);
                    } else {
                      rightPanel = "code";
                      setRightSidebarVisible(true);
                    }
                  }}
                >&lt;/&gt;</button>
              </div>
            </div>
            {#if curlDetectedFeedback || sendCancelledNotice}
              <div class="request-bar-row secondary">
                {#if curlDetectedFeedback}
                  <span class="hint">{curlDetectedFeedback}</span>
                {/if}
                {#if sendCancelledNotice}
                  <span class="hint">{sendCancelledNotice}</span>
                {/if}
              </div>
            {/if}
          </form>

          {#if missingVarHover && hoveredVarRect}
            <div
              class="hovered-var-token-highlight"
              style="top: {hoveredVarRect.top}px; left: {hoveredVarRect.left}px; width: {hoveredVarRect.right - hoveredVarRect.left}px; height: {hoveredVarRect.bottom - hoveredVarRect.top}px;"
              aria-hidden="true"
            ></div>
          {/if}

          {#if missingVarHover}
            {@const resolved = findResolvedVariable(missingVarHover.name)}
            {@const activeEnv = allEnvironments.find((e) => e.id === selectedEnvironmentId)}
            <div
              class="postman-var-popover"
              class:place-above={missingVarHover.placeAbove}
              role="dialog"
              aria-label={t(resolved ? "var.editValueFor" : "missingvar.addValueFor", { name: missingVarHover.name })}
              style="top: {missingVarHover.top}px; left: {missingVarHover.left}px;"
              onmouseenter={cancelHideMissingVarPopover}
              onmouseleave={() => scheduleHideMissingVarPopover(250)}
            >
              <div class="var-popover-header">
                <div class="var-popover-title-group">
                  <span class="var-popover-var-symbol" aria-hidden="true">&#123;&#123; &#125;&#125;</span>
                  <span class="var-popover-var-name">{missingVarHover.name}</span>
                </div>
                <div class="var-popover-badge-group">
                  {#if resolved}
                    {#if resolved.scope === "environment"}
                      <span class="var-scope-badge badge-env" title="Environment Variable">ENVIRONMENT</span>
                    {:else}
                      <span class="var-scope-badge badge-global" title="Global Variable">GLOBAL</span>
                    {/if}
                  {:else}
                    <span class="var-scope-badge badge-unresolved" title="Unresolved Variable">UNRESOLVED</span>
                  {/if}
                  <button
                    type="button"
                    class="var-popover-close-btn"
                    title="Close"
                    onclick={() => { missingVarHover = null; hoveredVarRect = null; }}
                  >&times;</button>
                </div>
              </div>

              {#if resolved && resolved.scope === "environment"}
                <div class="var-popover-env-context">
                  <span class="env-context-label">Environment:</span>
                  <span class="env-context-name">{activeEnv?.name ?? "Active Environment"}</span>
                </div>
              {:else if !resolved}
                <div class="var-popover-unresolved-hint">
                  Not resolved in current environment or globals.
                </div>
              {/if}

              <div class="var-popover-body">
                <label class="var-popover-label" for="popover-var-val">CURRENT VALUE</label>
                <div class="var-popover-input-wrap">
                  <input
                    id="popover-var-val"
                    class="var-popover-input"
                    placeholder={resolved ? "Value..." : "Set value to define variable..."}
                    value={missingVarDrafts[missingVarHover.name] ?? ""}
                    onfocus={() => { popoverInputFocused = true; cancelHideMissingVarPopover(); }}
                    onblur={() => { popoverInputFocused = false; scheduleHideMissingVarPopover(300); }}
                    oninput={(e) => (missingVarDrafts[missingVarHover!.name] = (e.target as HTMLInputElement).value)}
                    onkeydown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        saveVariableFromPopover(missingVarHover!.name);
                      } else if (e.key === "Escape") {
                        e.preventDefault();
                        missingVarHover = null;
                        hoveredVarRect = null;
                      }
                    }}
                  />
                </div>
              </div>

              <div class="var-popover-footer">
                <div class="var-popover-status">
                  {#if popoverSaveSuccess}
                    <span class="save-success-msg">&#10003; Saved</span>
                  {/if}
                </div>
                <div class="var-popover-actions">
                  <button
                    type="button"
                    class="var-popover-btn var-popover-btn-save"
                    onclick={() => saveVariableFromPopover(missingVarHover!.name)}
                  >
                    {#if resolved}
                      Save
                    {:else if selectedEnvironmentId}
                      Add to Environment
                    {:else}
                      Add to Global
                    {/if}
                  </button>
                </div>
              </div>
            </div>
          {/if}

          {#if autocomplete}
            <ul
              class="autocomplete-portal"
              role="listbox"
              style="top: {autocomplete.top}px; left: {autocomplete.left}px;"
            >
              {#each autocomplete.items as item, i (item.insertText)}
                <li>
                  <button
                    type="button"
                    class="autocomplete-item"
                    class:active={i === autocomplete.activeIndex}
                    onmousedown={(e) => { e.preventDefault(); applyAutocompleteItem(item); }}
                  >
                    <span class="autocomplete-label">{item.label}</span>
                    {#if item.detail}<span class="autocomplete-detail">{item.detail}</span>{/if}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}

          {#if urlPreview}
            <div class="url-preview-bar">
              <span class="preview-label">{t("request.resolvesTo")}</span> <code>{urlPreview.resolved}</code>
              {#if urlPreview.missing.length}
                <span class="warn-inline">{t("request.missing", { list: urlPreview.missing.join(", ") })}</span>
              {/if}
            </div>
          {/if}

          {#if requestDiagnostics?.all_missing?.length}
            <div class="warn-banner missing-vars-banner">
              {@render iconWarning()} {t("request.unresolvedVariables")}
              {#each requestDiagnostics.all_missing as varName (varName)}
                <span
                  class="missing-var-chip"
                  role="presentation"
                  onmouseenter={(e) => showMissingVarPopover(varName, e.currentTarget as HTMLElement)}
                  onmouseleave={() => scheduleHideMissingVarPopover()}
                >
                  <strong>{varName}</strong>
                </span>
              {/each}
              <span class="hint">{t("request.unresolvedHint", { scope: selectedEnvironmentId ? t("request.scopeEnvironment") : t("request.scopeGlobal") })}</span>
            </div>
          {/if}

          <div class="editor-tabs">
            <button type="button" class="editor-tab" class:active={activeEditorTab === "docs"} onclick={() => (activeEditorTab = "docs")}>
              {t("tab.docs")} {#if editDescription}<span class="tab-dot">•</span>{/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "params"} onclick={() => (activeEditorTab = "params")}>
              {t("tab.params")}
              {#if requestDiagnostics?.query_params_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInParams", { list: requestDiagnostics.query_params_missing.join(', ') })}>{@render iconWarning()} {requestDiagnostics.query_params_missing.length}</span>
              {:else if withoutEmptyKeyRows(editQueryParams).length}
                <span class="tab-badge">{withoutEmptyKeyRows(editQueryParams).length}</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "auth"} onclick={() => (activeEditorTab = "auth")}>
              {t("tab.auth")}
              {#if requestDiagnostics?.auth_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInAuth", { list: requestDiagnostics.auth_missing.join(', ') })}>{@render iconWarning()} {requestDiagnostics.auth_missing.length}</span>
              {:else if editAuthType !== "none"}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "headers"} onclick={() => (activeEditorTab = "headers")}>
              {t("tab.headers")}
              {#if withoutEmptyKeyRows(editHeaders).length}
                <span class="tab-badge">{withoutEmptyKeyRows(editHeaders).length}</span>
              {/if}
              {#if editHeaders.some(h => h.enabled && h.key.trim())}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "body"} onclick={() => (activeEditorTab = "body")}>
              {t("tab.body")}
              {#if requestDiagnostics?.body_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInBody", { list: requestDiagnostics.body_missing.join(', ') })}>{@render iconWarning()} {requestDiagnostics.body_missing.length}</span>
              {:else if editBody || editFormDataItems.length || editUrlEncodedItems.length}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "scripts"} onclick={() => (activeEditorTab = "scripts")}>
              {t("tab.scripts")} {#if editPreScript || editPostScript}<span class="tab-dot">•</span>{/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "settings"} onclick={() => (activeEditorTab = "settings")}>
              {t("tab.settings")}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "mock"} onclick={() => (activeEditorTab = "mock")}>
              {t("tab.mock")} {#if sampleResponses.length}<span class="tab-badge">{sampleResponses.length}</span>{/if}
            </button>
            <div class="response-stat-spacer"></div>
            <button type="button" class="editor-tab editor-tab-cookies" onclick={() => { responseSubTab = "cookies"; }}>
              Cookies
            </button>
          </div>

          <div class="editor-body-row">
          <div class="editor-pane" onmousemove={handlePaneMouseMoveDelegated} onmouseleave={() => scheduleHideMissingVarPopover()}>
          <div class="tab-content">
            {#if activeEditorTab === "params"}
              <div class="params-table">
                {#each editQueryParams as param, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={param.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                    <input placeholder={t("params.key")} bind:value={param.key} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
                    <input placeholder={t("params.value")} bind:value={param.value} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
                    {#if i < editQueryParams.length - 1 || param.key.trim()}
                      <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeQueryParam(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
                    {/if}
                  </div>
                {/each}
              </div>

            {:else if activeEditorTab === "headers"}
              <div class="params-table">
                {#each editHeaders as header, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={header.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                    <input
                      placeholder={t("params.key")}
                      bind:value={header.key}
                      oninput={(e) => {
                        growHeaders();
                        scheduleAutoSave();
                        updateAutocompleteFor(e.currentTarget as HTMLInputElement, "header", false, (v) => (header.key = v));
                      }}
                      onkeydown={handleAutocompleteKeydown}
                      onblur={hideAutocompleteSoon}
                    />
                    <input
                      placeholder={t("params.value")}
                      bind:value={header.value}
                      oninput={(e) => {
                        growHeaders();
                        scheduleAutoSave();
                        updateAutocompleteFor(e.currentTarget as HTMLInputElement, "var", false, (v) => (header.value = v));
                      }}
                      onkeydown={handleAutocompleteKeydown}
                      onblur={hideAutocompleteSoon}
                    />
                    <input placeholder={t("headers.description")} bind:value={header.description} oninput={() => { growHeaders(); scheduleAutoSave(); }} />
                    {#if i < editHeaders.length - 1 || header.key.trim()}
                      <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeHeader(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
                    {/if}
                  </div>
                {/each}
              </div>

            {:else if activeEditorTab === "auth"}
              <div class="params-table">
                <select bind:value={editAuthType} onchange={scheduleAutoSave}>
                  <option value="none">{t("auth.none")}</option>
                  <option value="bearer">{t("auth.bearer")}</option>
                  <option value="basic">{t("auth.basic")}</option>
                  <option value="api_key">{t("auth.apiKey")}</option>
                </select>
                {#if editAuthType === "bearer"}
                  <div class="params-row">
                    <input placeholder={t("auth.token")} bind:value={editAuthBearerToken} oninput={scheduleAutoSave} />
                  </div>
                {:else if editAuthType === "basic"}
                  <div class="params-row">
                    <input placeholder={t("auth.username")} bind:value={editAuthBasicUsername} oninput={scheduleAutoSave} />
                    <input placeholder={t("auth.password")} type="password" bind:value={editAuthBasicPassword} oninput={scheduleAutoSave} />
                  </div>
                {:else if editAuthType === "api_key"}
                  <div class="params-row">
                    <input placeholder={t("params.key")} bind:value={editAuthApiKeyKey} oninput={scheduleAutoSave} />
                    <input placeholder={t("params.value")} bind:value={editAuthApiKeyValue} oninput={scheduleAutoSave} />
                    <select bind:value={editAuthApiKeyLocation} onchange={scheduleAutoSave}>
                      <option value="header">{t("auth.locationHeader")}</option>
                      <option value="query">{t("auth.locationQuery")}</option>
                    </select>
                  </div>
                {/if}
              </div>

            {:else if activeEditorTab === "body"}
              <div class="params-table">
                <div class="body-mode-bar">
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="raw" onchange={scheduleAutoSave} /> {t("body.raw")}
                  </label>
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="form-data" onchange={scheduleAutoSave} /> {t("body.formData")}
                  </label>
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="x-www-form-urlencoded" onchange={scheduleAutoSave} /> {t("body.urlEncoded")}
                  </label>
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="binary" onchange={scheduleAutoSave} /> {t("body.binary")}
                  </label>
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="graphql" onchange={scheduleAutoSave} /> {t("body.graphql")}
                  </label>
                  {#if editBodyType === "raw"}
                    <select class="raw-type-select" value={rawContentType} onchange={(e) => setRawContentType((e.target as HTMLSelectElement).value)}>
                      {#each RAW_CONTENT_TYPES as type (type.id)}
                        <option value={type.id}>{t(type.label)}</option>
                      {/each}
                    </select>
                    {#if rawContentType === "json"}
                      <button type="button" onclick={() => { if (!editBody) editBody = "{\n  \n}"; scheduleAutoSave(); }}>{t("body.jsonTemplate")}</button>
                    {/if}
                    <button type="button" class="btn-danger-ghost" onclick={() => { editBody = ""; scheduleAutoSave(); }}>{t("body.clearBody")}</button>
                    {#if editBody.trim()}
                      <button type="button" onclick={prettifyBody} title={t("body.prettifyTitle")}>{t("body.prettify")}</button>
                    {/if}
                    {#if bodyPrettifyFeedback}<span class="warn-inline">{bodyPrettifyFeedback}</span>{/if}
                  {/if}
                </div>

                {#if editBodyType === "raw"}
                  <div class="code-editor-shell">
                    <div class="code-gutter" aria-hidden="true">
                      {#each (editBody || "").split("\n") as _, lineIdx (lineIdx)}
                        <span class="gutter-num">{lineIdx + 1}</span>
                      {/each}
                    </div>
                    <textarea
                      placeholder={t("body.rawPlaceholder")}
                      bind:value={editBody}
                      class="body-input code-editor-input"
                      rows="8"
                      spellcheck="false"
                      oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "var", false, (v) => (editBody = v)); }}
                      onkeydown={handleAutocompleteKeydown}
                      onblur={hideAutocompleteSoon}
                      onmousemove={handleBodyMouseMove}
                      onmouseleave={handleBodyMouseLeave}
                    ></textarea>
                  </div>
                {:else if editBodyType === "form-data"}
                  <div class="params-table">
                    {#each editFormDataItems as item, i (i)}
                      <div class="params-row">
                        <input type="checkbox" bind:checked={item.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                        <input placeholder={t("params.key")} bind:value={item.key} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
                        {#if item.is_file}
                          <input placeholder={t("body.filePath")} bind:value={item.file_path} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
                        {:else}
                          <input placeholder={t("params.value")} bind:value={item.value} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
                        {/if}
                        <label class="checkbox-label" title={t("body.fileHint")}>
                          <input type="checkbox" bind:checked={item.is_file} onchange={scheduleAutoSave} /> {t("body.file")}
                        </label>
                        {#if i < editFormDataItems.length - 1 || item.key.trim()}
                          <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeFormDataItem(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {:else if editBodyType === "x-www-form-urlencoded"}
                  <div class="params-table">
                    {#each editUrlEncodedItems as item, i (i)}
                      <div class="params-row">
                        <input type="checkbox" bind:checked={item.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                        <input placeholder={t("params.key")} bind:value={item.key} oninput={() => { growUrlEncodedItems(); scheduleAutoSave(); }} />
                        <input placeholder={t("params.value")} bind:value={item.value} oninput={() => { growUrlEncodedItems(); scheduleAutoSave(); }} />
                        {#if i < editUrlEncodedItems.length - 1 || item.key.trim()}
                          <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeUrlEncodedItem(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {:else if editBodyType === "binary"}
                  <div class="params-table">
                    <div class="params-row">
                      <input placeholder={t("body.binaryPathPlaceholder")} bind:value={editBinaryFilePath} oninput={scheduleAutoSave} />
                    </div>
                    <p class="hint">{t("body.binaryHint")}</p>
                  </div>
                {:else if editBodyType === "graphql"}
                  <div class="graphql-editor">
                    <h4>{t("body.graphqlQuery")}</h4>
                    <textarea
                      placeholder={t("body.graphqlQueryPlaceholder")}
                      bind:value={editGraphqlQuery}
                      class="body-input"
                      rows="6"
                      oninput={() => {
                        try {
                          const vars = editGraphqlVariables ? JSON.parse(editGraphqlVariables) : {};
                          editBody = JSON.stringify({ query: editGraphqlQuery, variables: vars }, null, 2);
                        } catch {
                          editBody = JSON.stringify({ query: editGraphqlQuery }, null, 2);
                        }
                        scheduleAutoSave();
                      }}
                    ></textarea>
                    <h4>{t("body.graphqlVariables")}</h4>
                    <textarea
                      placeholder={t("body.graphqlVariablesPlaceholder")}
                      bind:value={editGraphqlVariables}
                      class="body-input"
                      rows="3"
                      oninput={() => {
                        try {
                          const vars = editGraphqlVariables ? JSON.parse(editGraphqlVariables) : {};
                          editBody = JSON.stringify({ query: editGraphqlQuery, variables: vars }, null, 2);
                        } catch {
                          editBody = JSON.stringify({ query: editGraphqlQuery }, null, 2);
                        }
                        scheduleAutoSave();
                      }}
                    ></textarea>
                  </div>
                {/if}
              </div>

            {:else if activeEditorTab === "scripts"}
              <div class="scripts-layout">
                <div class="scripts-side">
                  <button type="button" class="scripts-side-item" class:active={activeScriptTab === "pre"} onclick={() => (activeScriptTab = "pre")}>
                    {t("scripts.pre")} {#if editPreScript}<span class="tab-dot">•</span>{/if}
                  </button>
                  <button type="button" class="scripts-side-item" class:active={activeScriptTab === "post"} onclick={() => (activeScriptTab = "post")}>
                    {t("scripts.post")} {#if editPostScript}<span class="tab-dot">•</span>{/if}
                  </button>
                </div>
                <div class="scripts-main">
                  {#if activeScriptTab === "pre"}
                    <p class="hint">{t("scripts.preHint")}</p>
                    <textarea
                      placeholder={t("scripts.prePlaceholder")}
                      bind:value={editPreScript}
                      class="body-input scripts-textarea"
                      oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "pm", false, (v) => (editPreScript = v)); }}
                      onkeydown={handleAutocompleteKeydown}
                      onblur={hideAutocompleteSoon}
                    ></textarea>
                  {:else}
                    <div class="field-header-row">
                      <p class="hint">{t("scripts.postHint")}</p>
                      <button
                        type="button"
                        class="btn-ghost btn-xs"
                        disabled={generatingTestsDocs}
                        onclick={() => generateTestsAndDocsWithAiAction("tests")}
                        title={t("scripts.generateTestsTitle")}
                      >
                        {generatingTestsDocs ? t("scripts.generating") : t("scripts.generateTests")}
                      </button>
                    </div>
                    {#if testsDocsFeedback}
                      <p class="action-feedback-inline">{testsDocsFeedback}</p>
                    {/if}
                    <textarea
                      placeholder={t("scripts.postPlaceholder")}
                      bind:value={editPostScript}
                      class="body-input scripts-textarea"
                      oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "pm", true, (v) => (editPostScript = v)); }}
                      onkeydown={handleAutocompleteKeydown}
                      onblur={hideAutocompleteSoon}
                    ></textarea>
                  {/if}
                </div>
              </div>

            {:else if activeEditorTab === "settings"}
              <div class="params-table settings-grid">
                <label class="settings-row">
                  <span>{t("reqSettings.timeout")}</span>
                  <input
                    type="number"
                    placeholder={t("reqSettings.timeoutPlaceholder")}
                    value={editTimeoutMs ?? ""}
                    oninput={(e) => {
                      const val = (e.target as HTMLInputElement).value;
                      editTimeoutMs = val ? parseInt(val, 10) : null;
                      scheduleAutoSave();
                    }}
                  />
                </label>
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={editFollowRedirects} onchange={scheduleAutoSave} />
                  {t("reqSettings.followRedirects")}
                </label>
                <label class="settings-row">
                  <span>{t("reqSettings.maxRedirects")}</span>
                  <input type="number" bind:value={editMaxRedirects} min="0" max="50" oninput={scheduleAutoSave} />
                </label>
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={editVerifySsl} onchange={scheduleAutoSave} />
                  {t("reqSettings.verifySsl")}
                </label>
                <label class="settings-row">
                  <span>{t("reqSettings.proxyUrl")}</span>
                  <input placeholder="http://127.0.0.1:8080" bind:value={editProxyUrl} oninput={scheduleAutoSave} />
                </label>
                <label class="settings-row">
                  <span>{t("reqSettings.httpVersion")}</span>
                  <select bind:value={editHttpVersion} onchange={scheduleAutoSave}>
                    <option value="">{t("reqSettings.httpVersionDefault")}</option>
                    <option value="HTTP/1.1">HTTP/1.1</option>
                    <option value="HTTP/2">HTTP/2</option>
                  </select>
                </label>
              </div>

            {:else if activeEditorTab === "docs"}
              <div class="params-table">
                <div class="field-header-row">
                  <h4>{t("docs.title")}</h4>
                  <button
                    type="button"
                    class="btn-ghost btn-xs"
                    disabled={generatingTestsDocs}
                    onclick={() => generateTestsAndDocsWithAiAction("docs")}
                    title={t("docs.generateTitle")}
                  >
                    {generatingTestsDocs ? t("scripts.generating") : t("docs.generate")}
                  </button>
                </div>
                {#if testsDocsFeedback}
                  <p class="action-feedback-inline">{testsDocsFeedback}</p>
                {/if}
                <textarea
                  placeholder={t("docs.placeholder")}
                  bind:value={editDescription}
                  class="body-input"
                  rows="8"
                  oninput={scheduleAutoSave}
                ></textarea>
              </div>

            {:else if activeEditorTab === "mock"}
              <div class="sample-responses-section">
                <div class="field-header-row">
                  <p class="hint">{t("sample.hint")}</p>
                  <button
                    type="button"
                    class="btn-ghost btn-xs"
                    disabled={generatingSample}
                    onclick={generateSampleResponseWithAiAction}
                    title={t("sample.generateTitle")}
                  >
                    {generatingSample ? t("scripts.generating") : t("sample.generate")}
                  </button>
                </div>
                {#if sampleFeedback}
                  <p class="action-feedback-inline">{sampleFeedback}</p>
                {/if}
                {#if sampleResponses.length}
                  <div class="sample-responses-list">
                    {#each sampleResponses as sr (sr.id)}
                      {#if renamingSampleResponseId === sr.id}
                        <div class="sample-response-card">
                          <form class="inline-form" onsubmit={(e) => { e.preventDefault(); submitRenameSampleResponse(sr.request_id); }}>
                            <input bind:value={renameSampleResponseValue} use:focusOnMount onblur={() => submitRenameSampleResponse(sr.request_id)} />
                            <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                            <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingSampleResponseId = null)}>{@render iconClose()}</button>
                          </form>
                        </div>
                      {:else}
                        <details class="sample-response-card">
                          <summary class="sample-response-summary">
                            <span class="badge badge-sample">{t("sample.badge")}</span>
                            <strong class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>
                              {sr.status}
                            </strong>
                            <span class="sample-name">{sr.name}</span>
                            <span class="hint">{new Date(sr.created_at).toLocaleTimeString()}</span>
                            <button
                              type="button"
                              class="btn-delete-icon"
                              onclick={(e) => { e.stopPropagation(); startRenameSampleResponse(sr); }}
                              title={t("sidebar.rename")}
                            >{@render iconEdit()}</button>
                            <button
                              type="button"
                              class="btn-delete-icon"
                              onclick={(e) => { e.stopPropagation(); deleteSampleResponseAction(sr.request_id, sr.id); }}
                              title={t("sample.delete")}
                            >{@render iconClose()}</button>
                          </summary>
                          <pre class="body-view">{sr.body ?? ""}</pre>
                        </details>
                      {/if}
                    {/each}
                  </div>
                {:else}
                  <p class="screen-empty-inline">{t("sample.empty")}</p>
                {/if}
              </div>
            {/if}
          </div>
          </div>

          {#if !responsePaneCollapsed && !responsePaneMaximized}
            <div
              class="response-pane-resize-handle"
              class:resizing={responsePaneResizing}
              onmousedown={startResponsePaneResize}
              ondblclick={() => { responsePaneHeight = 360; }}
              onkeydown={(e) => {
                responsePaneManuallyResized = true;
                if (e.key === "ArrowUp") responsePaneHeight = Math.min(window.innerHeight - 220, responsePaneHeight + 16);
                else if (e.key === "ArrowDown") responsePaneHeight = Math.max(160, responsePaneHeight - 16);
              }}
              role="slider"
              aria-orientation="horizontal"
              aria-label={t("response.resizeHandle")}
              aria-valuenow={responsePaneHeight}
              aria-valuemin={100}
              aria-valuemax={1200}
              tabindex="0"
            ></div>
          {/if}

          <div
            class="response-pane"
            class:collapsed={responsePaneCollapsed}
            class:maximized={responsePaneMaximized}
            class:is-resizing={responsePaneResizing}
            style="height: {responsePaneMaximized ? '100%' : responsePaneCollapsed ? '37px' : responsePaneHeight + 'px'}"
          >
          {#if sending}
            <div class="response-loading">
              <span class="spinner" aria-hidden="true"></span>
              {t("response.sending")}
            </div>
          {:else if activeResponse}
            <div class="response">
              <div class="response-stat-row">
                <span class="response-stat-status" class:status-ok={activeResponse.status < 400} class:status-err={activeResponse.status >= 400}>
                  {activeResponse.status} {activeResponse.status_text}
                </span>
                <span class="response-stat-item"><span class="response-stat-label">{t("response.time")}</span> {activeResponse.duration_ms} ms</span>
                <span class="response-stat-item"><span class="response-stat-label">{t("response.size")}</span> {formatByteSize(activeResponse.body_size)}</span>
                <div class="response-stat-spacer"></div>
                {#if copyFeedback}
                  <span class="hint">{copyFeedback}</span>
                {/if}
                <button type="button" class="icon-btn" title={t("response.copyTitle")} onclick={copyResponseBody}>{@render iconCopy()}</button>
                <button type="button" class="icon-btn" title={t("response.downloadTitle")} onclick={downloadResponseBody}>{@render iconImport()}</button>
                <button type="button" class="icon-btn" title={t("response.saveAsSample")} onclick={saveCurrentResponse}>{@render iconSave()}</button>
                <button
                  type="button"
                  class="icon-btn"
                  title={responsePaneCollapsed ? "Expand response panel" : "Collapse / Minimize response panel"}
                  onclick={() => setResponsePaneCollapsed(!responsePaneCollapsed)}
                >{#if responsePaneCollapsed}{@render iconChevronUp()}{:else}{@render iconMinus()}{/if}</button>
                <button
                  type="button"
                  class="icon-btn"
                  title={responsePaneMaximized ? "Restore response size" : "Maximize response panel"}
                  onclick={() => { responsePaneMaximized = !responsePaneMaximized; if (responsePaneMaximized) responsePaneCollapsed = false; }}
                >{#if responsePaneMaximized}{@render iconCompressDiagonal()}{:else}{@render iconExpandDiagonal()}{/if}</button>
              </div>
              {#if !responsePaneCollapsed}

              <div class="response-subtabs">
                <button type="button" class="response-subtab" class:active={responseSubTab === "body"} onclick={() => (responseSubTab = "body")}>{t("response.body")}</button>
                <button type="button" class="response-subtab" class:active={responseSubTab === "headers"} onclick={() => (responseSubTab = "headers")}>
                  {t("response.headers")}
                  {#if activeResponse.headers?.length}<span class="tab-badge">{activeResponse.headers.length}</span>{/if}
                </button>
                <button type="button" class="response-subtab" class:active={responseSubTab === "cookies"} onclick={() => (responseSubTab = "cookies")}>
                  {t("response.cookies")}
                  {#if activeResponse.cookies?.length}<span class="tab-badge">{activeResponse.cookies.length}</span>{/if}
                </button>
                <button type="button" class="response-subtab" class:active={responseSubTab === "tests"} onclick={() => (responseSubTab = "tests")}>
                  {t("response.tests")}
                  {#if activeResponseTests.length}
                    <span class="tab-badge" class:tab-badge-warn={activeResponseTests.some((test) => !test.passed)}>
                      {activeResponseTests.filter((test) => test.passed).length}/{activeResponseTests.length}
                    </span>
                  {/if}
                </button>
                <button type="button" class="response-subtab" class:active={responseSubTab === "history"} onclick={() => (responseSubTab = "history")}>
                  {t("response.history")}
                  {#if responseHistory.length}<span class="tab-badge">{responseHistory.length}</span>{/if}
                </button>

                {#if responseSubTab === "body"}
                  <div class="response-format-toggle">
                    <button type="button" class="btn-toggle" class:active={responseViewMode === "pretty"} onclick={() => (responseViewMode = "pretty")}>{t("response.pretty")}</button>
                    <button type="button" class="btn-toggle" class:active={responseViewMode === "raw"} onclick={() => (responseViewMode = "raw")}>{t("response.raw")}</button>
                    {#if responseBodyIsHtml}
                      <button type="button" class="btn-toggle" class:active={responseViewMode === "preview"} onclick={() => (responseViewMode = "preview")}>{t("response.preview")}</button>
                    {/if}
                  </div>
                {/if}
              </div>

              <div class="response-subtab-content">
                {#if responseSubTab === "body"}
                  {#if responseViewMode === "preview" && responseBodyIsHtml}
                    <iframe class="response-preview-frame" title={t("response.preview")} sandbox="" srcdoc={activeResponseBody}></iframe>
                  {:else if responseBodyIsJson}
                    <div class="code-editor-shell response-code-shell">
                      <div class="code-gutter" aria-hidden="true">
                        {#each (prettyResponseBody || "").split("\n") as _, lineIdx (lineIdx)}
                          <span class="gutter-num">{lineIdx + 1}</span>
                        {/each}
                      </div>
                      <pre class="body-view code-editor-pre">{@html highlightedResponseBody}</pre>
                    </div>
                  {:else}
                    <div class="code-editor-shell response-code-shell">
                      <div class="code-gutter" aria-hidden="true">
                        {#each (prettyResponseBody || "").split("\n") as _, lineIdx (lineIdx)}
                          <span class="gutter-num">{lineIdx + 1}</span>
                        {/each}
                      </div>
                      <pre class="body-view code-editor-pre">{prettyResponseBody}</pre>
                    </div>
                  {/if}
                  {#if activeResponseTruncated}
                    <p class="hint">{t("response.truncated")}</p>
                  {/if}
                {:else if responseSubTab === "headers"}
                  {#if activeResponse.headers?.length}
                    <div class="headers-list">
                      {#each activeResponse.headers as h}
                        <div class="header-line">
                          <strong>{h.key}:</strong> {h.value}
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="empty">{t("response.noHeaders")}</p>
                  {/if}
                {:else if responseSubTab === "cookies"}
                  {#if activeResponse.cookies?.length}
                    <div class="headers-list">
                      {#each activeResponse.cookies as c}
                        <div class="header-line">
                          <strong>{c.name}:</strong> {c.value}
                          {#if c.domain}<span class="hint">{t("cookie.domain", { value: c.domain })}</span>{/if}
                          {#if c.path}<span class="hint">{t("cookie.path", { value: c.path })}</span>{/if}
                          {#if c.http_only}<span class="badge">{t("cookie.httpOnly")}</span>{/if}
                          {#if c.secure}<span class="badge">{t("cookie.secure")}</span>{/if}
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="empty">{t("response.noCookies")}</p>
                  {/if}
                {:else if responseSubTab === "tests"}
                  {#if activeResponseTests.length}
                    <ul class="test-results-list">
                      {#each activeResponseTests as test, i (i)}
                        <li class="test-result-row" class:test-pass={test.passed} class:test-fail={!test.passed}>
                          <span class="test-result-icon">{#if test.passed}{@render iconCheck()}{:else}{@render iconClose()}{/if}</span>
                          <span class="test-result-name">{test.name}</span>
                          {#if !test.passed && test.error}<span class="test-result-error">{test.error}</span>{/if}
                        </li>
                      {/each}
                    </ul>
                  {:else}
                    <p class="empty">{t("response.testsHintFull")}</p>
                  {/if}
                {:else if responseSubTab === "history"}
                  {#if responseHistory.length}
                    <ul class="response-history-list">
                      {#each responseHistory as r (r.id)}
                        <li>
                          <button type="button" class="response-history-row" onclick={() => openHistoryResponse(r.id)}>
                            <span class="status-chip" class:status-ok={r.status < 400} class:status-err={r.status >= 400}>{r.status}</span>
                            <span class="response-history-duration">{r.duration_ms} ms</span>
                            <span class="response-history-time">{new Date(r.created_at).toLocaleString()}</span>
                          </button>
                        </li>
                      {/each}
                    </ul>
                  {:else}
                    <p class="empty">{t("history.empty")}</p>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <div class="response-pane-bar">
            <button
              type="button"
              class="icon-btn"
              title={responsePaneCollapsed ? t("response.expandPane") : t("response.collapsePane")}
              onclick={() => setResponsePaneCollapsed(!responsePaneCollapsed)}
            >{#if responsePaneCollapsed}{@render iconChevronUp()}{:else}{@render iconMinus()}{/if}</button>
            <span class="response-pane-bar-label">
              {t("response.title")}
            </span>
            <div class="response-stat-spacer"></div>
            <button
              type="button"
              class="icon-btn"
              title={responsePaneMaximized ? "Restore response size" : "Maximize response panel"}
              onclick={() => { responsePaneMaximized = !responsePaneMaximized; if (responsePaneMaximized) responsePaneCollapsed = false; }}
            >{#if responsePaneMaximized}{@render iconCompressDiagonal()}{:else}{@render iconExpandDiagonal()}{/if}</button>
          </div>
          {#if !responsePaneCollapsed}
            <div class="response-empty-state">
              <div class="empty-icon">{@render iconInboxEmpty()}</div>
              <p>{t("response.sendEmpty")}</p>
            </div>
          {/if}
        {/if}
          </div>
          </div>
          {/if}
        </section>
      {/if}
    </main>

    {#if activeScreen === "workspace" && !responseExpanded && selectedProjectId}
      {#if rightSidebarVisible}
        <div
          class="right-sidebar-resize-handle"
          class:resizing={rightSidebarResizing}
          onmousedown={startRightSidebarResize}
          onkeydown={(e) => {
            if (e.key === "ArrowLeft") rightSidebarWidth = Math.min(640, rightSidebarWidth + 16);
            else if (e.key === "ArrowRight") rightSidebarWidth = Math.max(260, rightSidebarWidth - 16);
          }}
          role="slider"
          aria-orientation="vertical"
          aria-label={t("rightSidebar.resizeHandle")}
          aria-valuenow={rightSidebarWidth}
          aria-valuemin={260}
          aria-valuemax={640}
          tabindex="0"
        ></div>
        <aside class="right-sidebar" style="width: {rightSidebarWidth}px">
          {#if rightPanel === "variables"}
            <div class="right-sidebar-header">
              <span class="right-sidebar-title">All variables</span>
              <div class="response-stat-spacer"></div>
              <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
                {@render iconClose()}
              </button>
            </div>
            <div class="right-drawer-body">
              <div class="env-scope-subhead">
                <span class="env-badge-pill">E</span>
                <span class="env-badge-name">{allEnvironments.find((e) => e.id === selectedEnvironmentId)?.name ?? "[Alansari] [Remittance] [Masoud] [DEV]"}</span>
              </div>
              <div class="drawer-search-row">
                <span class="drawer-search-icon">🔍</span>
                <input
                  type="search"
                  placeholder="Filter variables"
                  class="drawer-search-input"
                  bind:value={drawerVarSearch}
                />
              </div>
              <div class="drawer-variables-list">
                {#each drawerFilteredVariables as v (v.id)}
                  <div class="drawer-variable-item">
                    <div class="drawer-var-top">
                      <span class="drawer-var-key">{v.key}</span>
                      <span class="drawer-var-scope-badge">{v.scopeTag}</span>
                    </div>
                    <div class="drawer-var-bottom">
                      <span class="drawer-var-val">{v.is_secret ? "••••••••" : v.value}</span>
                      <button type="button" class="icon-btn icon-btn-ghost copy-btn" title="Copy value" onclick={() => copyTextToClipboard(v.value)}>
                        {@render iconCopy()}
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {:else if rightPanel === "info"}
            <div class="right-sidebar-header">
              <span class="right-sidebar-title">{@render iconInfo()} {t("bottom.info")}</span>
              <div class="response-stat-spacer"></div>
              <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
                {@render iconClose()}
              </button>
            </div>
            {#if !selectedRequest}
              <p class="screen-empty-inline">{t("request.selectPrompt")}</p>
            {:else}
              <div class="bottom-panel">
                <dl class="info-list info-list-grid">
                  <dt>{t("bottom.id")}</dt>
                  <dd>{selectedRequest.id}</dd>
                  <dt>{t("bottom.projectId")}</dt>
                  <dd>{selectedRequest.project_id}</dd>
                  <dt>{t("bottom.created")}</dt>
                  <dd>{new Date(selectedRequest.created_at).toLocaleString()}</dd>
                  <dt>{t("bottom.updated")}</dt>
                  <dd>{new Date(selectedRequest.updated_at).toLocaleString()}</dd>
                  <dt>{t("bottom.headersCount")}</dt>
                  <dd>{selectedRequest.headers.length}</dd>
                  <dt>{t("bottom.queryParamsCount")}</dt>
                  <dd>{selectedRequest.query_params.length}</dd>
                </dl>
              </div>
            {/if}
          {:else if rightPanel === "ai"}
            <div class="right-sidebar-header">
              <span class="right-sidebar-title">🪄 Postman AI</span>
              <div class="response-stat-spacer"></div>
              <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
                {@render iconClose()}
              </button>
            </div>
            <div class="right-drawer-body">
              <p class="hint" style="margin-bottom: var(--space-3);">Generate tests, mock payloads, or explain responses with AI.</p>
              <textarea class="form-textarea" placeholder="Ask Postman AI to generate test scripts..." bind:value={aiPrompt} rows="4"></textarea>
              <button type="button" class="btn-primary" style="margin-top: var(--space-2); width: 100%;" onclick={() => { showAiPanel = true; }}>
                Open AI Assistant
              </button>
            </div>
          {:else if rightPanel === "comments"}
            <div class="right-sidebar-header">
              <span class="right-sidebar-title">💬 Comments</span>
              <div class="response-stat-spacer"></div>
              <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
                {@render iconClose()}
              </button>
            </div>
            <div class="right-drawer-body">
              <p class="hint">Comments aren't available yet — this is a local, single-user app with no collaboration backend.</p>
            </div>
          {:else}
            <!-- Code snippet drawer matching p3.png -->
            <div class="right-sidebar-header code-snippet-header">
              <span class="right-sidebar-title">Code snippet</span>
              <div class="response-stat-spacer"></div>
              <div class="menu-wrap">
                <button type="button" class="icon-btn" title="Snippet Settings" onclick={() => (showSnippetSettings = !showSnippetSettings)}>
                  {@render iconSettings()}
                </button>
                {#if showSnippetSettings}
                  <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (showSnippetSettings = false)}></button>
                  <div class="dropdown-menu snippet-settings-dropdown" style="right: 0; min-width: 220px; padding: 6px 0;">
                    <div style="padding: 6px 12px; font-weight: 600; font-size: 11px; color: #888; border-bottom: 1px solid #333; margin-bottom: 4px;">SNIPPET SETTINGS</div>
                    <label class="dropdown-menu-item" style="display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 6px 12px;">
                      <input type="checkbox" checked={snippetIndentType === "tab"} onchange={(e) => { snippetIndentType = e.currentTarget.checked ? "tab" : "space"; }} />
                      <span>Use tabs for indent</span>
                    </label>
                    <label class="dropdown-menu-item" style="display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 6px 12px;">
                      <input type="checkbox" checked={snippetTrimTrailing} onchange={(e) => { snippetTrimTrailing = e.currentTarget.checked; }} />
                      <span>Trim trailing spaces</span>
                    </label>
                  </div>
                {/if}
              </div>
              <button type="button" class="icon-btn" title="Copy snippet to clipboard" onclick={copySnippetToClipboard}>
                {@render iconCopy()}
              </button>
              <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
                {@render iconClose()}
              </button>
            </div>

            <div class="code-snippet-drawer-body">
              <div class="code-snippet-target-row">
                <select bind:value={snippetTarget} class="snippet-target-picker">
                  <option value="bash">cURL</option>
                  <option value="windows_cmd">cURL (Windows CMD)</option>
                  <option value="power_shell">cURL (PowerShell)</option>
                  <option value="java_script_fetch">JavaScript - Fetch</option>
                  <option value="node_fetch">Node.js - Fetch</option>
                  <option value="python_requests">Python - Requests</option>
                  <option value="preload">Preload element</option>
                  <option value="har">HAR (sanitized)</option>
                </select>
                <select bind:value={snippetMode} class="snippet-mode-picker">
                  <option value="placeholder">Placeholder</option>
                  <option value="resolved">Resolved</option>
                </select>
              </div>

              {#if snippetError}
                <p class="error">{snippetError}</p>
              {:else if snippetLoading}
                <p class="hint">{t("bottom.generatingSnippet")}</p>
              {:else if snippet}
                <div class="code-snippet-gutter-box">
                  <div class="code-snippet-gutter" aria-hidden="true">
                    {#each snippet.split("\n") as _, idx}
                      <span class="gutter-line-no">{idx + 1}</span>
                    {/each}
                  </div>
                  <pre class="code-snippet-pre"><code>{@html highlightSnippetCode(snippet)}</code></pre>
                </div>
              {/if}
            </div>
          {/if}
        </aside>
      {/if}
    {/if}

    <!-- Persistent Right Utility Rail matching p3, p4 -->
    {#if utilityRailVisible && activeScreen === "workspace" && !responseExpanded}
      <aside class="right-utility-rail" aria-label="Utility rail">
        <button
          type="button"
          class="rail-action-btn"
          class:active={rightSidebarVisible && rightPanel === "ai"}
          title="Postman AI Assistant"
          onclick={() => {
            if (rightSidebarVisible && rightPanel === "ai") {
              setRightSidebarVisible(false);
              rightPanel = null;
            } else {
              rightPanel = "ai";
              setRightSidebarVisible(true);
            }
          }}
        >
          <span class="rail-action-icon">🪄</span>
          <span class="rail-action-label">AI</span>
        </button>

        <button
          type="button"
          class="rail-action-btn"
          class:active={rightSidebarVisible && rightPanel === "variables"}
          title="All variables (Environment quick look)"
          onclick={() => {
            if (rightSidebarVisible && rightPanel === "variables") {
              setRightSidebarVisible(false);
              rightPanel = null;
            } else {
              loadVariables();
              rightPanel = "variables";
              setRightSidebarVisible(true);
            }
          }}
        >
          <span class="rail-action-icon">{@render iconEye()}</span>
        </button>

        <button
          type="button"
          class="rail-action-btn"
          class:active={rightSidebarVisible && (rightPanel === "code" || !rightPanel)}
          title="Code snippet"
          onclick={() => {
            if (rightSidebarVisible && rightPanel === "code") {
              setRightSidebarVisible(false);
              rightPanel = null;
            } else {
              rightPanel = "code";
              setRightSidebarVisible(true);
            }
          }}
        >
          <span class="rail-action-code">&lt;/&gt;</span>
        </button>

        <button
          type="button"
          class="rail-action-btn"
          class:active={rightSidebarVisible && rightPanel === "comments"}
          title="Comments"
          onclick={() => {
            if (rightSidebarVisible && rightPanel === "comments") {
              setRightSidebarVisible(false);
              rightPanel = null;
            } else {
              rightPanel = "comments";
              setRightSidebarVisible(true);
            }
          }}
        >
          <span class="rail-action-icon">💬</span>
        </button>

        <button
          type="button"
          class="rail-action-btn"
          class:active={rightSidebarVisible && rightPanel === "info"}
          title="Request info"
          onclick={() => {
            if (rightSidebarVisible && rightPanel === "info") {
              setRightSidebarVisible(false);
              rightPanel = null;
            } else {
              rightPanel = "info";
              setRightSidebarVisible(true);
            }
          }}
        >
          <span class="rail-action-icon">{@render iconInfo()}</span>
        </button>
      </aside>
    {/if}
  </div>

  {#if showAiPanel}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showAiPanel = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showAiPanel = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container modal-wide">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <h3>{t("ai.title")}</h3>
            <span class="modal-sub">{t("ai.subtitle")}</span>
          </div>
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showAiPanel = false)}>{@render iconClose()}</button>
        </div>

        <div class="modal-tabs">
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "generate"}
            onclick={() => (aiActiveTab = "generate")}
          >
            {t("ai.tabGenerate")}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "source"}
            onclick={() => (aiActiveTab = "source")}
          >
            {t("ai.tabSource")} {#if sourceReport}<span class="badge badge-framework">{sourceReport.endpoints.length}</span>{/if}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "settings"}
            onclick={() => (aiActiveTab = "settings")}
          >
            {t("ai.tabSettings")} {#if !aiConfigured}<span class="tab-badge-alert">!</span>{/if}
          </button>
        </div>

        {#if aiActiveTab === "generate"}
          <div class="modal-body">
            {#if !aiConfigured}
              <div class="action-alert warning">
                <span>{t("ai.notConfiguredWarning")}</span>
                <button type="button" class="btn-primary btn-xs" onclick={() => (aiActiveTab = "settings")}>{t("ai.configureAi")}</button>
              </div>
            {/if}

            <div class="ai-context-options">
              <label class="checkbox-label" title={t("ai.includeExistingTitle")}>
                <input type="checkbox" bind:checked={aiIncludeExistingRequests} />
                {t("ai.includeExisting")}
              </label>
              <label class="checkbox-label" title={t("ai.includeVarsTitle")}>
                <input type="checkbox" bind:checked={aiIncludeVariables} />
                {t("ai.includeVars")}
              </label>
            </div>

            <form onsubmit={generateWithAi} class="ai-prompt-form">
              <textarea
                placeholder={t("ai.promptPlaceholder")}
                bind:value={aiPrompt}
                class="body-input"
                rows="3"
              ></textarea>
              <div class="params-row">
                <button type="submit" class="btn-primary" disabled={aiGenerating || !aiPrompt.trim()}>
                  {aiGenerating ? t("ai.generatingDefinition") : t("ai.generateRequest")}
                </button>
              </div>
            </form>

            {#if aiPreview}
              <div class="ai-preview-card">
                <div class="preview-title-row">
                  <span class="method method-{aiPreview.method.toLowerCase()}">{aiPreview.method}</span>
                  <span class="preview-name">{aiPreview.name}</span>
                  <code class="preview-url">{aiPreview.url}</code>
                </div>
                {#if aiPreview.description}
                  <p class="hint preview-desc">{aiPreview.description}</p>
                {/if}
                <div class="preview-meta-row">
                  <span>{t("ai.headersCount", { count: aiPreview.headers.length })}</span>
                  <span>{t("ai.queryParamsCount", { count: aiPreview.query_params.length })}</span>
                  {#if aiPreview.body}<span>{t("ai.hasBody")}</span>{/if}
                </div>
                {#if aiPreview.body}
                  <pre class="body-view preview-body-pre">{aiPreview.body}</pre>
                {/if}
                <div class="params-row">
                  <button
                    type="button"
                    class="btn-primary"
                    onclick={() => { addAiPreviewToProject(); showAiPanel = false; }}
                  >
                    {t("ai.addToProject")}
                  </button>
                  <button type="button" onclick={() => (aiPreview = null)}>{t("ai.discard")}</button>
                </div>
              </div>
            {/if}
          </div>

        {:else if aiActiveTab === "source"}
          <div class="modal-body">
            <p class="hint">
              {t("ai.sourceDesc")} <em>{t("ai.sourceReadOnly")}</em>
            </p>

            <div class="source-scan-bar">
              <input
                type="text"
                placeholder={t("ai.sourcePathPlaceholder")}
                bind:value={sourceDirectoryInput}
                class="url-input"
              />
              <button
                type="button"
                class="btn-primary"
                disabled={sourceScanning || !sourceDirectoryInput.trim()}
                onclick={scanSourceProjectAction}
              >
                {sourceScanning ? t("ai.scanning") : t("ai.scanCodebase")}
              </button>
            </div>

            {#if sourceActionFeedback}
              <p class="action-feedback-inline">{sourceActionFeedback}</p>
            {/if}

            {#if sourceReport}
              <div class="source-summary-panel">
                <div class="source-badges-row">
                  <span class="badge">{t("ai.typeLabel", { type: sourceReport.project_type })}</span>
                  <span class="badge">{t("ai.scannedLabel", { count: sourceReport.scanned_files_count })}</span>
                  <span class="badge badge-success">{t("ai.foundLabel", { count: sourceReport.endpoints.length })}</span>
                  {#each sourceReport.frameworks as fw}
                    <span class="badge badge-framework">{fw}</span>
                  {/each}
                  {#if sourceReport.has_openapi}
                    <span class="badge badge-openapi">{t("ai.openapiDetected", { path: sourceReport.openapi_path ?? t("ai.specDetected") })}</span>
                  {/if}
                </div>

                {#if sourceReport.endpoints.length > 0}
                  <div class="source-filter-row">
                    <input
                      type="text"
                      placeholder={t("ai.filterRoutesPlaceholder")}
                      bind:value={sourceFilter}
                      class="url-input"
                    />
                  </div>

                  <div class="discovered-endpoints-list">
                    {#each filteredSourceEndpoints as ep, idx (idx)}
                      <div class="discovered-endpoint-card">
                        <div class="ep-info">
                          <span class="method method-{ep.method.toLowerCase()}">{ep.method}</span>
                          <code class="ep-path">{ep.path}</code>
                          {#if ep.auth_hint}
                            <span class="badge badge-auth" title={t("ai.authDetectedTitle")}>{@render iconLock()} {ep.auth_hint}</span>
                          {/if}
                        </div>
                        <div class="ep-meta">
                          <span class="ep-file">{ep.source_file}{ep.line_number ? `:${ep.line_number}` : ''}</span>
                          {#if ep.description}
                            <span class="ep-summary">{ep.description}</span>
                          {/if}
                        </div>
                        <button
                          type="button"
                          class="btn-xs-primary"
                          disabled={!selectedProjectId}
                          onclick={() => { importDiscoveredEndpointAction(ep); showAiPanel = false; }}
                          title={t("ai.importEndpointTitle")}
                        >
                          {t("ai.importRequest")}
                        </button>
                      </div>
                    {/each}
                    {#if filteredSourceEndpoints.length === 0}
                      <p class="hint text-center">{t("ai.noRoutesMatch", { filter: sourceFilter })}</p>
                    {/if}
                  </div>
                {:else}
                  <p class="hint">{t("ai.noRoutesFound")}</p>
                {/if}
              </div>
            {/if}
          </div>

        {:else if aiActiveTab === "settings"}
          <div class="modal-body">
            <div class="ai-settings-grid">
              <div class="settings-field">
                <label for="ai-provider-select"><strong>{t("ai.providerLabel")}</strong></label>
                <select id="ai-provider-select" bind:value={aiProviderInput} onchange={onAiProviderChange} class="url-input">
                  {#each AI_PROVIDERS as p (p.id)}
                    <option value={p.id}>{t(p.labelKey)}</option>
                  {/each}
                </select>
                <span class="hint">{t("ai.providerHint")}</span>
              </div>

              <div class="settings-field">
                <label for="ai-api-key-input">
                  <strong>{t("ai.apiKeyLabel")}</strong>
                  {#if aiConfigured}
                    <span class="badge badge-success">{t("ai.configured")}</span>
                  {:else}
                    <span class="badge">{t("ai.notConfigured")}</span>
                  {/if}
                </label>
                <div class="password-input-row">
                  <input
                    id="ai-api-key-input"
                    type={aiShowKey ? "text" : "password"}
                    placeholder={aiSettings?.api_key || AI_API_KEY_PLACEHOLDER[aiProviderInput]}
                    bind:value={aiApiKeyInput}
                    class="url-input"
                  />
                  <button type="button" class="btn-ghost" onclick={() => (aiShowKey = !aiShowKey)}>
                    {aiShowKey ? t("ai.hide") : t("ai.show")}
                  </button>
                </div>
                <span class="hint">{t("ai.apiKeyHint")}</span>
              </div>

              <div class="settings-field">
                <label for="ai-model-input"><strong>{t("ai.modelLabel")}</strong></label>
                <input
                  id="ai-model-input"
                  type="text"
                  list="ai-model-suggestions"
                  placeholder={t("ai.modelPlaceholder")}
                  bind:value={aiModelInput}
                  class="url-input"
                />
                <datalist id="ai-model-suggestions">
                  {#each AI_MODEL_SUGGESTIONS[aiProviderInput] as m (m)}
                    <option value={m}></option>
                  {/each}
                </datalist>
                <span class="hint">{t("ai.modelHint")}</span>
              </div>

              <div class="settings-field">
                <label for="ai-base-url-input">
                  <strong>{t("ai.baseUrlLabel")}</strong>
                  {#if aiProviderInput === "custom"}
                    <span class="badge badge-warn">{t("ai.required")}</span>
                  {/if}
                </label>
                <input
                  id="ai-base-url-input"
                  type="text"
                  placeholder={aiProviderInput === "custom" ? t("ai.baseUrlPlaceholderCustom") : t("ai.baseUrlPlaceholder")}
                  bind:value={aiBaseUrlInput}
                  class="url-input"
                />
                <span class="hint">{aiProviderInput === "custom" ? t("ai.baseUrlHintCustom") : t("ai.baseUrlHint")}</span>
              </div>

              <div class="params-row">
                <button type="button" class="btn-primary" onclick={saveAiSettingsAction}>{t("ai.saveSettings")}</button>
                <button type="button" class="btn-secondary" disabled={aiTesting} onclick={testAiConnectionAction}>
                  {aiTesting ? t("ai.testing") : t("ai.testConnection")}
                </button>
              </div>

              {#if aiSettingsFeedback}
                <p class="action-feedback-inline text-success">{aiSettingsFeedback}</p>
              {/if}
              {#if aiTestFeedback}
                <div class="action-alert success">
                  <span>{@render iconCheckCircle()} {aiTestFeedback}</span>
                </div>
              {/if}
              {#if aiTestError}
                <div class="action-alert error">
                  <span>{@render iconXCircle()} {aiTestError}</span>
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <div class="modal-footer">
          <button type="button" onclick={() => (showAiPanel = false)}>{t("common.close")}</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showDiffModal}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showDiffModal = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showDiffModal = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container">
        <div class="modal-header">
          <h3>{t("diff.title")}</h3>
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showDiffModal = false)}>{@render iconClose()}</button>
        </div>
        <div class="modal-body">
          {#if !gitDiffContent.trim()}
            <p class="hint">{t("diff.noDiff")}</p>
          {:else}
            <pre class="diff-viewer">{gitDiffContent}</pre>
          {/if}
        </div>
        <div class="modal-footer">
          <button type="button" onclick={() => (showDiffModal = false)}>{t("common.close")}</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showHistoryModal}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showHistoryModal = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showHistoryModal = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container">
        <div class="modal-header">
          <h3>{t("diffHistory.title")}</h3>
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showHistoryModal = false)}>{@render iconClose()}</button>
        </div>
        <div class="modal-body">
          {#if gitHistory.length === 0}
            <p class="hint">{t("diffHistory.noHistory")}</p>
          {:else}
            <div class="history-list">
              {#each gitHistory as c}
                <div class="history-item">
                  <div class="commit-header">
                    <code class="commit-hash">{c.hash.slice(0, 8)}</code>
                    <span class="commit-author">{c.author}</span>
                    <span class="commit-date">{c.date}</span>
                  </div>
                  <p class="commit-msg">{c.message}</p>
                </div>
              {/each}
            </div>
          {/if}
        </div>
        <div class="modal-footer">
          <button type="button" onclick={() => (showHistoryModal = false)}>{t("common.close")}</button>
        </div>
      </div>
    </div>
  {/if}

</div>
{/if}
{:else if activeScreen === "globals"}
  <div class="globals-screen-container">
    <div class="globals-main-view">
      <div class="globals-header">
        <div class="globals-title-row">
          <h1 class="globals-title">Globals</h1>
          <button type="button" class="btn-link-action" onclick={handleAddGlobalVariableClick}>Add variable</button>
        </div>
        <div class="globals-desc-row">
          <span class="globals-desc">Globals are variables that are available across all workspaces.</span>
        </div>
        <div class="globals-toolbar">
          <div class="globals-search-wrapper">
            <span class="search-icon">🔍</span>
            <input
              type="search"
              placeholder="Filter variables"
              class="globals-search-input"
              bind:value={globalsSearch}
            />
          </div>
          <button type="button" class="btn-export-globals" onclick={exportGlobalVariables}>
            <span class="export-icon">&#8682;</span>
            <span>Export</span>
          </button>
        </div>
      </div>

      <div class="globals-table-wrapper">
        <table class="globals-table">
          <thead>
            <tr>
              <th class="col-check"><input type="checkbox" checked title="Select all" /></th>
              <th class="col-key">VARIABLE</th>
              <th class="col-init">INITIAL VALUE</th>
              <th class="col-curr">CURRENT VALUE</th>
            </tr>
          </thead>
          <tbody>
            {#each (globalsSearch ? filteredProjectVariables.filter(v => v.key.toLowerCase().includes(globalsSearch.toLowerCase()) || v.value.toLowerCase().includes(globalsSearch.toLowerCase())) : filteredProjectVariables) as v (v.id)}
              <tr>
                <td class="col-check">
                  <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} />
                </td>
                <td class="col-key font-mono font-bold">
                  <input class="table-cell-input" value={v.key} onblur={(e) => updateVariableKey(v, (e.target as HTMLInputElement).value)} />
                </td>
                <td class="col-init font-mono">
                  <input class="table-cell-input text-secondary" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
                </td>
                <td class="col-curr font-mono">
                  <input class="table-cell-input" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
                </td>
              </tr>
            {/each}
            <tr class="add-row">
              <td class="col-check"><input type="checkbox" disabled /></td>
              <td class="col-key">
                <input
                  class="table-cell-input placeholder-row"
                  placeholder="Add variable"
                  bind:value={newGlobalVarDraft.key}
                  onkeydown={(e) => { if (e.key === "Enter") commitNewGlobalVar(); }}
                />
              </td>
              <td class="col-init">
                <input
                  class="table-cell-input"
                  placeholder=""
                  bind:value={newGlobalVarDraft.value}
                  onkeydown={(e) => { if (e.key === "Enter") commitNewGlobalVar(); }}
                />
              </td>
              <td class="col-curr"></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
{:else if activeScreen === "environments"}
  {#if !selectedProjectId}
    {@render noProjectPicker(t("rail.environments"))}
  {:else}
    <div class="env-screen">
      <aside class="env-screen-side">
        <div class="env-screen-side-header">
          <span class="screen-kicker">{t("env.title")}</span>
          <button type="button" class="icon-btn" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
        </div>
        <div class="env-screen-project-row">
          <span class="env-screen-project-label">{t("env.project")}</span>
          {@render projectSwitcher()}
        </div>
        <div class="request-search-box">
          <input
            type="search"
            placeholder={t("env.searchEnvironments")}
            bind:value={envSearchQuery}
            class="request-search-input"
          />
        </div>
        <div class="env-screen-list">
          {#if !envSearchQuery}
            <button
              type="button"
              class="env-screen-item"
              class:active={!selectedEnvironmentId}
              onclick={() => { selectedEnvironmentId = null; loadVariables(); }}
            >
              {t("env.noEnvironment")}
            </button>
          {/if}
          {#each filteredEnvironmentsByProject as [projectName, envs] (projectName)}
            <div class="env-screen-group-label">{projectName}</div>
            {#each envs as env (env.id)}
              {#if renamingEnvironmentId === env.id}
                <form class="inline-form env-screen-rename-form" onsubmit={submitRenameEnvironment}>
                  <input bind:value={renameEnvironmentValue} use:focusOnMount onblur={submitRenameEnvironment} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingEnvironmentId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                <div class="env-screen-item-row" class:active={selectedEnvironmentId === env.id}>
                  <button
                    type="button"
                    class="env-screen-item"
                    onclick={() => { selectedEnvironmentId = env.id; loadVariables(); }}
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
            {#if envSearchQuery}
              <p class="screen-empty-inline">{t("palette.noMatches")}</p>
            {/if}
          {/each}
        </div>
      </aside>

      <section class="screen-page">
        <div class="screen-page-header">
          <span class="screen-kicker">{t("env.editing")}</span>
          <h1 class="screen-title">{selectedEnvironmentId ? (allEnvironments.find((e) => e.id === selectedEnvironmentId)?.name ?? t("env.fallbackName")) : t("env.globalAll")}</h1>
        </div>

        <div class="screen-page-body">
          <div class="request-search-box">
            <input
              type="search"
              placeholder={t("env.searchVariables")}
              bind:value={envVarSearchQuery}
              class="request-search-input"
            />
            {#if envVarSearchQuery}
              <span class="request-count-badge">{filteredProjectVariables.length + filteredEnvironmentVariables.length}/{projectVariables.length + environmentVariables.length}</span>
            {/if}
          </div>
          <h4>{t("env.globalVariables", { count: filteredProjectVariables.length })}</h4>
          <div class="params-table">
            {#each filteredProjectVariables as v (v.id)}
              <div class="params-row">
                <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title={t("params.enabled")} />
                <span class="var-key">{v.key}</span>
                <span class="var-val">{revealedSecrets[v.id] ?? v.value}</span>
                {#if v.is_local}
                  <span class="badge badge-local" title={t("env.localBadgeTitle")}>{t("env.localBadge")}</span>
                {/if}
                {#if v.is_secret}
                  <span class="badge">{t("env.secretBadge")}</span>
                  {#if !revealedSecrets[v.id]}
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
                bind:value={newGlobalVarDraft.key}
                onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewGlobalVar(); } }}
              />
              <input
                placeholder={t("params.value")}
                bind:value={newGlobalVarDraft.value}
                onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewGlobalVar(); } }}
              />
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={newGlobalVarDraft.isSecret} /> {t("env.secret")}
              </label>
              <label class="checkbox-label" title={t("env.localHint")}>
                <input type="checkbox" bind:checked={newGlobalVarDraft.isLocal} /> {t("env.local")}
              </label>
            </div>
          </div>

          {#if selectedEnvironmentId}
            <h4>{t("env.environmentVariables", { count: filteredEnvironmentVariables.length })}</h4>
            <div class="params-table">
              {#each filteredEnvironmentVariables as v (v.id)}
                <div class="params-row">
                  <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title={t("params.enabled")} />
                  <span class="var-key">{v.key}</span>
                  <span class="var-val">{revealedSecrets[v.id] ?? v.value}</span>
                  {#if v.is_local}
                    <span class="badge badge-local" title={t("env.localBadgeTitle")}>{t("env.localBadge")}</span>
                  {/if}
                  {#if v.is_secret}
                    <span class="badge">{t("env.secretBadge")}</span>
                    {#if !revealedSecrets[v.id]}
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
                  bind:value={newEnvVarDraft.key}
                  onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
                />
                <input
                  placeholder={t("params.value")}
                  bind:value={newEnvVarDraft.value}
                  onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
                />
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={newEnvVarDraft.isSecret} /> {t("env.secret")}
                </label>
                <label class="checkbox-label" title={t("env.localHint")}>
                  <input type="checkbox" bind:checked={newEnvVarDraft.isLocal} /> {t("env.local")}
                </label>
              </div>
            </div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
{:else if activeScreen === "git"}
  {#if !activeWorkspaceId}
    <div class="screen-empty">
      <div class="empty-icon">{@render iconInboxEmpty()}</div>
      <p>{t("rail.loading")}</p>
    </div>
  {:else}
    <section class="screen-page">
      <div class="screen-page-header">
        <span class="screen-kicker">{t("git.title")}</span>
        <div class="screen-title-row">
          <h1 class="screen-title">{workspaces.find((w) => w.id === activeWorkspaceId)?.name ?? t("workspace.defaultName")}</h1>
        </div>
        <p class="screen-subtitle">{t("git.workspaceScopeHint")}</p>
      </div>

      <div class="modal-tabs" style="flex:none; padding: 0 var(--space-6);">
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "sync"}
            onclick={() => (gitActiveTab = "sync")}
          >
            {t("git.repoSync")}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "conflicts"}
            onclick={() => (gitActiveTab = "conflicts")}
          >
            {t("git.conflicts")}
            {#if gitStatus?.has_conflicts}
              <span class="tab-badge-alert">{gitStatus.conflict_files.length}</span>
            {/if}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "github"}
            onclick={() => (gitActiveTab = "github")}
          >
            {t("git.githubAuth")}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "projectfile"}
            onclick={() => (gitActiveTab = "projectfile")}
          >
            {t("git.projectFile")}
          </button>
        </div>

        <div class="screen-page-body">
          {#if gitActionFeedback}
            <div class="action-alert success">{gitActionFeedback}</div>
          {/if}
          {#if gitActionError}
            <div class="action-alert error">{gitActionError}</div>
          {/if}

          {#if gitActiveTab === "sync"}
            {#if legacyGitCandidates.length}
              <div class="git-panel-section legacy-git-banner">
                <h4>{t("git.legacyFoundTitle")}</h4>
                <p class="hint">{t("git.legacyFoundHint")}</p>
                <ul class="legacy-git-list">
                  {#each legacyGitCandidates as candidate (candidate.project_id)}
                    <li class="legacy-git-row">
                      <div class="legacy-git-row-info">
                        <strong>{candidate.project_name}</strong>
                        <span class="hint">{candidate.settings.repo_path}</span>
                      </div>
                      <button type="button" class="btn-primary btn-xs" onclick={() => adoptLegacyGitSettings(candidate)}>{t("git.legacyUseThis")}</button>
                    </li>
                  {/each}
                </ul>
                <button type="button" class="btn-ghost btn-xs" onclick={dismissLegacyGitCandidates}>{t("git.legacyDismiss")}</button>
              </div>
            {/if}
            <div class="git-panel-section">
              <h4>{t("git.repositorySettings")}</h4>
              <div class="form-row-stacked">
                <label for="git-repo-path-input">{t("git.repoPathLabel")}</label>
                <div class="input-with-actions">
                  <input
                    id="git-repo-path-input"
                    type="text"
                    placeholder={t("git.repoPathPlaceholder")}
                    bind:value={gitRepoPathInput}
                    class="path-input"
                  />
                  <button type="button" onclick={saveGitSettingsAction}>{t("git.savePath")}</button>
                  <button type="button" onclick={() => refreshGitStatus()} disabled={!gitRepoPathInput.trim() || gitStatusLoading}>
                    {gitStatusLoading ? t("git.checkingStatus") : t("git.checkStatus")}
                  </button>
                </div>
                <p class="hint">{t("git.repoPathHint")}</p>
              </div>

              {#if !gitStatus || !gitStatus.is_repo}
                <div class="alert-box-warning">
                  <p><strong>{t("git.notARepoTitle")}</strong> {t("git.notARepoDesc")}</p>
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={!gitRepoPathInput.trim() || gitLoading}
                    onclick={initializeGitRepoAction}
                  >
                    {gitLoading ? t("git.initializing") : t("git.initializeRepo")}
                  </button>
                </div>
              {:else}
                <div class="git-status-card">
                  <div class="status-summary-row">
                    <span class="status-label">{t("git.branch")}</span>
                    <strong>{gitStatus.branch}</strong>
                    <span class="status-sep">|</span>
                    <span class="status-label">{t("git.status")}</span>
                    <span class="git-badge-kind kind-{gitStatus.status_kind}">{gitStatus.status_kind.toUpperCase()}</span>
                    {#if gitStatus.ahead > 0}
                      <span class="badge-ahead">{t("git.unpushed", { count: gitStatus.ahead })}</span>
                    {/if}
                    {#if gitStatus.behind > 0}
                      <span class="badge-behind">{t("git.unpulled", { count: gitStatus.behind })}</span>
                    {/if}
                  </div>

                  {#if gitStatus.staged_files.length > 0 || gitStatus.unstaged_files.length > 0 || gitStatus.untracked_files.length > 0}
                    <div class="files-changed-summary">
                      {#if gitStatus.staged_files.length > 0}
                        <p class="file-category">{t("git.staged")} <code>{gitStatus.staged_files.join(", ")}</code></p>
                      {/if}
                      {#if gitStatus.unstaged_files.length > 0}
                        <p class="file-category">{t("git.modified")} <code>{gitStatus.unstaged_files.join(", ")}</code></p>
                      {/if}
                      {#if gitStatus.untracked_files.length > 0}
                        <p class="file-category">{t("git.untracked")} <code>{gitStatus.untracked_files.join(", ")}</code></p>
                      {/if}
                    </div>
                  {:else}
                    <p class="working-tree-clean">{t("git.workingTreeClean")}</p>
                  {/if}
                </div>

                <div class="git-commit-box">
                  <h4>{t("git.manualSync")}</h4>
                  <div class="commit-input-row">
                    <input
                      type="text"
                      placeholder={t("git.commitMessagePlaceholder")}
                      bind:value={gitCommitMessage}
                    />
                    <button
                      type="button"
                      class="btn-primary"
                      disabled={gitLoading}
                      onclick={commitAndPushAction}
                    >
                      {gitLoading ? t("git.syncing") : t("git.commitPush")}
                    </button>
                    <button
                      type="button"
                      disabled={gitLoading}
                      onclick={pullRepositoryAction}
                    >
                      {gitLoading ? t("git.pulling") : t("git.pullRemote")}
                    </button>
                  </div>
                  <div class="quick-git-actions">
                    <button type="button" class="icon-btn-text" onclick={saveWorkspaceToRepoAction} disabled={gitLoading}>
                      {t("git.saveProjectFile")}
                    </button>
                    <button type="button" class="icon-btn-text" onclick={viewDiffAction} disabled={gitLoading}>
                      {t("git.viewDiff")}
                    </button>
                    <button type="button" class="icon-btn-text" onclick={viewHistoryAction} disabled={gitLoading}>
                      {t("git.commitHistory")}
                    </button>
                  </div>
                </div>

                <div class="auto-sync-box">
                  <label class="checkbox-label">
                    <input type="checkbox" bind:checked={gitAutoSyncInput} onchange={saveGitSettingsAction} />
                    <strong>{t("git.enableAutoSync")}</strong>
                  </label>
                  <p class="hint">{t("git.autoSyncDesc")}</p>
                  {#if gitSettings?.last_sync_at}
                    <p class="hint">{t("git.lastSynced", { time: new Date(gitSettings.last_sync_at).toLocaleString() })}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "conflicts"}
            <div class="git-panel-section">
              <h4>{t("git.conflictDetection")}</h4>
              {#if !gitStatus?.has_conflicts || gitStatus.conflict_files.length === 0}
                <div class="clean-box">
                  <p>{t("git.noConflicts")}</p>
                </div>
              {:else}
                <div class="conflict-alert-box">
                  <p><strong>{t("git.conflictsDetectedTitle")}</strong> {t("git.conflictsDetectedDesc")}</p>
                </div>
                <div class="conflicts-list">
                  {#each gitStatus.conflict_files as file}
                    <div class="conflict-item-card">
                      <div class="conflict-item-header">
                        <span class="conflict-filename">{@render iconFileText()} {file}</span>
                        <div class="conflict-choices">
                          <button
                            type="button"
                            class="btn-choice"
                            title={t("git.view3wayDiffTitle")}
                            onclick={() => loadConflictVersions(file)}
                          >
                            {selectedConflictFile === file && conflictVersions ? t("git.viewing3wayDiff") : t("git.view3wayDiff")}
                          </button>
                          <button
                            type="button"
                            class="btn-choice local"
                            title={t("git.keepLocalTitle")}
                            onclick={() => { resolveConflictAction(file, "ours"); if (selectedConflictFile === file) { selectedConflictFile = null; conflictVersions = null; } }}
                            disabled={gitLoading}
                          >
                            {t("git.keepLocal")}
                          </button>
                          <button
                            type="button"
                            class="btn-choice remote"
                            title={t("git.keepRemoteTitle")}
                            onclick={() => { resolveConflictAction(file, "theirs"); if (selectedConflictFile === file) { selectedConflictFile = null; conflictVersions = null; } }}
                            disabled={gitLoading}
                          >
                            {t("git.keepRemote")}
                          </button>
                        </div>
                      </div>
                      <p class="hint">{t("git.conflictDecideHint")}</p>

                      {#if selectedConflictFile === file}
                        <div class="conflict-3way">
                          {#if conflictVersionsLoading}
                            <p class="hint">{t("git.loadingVersions")}</p>
                          {:else if conflictVersions}
                            <div class="conflict-3way-col">
                              <span class="screen-kicker">{t("git.baseAncestor")}</span>
                              <pre class="body-view conflict-3way-pre">{conflictVersions.base ?? t("git.noCommonAncestor")}</pre>
                            </div>
                            <div class="conflict-3way-col">
                              <span class="screen-kicker">{t("git.localOurs")}</span>
                              <pre class="body-view conflict-3way-pre">{conflictVersions.local ?? t("git.absentLocally")}</pre>
                            </div>
                            <div class="conflict-3way-col">
                              <span class="screen-kicker">{t("git.remoteTheirs")}</span>
                              <pre class="body-view conflict-3way-pre">{conflictVersions.remote ?? t("git.absentRemote")}</pre>
                            </div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "github"}
            <div class="git-panel-section">
              <h4>{t("git.githubCollab")}</h4>
              <p class="hint">{t("git.githubTokenDesc")}</p>

              <div class="form-row-stacked">
                <label for="github-pat-input">{t("git.patLabel")}</label>
                <div class="input-with-actions">
                  <input
                    id="github-pat-input"
                    type={githubShowToken ? "text" : "password"}
                    placeholder="ghp_..."
                    bind:value={githubTokenInput}
                  />
                  <button type="button" onclick={() => (githubShowToken = !githubShowToken)}>
                    {githubShowToken ? t("git.hide") : t("git.show")}
                  </button>
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={!githubTokenInput.trim() || githubValidating}
                    onclick={() => { saveGitSettingsAction(); verifyGitHubTokenAction(true); }}
                  >
                    {githubValidating ? t("git.verifying") : t("git.verifyToken")}
                  </button>
                </div>
              </div>

              {#if githubUser}
                <div class="github-profile-card">
                  {#if githubUser.avatar_url}
                    <img src={githubUser.avatar_url} alt={githubUser.login} class="github-avatar" />
                  {/if}
                  <div class="github-profile-info">
                    <strong>{githubUser.name ?? githubUser.login}</strong>
                    <span class="hint">@{githubUser.login}</span>
                    {#if githubUser.email}
                      <span class="hint">{githubUser.email}</span>
                    {/if}
                  </div>
                  <span class="badge badge-success">{t("git.authenticated")}</span>
                </div>
              {/if}

              <div class="form-row-stacked">
                <label for="git-remote-url-input">{t("git.remoteUrlLabel")}</label>
                <div class="input-with-actions">
                  <input
                    id="git-remote-url-input"
                    type="text"
                    placeholder="https://github.com/owner/repository.git"
                    bind:value={gitRemoteUrlInput}
                  />
                  <button type="button" onclick={saveGitSettingsAction}>{t("git.saveRemote")}</button>
                  <button
                    type="button"
                    disabled={!githubTokenInput.trim() || !gitRemoteUrlInput.trim() || githubValidating}
                    onclick={checkGitHubRepoAction}
                  >
                    {t("git.checkPermissions")}
                  </button>
                </div>
              </div>

              {#if githubRepoInfo}
                <div class="repo-permissions-card">
                  <h5>{t("git.repository", { name: githubRepoInfo.full_name })}</h5>
                  <div class="perm-badges">
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.pull}>
                      {t("git.readPull", { state: githubRepoInfo.permissions?.pull ? t("git.granted") : t("git.denied") })}
                    </span>
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.push}>
                      {t("git.writePush", { state: githubRepoInfo.permissions?.push ? t("git.granted") : t("git.denied") })}
                    </span>
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.admin}>
                      {t("git.admin", { state: githubRepoInfo.permissions?.admin ? t("git.granted") : t("git.denied") })}
                    </span>
                  </div>
                  <p class="hint">{t("git.defaultBranchLine", { branch: githubRepoInfo.default_branch, visibility: githubRepoInfo.private ? t("git.private") : t("git.public") })}</p>
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "projectfile"}
            <div class="git-panel-section">
              <h4>{t("git.canonicalFormat")}</h4>
              <p class="hint">{t("git.canonicalFormatDesc")}</p>
              {#if !selectedProjectId}
                <p class="hint">{t("git.selectProjectFirst")}</p>
              {/if}

              <div class="projectfile-options">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={projectFileMaskSecrets} />
                  {t("git.maskSecrets")}
                </label>
                <button type="button" class="btn-primary" onclick={exportProjectFileAction} disabled={!selectedProjectId}>
                  {t("git.generateJson")}
                </button>
              </div>

              {#if projectFileJson}
                <div class="json-preview-box">
                  <div class="json-preview-toolbar">
                    <span>light-postman.json</span>
                    <div class="toolbar-actions">
                      <button type="button" onclick={downloadProjectFile}>{t("git.downloadFile")}</button>
                      <button type="button" onclick={importProjectFileAction}>{t("git.importIntoProject")}</button>
                    </div>
                  </div>
                  <textarea rows="12" bind:value={projectFileJson} class="code-area"></textarea>
                  {#if projectFileStatus}
                    <p class="hint">{projectFileStatus}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </div>
    </section>
  {/if}
{:else if activeScreen === "launcher"}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("launcher.kicker")}</span>
      <h1 class="screen-title">{t("launcher.title")}</h1>
      <p class="screen-subtitle">{t("launcher.subtitle", { count: projects.length })}</p>
    </div>

    {#if projects.length === 0}
      <div class="screen-empty">
        <div class="empty-icon">{@render iconFolder()}</div>
        <p>{t("launcher.noProjects")}</p>
        <div style="display:flex; gap: var(--space-3); margin-top: var(--space-4);">
          <button type="button" class="btn-primary" onclick={async () => { await quickCreateProject(); activeScreen = "workspace"; }}>+ {t("launcher.newProject")}</button>
          <button type="button" class="btn-secondary" onclick={() => { collectionImportTarget = "new"; importActiveTab = "collection"; collectionImportReport = null; collectionImportError = ""; showImportDialog = true; }}>{t("launcher.importCollection")}</button>
        </div>
      </div>
    {:else}
      <div class="launcher-grid">
        {#each projects as p (p.id)}
          <button
            type="button"
            class="launcher-card"
            class:active={p.id === selectedProjectId}
            onclick={() => { selectProject(p.id); activeScreen = "workspace"; }}
          >
            <span class="screen-kicker" class:current={p.id === selectedProjectId}>{p.id === selectedProjectId ? t("launcher.openNow") : t("launcher.updated", { date: new Date(p.updated_at).toLocaleDateString() })}</span>
            <div class="launcher-card-name">{p.name}</div>
            <div class="hr"></div>
            <div class="launcher-card-meta">
              <span>{t("launcher.requestCount", { count: projectRequestCounts[p.id] ?? 0 })}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}

    <div class="screen-page-body" style="flex:none; display:flex; gap: var(--space-3);">
      <button type="button" class="btn-primary" onclick={async () => { await quickCreateProject(); activeScreen = "workspace"; }}>+ {t("launcher.newProject")}</button>
      <button type="button" class="btn-secondary" onclick={() => { collectionImportTarget = "new"; importActiveTab = "collection"; collectionImportReport = null; collectionImportError = ""; showImportDialog = true; }}>{t("launcher.importCollection")}</button>
    </div>
  </section>
{:else if activeScreen === "history"}
  {#if !selectedProjectId}
    {@render noProjectPicker(t("rail.history"))}
  {:else}
    <section class="screen-page">
      <div class="history-toolbar">
        <span class="history-toolbar-project-label">{t("env.project")}</span>
        {@render projectSwitcher()}
        <div class="hr-v"></div>
        <input class="history-search" placeholder={t("history.filterPlaceholder")} bind:value={historySearchQuery} />
        <div class="hr-v"></div>
        <span class="screen-empty-inline">{t("history.resultsCount", { shown: filteredProjectHistory.length, total: projectHistory.length })}</span>
        <div class="response-stat-spacer"></div>
        <button type="button" class="history-filter-chip" class:active={historyShowFailuresOnly} onclick={() => (historyShowFailuresOnly = !historyShowFailuresOnly)}>
          {t("history.failuresOnly")}
        </button>
        <button type="button" class="btn-ghost btn-xs" onclick={refreshProjectHistory}>{t("history.refresh")}</button>
      </div>

      <div class="history-header-row">
        <span>{t("history.colMethod")}</span>
        <span>{t("history.colRequest")}</span>
        <span>{t("history.colStatus")}</span>
        <span>{t("history.colTime")}</span>
        <span>{t("history.colWhen")}</span>
      </div>

      <div class="history-rows">
        {#if historyLoading}
          <p class="screen-empty-inline" style="padding: var(--space-4);">{t("history.loading")}</p>
        {:else if filteredProjectHistory.length === 0}
          <p class="screen-empty-inline" style="padding: var(--space-4);">
            {projectHistory.length === 0 ? t("history.noRequestsSent") : t("history.noResultsMatch")}
          </p>
        {:else}
          {#each filteredProjectHistory as h (h.id)}
            <button
              type="button"
              class="history-row"
              onclick={async () => { await openRequest(h.request_id); await openHistoryResponse(h.id); activeScreen = "workspace"; responseExpanded = true; }}
            >
              <span class="history-method">{h.method}</span>
              <span class="history-path">{h.request_name} · {h.url}</span>
              <span class:status-ok={h.status < 400} class:status-err={h.status >= 400}>{h.status}</span>
              <span>{h.duration_ms} ms</span>
              <span>{new Date(h.created_at).toLocaleString()}</span>
            </button>
          {/each}
        {/if}
      </div>
    </section>
  {/if}
{:else if activeScreen === "settings"}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("rail.settings")}</span>
      <h1 class="screen-title">{t("settings.title")}</h1>
      <p class="screen-subtitle">{t("settings.subtitle")}</p>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.autoSyncInterval")}</div>
        <div class="screen-empty-inline">{t("settings.autoSyncHint")}</div>
      </div>
      <div class="seg">
        {#each [[30000, "30s"], [60000, "60s"], [120000, "2m"], [300000, "5m"]] as [ms, label]}
          <button type="button" class="seg-opt" class:active={autoSyncIntervalMs === ms} onclick={() => setAutoSyncIntervalMs(ms as number)}>{label}</button>
        {/each}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.appearance")}</div>
        <div class="screen-empty-inline">{t("settings.appearanceHint")}</div>
      </div>
      <div class="seg">
        {#each THEME_OPTIONS as opt (opt.id)}
          <button type="button" class="seg-opt" class:active={themeMode === opt.id} onclick={() => setThemeMode(opt.id)}>{t(opt.label)}</button>
        {/each}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">Sidebar Sections</div>
        <div class="screen-empty-inline">Choose which sections appear in the sidebar accordion.</div>
      </div>
      <div class="settings-sections-checkbox-grid">
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.collections}>
          <input type="checkbox" checked={sidebarSectionsVisible.collections} onchange={() => toggleSidebarSectionVisibility("collections")} />
          <span>Collections</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.environments}>
          <input type="checkbox" checked={sidebarSectionsVisible.environments} onchange={() => toggleSidebarSectionVisibility("environments")} />
          <span>Environments</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.datasets}>
          <input type="checkbox" checked={sidebarSectionsVisible.datasets} onchange={() => toggleSidebarSectionVisibility("datasets")} />
          <span>Datasets</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.documents}>
          <input type="checkbox" checked={sidebarSectionsVisible.documents} onchange={() => toggleSidebarSectionVisibility("documents")} />
          <span>Documents</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.specs}>
          <input type="checkbox" checked={sidebarSectionsVisible.specs} onchange={() => toggleSidebarSectionVisibility("specs")} />
          <span>Specs</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.mocks}>
          <input type="checkbox" checked={sidebarSectionsVisible.mocks} onchange={() => toggleSidebarSectionVisibility("mocks")} />
          <span>Mocks</span>
        </label>
        <label class="settings-checkbox-pill" class:active={sidebarSectionsVisible.flows}>
          <input type="checkbox" checked={sidebarSectionsVisible.flows} onchange={() => toggleSidebarSectionVisibility("flows")} />
          <span>Flows</span>
        </label>
      </div>
    </div>

    <div class="settings-screen-block">
      <p class="screen-subtitle">{t("theme.subtitle")}</p>
      <div class="theme-compare">
        {#each THEME_OPTIONS as opt (opt.id)}
          <button type="button" class="theme-compare-col" class:active={themeMode === opt.id} onclick={() => setThemeMode(opt.id)}>
            <div class="theme-compare-header">
              <span class="screen-kicker">{t(opt.label)}</span>
              {#if themeMode === opt.id}<span class="theme-active-badge">{t("theme.active")}</span>{/if}
            </div>
            <div
              class="theme-swatch"
              data-theme={opt.id}
              style="{surfaceStyleOverride(surfaceTint, opt.id)} {accentStyleOverride(accentColor)} {fontStyleOverride(headingFontOverride, bodyFontOverride)} {textColorStyleOverride(textColorOverride)}"
            >
              <div class="theme-swatch-topbar">
                <span>{t("rail.brand")}</span>
                <span class="theme-swatch-sync">SYNC</span>
              </div>
              <div class="theme-swatch-body">
                <div class="theme-swatch-side">
                  <div class="screen-kicker">Explorer</div>
                  <div>List charges</div>
                  <div>Create charge</div>
                  <div class="theme-swatch-active">Refund charge</div>
                </div>
                <div class="theme-swatch-main">
                  <div class="theme-swatch-path">POST /v1/charges/:id/refund</div>
                  <div class="hr"></div>
                  <div class="theme-swatch-status">200 OK</div>
                  <div class="screen-empty-inline">214 ms · 1.2 KB</div>
                </div>
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.accentColor")}</div>
        <div class="screen-empty-inline">{t("settings.accentColorHint")}</div>
      </div>
      <div class="accent-picker">
        {#each ACCENT_PRESETS as preset (preset.id)}
          <button
            type="button"
            class="accent-swatch"
            class:active={accentColor === preset.hex}
            style="background: {preset.hex}"
            title={t(`accent.${preset.id}`)}
            onclick={() => setAccentColor(preset.hex)}
          ></button>
        {/each}
        <label class="accent-swatch accent-swatch-custom" style={accentColor && !ACCENT_PRESETS.some((p) => p.hex === accentColor) ? `background: ${accentColor}` : ""} title={t("accent.custom")}>
          <input type="color" value={accentColor ?? "#c1603f"} oninput={(e) => setAccentColor((e.currentTarget as HTMLInputElement).value)} />
          {#if !accentColor || ACCENT_PRESETS.some((p) => p.hex === accentColor)}<span class="accent-swatch-plus">+</span>{/if}
        </label>
        {#if accentColor}
          <button type="button" class="btn-ghost accent-reset" onclick={() => setAccentColor(null)}>{t("accent.reset")}</button>
        {/if}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.surfaceTint")}</div>
        <div class="screen-empty-inline">{surfaceTintAvailable ? t("settings.surfaceTintHint") : t("settings.surfaceTintUnavailable")}</div>
      </div>
      {#if surfaceTintAvailable}
        <div class="accent-picker">
          {#each SURFACE_TINTS as tint (tint.id)}
            <button
              type="button"
              class="accent-swatch"
              class:active={surfaceTint === tint.id}
              style="background: {themeMode === 'dark' ? tint.dark.bg : tint.light.bg}"
              title={t(`surfaceTint.${tint.id}`)}
              onclick={() => setSurfaceTint(tint.id)}
            ></button>
          {/each}
          <label class="accent-swatch accent-swatch-custom" style={isCustomSurfaceTint(surfaceTint) ? `background: ${surfaceTint}` : ""} title={t("accent.custom")}>
            <input type="color" value={isCustomSurfaceTint(surfaceTint) ? surfaceTint : "#faf6ef"} oninput={(e) => setSurfaceTint((e.currentTarget as HTMLInputElement).value)} />
            {#if !isCustomSurfaceTint(surfaceTint)}<span class="accent-swatch-plus">+</span>{/if}
          </label>
          {#if surfaceTint}
            <button type="button" class="btn-ghost accent-reset" onclick={() => setSurfaceTint(null)}>{t("accent.reset")}</button>
          {/if}
        </div>
      {/if}
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.headingFont")}</div>
        <div class="screen-empty-inline">{t("settings.headingFontHint")}</div>
      </div>
      <select
        class="url-input"
        value={headingFontOverride ?? ""}
        onchange={(e) => setHeadingFontOverride((e.currentTarget as HTMLSelectElement).value || null)}
      >
        <option value="">{t("settings.themeDefault")}</option>
        {#each THEME_FONT_OPTIONS as font (font.id)}
          <option value={font.id}>{font.label}</option>
        {/each}
      </select>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.bodyFont")}</div>
        <div class="screen-empty-inline">{t("settings.bodyFontHint")}</div>
      </div>
      <select
        class="url-input"
        value={bodyFontOverride ?? ""}
        onchange={(e) => setBodyFontOverride((e.currentTarget as HTMLSelectElement).value || null)}
      >
        <option value="">{t("settings.themeDefault")}</option>
        {#each THEME_FONT_OPTIONS as font (font.id)}
          <option value={font.id}>{font.label}</option>
        {/each}
      </select>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.textColor")}</div>
        <div class="screen-empty-inline">{t("settings.textColorHint")}</div>
        {#if textColorContrastWarning}<div class="warn-inline">{textColorContrastWarning}</div>{/if}
      </div>
      <div class="accent-picker">
        <label class="accent-swatch accent-swatch-custom" style={textColorOverride ? `background: ${textColorOverride}` : ""} title={t("accent.custom")}>
          <input type="color" value={textColorOverride ?? "#2b2620"} oninput={(e) => setTextColorOverride((e.currentTarget as HTMLInputElement).value)} />
          {#if !textColorOverride}<span class="accent-swatch-plus">+</span>{/if}
        </label>
        {#if textColorOverride}
          <button type="button" class="btn-ghost accent-reset" onclick={() => setTextColorOverride(null)}>{t("accent.reset")}</button>
        {/if}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.uiScale")}</div>
        <div class="screen-empty-inline">{t("settings.uiScaleHint")}</div>
      </div>
      <div class="seg">
        {#each [[85, t("settings.scaleSmall")], [100, t("settings.scaleDefault")], [115, t("settings.scaleLarge")], [130, t("settings.scaleExtraLarge")]] as [pct, label]}
          <button type="button" class="seg-opt" class:active={uiScale === pct} onclick={() => setUiScale(pct as number)}>{label}</button>
        {/each}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.language")}</div>
        <div class="screen-empty-inline">{t("settings.languageHint")}</div>
      </div>
      <div class="seg">
        <button type="button" class="seg-opt" class:active={locale === "en"} onclick={() => setLocale("en")}>{t("settings.languageEnglish")}</button>
        <button type="button" class="seg-opt" class:active={locale === "ar"} onclick={() => setLocale("ar")}>{t("settings.languageArabic")}</button>
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.shortcuts")}</div>
        <div class="screen-empty-inline">{t("settings.shortcutsHint")}</div>
      </div>
      <div class="shortcuts-list">
        {#each SHORTCUT_DEFS as def (def.id)}
          <label class="shortcut-toggle-row">
            <input
              type="checkbox"
              checked={shortcutsEnabled[def.id]}
              onchange={(e) => setShortcutEnabled(def.id, (e.target as HTMLInputElement).checked)}
            />
            <span class="shortcut-label">{t(def.label)}</span>
            <kbd class="shortcut-keys">{def.keys}</kbd>
          </label>
        {/each}
      </div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.responseMemory")}</div>
        <div class="screen-empty-inline">{t("settings.responseMemoryHint")}</div>
      </div>
      <div class="settings-screen-row-value">256 KB</div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.scriptTimeout")}</div>
        <div class="screen-empty-inline">{t("settings.scriptTimeoutHint")}</div>
      </div>
      <div class="settings-screen-row-value">1.5s / 2s</div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.responseCache")}</div>
        <div class="screen-empty-inline">{t("settings.responseCacheHint")}</div>
      </div>
      <div class="settings-screen-row-value">{t("settings.none")}</div>
    </div>

    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.historyRetention")}</div>
        <div class="screen-empty-inline">{t("settings.historyRetentionHint")}</div>
      </div>
      <div class="settings-screen-row-value">{t("settings.unbounded")}</div>
    </div>

    {#if systemDiagnostics}
      <div class="settings-screen-row">
        <div>
          <div class="settings-screen-row-label">{t("settings.currentProcess")}</div>
          <div class="screen-empty-inline">{t("settings.currentProcessHint")}</div>
        </div>
        <div class="settings-screen-diagnostics-grid">
          <span>{t("settings.memoryRss")}</span><strong>{formatByteSize(systemDiagnostics.process_rss_bytes)}</strong>
          <span>{t("settings.database")}</span><strong>{formatByteSize(systemDiagnostics.db_size_bytes)}</strong>
          <span>{t("settings.walFile")}</span><strong>{formatByteSize(systemDiagnostics.db_wal_size_bytes)}</strong>
          <span>{t("settings.projects")}</span><strong>{systemDiagnostics.total_projects}</strong>
          <span>{t("settings.requests")}</span><strong>{systemDiagnostics.total_requests}</strong>
          <span>{t("settings.responsesStored")}</span><strong>{systemDiagnostics.total_responses}</strong>
          <span>{t("settings.consoleEvents")}</span><strong>{systemDiagnostics.console_events_count}</strong>
          <span>{t("settings.uptime")}</span><strong>{Math.floor(systemDiagnostics.uptime_seconds / 60)} min</strong>
        </div>
      </div>
    {/if}
  </section>
{/if}
  </div>

  {#if showConsole}
    <div
      class="console-resize-handle"
      class:resizing={consoleResizing}
      onmousedown={startConsoleResize}
      onkeydown={(e) => {
        if (e.key === "ArrowUp") consoleHeight = Math.min(window.innerHeight - 160, consoleHeight + 16);
        else if (e.key === "ArrowDown") consoleHeight = Math.max(120, consoleHeight - 16);
      }}
      role="slider"
      aria-orientation="horizontal"
      aria-label={t("console.resizeHandle")}
      aria-valuenow={consoleHeight}
      aria-valuemin={120}
      aria-valuemax={900}
      tabindex="0"
    ></div>
    <div class="console-drawer" style="height: {consoleHeight}px">
      <div class="console-header">
        <div class="console-title-group">
          <span class="console-title">{t("console.title")}</span>
          <span class="console-count-badge">{t("console.eventsCount", { count: filteredConsoleEvents.length })}</span>
        </div>

        <div class="console-toolbar">
          <select bind:value={consoleLevelFilter} class="console-select">
            <option value="all">{t("console.allLevels")}</option>
            <option value="info">{t("console.info")}</option>
            <option value="warn">{t("console.warn")}</option>
            <option value="error">{t("console.error")}</option>
            <option value="debug">{t("console.debug")}</option>
          </select>

          <label class="console-check-label" title={t("console.activeRequestOnlyTitle")}>
            <input type="checkbox" bind:checked={consoleActiveRequestOnly} />
            {t("console.activeRequestOnly")}
          </label>

          <input
            type="search"
            placeholder={t("console.filterPlaceholder")}
            bind:value={consoleSearchFilter}
            class="console-search"
          />

          <button type="button" class="console-btn" onclick={refreshConsoleEvents} title={t("console.refreshTitle")}>{t("console.refresh")}</button>
          <button type="button" class="console-btn" onclick={clearConsole} title={t("console.clearTitle")}>{t("console.clear")}</button>
          <button type="button" class="console-btn" onclick={copyConsoleLog} title={t("console.copyTitle")}>{t("console.copy")}</button>
          <button type="button" class="console-btn" onclick={exportConsoleJson} title={t("console.exportJsonTitle")}>{t("console.exportJson")}</button>
          <button type="button" class="console-close-btn" onclick={() => (showConsole = false)} title={t("console.closeTitle")}>{@render iconClose()}</button>
        </div>
      </div>

      <div class="console-body">
        {#if filteredConsoleEvents.length === 0}
          <div class="console-empty">{t("console.empty")}</div>
        {:else}
          <div class="console-events-list">
            {#each filteredConsoleEvents as evt (evt.id)}
              <div class="console-row" class:error-row={evt.level === "error"} class:warn-row={evt.level === "warn"}>
                <div
                  class="console-row-summary"
                  onclick={() => toggleEventExpanded(evt.id)}
                  role="button"
                  tabindex="0"
                  onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") toggleEventExpanded(evt.id); }}
                >
                  <span class="evt-expander">{#if expandedEventIds.has(evt.id)}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</span>
                  <span class="evt-time">{formatConsoleTime(evt.timestamp)}</span>
                  <span class="evt-level level-{evt.level}">{evt.level.toUpperCase()}</span>
                  <span class="evt-type">{evt.event_type}</span>
                  <span class="evt-cid" title={t("console.correlationIdTitle", { id: evt.correlation_id ?? "" })}>#{evt.correlation_id ? evt.correlation_id.slice(0, 8) : "—"}</span>
                  <span class="evt-msg">{evt.message}</span>
                </div>

                {#if expandedEventIds.has(evt.id) && evt.details}
                  {@const d = evt.details as Record<string, unknown>}
                  {@const known = ["request_start", "response_received", "cookie_injected", "request_error", "test_assertion"].includes(evt.event_type)}
                  <div class="console-row-details">
                    <div class="details-actions">
                      {#if known}
                        <button type="button" class="console-mini-btn" onclick={() => toggleRawDetails(evt.id)}>
                          {rawDetailsVisible.has(evt.id) ? t("console.hideRawJson") : t("console.viewRawJson")}
                        </button>
                      {/if}
                      <button type="button" class="console-mini-btn" onclick={() => copyEventDetails(evt)}>{t("console.copyDetailsJson")}</button>
                    </div>

                    {#if evt.event_type === "request_start"}
                      <div class="detail-kv-grid">
                        <span class="detail-k">{t("console.method")}</span><span class="detail-v">{detailStr(d, "method")}</span>
                        <span class="detail-k">{t("console.url")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "url")}</span>
                        <span class="detail-k">{t("console.authType")}</span><span class="detail-v">{detailStr(d, "auth_type")}</span>
                        <span class="detail-k">{t("console.bodyBytes")}</span><span class="detail-v">{formatByteSize(Number(d.body_bytes ?? 0))}</span>
                        <span class="detail-k">{t("console.timeout")}</span><span class="detail-v">{detailStr(d, "timeout_ms")} ms</span>
                        <span class="detail-k">{t("console.followRedirects")}</span><span class="detail-v">{String(d.follow_redirects)}</span>
                        <span class="detail-k">{t("console.verifySsl")}</span><span class="detail-v">{String(d.verify_ssl)}</span>
                        {#if detailStr(d, "proxy")}
                          <span class="detail-k">{t("console.proxy")}</span><span class="detail-v">{detailStr(d, "proxy")}</span>
                        {/if}
                        {#if detailStr(d, "http_version")}
                          <span class="detail-k">{t("console.httpVersion")}</span><span class="detail-v">{detailStr(d, "http_version")}</span>
                        {/if}
                      </div>
                      {#if asHeaderRows(d.headers).length}
                        <table class="detail-header-table">
                          <thead><tr><th>{t("params.key")}</th><th>{t("params.value")}</th></tr></thead>
                          <tbody>
                            {#each asHeaderRows(d.headers) as h, i (i)}
                              <tr class:disabled-row={h.enabled === false}><td>{h.key}</td><td>{h.value}</td></tr>
                            {/each}
                          </tbody>
                        </table>
                      {/if}
                    {:else if evt.event_type === "response_received"}
                      <div class="detail-kv-grid">
                        <span class="detail-k">{t("console.status")}</span><span class="detail-v">{detailStr(d, "status")} {detailStr(d, "status_text")}</span>
                        <span class="detail-k">{t("console.duration")}</span><span class="detail-v">{detailStr(d, "duration_ms")} ms</span>
                        <span class="detail-k">{t("console.bodySize")}</span><span class="detail-v">{formatByteSize(Number(d.body_size ?? 0))}</span>
                        {#if detailStr(d, "content_type")}
                          <span class="detail-k">{t("console.contentType")}</span><span class="detail-v">{detailStr(d, "content_type")}</span>
                        {/if}
                      </div>
                      {#if asHeaderRows(d.headers).length}
                        <table class="detail-header-table">
                          <thead><tr><th>{t("params.key")}</th><th>{t("params.value")}</th></tr></thead>
                          <tbody>
                            {#each asHeaderRows(d.headers) as h, i (i)}
                              <tr><td>{h.key}</td><td>{h.value}</td></tr>
                            {/each}
                          </tbody>
                        </table>
                      {/if}
                      {#if asCookieRows(d.cookies).length}
                        <table class="detail-header-table">
                          <thead><tr><th>{t("console.cookieName")}</th><th>{t("params.value")}</th><th>{t("console.cookieDomain")}</th></tr></thead>
                          <tbody>
                            {#each asCookieRows(d.cookies) as c, i (i)}
                              <tr><td>{c.name}</td><td>{c.value || "—"}</td><td>{c.domain}{c.path}</td></tr>
                            {/each}
                          </tbody>
                        </table>
                      {/if}
                    {:else if evt.event_type === "cookie_injected"}
                      <p class="detail-list-label">{t("console.injectedCookies")}</p>
                      <ul class="detail-plain-list">
                        {#each (Array.isArray(d.cookies) ? d.cookies : []) as c, i (i)}
                          <li>{String(c)}</li>
                        {/each}
                      </ul>
                    {:else if evt.event_type === "request_error"}
                      <div class="detail-kv-grid">
                        <span class="detail-k">{t("console.error")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "error")}</span>
                      </div>
                    {:else if evt.event_type === "test_assertion"}
                      <div class="detail-kv-grid">
                        <span class="detail-k">{t("console.testName")}</span><span class="detail-v">{detailStr(d, "name")}</span>
                        <span class="detail-k">{t("console.testPassed")}</span><span class="detail-v">{d.passed ? t("console.pass") : t("console.fail")}</span>
                        {#if detailStr(d, "error")}
                          <span class="detail-k">{t("console.testError")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "error")}</span>
                        {/if}
                      </div>
                    {/if}

                    {#if !known || rawDetailsVisible.has(evt.id)}
                      <pre class="console-json-view">{JSON.stringify(evt.details, null, 2)}</pre>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <footer class="app-status-bar">
    <div class="status-left">
      <button
        type="button"
        class="status-btn icon-only"
        class:active={sidebarVisible}
        title={sidebarVisible ? "Hide sidebar (Project tree)" : "Show sidebar (Project tree)"}
        onclick={() => setSidebarVisible(!sidebarVisible)}
      >
        {@render iconSidebar()}
      </button>
      {#if selectedProjectId}
        <button
          type="button"
          class="status-btn git-btn"
          title={t("footer.gitSyncTitle")}
          onclick={() => { activeScreen = "git"; if (gitRepoPathInput) refreshGitStatus(); }}
        >
          <span class="git-icon">{@render iconGitBranch()}</span>
          <span>{gitStatus?.branch || "main"}</span>
          <span class="git-sync-arrows">&#8644;</span>
        </button>
      {/if}
      <button
        type="button"
        class="status-btn console-btn"
        class:active={showConsole}
        onclick={() => {
          showConsole = !showConsole;
          if (showConsole) refreshConsoleEvents();
        }}
      >
        <span>{t("console.title")}</span>
        {#if consoleErrorCount > 0}<span class="status-badge-err">! {consoleErrorCount}</span>{/if}
        {#if consoleWarnCount > 0}<span class="status-badge-warn">! {consoleWarnCount}</span>{/if}
      </button>
    </div>
    <div class="status-right">
      <button
        type="button"
        class="status-btn"
        class:active={activeScreen === "globals"}
        onclick={() => {
          loadVariables();
          activeScreen = "globals";
          rightPanel = "variables";
          setRightSidebarVisible(true);
        }}
      >
        <span>Globals</span>
      </button>
      <button type="button" class="status-btn" class:active={activeScreen === "environments"} onclick={() => { loadVariables(); activeScreen = "environments"; }}>
        <span>Environments</span>
      </button>
      <button type="button" class="status-btn" class:active={activeScreen === "settings"} onclick={() => (activeScreen = activeScreen === "settings" ? "workspace" : "settings")}>
        <span>Tools</span>
      </button>
      <button
        type="button"
        class="status-btn icon-only"
        class:active={utilityRailVisible}
        title={utilityRailVisible ? "Hide menu items rail" : "Show menu items rail"}
        onclick={toggleUtilityRail}
      >
        {@render iconLayout()}
      </button>
    </div>
  </footer>

  {#if showInviteModal}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showInviteModal = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showInviteModal = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container" style="max-width: 460px;">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <h3>Invite to Workspace</h3>
            <span class="modal-sub">Collaborate in real time with your engineering team.</span>
          </div>
          <button type="button" class="modal-close-btn" title="Close" onclick={() => (showInviteModal = false)}>{@render iconClose()}</button>
        </div>
        <div style="padding: 16px 20px; display: flex; flex-direction: column; gap: 14px;">
          <div>
            <label style="display: block; font-size: 11px; font-weight: 600; color: #888; margin-bottom: 6px;">EMAIL ADDRESS</label>
            <input
              type="email"
              placeholder="colleague@organization.com"
              bind:value={inviteEmail}
              class="request-search-input"
              style="width: 100%; box-sizing: border-box; height: 34px;"
            />
          </div>
          <div>
            <label style="display: block; font-size: 11px; font-weight: 600; color: #888; margin-bottom: 6px;">ROLE</label>
            <select bind:value={inviteRole} class="snippet-target-picker" style="width: 100%;">
              <option value="editor">Editor (can edit and send requests)</option>
              <option value="viewer">Viewer (read-only)</option>
              <option value="admin">Admin (full workspace control)</option>
            </select>
          </div>
        </div>
        <div class="modal-actions" style="display: flex; justify-content: flex-end; gap: 10px; padding: 14px 20px; border-top: 1px solid #333;">
          <button type="button" class="btn-cancel" onclick={() => (showInviteModal = false)}>Cancel</button>
          <button type="button" class="btn-send" onclick={() => {
            showInviteModal = false;
            errorMessage = "Workspace invites aren't available yet — this is a local, single-user app with no account/collaboration backend.";
          }}>Send Invite</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showUpgradeModal}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showUpgradeModal = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showUpgradeModal = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container" style="max-width: 650px;">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <h3>Choose a Plan for Your Team</h3>
            <span class="modal-sub">Unlock unlimited mock requests, advanced test flows, and cloud sync.</span>
          </div>
          <button type="button" class="modal-close-btn" title="Close" onclick={() => (showUpgradeModal = false)}>{@render iconClose()}</button>
        </div>
        <div style="padding: 20px; display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
          <div style="background: #202020; border: 1px solid #333; border-radius: 6px; padding: 16px;">
            <div style="font-weight: 700; font-size: 14px; color: #fff;">Team Plan</div>
            <div style="font-size: 20px; font-weight: 700; color: #ff6c37; margin: 8px 0;">$14 <span style="font-size: 11px; color: #888;">/ user / month</span></div>
            <ul style="font-size: 12px; color: #aaa; padding-left: 18px; margin: 10px 0; line-height: 1.6;">
              <li>Unlimited collections & requests</li>
              <li>Collaborative shared environments</li>
              <li>Up to 50,000 mock calls / month</li>
            </ul>
            <button type="button" class="btn-send" style="width: 100%; margin-top: 10px;" onclick={() => {
              showUpgradeModal = false;
              exportFeedback = "Upgraded to Team Plan! Enjoy enhanced features.";
              setTimeout(() => { exportFeedback = ""; }, 3500);
            }}>Upgrade to Team</button>
          </div>
          <div style="background: #202020; border: 1px solid #ff6c37; border-radius: 6px; padding: 16px; position: relative;">
            <span style="position: absolute; top: -10px; right: 12px; background: #ff6c37; color: #fff; font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 10px;">POPULAR</span>
            <div style="font-weight: 700; font-size: 14px; color: #fff;">Enterprise</div>
            <div style="font-size: 20px; font-weight: 700; color: #0cbb52; margin: 8px 0;">$29 <span style="font-size: 11px; color: #888;">/ user / month</span></div>
            <ul style="font-size: 12px; color: #aaa; padding-left: 18px; margin: 10px 0; line-height: 1.6;">
              <li>Dedicated mock servers & flows</li>
              <li>SSO (SAML / Okta) integration</li>
              <li>Priority SLA & 24/7 technical support</li>
            </ul>
            <button type="button" class="btn-send" style="width: 100%; margin-top: 10px; background: #0cbb52;" onclick={() => {
              showUpgradeModal = false;
              exportFeedback = "Upgraded to Enterprise! Welcome aboard.";
              setTimeout(() => { exportFeedback = ""; }, 3500);
            }}>Upgrade to Enterprise</button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if showImportDialog}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showImportDialog = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showImportDialog = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container modal-wide">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <h3>{t("import.title")}</h3>
            <span class="modal-sub">{t("import.subtitle")}</span>
          </div>
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showImportDialog = false)}>{@render iconClose()}</button>
        </div>

        <div class="modal-tabs">
          <button type="button" class="modal-tab-btn" class:active={importActiveTab === "collection"} onclick={() => (importActiveTab = "collection")}>{t("import.tabCollection")}</button>
          <button type="button" class="modal-tab-btn" class:active={importActiveTab === "environment"} onclick={() => (importActiveTab = "environment")}>{t("import.tabEnvironment")}</button>
          <button type="button" class="modal-tab-btn" class:active={importActiveTab === "curl"} onclick={() => (importActiveTab = "curl")}>{t("import.tabCurl")}</button>
          <button type="button" class="modal-tab-btn" class:active={importActiveTab === "localWorkspace"} onclick={() => (importActiveTab = "localWorkspace")}>{t("import.tabLocalWorkspace")}</button>
        </div>

        <div class="modal-body">
          {#if importActiveTab === "collection"}
            <p class="hint">{t("import.collectionHint")}</p>
            <div class="file-dropzone">
              <label class="file-label">
                <span>{t("import.chooseJsonFile")}</span>
                <input type="file" accept=".json,application/json" onchange={handleCollectionFileUpload} />
              </label>
            </div>
            <textarea placeholder={t("import.pasteCollectionPlaceholder")} bind:value={collectionImportText} rows="6" class="body-input"></textarea>

            {#if selectedProjectId}
              <div class="radio-row">
                <label class="radio-label">
                  <input type="radio" name="collectionTargetScreen" value="new" bind:group={collectionImportTarget} />
                  {t("import.newProject")}
                </label>
                <label class="radio-label">
                  <input type="radio" name="collectionTargetScreen" value="current" bind:group={collectionImportTarget} />
                  {t("import.currentProject")}
                </label>
              </div>
            {/if}

            {#if collectionImportError}<p class="error">{collectionImportError}</p>{/if}

            {#if collectionImportReport}
              <div class="import-report-card">
                <h4>{t("import.complete")}</h4>
                <p>{t("import.project", { name: collectionImportReport.project_name })}</p>
                <p>{t("import.requests", { count: collectionImportReport.requests_count })}</p>
                <p>{t("import.variables", { count: collectionImportReport.variables_count })}</p>
                <p>{t("import.sampleResponses", { count: collectionImportReport.sample_responses_count })}</p>
                {#if collectionImportReport.warnings.length > 0}
                  <div class="warnings-box">
                    <h5>{t("import.compatNotes")}</h5>
                    <ul>
                      {#each collectionImportReport.warnings as warn}<li>{warn}</li>{/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/if}

            <div class="params-row">
              <button type="button" class="btn-primary" disabled={!collectionImportText.trim() || collectionImportLoading} onclick={importPostmanCollectionAction}>
                {collectionImportLoading ? t("import.importing") : t("import.importCollection")}
              </button>
            </div>
          {:else if importActiveTab === "environment"}
            <p class="hint">{t("import.environmentHint")}</p>
            <div class="file-dropzone">
              <label class="file-label">
                <span>{t("import.chooseJsonFile")}</span>
                <input type="file" accept=".json,application/json" onchange={handleEnvironmentFileUpload} />
              </label>
            </div>
            <textarea placeholder={t("import.pasteEnvironmentPlaceholder")} bind:value={environmentImportText} rows="4" class="body-input"></textarea>

            {#if environmentImportError}<p class="error">{environmentImportError}</p>{/if}

            {#if environmentImportReport}
              <div class="import-report-card">
                <h4>{t("import.environmentImported")}</h4>
                <p>{t("import.environment", { name: environmentImportReport.environment_name })}</p>
                <p>{t("import.variables", { count: environmentImportReport.variables_count })}</p>
                {#if environmentImportReport.warnings.length > 0}
                  <div class="warnings-box">
                    <h5>{t("import.compatNotes")}</h5>
                    <ul>
                      {#each environmentImportReport.warnings as warn}<li>{warn}</li>{/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/if}

            <div class="params-row">
              <button type="button" class="btn-primary" disabled={!environmentImportText.trim() || environmentImportLoading} onclick={importPostmanEnvironmentAction}>
                {environmentImportLoading ? t("import.importing") : t("import.importEnvironment")}
              </button>
            </div>
          {:else if importActiveTab === "curl"}
            <p class="hint">{t("import.curlHint")}</p>
            {#if !selectedProjectId}
              <p class="screen-empty-inline">{t("import.selectProjectFirst")}</p>
            {:else}
              <form onsubmit={importCurlCommand}>
                <input placeholder={t("import.requestNamePlaceholder")} bind:value={curlImportName} />
                <textarea
                  placeholder={t("import.curlPlaceholder")}
                  bind:value={curlImportText}
                  rows="6"
                  class="body-input"
                ></textarea>
                {#if curlImportError}<p class="error">{curlImportError}</p>{/if}
                <div class="params-row">
                  <button type="submit" class="btn-primary">{t("import.importRequest")}</button>
                </div>
              </form>
            {/if}
          {:else if importActiveTab === "localWorkspace"}
            <p class="hint">{t("import.localWorkspaceHint")}</p>
            <input
              type="text"
              placeholder={t("import.localWorkspacePathPlaceholder")}
              bind:value={localWorkspacePathInput}
              class="url-input"
            />

            {#if localWorkspaceImportError}<p class="error">{localWorkspaceImportError}</p>{/if}

            {#if localWorkspaceImportReport}
              <div class="import-report-card">
                <h4>{t("import.complete")}</h4>
                <p>{t("import.localWorkspaceProjects", { count: localWorkspaceImportReport.projects_created })}</p>
                <p>{t("import.localWorkspaceFolders", { count: localWorkspaceImportReport.folders_created })}</p>
                <p>{t("import.requests", { count: localWorkspaceImportReport.requests_imported })}</p>
                <p>{t("import.localWorkspaceSamples", { count: localWorkspaceImportReport.samples_imported })}</p>
                <p>{t("import.localWorkspaceEnvironments", { count: localWorkspaceImportReport.environments_imported })}</p>
                <p>{t("import.variables", { count: localWorkspaceImportReport.variables_imported })}</p>
                {#if localWorkspaceImportReport.warnings.length > 0}
                  <div class="warnings-box">
                    <h5>{t("import.compatNotes")}</h5>
                    <ul>
                      {#each localWorkspaceImportReport.warnings as warn}<li>{warn}</li>{/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/if}

            <div class="params-row">
              <button
                type="button"
                class="btn-primary"
                disabled={!localWorkspacePathInput.trim() || localWorkspaceImportLoading}
                onclick={importLocalWorkspaceAction}
              >
                {localWorkspaceImportLoading ? t("import.importing") : t("import.importLocalWorkspace")}
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if paletteOpen}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) closePalette(); }} onkeydown={(e) => { if (e.key === "Escape") closePalette(); }} role="dialog" aria-modal="true" tabindex="0">
      <div class="palette">
        <div class="palette-header">
          <span class="screen-kicker">{t("palette.goTo")}</span>
          <input class="palette-input" placeholder={t("palette.placeholder")} bind:value={paletteQuery} bind:this={paletteInputEl} />
        </div>
        <div class="palette-scope-row">
          <button type="button" class="palette-scope-btn" class:active={paletteScope === "all"} onclick={() => (paletteScope = "all")}>{t("palette.scopeAll")}</button>
          <button type="button" class="palette-scope-btn" class:active={paletteScope === "projects"} onclick={() => (paletteScope = "projects")}>{t("palette.scopeProjects")}</button>
          <button type="button" class="palette-scope-btn" class:active={paletteScope === "apis"} onclick={() => (paletteScope = "apis")}>{t("palette.scopeApis")}</button>
        </div>
        {#if paletteScope !== "projects"}
          <div class="palette-scope-row palette-field-row">
            <span class="palette-field-label">{t("palette.fieldsLabel")}</span>
            <button type="button" class="palette-scope-btn" class:active={paletteFields.name} aria-pressed={paletteFields.name} onclick={() => togglePaletteField("name")}>{t("palette.fieldName")}</button>
            <button type="button" class="palette-scope-btn" class:active={paletteFields.url} aria-pressed={paletteFields.url} onclick={() => togglePaletteField("url")}>{t("palette.fieldUrl")}</button>
            <button type="button" class="palette-scope-btn" class:active={paletteFields.body} aria-pressed={paletteFields.body} onclick={() => togglePaletteField("body")}>{t("palette.fieldBody")}</button>
          </div>
        {/if}
        <div class="palette-results">
          {#each paletteItems as p, i (i)}
            <button type="button" class="palette-item" onclick={p.onSelect}>
              <span class="palette-item-method">{p.method ?? ""}</span>
              <span class="palette-item-label">
                {#if p.project}<span class="palette-item-breadcrumb">{p.project} ›</span>{/if}
                {p.label}
              </span>
              <div class="response-stat-spacer"></div>
              <span class="palette-item-hint">{p.hint}</span>
            </button>
          {:else}
            <p class="screen-empty-inline">{t("palette.noMatches")}</p>
          {/each}
        </div>
        <div class="palette-footer">
          <span>{t("palette.openHint")}</span>
          <span>{t("palette.dismissHint")}</span>
        </div>
      </div>
    </div>
  {/if}

  {#if textCopiedNotice}
    <div style="position: fixed; bottom: 40px; right: 24px; z-index: 99999; background: #0cbb52; color: #fff; padding: 8px 16px; border-radius: 4px; font-weight: 500; font-size: 13px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); display: flex; align-items: center; gap: 8px;">
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 8.5l3.5 3.5L13 4"/></svg>
      <span>{textCopiedNotice}</span>
    </div>
  {/if}

  {#if tabContextMenu.visible}
    <div
      class="dropdown-backdrop"
      style="background: transparent; z-index: 9998;"
      onclick={closeTabContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeTabContextMenu(); }}
      role="presentation"
    ></div>
    <div
      class="postman-tab-context-menu"
      style="top: {tabContextMenu.y}px; left: {tabContextMenu.x}px;"
    >
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuNewRequest}>
        <span class="tab-menu-label">New Request</span>
        <span class="tab-menu-shortcut">Ctrl+T</span>
      </button>
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuDuplicateTab}>
        <span class="tab-menu-label">Duplicate Tab</span>
      </button>

      <div class="tab-context-menu-divider"></div>

      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseTab}>
        <span class="tab-menu-label">Close Tab</span>
        <span class="tab-menu-shortcut">Ctrl+W</span>
      </button>
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuForceCloseTab}>
        <span class="tab-menu-label">Force Close Tab</span>
        <span class="tab-menu-shortcut">Alt+Ctrl+W</span>
      </button>
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseOtherTabs}>
        <span class="tab-menu-label">Close Other Tabs</span>
      </button>
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseAllTabs}>
        <span class="tab-menu-label">Close All Tabs</span>
      </button>
      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuForceCloseAllTabs}>
        <span class="tab-menu-label">Force Close All Tabs</span>
      </button>

      <div class="tab-context-menu-divider"></div>

      <button type="button" class="tab-context-menu-item" onclick={handleTabMenuRevealInSidebar}>
        <span class="tab-menu-label">Reveal in Sidebar</span>
      </button>
    </div>
  {/if}

  {#if requestContextMenu.visible}
    <div
      class="dropdown-backdrop"
      style="background: transparent; z-index: 9998;"
      onclick={closeRequestContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeRequestContextMenu(); }}
      role="presentation"
    ></div>
    <div
      class="postman-context-menu"
      style="top: {requestContextMenu.y}px; left: {requestContextMenu.x}px;"
    >
      <button type="button" class="context-menu-item" onclick={() => copyContextUrl(requestContextMenu.request)}>
        <span>Copy URL</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlBash(requestContextMenu.request)}>
        <span>Copy as cURL (bash)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlCmd(requestContextMenu.request)}>
        <span>Copy as cURL (cmd)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlPowerShell(requestContextMenu.request)}>
        <span>Copy as cURL (PowerShell)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsFetch(requestContextMenu.request)}>
        <span>Copy as Fetch</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsNodeFetch(requestContextMenu.request)}>
        <span>Copy as Node.js - Fetch</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsPythonRequests(requestContextMenu.request)}>
        <span>Copy as Python - Requests</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsPreload(requestContextMenu.request)}>
        <span>Copy as Preload element</span>
      </button>
      <button type="button" class="context-menu-item" onclick={() => copyContextAsHar(requestContextMenu.request)}>
        <span>Copy as HAR (sanitized)</span>
      </button>
      <div class="context-menu-divider"></div>
      <button type="button" class="context-menu-item" onclick={copyAllUrlsAction}>
        <span>Copy all URLs</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsCurlBashAction}>
        <span>Copy all as cURL (bash)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsCurlCmdAction}>
        <span>Copy all as cURL (cmd)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsCurlPowerShellAction}>
        <span>Copy all as cURL (PowerShell)</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsFetchAction}>
        <span>Copy all as Fetch</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsNodeFetchAction}>
        <span>Copy all as Node.js - Fetch</span>
      </button>
      <button type="button" class="context-menu-item" onclick={copyAllAsHarAction}>
        <span>Copy all as HAR (sanitized)</span>
      </button>
    </div>
  {/if}
</div>
