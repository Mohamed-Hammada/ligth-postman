<script lang="ts">
  import { onMount } from "svelte";
  import { translate, RTL_LOCALES, type Locale } from "$lib/i18n";
  import {
    api,
    describeError,
    type Auth,
    type CollectionImportReport,
    type Environment,
    type EnvironmentImportReport,
    type Folder,
    type GeneratedApiDefinition,
    type FormDataPart,
    type HeaderEntry,
    type Project,
    type QueryParam,
    type UrlEncodedItem,
    type RequestFull,
    type RequestDiagnostics,
    type RequestSettings,
    type RequestSummary,
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
    type ProjectGitSettings,
    type GitHubUser,
    type GitHubRepoInfo,
    type AiSettings,
    type DiscoveredEndpoint,
    type GeneratedTestsAndDocs,
    type SampleResponse,
    type SourceProjectReport,
    type UpdateAiSettingsInput,
    type SystemDiagnostics,
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

  let projects = $state<Project[]>([]);
  let selectedProjectId = $state<string | null>(null);

  let requests = $state<RequestSummary[]>([]);
  let selectedRequest = $state<RequestFull | null>(null);

  // Flat (non-nested) folders — a request either sits directly under its project or under one
  // folder in that project (see models::Folder on the backend).
  let folders = $state<Folder[]>([]);
  let expandedFolderIds = $state<Set<string>>(new Set());
  let renamingFolderId = $state<string | null>(null);
  let renameFolderValue = $state("");

  let environments = $state<Environment[]>([]);
  let selectedEnvironmentId = $state<string | null>(null);
  let renamingEnvironmentId = $state<string | null>(null);
  let renameEnvironmentValue = $state("");
  let urlPreview = $state<ResolvedTemplate | null>(null);

  let projectVariables = $state<VariableView[]>([]);
  let environmentVariables = $state<VariableView[]>([]);
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
  let activeEditorTab = $state<"params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "code">("params");
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

  // Import screen tab (collection / environment / cURL) — the full-page Import screen.
  let importActiveTab = $state<"collection" | "environment" | "curl">("collection");

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

  let snippetMode = $state<SnippetMode>("placeholder");
  // Windows CMD curl by default — this app's dev/target environment is Windows; anything else
  // is one click away in the same dropdown.
  let snippetTarget = $state<SnippetTarget>("windows_cmd");
  let snippet = $state("");
  let snippetError = $state("");

  let errorMessage = $state("");
  let loadingRequests = $state(false);

  let sending = $state(false);
  let activeResponse = $state<ResponseMeta | null>(null);
  let activeResponseBody = $state("");
  let activeResponseTruncated = $state(false);
  let responseHistory = $state<ResponseSummary[]>([]);

  // Response Pretty / Raw viewer (LP-0403)
  let responseViewMode = $state<"pretty" | "raw">("pretty");
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

  // Response area sub-tabs (Body / Headers / Cookies / Tests) — replaces the old flat stacked layout.
  let responseSubTab = $state<"body" | "headers" | "cookies" | "tests">("body");

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

  // Screen navigation shell — a real left-rail switcher between full-page screens. Each
  // screen reuses the exact same state/functions the old modal-based UI used; nothing here
  // introduces a second source of truth for projects/requests/environments/git/etc.
  type ScreenId = "workspace" | "environments" | "git" | "import" | "launcher" | "history" | "settings" | "theme";
  let activeScreen = $state<ScreenId>("workspace");
  // `label` stores an i18n key (see SHORTCUT_DEFS above for why) — resolve via t(s.label).
  const SCREENS: { id: ScreenId; label: string; icon: string }[] = [
    { id: "workspace", label: "rail.workspace", icon: "🗂️" },
    { id: "environments", label: "rail.environments", icon: "🌐" },
    { id: "git", label: "rail.git", icon: "⎇" },
    { id: "import", label: "rail.import", icon: "📥" },
    { id: "launcher", label: "rail.launcher", icon: "🚀" },
    { id: "history", label: "rail.history", icon: "🕘" },
    { id: "settings", label: "rail.settings", icon: "⚙️" },
    { id: "theme", label: "rail.theme", icon: "🌓" },
  ];

  // Sidebar show/hide, for the Workspace screen's project/request explorer.
  let sidebarVisible = $state(true);
  let screensRailVisible = $state(true);
  // The full-detail response view (stat sidebar, tests, bigger body) used to be its own rail
  // screen, but it's meaningless without Workspace's request context — it's now an expand
  // mode reached from the inline response panel instead of a peer top-level destination.
  let responseExpanded = $state(false);

  // Real, persisted light/dark toggle — light is this design system's own default, dark is a
  // genuine alternate palette (see the :root[data-theme="dark"] block), not a static mockup.
  let themeMode = $state<"light" | "dark">("light");

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
  function setThemeMode(mode: "light" | "dark") {
    themeMode = mode;
    try {
      localStorage.setItem("lp-theme", mode);
    } catch {
      // localStorage can throw in a locked-down webview profile — theme just won't persist.
    }
  }

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
    if (!selectedProjectId) {
      projectHistory = [];
      return;
    }
    historyLoading = true;
    try {
      projectHistory = await api.listProjectHistory(selectedProjectId, 200);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      historyLoading = false;
    }
  }
  let filteredProjectHistory = $derived.by(() => {
    const q = historySearchQuery.trim().toLowerCase();
    return projectHistory.filter((h) => {
      if (historyShowFailuresOnly && h.status < 400) return false;
      if (!q) return true;
      return (
        h.request_name.toLowerCase().includes(q) ||
        h.method.toLowerCase().includes(q) ||
        h.url.toLowerCase().includes(q) ||
        String(h.status).includes(q)
      );
    });
  });

  // Command palette — real fuzzy-ish search over actual projects/requests, real actions only
  // (open the project, open the request). No fabricated "commands".
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteInputEl = $state<HTMLInputElement | null>(null);
  function openPalette() {
    paletteOpen = true;
    paletteQuery = "";
  }
  function closePalette() {
    paletteOpen = false;
  }

  // Keyboard shortcuts — each one maps to a real, already-existing action (nothing fabricated
  // for the sake of having a shortcuts list). Individually toggleable from Settings, persisted
  // to localStorage the same way theme/auto-sync-interval already are.
  type ShortcutId = "commandPalette" | "sendRequest" | "saveRequest" | "newRequest";
  // `label` stores an i18n key, not literal text — SHORTCUT_DEFS is a plain const (evaluated
  // once), so resolving the string at definition time would freeze it in whatever locale was
  // active then. Resolve it at render time instead: t(def.label).
  const SHORTCUT_DEFS: { id: ShortcutId; label: string; keys: string }[] = [
    { id: "commandPalette", label: "shortcut.commandPalette", keys: "Ctrl/⌘ K" },
    { id: "sendRequest", label: "shortcut.sendRequest", keys: "Ctrl/⌘ Enter" },
    { id: "saveRequest", label: "shortcut.saveRequest", keys: "Ctrl/⌘ S" },
    { id: "newRequest", label: "shortcut.newRequest", keys: "Ctrl/⌘ N" },
  ];
  let shortcutsEnabled = $state<Record<ShortcutId, boolean>>({
    commandPalette: true,
    sendRequest: true,
    saveRequest: true,
    newRequest: true,
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
  type PaletteItem = { kind: "project" | "request"; label: string; method?: string; hint: string; onSelect: () => void };
  let paletteItems = $derived.by((): PaletteItem[] => {
    const q = paletteQuery.trim().toLowerCase();
    const items: PaletteItem[] = [];
    for (const p of projects) {
      if (!q || p.name.toLowerCase().includes(q)) {
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
    if (selectedProjectId) {
      for (const r of requests) {
        if (!q || r.name.toLowerCase().includes(q) || r.url.toLowerCase().includes(q)) {
          items.push({
            kind: "request",
            label: r.name,
            method: r.method,
            hint: r.url,
            onSelect: () => {
              openRequest(r.id);
              activeScreen = "workspace";
              closePalette();
            },
          });
        }
      }
    }
    return items.slice(0, 30);
  });

  let aiConfigured = $state(false);
  let showAiPanel = $state(false);
  let rightPanel = $state<"code" | "info" | null>(null);
  let aiActiveTab = $state<"generate" | "source" | "settings">("generate");
  let aiPrompt = $state("");
  let aiGenerating = $state(false);
  let aiPreview = $state<GeneratedApiDefinition | null>(null);

  // Project context toggles (LP-0807, LP-0808, LP-0809)
  let aiIncludeExistingRequests = $state(true);
  let aiIncludeVariables = $state(true);

  // AI Settings (LP-0803)
  let aiSettings = $state<AiSettings | null>(null);
  let aiApiKeyInput = $state("");
  let aiModelInput = $state("claude-3-5-sonnet-20241022");
  let aiBaseUrlInput = $state("");
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
    const q = sourceFilter.trim().toLowerCase();
    if (!q) return sourceReport.endpoints;
    return sourceReport.endpoints.filter(
      (e) =>
        e.path.toLowerCase().includes(q) ||
        e.method.toLowerCase().includes(q) ||
        e.source_file.toLowerCase().includes(q),
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
    activeEditorTab: "params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "code";
    activeResponse: ResponseMeta | null;
    activeResponseBody: string;
    activeResponseTruncated: boolean;
  }

  let openTabs = $state<RequestTab[]>([]);
  const tabDrafts = new Map<string, RequestDraft>();

  // Project search — filters the sidebar's project list by name.
  let projectSearchQuery = $state("");

  // Advanced sort: projects list + requests/folders within a project. Persisted so the chosen
  // order survives a reload, same as the other small UI prefs (uiScale, locale, theme).
  type ProjectSortField = "name" | "created" | "updated";
  type RequestSortField = "name" | "method" | "updated";
  type SortDir = "asc" | "desc";
  let projectSortField = $state<ProjectSortField>("name");
  let projectSortDir = $state<SortDir>("asc");
  let requestSortField = $state<RequestSortField>("name");
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
    try { localStorage.setItem("lp-request-sort", JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
  }
  function toggleRequestSortDir() {
    requestSortDir = requestSortDir === "asc" ? "desc" : "asc";
    try { localStorage.setItem("lp-request-sort", JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
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

  const PROJECT_SORT_FIELDS: { field: ProjectSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "created", label: "sidebar.sortByCreated" },
    { field: "updated", label: "sidebar.sortByUpdated" },
  ];
  const REQUEST_SORT_FIELDS: { field: RequestSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "method", label: "sidebar.sortByMethod" },
    { field: "updated", label: "sidebar.sortByUpdated" },
  ];
  let projectSortMenuOpen = $state(false);
  let requestSortMenuOpen = $state(false);

  // Only one project's "more actions" overflow menu is open at a time.
  let openProjectMenuId = $state<string | null>(null);

  function sortProjectList(list: Project[]): Project[] {
    const dir = projectSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (projectSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (projectSortField === "created") return a.created_at.localeCompare(b.created_at) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }
  function sortRequestList(list: RequestSummary[]): RequestSummary[] {
    const dir = requestSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (requestSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (requestSortField === "method") return a.method.localeCompare(b.method) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }

  let filteredProjects = $derived.by(() => {
    const q = projectSearchQuery.trim().toLowerCase();
    const base = q ? projects.filter((p) => p.name.toLowerCase().includes(q)) : projects;
    return sortProjectList(base);
  });

  // Request search & windowing (LP-0409, LP-0410)
  let requestSearchQuery = $state("");
  let filteredRequests = $derived.by(() => {
    const q = requestSearchQuery.trim().toLowerCase();
    const base = q
      ? requests.filter(
          (r) =>
            r.name.toLowerCase().includes(q) ||
            r.method.toLowerCase().includes(q) ||
            r.url.toLowerCase().includes(q),
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
  let sortedFolders = $derived.by(() => {
    const dir = requestSortDir === "asc" ? 1 : -1;
    return [...folders].sort((a, b) => {
      if (requestSortField === "updated") return a.updated_at.localeCompare(b.updated_at) * dir;
      return a.name.localeCompare(b.name) * dir;
    });
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
        const q = consoleSearchFilter.toLowerCase();
        const inMsg = e.message.toLowerCase().includes(q);
        const inType = e.event_type.toLowerCase().includes(q);
        const inCid = e.correlation_id.toLowerCase().includes(q);
        const inDetails = e.details ? JSON.stringify(e.details).toLowerCase().includes(q) : false;
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

  // Git & Collaboration state (LP-0701 - LP-0713)
  let showDiffModal = $state(false);
  let showHistoryModal = $state(false);
  let gitActiveTab = $state<"sync" | "conflicts" | "github" | "projectfile">("sync");

  let gitSettings = $state<ProjectGitSettings | null>(null);
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

  $effect(() => {
    if (selectedRequest && selectedRequest.id !== hydratedRequestId) {
      hydratedRequestId = selectedRequest.id;
      if (!tabDrafts.has(selectedRequest.id)) {
        editName = selectedRequest.name;
        editMethod = selectedRequest.method;
        editUrl = selectedRequest.url;
        editHeaders = withTrailingEmptyRow(selectedRequest.headers.map((h) => ({ ...h })), () => ({ key: "", value: "", enabled: true, description: "" }));
        editQueryParams = withTrailingEmptyRow(selectedRequest.query_params.map((p) => ({ ...p })), () => ({ key: "", value: "", enabled: true }));
        const parsedBody = parseBodyForEditing(selectedRequest.body);
        editBodyType = parsedBody.bodyType;
        editBody = parsedBody.rawBody;
        editGraphqlQuery = parsedBody.graphqlQuery;
        editGraphqlVariables = parsedBody.graphqlVariables;
        editFormDataItems = withTrailingEmptyRow(parsedBody.formDataItems, () => ({ key: "", value: "", enabled: true, is_file: false, file_path: null }));
        editUrlEncodedItems = withTrailingEmptyRow(parsedBody.urlEncodedItems, () => ({ key: "", value: "", enabled: true }));
        editBinaryFilePath = parsedBody.binaryFilePath;
        editDescription = selectedRequest.description ?? "";

        const auth = selectedRequest.auth;
        editAuthType = auth.type;
        editAuthBearerToken = auth.type === "bearer" ? auth.token : "";
        editAuthBasicUsername = auth.type === "basic" ? auth.username : "";
        editAuthBasicPassword = auth.type === "basic" ? auth.password : "";
        editAuthApiKeyKey = auth.type === "api_key" ? auth.key : "";
        editAuthApiKeyValue = auth.type === "api_key" ? auth.value : "";
        editAuthApiKeyLocation = auth.type === "api_key" ? auth.location : "header";

        // Scripts
        editPreScript = selectedRequest.pre_request_script ?? "";
        editPostScript = selectedRequest.post_request_script ?? "";

        // Settings
        const settings = selectedRequest.settings;
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

  async function copyAsCurl() {
    if (!selectedRequest) return;
    snippetError = "";
    try {
      snippet = await api.generateCurlSnippet(selectedRequest.id, selectedEnvironmentId, snippetMode, snippetTarget);
    } catch (err) {
      snippetError = describeError(err);
    }
  }

  async function copySnippetToClipboard() {
    if (!snippet) return;
    try {
      await navigator.clipboard.writeText(snippet);
    } catch (err) {
      snippetError = describeError(err);
    }
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
    loadProjects();
    loadAiSettings();
    api.isAiConfigured().then((configured) => (aiConfigured = configured));
    refreshConsoleEvents();
    refreshSystemDiagnostics();
    try {
      const saved = localStorage.getItem("lp-theme");
      if (saved === "dark" || saved === "light") themeMode = saved;
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
        if (parsed.field === "name" || parsed.field === "created" || parsed.field === "updated") projectSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") projectSortDir = parsed.dir;
      }
      const savedRequestSort = localStorage.getItem("lp-request-sort");
      if (savedRequestSort) {
        const parsed = JSON.parse(savedRequestSort);
        if (parsed.field === "name" || parsed.field === "method" || parsed.field === "updated") requestSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") requestSortDir = parsed.dir;
      }
    } catch {
      // ignore — settings just stay at their defaults
    }

    sidebarWidth = adaptiveSidebarWidth();
    const onWindowResize = () => {
      if (!sidebarManuallyResized) sidebarWidth = adaptiveSidebarWidth();
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
    if (activeScreen === "launcher") {
      refreshProjectRequestCounts();
    }
    if (activeScreen === "history") {
      refreshProjectHistory();
    }
    if (activeScreen !== "workspace") {
      responseExpanded = false;
    }
  });

  function saveCurrentDraft() {
    if (!selectedRequest) return;
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
      editFormDataItems,
      editUrlEncodedItems,
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
    // A surviving draft means the autosave debounce didn't get to fire before the user tabbed
    // away — pick up where it left off instead of leaving those edits stuck unsaved.
    scheduleAutoSave();
  }

  function isTabDirty(tabId: string): boolean {
    if (tabId === selectedRequest?.id) {
      return (
        editName !== selectedRequest.name ||
        editMethod !== selectedRequest.method ||
        editUrl !== selectedRequest.url ||
        editBody !== (selectedRequest.body ?? "") ||
        editDescription !== (selectedRequest.description ?? "") ||
        editPreScript !== (selectedRequest.pre_request_script ?? "") ||
        editPostScript !== (selectedRequest.post_request_script ?? "")
      );
    }
    return tabDrafts.has(tabId);
  }

  function closeTab(id: string) {
    tabDrafts.delete(id);
    const idx = openTabs.findIndex((t) => t.id === id);
    if (idx === -1) return;
    const wasActive = selectedRequest?.id === id;
    openTabs = openTabs.filter((t) => t.id !== id);

    if (wasActive) {
      if (openTabs.length > 0) {
        const nextTab = openTabs[Math.max(0, idx - 1)];
        openRequest(nextTab.id);
      } else {
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
      copyFeedback = "Console log copied!";
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
      copyFeedback = "Event JSON copied!";
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
    try {
      projects = await api.listProjects();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function loadAiSettings() {
    try {
      const s = await api.getAiSettings();
      aiSettings = s;
      aiConfigured = s.is_configured;
      aiApiKeyInput = s.api_key ?? "";
      aiModelInput = s.model || "claude-3-5-sonnet-20241022";
      aiBaseUrlInput = s.base_url ?? "";
    } catch (err) {
      console.error("Failed to load AI settings", err);
    }
  }

  async function saveAiSettingsAction() {
    aiTestError = "";
    aiTestFeedback = "";
    aiSettingsFeedback = "";
    try {
      await api.saveAiSettings({
        api_key: aiApiKeyInput.trim() || null,
        model: aiModelInput.trim() || null,
        base_url: aiBaseUrlInput.trim() || null,
      });
      await loadAiSettings();
      aiSettingsFeedback = "AI settings saved successfully!";
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

  async function deleteSampleResponseAction(requestId: string, id: string) {
    try {
      await api.deleteSampleResponse(id);
      await loadSampleResponses(requestId);
    } catch (err) {
      errorMessage = describeError(err);
    }
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
        { id: request.id, project_id: request.project_id, folder_id: request.folder_id, name: request.name, method: request.method, url: request.url, updated_at: request.updated_at },
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
  async function quickCreateProject() {
    try {
      const project = await api.createProject("New Project");
      projects = [project, ...projects];
      await selectProject(project.id);
      startRenameProject(project);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Only ever loads request *metadata* for the selected project — bodies/headers
  // stay on disk until a specific request tab is opened (README §4/§20).
  async function selectProject(id: string) {
    selectedProjectId = id;
    selectedRequest = null;
    selectedEnvironmentId = null;
    openTabs = [];
    tabDrafts.clear();
    loadingRequests = true;
    try {
      requests = await api.listRequests(id);
      folders = await api.listFolders(id);
      environments = await api.listEnvironments(id);
      // Auto-select the project's preferred environment, if it set one and that environment
      // still exists (it may have been deleted since — the backend already clears the
      // reference then, but the frontend's stale `projects` entry might not have refreshed yet).
      const defaultEnvId = projects.find((p) => p.id === id)?.default_environment_id;
      if (defaultEnvId && environments.some((e) => e.id === defaultEnvId)) {
        selectedEnvironmentId = defaultEnvId;
      }
      await loadVariables();
      await loadGitSettings(id);
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
  async function quickCreateEnvironment() {
    if (!selectedProjectId) return;
    try {
      const env = await api.createEnvironment(selectedProjectId, "New Environment");
      environments = [...environments, env];
      selectedEnvironmentId = env.id;
      await loadVariables();
      startRenameEnvironment(env);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function startRenameEnvironment(env: Environment) {
    renamingEnvironmentId = env.id;
    renameEnvironmentValue = env.name;
  }

  async function submitRenameEnvironment() {
    const id = renamingEnvironmentId;
    const value = renameEnvironmentValue.trim();
    renamingEnvironmentId = null;
    if (!id || !value) return;
    try {
      const updated = await api.updateEnvironment({ id, name: value });
      environments = environments.map((e) => (e.id === updated.id ? updated : e));
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function deleteEnvironmentAction(id: string) {
    try {
      await api.deleteEnvironment(id);
      environments = environments.filter((e) => e.id !== id);
      if (selectedEnvironmentId === id) {
        selectedEnvironmentId = null;
        await loadVariables();
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
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

  // "+" for folder next to a project — creates the folder, then drops straight into
  // inline-rename, same pattern as quickCreateRequest/quickCreateProject.
  async function quickCreateFolder(projectId: string) {
    try {
      if (selectedProjectId !== projectId) {
        await selectProject(projectId);
      }
      const folder = await api.createFolder({ project_id: projectId, name: "New Folder" });
      folders = [...folders, folder];
      expandedFolderIds = new Set([...expandedFolderIds, folder.id]);
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

  // Ungroups the folder's requests back to the project root instead of deleting them —
  // matches the backend's own delete_folder semantics (see folder_store.rs).
  async function deleteFolderAction(id: string) {
    try {
      await api.deleteFolder(id);
      folders = folders.filter((f) => f.id !== id);
      requests = requests.map((r) => (r.folder_id === id ? { ...r, folder_id: null } : r));
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function toggleFolderExpanded(id: string) {
    const next = new Set(expandedFolderIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedFolderIds = next;
  }

  // Hydrate the full request only when the user actually opens it.
  async function openRequest(id: string) {
    if (selectedRequest?.id === id) return;
    saveCurrentDraft();

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
      if (tabDrafts.has(id)) {
        restoreDraft(tabDrafts.get(id)!);
      } else {
        activeResponse = null;
        activeResponseBody = "";
      }
      responseHistory = await api.listResponseSummaries(id);
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
      const body = await api.getResponseBody(meta.id);
      activeResponseBody = body.text;
      activeResponseTruncated = body.truncated;
      responseHistory = await api.listResponseSummaries(requestId);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      sending = false;
      await refreshConsoleEvents();
    }
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

  async function deleteProject(id: string) {
    try {
      await api.deleteProject(id);
      projects = projects.filter((p) => p.id !== id);
      if (selectedProjectId === id) {
        selectedProjectId = null;
        requests = [];
        selectedRequest = null;
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
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
          ? { id: updated.id, project_id: updated.project_id, folder_id: updated.folder_id, name: updated.name, method: updated.method, url: updated.url, updated_at: updated.updated_at }
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
      });
      requests = [
        {
          id: request.id,
          project_id: request.project_id,
          folder_id: request.folder_id,
          name: request.name,
          method: request.method,
          url: request.url,
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
    if (!collectionImportText.trim()) return;
    collectionImportLoading = true;
    collectionImportError = "";
    collectionImportReport = null;
    try {
      const targetId = collectionImportTarget === "current" ? selectedProjectId : null;
      const report = await api.importPostmanCollection(collectionImportText.trim(), targetId);
      collectionImportReport = report;
      await loadProjects();
      if (!selectedProjectId || collectionImportTarget === "new") {
        await selectProject(report.project_id);
      } else {
        await selectProject(selectedProjectId);
      }
      collectionImportText = "";
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
      environments = await api.listEnvironments(selectedProjectId);
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
      const env = environments.find((e) => e.id === selectedEnvironmentId);
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
    if (!selectedProjectId) {
      projectVariables = [];
      environmentVariables = [];
      return;
    }
    try {
      projectVariables = await api.listVariablesForScope("global", selectedProjectId);
      if (selectedEnvironmentId) {
        environmentVariables = await api.listVariablesForScope("environment", selectedEnvironmentId);
      } else {
        environmentVariables = [];
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

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
    if (!selectedEnvironmentId || !key) return;
    try {
      await api.createVariable({
        scope: "environment",
        environment_id: selectedEnvironmentId,
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
  let missingVarHover = $state<{ name: string; top: number; left: number } | null>(null);
  let missingVarHoverHideTimer: ReturnType<typeof setTimeout> | null = null;

  function showMissingVarPopover(name: string, target: HTMLElement) {
    if (missingVarHoverHideTimer) {
      clearTimeout(missingVarHoverHideTimer);
      missingVarHoverHideTimer = null;
    }
    const rect = target.getBoundingClientRect();
    missingVarHover = { name, top: rect.bottom, left: rect.left };
  }

  function scheduleHideMissingVarPopover() {
    if (missingVarHoverHideTimer) clearTimeout(missingVarHoverHideTimer);
    missingVarHoverHideTimer = setTimeout(() => {
      missingVarHover = null;
      missingVarHoverHideTimer = null;
    }, 150);
  }

  function cancelHideMissingVarPopover() {
    if (missingVarHoverHideTimer) {
      clearTimeout(missingVarHoverHideTimer);
      missingVarHoverHideTimer = null;
    }
  }

  async function deleteVariable(id: string) {
    try {
      await api.deleteVariable(id);
      await loadVariables();
      await refreshDiagnostics();
    } catch (err) {
      errorMessage = describeError(err);
    }
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
      copyFeedback = "Copied!";
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

  async function deleteRequest(id: string) {
    try {
      await api.deleteRequest(id);
      requests = requests.filter((r) => r.id !== id);
      closeTab(id);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // --- Git & Collaboration Functions (LP-0701 - LP-0713) ---
  async function loadGitSettings(projectId: string) {
    try {
      const s = await api.getProjectGitSettings(projectId);
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
    if (!selectedProjectId) return;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      const settings: ProjectGitSettings = {
        project_id: selectedProjectId,
        repo_path: gitRepoPathInput.trim() || null,
        remote_url: gitRemoteUrlInput.trim() || null,
        branch: gitBranchInput.trim() || "main",
        auto_sync: gitAutoSyncInput,
        github_token: githubTokenInput.trim() || null,
        last_sync_at: gitSettings?.last_sync_at ?? null,
      };
      await api.saveProjectGitSettings(settings);
      gitSettings = settings;
      gitActionFeedback = "Settings saved successfully.";
      if (settings.repo_path) {
        await refreshGitStatus(settings.repo_path);
      }
    } catch (err) {
      gitActionError = describeError(err);
    }
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
      if (selectedProjectId) {
        await api.saveProjectToRepo(selectedProjectId, dir, false);
      }
      await refreshGitStatus(dir);
      gitActionFeedback = "Git repository initialized successfully with .gitignore and canonical project file.";
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function saveProjectToRepoAction() {
    if (!selectedProjectId || !gitRepoPathInput.trim()) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      const path = await api.saveProjectToRepo(selectedProjectId, gitRepoPathInput.trim(), false);
      await refreshGitStatus();
      gitActionFeedback = `Saved canonical project file to ${path}`;
    } catch (err) {
      gitActionError = describeError(err);
    } finally {
      gitLoading = false;
    }
  }

  async function commitAndPushAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir || !selectedProjectId) return;
    const msg = gitCommitMessage.trim() || "Update project from Light Postman";
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.saveProjectToRepo(selectedProjectId, dir, false);
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
        await api.saveProjectGitSettings(gitSettings);
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

  async function pullRepositoryAction() {
    const dir = gitRepoPathInput.trim();
    if (!dir || !selectedProjectId) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.gitPullRepository(dir, "origin", gitBranchInput.trim() || "main");
      await api.loadProjectFromRepo(dir, selectedProjectId);
      await selectProject(selectedProjectId);
      gitActionFeedback = "Successfully pulled from remote and reloaded project.";
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
    if (!dir || !selectedProjectId) return;
    gitLoading = true;
    gitActionError = "";
    gitActionFeedback = "";
    try {
      await api.gitResolveConflict(dir, file, choice);
      gitActionFeedback = `Resolved conflict on '${file}' with choice '${choice}'.`;
      if (choice === "theirs" && file.includes("light-postman.json")) {
        await api.loadProjectFromRepo(dir, selectedProjectId);
        await selectProject(selectedProjectId);
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
    if (!projectFileJson.trim()) return;
    try {
      const proj = await api.importProjectFile(projectFileJson, selectedProjectId);
      projectFileStatus = `Project '${proj.name}' imported successfully.`;
      await selectProject(proj.id);
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
    if (gitAutoSyncInput && gitRepoPathInput.trim() && selectedProjectId) {
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
</script>


<div class="app-shell" data-theme={themeMode === "dark" ? "dark" : "light"}>
  <nav class="screens-rail" class:collapsed={!screensRailVisible}>
    <div class="rail-brand">
      {#if screensRailVisible}<span>{t("rail.brand")}</span>{/if}
      <button
        type="button"
        class="icon-btn"
        title={screensRailVisible ? t("rail.hide") : t("rail.show")}
        onclick={() => (screensRailVisible = !screensRailVisible)}
      >☰</button>
    </div>
    <div class="rail-screens">
      {#each SCREENS as s (s.id)}
        <button
          type="button"
          class="rail-screen"
          class:active={activeScreen === s.id}
          title={t(s.label)}
          onclick={() => (activeScreen = s.id)}
        >
          <span class="rail-screen-icon">{s.icon}</span>
          {#if screensRailVisible}<span class="rail-screen-label">{t(s.label)}</span>{/if}
        </button>
      {/each}
    </div>
    {#if screensRailVisible}
      <div class="rail-budget">
        <span class="rail-budget-label">{t("rail.budget")}</span>
        {#if systemDiagnostics}
          <span class="rail-budget-value">{formatByteSize(systemDiagnostics.process_rss_bytes)}</span>
          <span class="rail-budget-meta">{t("rail.tabsOpen", { count: openTabs.length })} · DB {formatByteSize(systemDiagnostics.db_size_bytes + systemDiagnostics.db_wal_size_bytes)}</span>
        {:else}
          <span class="rail-budget-meta">{t("rail.loading")}</span>
        {/if}
      </div>
    {/if}
  </nav>

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
        <p class="screen-empty">{t("env.noProjectsYet")}</p>
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
    <li class="request-item-wrapper">
      <div class="request-item" class:active={req.id === selectedRequest?.id}>
        {#if renamingRequestId === req.id && req.id !== selectedRequest?.id}
          <form class="inline-form" onsubmit={submitRenameRequest}>
            <input bind:value={renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
            <button type="submit" title={t("sidebar.save")}>✓</button>
            <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingRequestId = null)}>✕</button>
          </form>
        {:else}
          <button
            type="button"
            class="tree-expand-btn"
            title={expandedTreeRequestIds.has(req.id) ? t("sidebar.collapseSamples") : t("sidebar.expandSamples")}
            onclick={() => toggleTreeRequestExpanded(req.id)}
          >{expandedTreeRequestIds.has(req.id) ? "▾" : "▸"}</button>
          <button type="button" class="request-link" onclick={() => openRequest(req.id)} ondblclick={() => startRenameRequest(req.id, req.name)}>
            <span class="method-badge method-{req.method.toLowerCase()}">{req.method}</span>
            <span class="request-name">{req.name}</span>
            {#if sampleCount}<span class="tab-badge">{sampleCount}</span>{/if}
          </button>
          <button class="icon-btn icon-btn-ghost" title={t("sidebar.delete")} onclick={() => deleteRequest(req.id)}>🗑</button>
        {/if}
      </div>
      {#if expandedTreeRequestIds.has(req.id)}
        <ul class="sample-tree-list">
          {#each sampleResponsesByRequestId.get(req.id) ?? [] as sr (sr.id)}
            <li class="sample-tree-item">
              {#if renamingSampleResponseId === sr.id}
                <form class="inline-form" onsubmit={(e) => { e.preventDefault(); submitRenameSampleResponse(req.id); }}>
                  <input bind:value={renameSampleResponseValue} use:focusOnMount onblur={() => submitRenameSampleResponse(req.id)} />
                  <button type="submit" title={t("sidebar.save")}>✓</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingSampleResponseId = null)}>✕</button>
                </form>
              {:else}
                <button
                  type="button"
                  class="sample-tree-link"
                  title={t("sample.badge")}
                  onclick={() => openRequest(req.id)}
                  ondblclick={() => startRenameSampleResponse(sr)}
                >
                  <span class="status-chip" class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>{sr.status}</span>
                  <span class="sample-tree-name">{sr.name}</span>
                </button>
                <button class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameSampleResponse(sr)}>✎</button>
                <button class="icon-btn icon-btn-ghost" title={t("sample.delete")} onclick={() => deleteSampleResponseAction(req.id, sr.id)}>🗑</button>
              {/if}
            </li>
          {:else}
            <li class="empty">{t("sidebar.noSamplesYet")}</li>
          {/each}
        </ul>
      {/if}
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
        <p class="screen-empty">{t("response.openFromWorkspace")}</p>
      {:else if !activeResponse}
        <p class="screen-empty">{t("response.sendToSee")}</p>
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
              <div class="response-stat-spacer"></div>
              {#if responseSubTab === "body"}
                <button type="button" class="btn-ghost btn-xs" class:active={responseViewMode === "pretty"} onclick={() => (responseViewMode = "pretty")}>{t("response.pretty")}</button>
                <button type="button" class="btn-ghost btn-xs" class:active={responseViewMode === "raw"} onclick={() => (responseViewMode = "raw")}>{t("response.raw")}</button>
              {/if}
              <button type="button" class="btn-ghost btn-xs" onclick={copyResponseBody}>{t("response.copyAction")}</button>
              <button type="button" class="btn-ghost btn-xs" onclick={downloadResponseBody}>{t("response.saveToFile")}</button>
            </div>
            <div class="response-screen-content">
              {#if responseSubTab === "body"}
                <pre class="body-view screen-body-view">{prettyResponseBody}</pre>
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
  <header class="topbar">
    <div class="topbar-left">
      <span class="brand">⚡ {t("topbar.brand")}</span>
    </div>
    <div class="topbar-center">
      {#if selectedProjectId}
        <div class="env-bar">
          <select
            value={selectedEnvironmentId ?? "__none__"}
            class="env-select"
            onchange={(e) => {
              const target = e.target as HTMLSelectElement;
              const val = target.value;
              if (val === "__new__") {
                target.value = selectedEnvironmentId ?? "__none__";
                quickCreateEnvironment();
                return;
              }
              selectedEnvironmentId = val === "__none__" ? null : val;
              loadVariables();
            }}
          >
            <option value="__new__">{t("topbar.newEnvironment")}</option>
            <option value="__none__">{t("topbar.noEnvironment")}</option>
            {#each environments as env (env.id)}
              <option value={env.id}>{env.name}</option>
            {/each}
          </select>
          {#if selectedProjectId}
            {@const isDefault = projects.find((p) => p.id === selectedProjectId)?.default_environment_id === selectedEnvironmentId}
            <button
              type="button"
              class="icon-btn"
              class:active={isDefault}
              title={isDefault ? t("topbar.unsetDefaultEnv") : t("topbar.setDefaultEnv")}
              onclick={toggleDefaultEnvironment}
            >
              {isDefault ? "★" : "☆"}
            </button>
          {/if}
          {#if renamingEnvironmentId}
            <form class="inline-form" onsubmit={submitRenameEnvironment}>
              <input bind:value={renameEnvironmentValue} use:focusOnMount onblur={submitRenameEnvironment} />
              <button type="submit" title={t("sidebar.save")}>✓</button>
              <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingEnvironmentId = null)}>✕</button>
            </form>
          {/if}
          <button
            type="button"
            class="icon-btn"
            title={t("topbar.manageVariables")}
            onclick={() => { loadVariables(); activeScreen = "environments"; }}
          >
            👁
          </button>
          {#if selectedEnvironmentId}
            <button
              type="button"
              class="icon-btn"
              title={t("topbar.exportEnvironment")}
              onclick={exportPostmanEnvironmentAction}
            >
              ⤓
            </button>
          {/if}
        </div>
      {/if}
    </div>
    <div class="topbar-right">
      <button type="button" class="palette-trigger" title={t("topbar.searchPlaceholder")} onclick={openPalette}>
        <span>{t("topbar.searchPlaceholder")}</span>
        <span class="palette-kbd">⌘K</span>
      </button>
      <button
        type="button"
        class="btn-ghost"
        title={aiConfigured ? t("topbar.askAiTitle") : t("topbar.setupAiTitle")}
        onclick={() => (showAiPanel = true)}
      >
        ✨ {aiConfigured ? t("topbar.askAi") : t("topbar.setupAi")}
      </button>
      <button type="button" class="btn-ghost" title={t("topbar.importTitle")} onclick={() => { importActiveTab = "collection"; collectionImportReport = null; collectionImportError = ""; activeScreen = "import"; }}>
        📥 {t("topbar.import")}
      </button>
    </div>
  </header>

  {#if exportFeedback}
    <div class="success-banner">{exportFeedback}</div>
  {/if}
  {#if errorMessage}
    <div class="error-banner">
      {errorMessage}
      <button type="button" class="dismiss-btn" title={t("error.dismiss")} onclick={() => (errorMessage = "")}>✕</button>
    </div>
  {/if}

  <div class="workspace">
    {#if sidebarVisible}
    <aside class="sidebar" style="width: {sidebarWidth}px">
      <div class="sidebar-header">
        <span class="sidebar-title">{t("sidebar.projects")}</span>
        <div class="sidebar-header-actions">
          <button
            type="button"
            class="icon-btn"
            title={t("sidebar.importCollection")}
            onclick={() => {
              collectionImportTarget = "new";
              importActiveTab = "collection";
              collectionImportReport = null;
              collectionImportError = "";
              activeScreen = "import";
            }}
          >📥</button>
          <button type="button" class="icon-btn" title={t("sidebar.newProject")} onclick={quickCreateProject}>+</button>
          <button type="button" class="icon-btn" title={t("sidebar.hide")} onclick={() => (sidebarVisible = false)}>«</button>
        </div>
      </div>

      <div class="project-search-box">
        <input
          type="search"
          placeholder={t("sidebar.searchProjects")}
          bind:value={projectSearchQuery}
          class="project-search-input"
        />
        {#if projectSearchQuery}
          <span class="request-count-badge">{filteredProjects.length}/{projects.length}</span>
        {/if}
        <div class="menu-wrap">
          <button
            type="button"
            class="icon-btn"
            title={t("sidebar.sortOptions")}
            onclick={() => (projectSortMenuOpen = !projectSortMenuOpen)}
          >⋮</button>
          {#if projectSortMenuOpen}
            <button type="button" class="dropdown-backdrop" aria-label={t("common.close")} onclick={() => (projectSortMenuOpen = false)}></button>
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
                    <span class="sort-dir-indicator">{projectSortDir === "asc" ? "↑" : "↓"}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <div class="project-list">
        {#each filteredProjects as project (project.id)}
          <div class="project-node">
            <div class="project-row" class:active={project.id === selectedProjectId}>
              {#if renamingProjectId === project.id}
                <form class="inline-form" onsubmit={submitRenameProject}>
                  <input bind:value={renameProjectValue} use:focusOnMount onblur={submitRenameProject} />
                  <button type="submit" title={t("sidebar.save")}>✓</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingProjectId = null)}>✕</button>
                </form>
              {:else}
                <button type="button" class="project-link" onclick={() => selectProject(project.id)} ondblclick={() => startRenameProject(project)}>
                  <span class="folder-icon">{project.id === selectedProjectId ? "📂" : "📁"}</span>
                  <span class="project-name">{project.name}</span>
                </button>
                <div class="project-row-actions" class:force-visible={openProjectMenuId === project.id}>
                  <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id)}>+</button>
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.moreActions")}
                      onclick={() => (openProjectMenuId = openProjectMenuId === project.id ? null : project.id)}
                    >⋮</button>
                    {#if openProjectMenuId === project.id}
                      <button type="button" class="dropdown-backdrop" aria-label={t("common.close")} onclick={() => (openProjectMenuId = null)}></button>
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
                            activeScreen = "import";
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

            {#if project.id === selectedProjectId}
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
                    >{expandedFolderIds.size < folders.length ? "⊞" : "⊟"}</button>
                  {/if}
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.sortOptions")}
                      onclick={() => (requestSortMenuOpen = !requestSortMenuOpen)}
                    >⋮</button>
                    {#if requestSortMenuOpen}
                      <button type="button" class="dropdown-backdrop" aria-label={t("common.close")} onclick={() => (requestSortMenuOpen = false)}></button>
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
                              <span class="sort-dir-indicator">{requestSortDir === "asc" ? "↑" : "↓"}</span>
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
                      <button type="button" title={t("sidebar.prevPage")} disabled={requestPage === 0} onclick={() => (requestPage = Math.max(0, requestPage - 1))}>◀</button>
                      <span>{requestPage + 1} / {totalRequestPages}</span>
                      <button type="button" title={t("sidebar.nextPage")} disabled={requestPage >= totalRequestPages - 1} onclick={() => (requestPage = Math.min(totalRequestPages - 1, requestPage + 1))}>▶</button>
                    </div>
                  {/if}
                {:else}
                  {#each sortedFolders as folder (folder.id)}
                    <div class="folder-node">
                      <div class="folder-row">
                        {#if renamingFolderId === folder.id}
                          <form class="inline-form" onsubmit={submitRenameFolder}>
                            <input bind:value={renameFolderValue} use:focusOnMount onblur={submitRenameFolder} />
                            <button type="submit" title={t("sidebar.save")}>✓</button>
                            <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingFolderId = null)}>✕</button>
                          </form>
                        {:else}
                          <button type="button" class="folder-link" onclick={() => toggleFolderExpanded(folder.id)} ondblclick={() => startRenameFolder(folder)}>
                            <span class="folder-icon">{expandedFolderIds.has(folder.id) ? "📂" : "📁"}</span>
                            <span class="project-name">{folder.name}</span>
                          </button>
                          <div class="project-row-actions">
                            <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id, folder.id)}>+</button>
                            <button class="icon-btn" title={t("sidebar.rename")} onclick={() => startRenameFolder(folder)}>✎</button>
                            <button class="icon-btn" title={t("sidebar.deleteFolder")} onclick={() => deleteFolderAction(folder.id)}>🗑</button>
                          </div>
                        {/if}
                      </div>
                      {#if expandedFolderIds.has(folder.id)}
                        <ul class="request-list folder-request-list">
                          {#each requestsByFolderId.get(folder.id) ?? [] as req (req.id)}
                            {@render requestRow(req)}
                          {:else}
                            <li class="empty">{t("sidebar.noRequestsInFolder")}</li>
                          {/each}
                        </ul>
                      {/if}
                    </div>
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
            {/if}
          </div>
        {:else}
          <p class="empty">{projectSearchQuery ? t("sidebar.noMatchingProjects") : t("sidebar.noProjectsYet")}</p>
        {/each}
      </div>
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
    <button type="button" class="sidebar-expand-btn" title={t("sidebar.show")} onclick={() => (sidebarVisible = true)}>»</button>
    {/if}

    <main class="main">
      {#if !selectedProjectId}
        <div class="empty-state">
          <div class="empty-icon">📁</div>
          <p>{t("workspace.selectProject")}</p>
        </div>
      {:else if !selectedRequest}
        <div class="empty-state">
          <div class="empty-icon">📨</div>
          <p>{t("request.selectPrompt")}</p>
        </div>
      {:else}
        <section class="detail">
          {#if openTabs.length > 0}
            <div class="request-tabs-bar">
              {#each openTabs as tab (tab.id)}
                <div class="request-tab-pill" class:active={tab.id === selectedRequest?.id}>
                  <button
                    type="button"
                    class="tab-pill-btn"
                    onclick={() => openRequest(tab.id)}
                  >
                    <span class="tab-method-badge method-{tab.method.toLowerCase()}">{tab.method}</span>
                    <span class="tab-title">{tab.name}</span>
                    {#if isTabDirty(tab.id)}
                      <span class="dirty-dot" title={t("tab.unsavedChanges")}>•</span>
                    {/if}
                  </button>
                  <button
                    type="button"
                    class="tab-close-btn"
                    title={t("tab.closeTab")}
                    onclick={(e) => {
                      e.stopPropagation();
                      closeTab(tab.id);
                    }}
                  >
                    ✕
                  </button>
                </div>
              {/each}
            </div>
          {/if}

          <div class="breadcrumb-row">
            <span class="breadcrumb-icon">🧭</span>
            <span class="breadcrumb-path">{projects.find((p) => p.id === selectedProjectId)?.name ?? ""}</span>
            <span class="breadcrumb-sep">›</span>
            {#if renamingRequestId === selectedRequest.id}
              <form class="inline-form" onsubmit={submitRenameRequest}>
                <input bind:value={renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
                <button type="submit" title={t("sidebar.save")}>✓</button>
                <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingRequestId = null)}>✕</button>
              </form>
            {:else}
              <button
                type="button"
                class="breadcrumb-current breadcrumb-current-btn"
                title={t("breadcrumb.renameHint")}
                onclick={() => startRenameRequest(selectedRequest!.id, selectedRequest!.name)}
              >
                {selectedRequest.name} <span class="breadcrumb-edit-hint">✎</span>
              </button>
            {/if}
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
                          onmouseenter={(e) => missing && showMissingVarPopover(tok.name, e.currentTarget as HTMLElement)}
                          onmouseleave={() => missing && scheduleHideMissingVarPopover()}
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
                    oninput={scheduleAutoSave}
                    onpaste={handleUrlPaste}
                    onscroll={syncUrlOverlayScroll}
                  />
                </div>
              </div>
              <div class="send-action">
                {#if sending}
                  <button type="button" class="btn-cancel" onclick={cancelCurrentSend}>{t("request.cancel")}</button>
                {:else}
                  <button type="button" class="btn-send" onclick={sendCurrentRequest}>{t("request.send")}</button>
                {/if}
                <button
                  type="button"
                  class="btn-save"
                  class:is-error={autoSaveStatus === "error"}
                  class:is-unsaved={autoSaveStatus === "unsaved"}
                  title={t("request.saveNow")}
                  onclick={() => saveRequest()}
                >
                  {#if autoSaveStatus === "saving"}{t("request.saving")}
                  {:else if autoSaveStatus === "unsaved"}{t("request.unsaved")}
                  {:else if autoSaveStatus === "error"}{t("request.saveFailed")}
                  {:else}{t("request.saved")}{/if}
                </button>
              </div>
            </div>
            <div class="request-bar-row secondary">
              {#if curlDetectedFeedback}
                <span class="hint">{curlDetectedFeedback}</span>
              {/if}
              <div class="response-stat-spacer"></div>
              <button type="button" class="icon-btn" title={t("request.deleteRequest")} onclick={() => deleteRequest(selectedRequest!.id)}>🗑</button>
            </div>
          </form>

          {#if missingVarHover}
            <div
              class="missing-var-popover-portal"
              role="group"
              aria-label={t("missingvar.addValueFor", { name: missingVarHover.name })}
              style="top: {missingVarHover.top}px; left: {missingVarHover.left}px;"
              onmouseenter={cancelHideMissingVarPopover}
              onmouseleave={scheduleHideMissingVarPopover}
            >
              <input
                placeholder={t("missingvar.valueFor", { name: missingVarHover.name })}
                value={missingVarDrafts[missingVarHover.name] ?? ""}
                oninput={(e) => (missingVarDrafts[missingVarHover!.name] = (e.target as HTMLInputElement).value)}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    addMissingVariable(missingVarHover!.name);
                  }
                }}
              />
              <button type="button" onclick={() => addMissingVariable(missingVarHover!.name)}>{t("missingvar.add")}</button>
            </div>
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
              ⚠️ {t("request.unresolvedVariables")}
              {#each requestDiagnostics.all_missing as varName (varName)}
                <span
                  class="missing-var-chip"
                  role="presentation"
                  onmouseenter={(e) => showMissingVarPopover(varName, e.currentTarget as HTMLElement)}
                  onmouseleave={scheduleHideMissingVarPopover}
                >
                  <strong>{varName}</strong>
                </span>
              {/each}
              <span class="hint">{t("request.unresolvedHint", { scope: selectedEnvironmentId ? t("request.scopeEnvironment") : t("request.scopeGlobal") })}</span>
            </div>
          {/if}

          <div class="editor-tabs">
            <button type="button" class="editor-tab" class:active={activeEditorTab === "params"} onclick={() => (activeEditorTab = "params")}>
              {t("tab.params")}
              {#if requestDiagnostics?.query_params_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInParams", { list: requestDiagnostics.query_params_missing.join(', ') })}>⚠ {requestDiagnostics.query_params_missing.length}</span>
              {:else if withoutEmptyKeyRows(editQueryParams).length}
                <span class="tab-badge">{withoutEmptyKeyRows(editQueryParams).length}</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "headers"} onclick={() => (activeEditorTab = "headers")}>
              {t("tab.headers")}
              {#if requestDiagnostics?.headers_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInHeaders", { list: requestDiagnostics.headers_missing.join(', ') })}>⚠ {requestDiagnostics.headers_missing.length}</span>
              {:else if withoutEmptyKeyRows(editHeaders).length}
                <span class="tab-badge">{withoutEmptyKeyRows(editHeaders).length}</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "auth"} onclick={() => (activeEditorTab = "auth")}>
              {t("tab.auth")}
              {#if requestDiagnostics?.auth_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInAuth", { list: requestDiagnostics.auth_missing.join(', ') })}>⚠ {requestDiagnostics.auth_missing.length}</span>
              {:else if editAuthType !== "none"}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "body"} onclick={() => (activeEditorTab = "body")}>
              {t("tab.body")}
              {#if requestDiagnostics?.body_missing?.length}
                <span class="tab-badge-warn" title={t("tab.missingInBody", { list: requestDiagnostics.body_missing.join(', ') })}>⚠ {requestDiagnostics.body_missing.length}</span>
              {:else if editBody}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "scripts"} onclick={() => (activeEditorTab = "scripts")}>
              {t("tab.scripts")} {#if editPreScript || editPostScript}<span class="tab-dot">•</span>{/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "settings"} onclick={() => (activeEditorTab = "settings")}>
              {t("tab.settings")}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "docs"} onclick={() => (activeEditorTab = "docs")}>
              {t("tab.docs")} {#if editDescription}<span class="tab-dot">•</span>{/if}
            </button>
          </div>

          <div class="editor-body-row">
          <div class="editor-main-col">
          <div class="tab-content">
            {#if activeEditorTab === "params"}
              <div class="params-table">
                {#each editQueryParams as param, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={param.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                    <input placeholder={t("params.key")} bind:value={param.key} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
                    <input placeholder={t("params.value")} bind:value={param.value} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
                    {#if i < editQueryParams.length - 1 || param.key.trim()}
                      <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeQueryParam(i); scheduleAutoSave(); }}>🗑</button>
                    {/if}
                  </div>
                {/each}
              </div>

            {:else if activeEditorTab === "headers"}
              <div class="params-table">
                {#each editHeaders as header, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={header.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
                    <input placeholder={t("params.key")} bind:value={header.key} oninput={() => { growHeaders(); scheduleAutoSave(); }} />
                    <input placeholder={t("params.value")} bind:value={header.value} oninput={() => { growHeaders(); scheduleAutoSave(); }} />
                    <input placeholder={t("headers.description")} bind:value={header.description} oninput={() => { growHeaders(); scheduleAutoSave(); }} />
                    {#if i < editHeaders.length - 1 || header.key.trim()}
                      <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeHeader(i); scheduleAutoSave(); }}>🗑</button>
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
                    <button type="button" onclick={() => { if (!editBody) editBody = "{\n  \n}"; scheduleAutoSave(); }}>{t("body.jsonTemplate")}</button>
                    <button type="button" onclick={() => { editBody = ""; scheduleAutoSave(); }}>{t("body.clearBody")}</button>
                  {/if}
                </div>

                {#if editBodyType === "raw"}
                  <textarea
                    placeholder={t("body.rawPlaceholder")}
                    bind:value={editBody}
                    class="body-input"
                    rows="8"
                    oninput={scheduleAutoSave}
                  ></textarea>
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
                          <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeFormDataItem(i); scheduleAutoSave(); }}>🗑</button>
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
                          <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeUrlEncodedItem(i); scheduleAutoSave(); }}>🗑</button>
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
                      oninput={scheduleAutoSave}
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
                      oninput={scheduleAutoSave}
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

            {/if}
          </div>

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
                <button type="button" class="icon-btn" title={t("response.copyTitle")} onclick={copyResponseBody}>⧉</button>
                <button type="button" class="icon-btn" title={t("response.downloadTitle")} onclick={downloadResponseBody}>⭳</button>
                <button type="button" class="icon-btn" title={t("response.expand")} onclick={() => (responseExpanded = true)}>⤢</button>
              </div>

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

                {#if responseSubTab === "body"}
                  <div class="response-format-toggle">
                    <button type="button" class="btn-toggle" class:active={responseViewMode === "pretty"} onclick={() => (responseViewMode = "pretty")}>{t("response.pretty")}</button>
                    <button type="button" class="btn-toggle" class:active={responseViewMode === "raw"} onclick={() => (responseViewMode = "raw")}>{t("response.raw")}</button>
                  </div>
                {/if}
              </div>

              <div class="response-subtab-content">
                {#if responseSubTab === "body"}
                  <pre class="body-view">{prettyResponseBody}</pre>
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
                          <span class="test-result-icon">{test.passed ? "✓" : "✗"}</span>
                          <span class="test-result-name">{test.name}</span>
                          {#if !test.passed && test.error}<span class="test-result-error">{test.error}</span>{/if}
                        </li>
                      {/each}
                    </ul>
                  {:else}
                    <p class="empty">{t("response.testsHintFull")}</p>
                  {/if}
                {/if}
              </div>
            </div>
          {:else}
            <div class="response-empty-state">
              <div class="empty-icon">📭</div>
              <p>{t("response.sendEmpty")}</p>
            </div>
          {/if}

          <div class="sample-responses-section">
            <div class="field-header-row">
              <h3>{t("sample.title", { count: sampleResponses.length })}</h3>
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
                        onclick={(e) => { e.stopPropagation(); deleteSampleResponseAction(sr.request_id, sr.id); }}
                        title={t("sample.delete")}
                      >✕</button>
                    </summary>
                    <pre class="body-view">{sr.body ?? ""}</pre>
                  </details>
                {/each}
              </div>
            {/if}
          </div>
          </div>
          </div>

          {#if rightPanel === "code"}
            <div class="bottom-panel">
              <h3 class="right-panel-title">{t("bottom.codeSnippet")}</h3>
              <div class="params-row">
                <select bind:value={snippetTarget}>
                  <option value="windows_cmd">{t("bottom.targetWindowsCmd")}</option>
                  <option value="powershell">{t("bottom.targetPowershell")}</option>
                  <option value="bash">{t("bottom.targetBash")}</option>
                  <option value="python">{t("bottom.targetPython")}</option>
                  <option value="javascript">{t("bottom.targetJavascript")}</option>
                </select>
                <select bind:value={snippetMode}>
                  <option value="placeholder">{t("bottom.placeholderSafe")}</option>
                  <option value="resolved">{t("bottom.resolvedReal")}</option>
                </select>
                <button type="button" class="btn-primary" onclick={copyAsCurl}>{t("bottom.generateSnippet")}</button>
                {#if snippet}
                  <button type="button" onclick={copySnippetToClipboard}>{t("bottom.copyClipboard")}</button>
                {/if}
              </div>
              {#if snippetError}
                <p class="error">{snippetError}</p>
              {/if}
              {#if snippet}
                <pre class="body-view bottom-panel-code">{snippet}</pre>
              {/if}
            </div>
          {:else if rightPanel === "info" && selectedRequest}
            <div class="bottom-panel">
              <h3 class="right-panel-title">{t("bottom.requestInfo")}</h3>
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

          <div class="bottom-bar">
            <button
              type="button"
              class="bottom-bar-tab"
              class:active={rightPanel === "code"}
              onclick={() => (rightPanel = rightPanel === "code" ? null : "code")}
            >&lt;/&gt; {t("bottom.codeSnippet")}</button>
            <button
              type="button"
              class="bottom-bar-tab"
              class:active={rightPanel === "info"}
              onclick={() => (rightPanel = rightPanel === "info" ? null : "info")}
            >ⓘ {t("bottom.info")}</button>
          </div>

          <h2>{t("history.title")}</h2>
          {#if responseHistory.length}
            <ul class="requests">
              {#each responseHistory as r (r.id)}
                <li class="row-item">
                  <button class="link" onclick={() => openHistoryResponse(r.id)}>
                    <span class="method">{r.status}</span>
                    <span class="name">{r.duration_ms} ms</span>
                    <span class="url">{new Date(r.created_at).toLocaleString()}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="empty">{t("history.empty")}</p>
          {/if}
        </section>
      {/if}
    </main>
  </div>

  {#if showConsole}
    <div class="console-drawer">
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
          <button type="button" class="console-close-btn" onclick={() => (showConsole = false)} title={t("console.closeTitle")}>✕</button>
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
                  <span class="evt-expander">{expandedEventIds.has(evt.id) ? "▼" : "▶"}</span>
                  <span class="evt-time">{formatConsoleTime(evt.timestamp)}</span>
                  <span class="evt-level level-{evt.level}">{evt.level.toUpperCase()}</span>
                  <span class="evt-type">{evt.event_type}</span>
                  <span class="evt-cid" title={t("console.correlationIdTitle", { id: evt.correlation_id })}>#{evt.correlation_id.slice(0, 8)}</span>
                  <span class="evt-msg">{evt.message}</span>
                </div>

                {#if expandedEventIds.has(evt.id) && evt.details}
                  <div class="console-row-details">
                    <div class="details-actions">
                      <button type="button" class="console-mini-btn" onclick={() => copyEventDetails(evt)}>{t("console.copyDetailsJson")}</button>
                    </div>
                    <pre class="console-json-view">{JSON.stringify(evt.details, null, 2)}</pre>
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
        class="console-toggle-btn"
        class:active={showConsole}
        onclick={() => {
          showConsole = !showConsole;
          if (showConsole) refreshConsoleEvents();
        }}
      >
        <span>{t("console.title")}</span>
        {#if consoleErrorCount > 0}
          <span class="status-badge-error">✖ {consoleErrorCount}</span>
        {/if}
        {#if consoleWarnCount > 0}
          <span class="status-badge-warn">⚠ {consoleWarnCount}</span>
        {/if}
      </button>
    </div>
    <div class="status-right">
      {#if selectedProjectId}
        <button
          type="button"
          class="git-status-pill git-kind-{gitStatus?.status_kind ?? 'unconfigured'}"
          class:git-has-conflict={gitStatus?.has_conflicts}
          title={t("footer.gitSyncTitle")}
          onclick={() => {
            activeScreen = "git";
            if (gitRepoPathInput) refreshGitStatus();
          }}
        >
          <span class="git-icon">⎇</span>
          {#if !gitSettings?.repo_path}
            <span>{t("footer.gitNotConfigured")}</span>
          {:else if gitStatusLoading}
            <span>{t("footer.gitChecking")}</span>
          {:else if gitStatus?.has_conflicts}
            <span class="git-alert">{t("footer.gitConflict", { count: gitStatus.conflict_files.length })}</span>
          {:else if gitStatus}
            <span>{t("footer.gitBranchStatus", { branch: gitStatus.branch, status: gitStatus.status_kind })}</span>
            {#if gitStatus.ahead > 0}<span class="git-ahead">↑{gitStatus.ahead}</span>{/if}
            {#if gitStatus.behind > 0}<span class="git-behind">↓{gitStatus.behind}</span>{/if}
          {:else}
            <span>{t("footer.gitLabel", { branch: gitBranchInput })}</span>
          {/if}
        </button>
        <span class="status-info">{t("footer.project", { name: projects.find((p) => p.id === selectedProjectId)?.name ?? selectedProjectId })}</span>
      {/if}
      {#if selectedEnvironmentId}
        <span class="status-info">{t("footer.env", { name: environments.find((e) => e.id === selectedEnvironmentId)?.name ?? selectedEnvironmentId })}</span>
      {/if}
    </div>
  </footer>


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
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showAiPanel = false)}>✕</button>
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
                            <span class="badge badge-auth" title={t("ai.authDetectedTitle")}>🔒 {ep.auth_hint}</span>
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
                    placeholder={aiSettings?.api_key || "sk-ant-api03-..."}
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
                <label for="ai-model-select"><strong>{t("ai.modelLabel")}</strong></label>
                <select id="ai-model-select" bind:value={aiModelInput} class="url-input">
                  <option value="claude-3-5-sonnet-20241022">{t("ai.modelSonnet")}</option>
                  <option value="claude-3-5-haiku-20241022">{t("ai.modelHaiku")}</option>
                  <option value="claude-3-opus-20240229">{t("ai.modelOpus")}</option>
                </select>
              </div>

              <div class="settings-field">
                <label for="ai-base-url-input"><strong>{t("ai.baseUrlLabel")}</strong></label>
                <input
                  id="ai-base-url-input"
                  type="text"
                  placeholder={t("ai.baseUrlPlaceholder")}
                  bind:value={aiBaseUrlInput}
                  class="url-input"
                />
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
                  <span>✅ {aiTestFeedback}</span>
                </div>
              {/if}
              {#if aiTestError}
                <div class="action-alert danger">
                  <span>❌ {aiTestError}</span>
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
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showDiffModal = false)}>✕</button>
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
          <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (showHistoryModal = false)}>✕</button>
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
        <div class="env-screen-list">
          <button
            type="button"
            class="env-screen-item"
            class:active={!selectedEnvironmentId}
            onclick={() => { selectedEnvironmentId = null; loadVariables(); }}
          >
            {t("env.noEnvironment")}
          </button>
          {#each environments as env (env.id)}
            {#if renamingEnvironmentId === env.id}
              <form class="inline-form env-screen-rename-form" onsubmit={submitRenameEnvironment}>
                <input bind:value={renameEnvironmentValue} use:focusOnMount onblur={submitRenameEnvironment} />
                <button type="submit" title={t("sidebar.save")}>✓</button>
                <button type="button" title={t("sidebar.cancel")} onclick={() => (renamingEnvironmentId = null)}>✕</button>
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
                <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameEnvironment(env)}>✎</button>
                <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.delete")} onclick={() => deleteEnvironmentAction(env.id)}>🗑</button>
              </div>
            {/if}
          {/each}
        </div>
      </aside>

      <section class="screen-page">
        <div class="screen-page-header">
          <span class="screen-kicker">{t("env.editing")}</span>
          <h1 class="screen-title">{selectedEnvironmentId ? (environments.find((e) => e.id === selectedEnvironmentId)?.name ?? t("env.fallbackName")) : t("env.globalAll")}</h1>
        </div>

        <div class="screen-page-body">
          <h4>{t("env.globalVariables", { count: projectVariables.length })}</h4>
          <div class="params-table">
            {#each projectVariables as v (v.id)}
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
                    <button type="button" class="icon-btn" title={t("env.reveal")} onclick={() => revealSecret(v.id)}>👁</button>
                  {/if}
                {/if}
                <button type="button" class="icon-btn" title={v.is_local ? t("env.makeShared") : t("env.makeLocalOnly")} onclick={() => toggleVariableLocal(v)}>{v.is_local ? "💻" : "🌐"}</button>
                <button type="button" class="icon-btn" title={t("env.toggleSecret")} onclick={() => toggleVariableSecret(v)}>🔒</button>
                <button type="button" class="icon-btn" title={t("sidebar.delete")} onclick={() => deleteVariable(v.id)}>🗑</button>
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
            <h4>{t("env.environmentVariables", { count: environmentVariables.length })}</h4>
            <div class="params-table">
              {#each environmentVariables as v (v.id)}
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
                      <button type="button" class="icon-btn" title={t("env.reveal")} onclick={() => revealSecret(v.id)}>👁</button>
                    {/if}
                  {/if}
                  <button type="button" class="icon-btn" title={v.is_local ? t("env.makeShared") : t("env.makeLocalOnly")} onclick={() => toggleVariableLocal(v)}>{v.is_local ? "💻" : "🌐"}</button>
                  <button type="button" class="icon-btn" title={t("env.toggleSecret")} onclick={() => toggleVariableSecret(v)}>🔒</button>
                  <button type="button" class="icon-btn" title={t("sidebar.delete")} onclick={() => deleteVariable(v.id)}>🗑</button>
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
  {#if !selectedProjectId}
    {@render noProjectPicker(t("git.title"))}
  {:else}
    <section class="screen-page">
      <div class="screen-page-header">
        <span class="screen-kicker">{t("git.title")}</span>
        <div class="screen-title-row">
          <h1 class="screen-title">{projects.find((p) => p.id === selectedProjectId)?.name ?? selectedProjectId}</h1>
          {@render projectSwitcher()}
        </div>
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
                    <button type="button" class="icon-btn-text" onclick={saveProjectToRepoAction} disabled={gitLoading}>
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
                        <span class="conflict-filename">📄 {file}</span>
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

              <div class="projectfile-options">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={projectFileMaskSecrets} />
                  {t("git.maskSecrets")}
                </label>
                <button type="button" class="btn-primary" onclick={exportProjectFileAction}>
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
{:else if activeScreen === "import"}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("import.title")}</span>
      <h1 class="screen-title">{t("import.subtitle")}</h1>
    </div>

    <div class="modal-tabs" style="flex:none; padding: 0 var(--space-6);">
      <button type="button" class="modal-tab-btn" class:active={importActiveTab === "collection"} onclick={() => (importActiveTab = "collection")}>{t("import.tabCollection")}</button>
      <button type="button" class="modal-tab-btn" class:active={importActiveTab === "environment"} onclick={() => (importActiveTab = "environment")}>{t("import.tabEnvironment")}</button>
      <button type="button" class="modal-tab-btn" class:active={importActiveTab === "curl"} onclick={() => (importActiveTab = "curl")}>{t("import.tabCurl")}</button>
    </div>

    <div class="screen-page-body">
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
      {/if}
    </div>
  </section>
{:else if activeScreen === "launcher"}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("launcher.kicker")}</span>
      <h1 class="screen-title">{t("launcher.title")}</h1>
      <p class="screen-subtitle">{t("launcher.subtitle", { count: projects.length })}</p>
    </div>

    {#if projects.length === 0}
      <p class="screen-empty">{t("launcher.noProjects")}</p>
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
      <button type="button" class="btn-primary" onclick={() => (activeScreen = "import")}>{t("launcher.importCollection")}</button>
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
        <button type="button" class="seg-opt" class:active={themeMode === "light"} onclick={() => setThemeMode("light")}>{t("settings.light")}</button>
        <button type="button" class="seg-opt" class:active={themeMode === "dark"} onclick={() => setThemeMode("dark")}>{t("settings.dark")}</button>
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
{:else if activeScreen === "theme"}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("settings.appearance")}</span>
      <h1 class="screen-title">{t("theme.title")}</h1>
      <p class="screen-subtitle">{t("theme.subtitle")} {t("theme.youAreOn", { mode: themeMode === "dark" ? t("settings.dark") : t("settings.light") })}</p>
    </div>
    <div class="theme-compare">
      <button type="button" class="theme-compare-col" class:active={themeMode === "light"} onclick={() => setThemeMode("light")}>
        <div class="theme-compare-header">
          <span class="screen-kicker">{t("settings.light")} — {t("settings.scaleDefault")}</span>
          {#if themeMode === "light"}<span class="theme-active-badge">{t("theme.active")}</span>{/if}
        </div>
        <div class="theme-swatch" data-theme="light">
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
      <button type="button" class="theme-compare-col" class:active={themeMode === "dark"} onclick={() => setThemeMode("dark")}>
        <div class="theme-compare-header">
          <span class="screen-kicker">{t("settings.dark")}</span>
          {#if themeMode === "dark"}<span class="theme-active-badge">{t("theme.active")}</span>{/if}
        </div>
        <div class="theme-swatch" data-theme="dark">
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
    </div>
  </section>
{/if}
  </div>

  {#if paletteOpen}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) closePalette(); }} onkeydown={(e) => { if (e.key === "Escape") closePalette(); }} role="dialog" aria-modal="true" tabindex="0">
      <div class="palette">
        <div class="palette-header">
          <span class="screen-kicker">{t("palette.goTo")}</span>
          <input class="palette-input" placeholder={t("palette.placeholder")} bind:value={paletteQuery} bind:this={paletteInputEl} />
        </div>
        <div class="palette-results">
          {#each paletteItems as p, i (i)}
            <button type="button" class="palette-item" onclick={p.onSelect}>
              <span class="palette-item-method">{p.method ?? ""}</span>
              <span class="palette-item-label">{p.label}</span>
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
</div>

<style>
  /* "Modernist" design system: flat, architectural, near-mono red-on-off-white, a visible
     modular grid, zero corner radius, strong 2px rules, Archivo throughout. Adopted wholesale
     (see the design-system export this was derived from) rather than layered on top of the
     app's previous dark Postman-style palette — every existing rule below still reads through
     these same custom-property names, so the remap alone repaints the whole app. Light is the
     system's own default (not an OS-follow); dark is a real, deliberate alternate palette
     toggled via `data-theme`, built from the same tonal ramps per the system's own guidance
     ("dark inverts the ground and lifts the accent one ramp step"). */
  @import url('https://fonts.googleapis.com/css2?family=Archivo:wght@400;600;800&display=swap');

  :root {
    --color-bg: #f3f2f2;
    --color-bg-secondary: #eae9e9;
    --color-bg-tertiary: #eae7e7;
    --color-bg-hover: color-mix(in srgb, #201e1d 7%, transparent);
    --color-sidebar-bg: #f3f2f2;
    --color-panel-bg: #eae9e9;
    --color-border: #d7d3d3;
    --color-border-strong: color-mix(in srgb, #201e1d 40%, transparent);
    --color-text: #201e1d;
    --color-text-secondary: #605d5d;
    --color-text-tertiary: #7d7979;
    --color-primary: #ec3013;
    --color-primary-hover: #dd2b0f;
    --color-primary-contrast: #f3f2f2;
    --color-accent: #ec3013;
    --color-accent-hover: #dd2b0f;
    --color-accent-contrast: #f3f2f2;
    --color-success: var(--color-text);
    --color-success-bg: transparent;
    --color-danger: #ec3013;
    --color-danger-bg: #fff2ef;
    --color-warn: #ae1800;
    --color-warn-bg: #fff2ef;
    --color-focus: #ec3013;

    /* The system deliberately does not color-code HTTP verbs (no rainbow GET/POST/etc.) —
       method text is plain ink, and turns the accent only when its row/tab is active. */
    --method-get: var(--color-text-secondary);
    --method-post: var(--color-text-secondary);
    --method-put: var(--color-text-secondary);
    --method-patch: var(--color-text-secondary);
    --method-delete: var(--color-text-secondary);
    --method-head: var(--color-text-secondary);
    --method-options: var(--color-text-secondary);
    --method-trace: var(--color-text-secondary);

    /* Tonal ramps (OKLCH-derived in the source system) — light steps (100-300) for tinted
       fills/hovers, 500 as a role's base, dark steps (700-900) for text on tinted fills. */
    --color-neutral-100: #f8f4f4;
    --color-neutral-200: #eae7e7;
    --color-neutral-300: #d7d3d3;
    --color-neutral-400: #bab6b6;
    --color-neutral-500: #9b9797;
    --color-neutral-600: #7d7979;
    --color-neutral-700: #605d5d;
    --color-neutral-800: #444141;
    --color-neutral-900: #2d2b2b;
    --color-accent-100: #fff2ef;
    --color-accent-200: #ffe0d9;
    --color-accent-300: #ffc4b8;
    --color-accent-400: #ff9783;
    --color-accent-500: #ff563c;
    --color-accent-600: #dd2b0f;
    --color-accent-700: #ae1800;
    --color-accent-800: #7c1405;
    --color-accent-900: #4d170e;

    --space-1: 4px;
    --space-2: 8px;
    --space-3: 12px;
    --space-4: 16px;
    --space-6: 24px;
    --space-8: 32px;

    --radius-sm: 0px;
    --radius-md: 0px;
    --radius-lg: 0px;
    --shadow-sm: 0 1px 2px color-mix(in srgb, #2d2b2b 14%, transparent);
    --shadow-md: 0 3px 10px color-mix(in srgb, #2d2b2b 16%, transparent);
    --shadow-lg: 0 12px 32px color-mix(in srgb, #2d2b2b 22%, transparent);
    --font-sans: "Archivo", system-ui, sans-serif;
    --font-heading: "Archivo", system-ui, sans-serif;
    --font-mono: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;

    color-scheme: light;
    color: var(--color-text);
    background: var(--color-bg);
    font-family: var(--font-sans);
    font-size: 12.5px;
  }

  /* Not scoped to :root — also applies to nested [data-theme="dark"] elements (the Theme
     screen's side-by-side comparison swatches), since custom properties inherit normally. */
  [data-theme="dark"] {
    --color-bg: #201e1d;
    --color-bg-secondary: #2d2b2b;
    --color-bg-tertiary: #363433;
    --color-bg-hover: color-mix(in srgb, #f8f4f4 8%, transparent);
    --color-sidebar-bg: #201e1d;
    --color-panel-bg: #2d2b2b;
    --color-border: #444141;
    --color-border-strong: color-mix(in srgb, #f8f4f4 25%, transparent);
    --color-text: #f8f4f4;
    --color-text-secondary: #bab6b6;
    --color-text-tertiary: #9b9797;
    --color-primary: #ff563c;
    --color-primary-hover: #ff9783;
    --color-primary-contrast: #201e1d;
    --color-accent: #ff563c;
    --color-accent-hover: #ff9783;
    --color-accent-contrast: #201e1d;
    --color-danger: #ff563c;
    --color-danger-bg: color-mix(in srgb, #ff563c 16%, transparent);
    --color-warn: #ff9783;
    --color-warn-bg: color-mix(in srgb, #ff563c 16%, transparent);
    --color-focus: #ff563c;

    --method-get: var(--color-text-secondary);
    --method-post: var(--color-text-secondary);
    --method-put: var(--color-text-secondary);
    --method-patch: var(--color-text-secondary);
    --method-delete: var(--color-text-secondary);
    --method-head: var(--color-text-secondary);
    --method-options: var(--color-text-secondary);
    --method-trace: var(--color-text-secondary);

    color-scheme: dark;
  }

  /* Mirrors the :root light values, scoped so a nested [data-theme="light"] element (the Theme
     screen's comparison swatch) resets back to light even while the app itself is on dark. */
  [data-theme="light"] {
    --color-bg: #f3f2f2;
    --color-bg-secondary: #eae9e9;
    --color-bg-tertiary: #eae7e7;
    --color-bg-hover: color-mix(in srgb, #201e1d 7%, transparent);
    --color-sidebar-bg: #f3f2f2;
    --color-panel-bg: #eae9e9;
    --color-border: #d7d3d3;
    --color-border-strong: color-mix(in srgb, #201e1d 40%, transparent);
    --color-text: #201e1d;
    --color-text-secondary: #605d5d;
    --color-text-tertiary: #7d7979;
    --color-primary: #ec3013;
    --color-primary-hover: #dd2b0f;
    --color-primary-contrast: #f3f2f2;
    --color-accent: #ec3013;
    --color-accent-hover: #dd2b0f;
    --color-accent-contrast: #f3f2f2;
    --color-danger: #ec3013;
    --color-danger-bg: #fff2ef;
    --color-warn: #ae1800;
    --color-warn-bg: #fff2ef;
    --color-focus: #ec3013;

    --method-get: var(--color-text-secondary);
    --method-post: var(--color-text-secondary);
    --method-put: var(--color-text-secondary);
    --method-patch: var(--color-text-secondary);
    --method-delete: var(--color-text-secondary);
    --method-head: var(--color-text-secondary);
    --method-options: var(--color-text-secondary);
    --method-trace: var(--color-text-secondary);

    color-scheme: light;
  }

  :global(body) {
    margin: 0;
    background: var(--color-bg);
  }

  * {
    box-sizing: border-box;
  }

  .app-shell {
    display: flex;
    height: 100vh;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: var(--font-sans);
  }

  .screens-rail {
    width: 220px;
    flex: none;
    display: flex;
    flex-direction: column;
    border-right: 2px solid var(--color-border-strong);
    background: var(--color-bg);
    overflow: hidden;
    transition: width 0.15s ease;
  }

  .screens-rail.collapsed {
    width: 52px;
  }

  .rail-brand {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-3) var(--space-3) var(--space-4);
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 0.95rem;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    border-bottom: 2px solid var(--color-border-strong);
  }

  .screens-rail.collapsed .rail-brand {
    justify-content: center;
    padding: var(--space-4) var(--space-2) var(--space-3) var(--space-2);
  }

  .rail-screens {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .rail-screen {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-left: 4px solid transparent;
    padding: 0.65rem var(--space-4);
    font-family: var(--font-sans);
    font-size: 0.8rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .screens-rail.collapsed .rail-screen {
    justify-content: center;
    padding: 0.65rem 0;
  }

  .rail-screen-icon {
    font-size: 1rem;
    flex: none;
  }

  .rail-screen-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rail-screen:hover {
    background: var(--color-bg-secondary);
  }

  .rail-screen.active {
    background: var(--color-bg-secondary);
    border-left-color: var(--color-accent);
    font-weight: 800;
  }

  .rail-budget {
    flex: none;
    border-top: 2px solid var(--color-border-strong);
    padding: var(--space-3) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .rail-budget-label {
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
  }

  .rail-budget-value {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 1.15rem;
  }

  .rail-budget-meta {
    font-size: 0.68rem;
    color: var(--color-text-tertiary);
  }

  .screen-area {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: var(--font-sans);
    color: var(--color-text);
    background: var(--color-bg);
    overflow: hidden;
  }

  /* — Generic full-page screens (Response/Environments/Git/Import/Launcher/History/
     Settings/Theme) — one consistent kicker+title header pattern, reused everywhere. — */
  .screen-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .screen-page-header {
    flex: none;
    padding: var(--space-8) var(--space-6) var(--space-4);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .screen-kicker {
    display: block;
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    margin-bottom: 4px;
  }

  .screen-title-row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .project-picker-select {
    font-family: var(--font-sans);
    font-size: 0.8rem;
  }

  .project-picker-select-sm {
    font-size: 0.75rem;
    padding: 0.25rem 0.4rem;
  }

  .screen-title {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 1.8rem;
    line-height: 1.1;
    margin: 0;
  }

  .screen-subtitle {
    font-size: 0.85rem;
    color: var(--color-text-secondary);
    max-width: 640px;
    margin-top: 4px;
  }

  .screen-empty {
    padding: var(--space-8) var(--space-6);
    color: var(--color-text-tertiary);
    font-size: 0.85rem;
  }

  .screen-empty-inline {
    color: var(--color-text-tertiary);
    font-size: 0.78rem;
  }

  .screen-page-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--space-6);
  }

  /* — Environments screen — */
  .env-screen {
    height: 100%;
    display: flex;
  }

  .env-screen-side {
    width: 260px;
    flex: none;
    border-right: 2px solid var(--color-border-strong);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .env-screen-side-header {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-4) var(--space-3);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .env-screen-project-row {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .env-screen-project-label {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
  }

  .env-screen-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .env-screen-item {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-left: 4px solid transparent;
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-3) var(--space-4);
    font: inherit;
    font-size: 0.8rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .env-screen-item:hover {
    background: var(--color-bg-secondary);
  }

  .env-screen-item.active {
    background: var(--color-bg-secondary);
    border-left-color: var(--color-accent);
    font-weight: 800;
  }

  .env-screen-item-row {
    display: flex;
    align-items: center;
    border-left: 4px solid transparent;
    border-bottom: 1px solid var(--color-border);
  }

  .env-screen-item-row:hover {
    background: var(--color-bg-secondary);
  }

  .env-screen-item-row:hover .icon-btn-ghost,
  .env-screen-item-row:focus-within .icon-btn-ghost {
    display: inline-block;
  }

  .env-screen-item-row.active {
    background: var(--color-bg-secondary);
    border-left-color: var(--color-accent);
  }

  .env-screen-item-row.active .env-screen-item {
    font-weight: 800;
  }

  .env-screen-item-row .env-screen-item {
    flex: 1;
    min-width: 0;
    border: none;
    padding: var(--space-3) var(--space-2) var(--space-3) calc(var(--space-4) - 4px);
  }

  .env-screen-item-row .icon-btn-ghost {
    flex: none;
    margin-right: var(--space-2);
  }

  .env-screen-rename-form {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--color-border);
  }

  .hr {
    height: 2px;
    border: 0;
    margin: var(--space-3) 0;
    background: var(--color-border-strong);
  }

  /* — Launcher screen — */
  .launcher-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .launcher-card {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    text-align: left;
    background: transparent;
    border: none;
    border-right: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-4);
    cursor: pointer;
    font: inherit;
    color: var(--color-text);
  }

  .launcher-card:hover {
    background: var(--color-bg-secondary);
  }

  .launcher-card.active {
    background: var(--color-bg-secondary);
  }

  .screen-kicker.current {
    color: var(--color-accent-700);
  }

  .launcher-card-name {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 1.1rem;
    margin: 2px 0 4px;
  }

  .launcher-card-meta {
    display: flex;
    gap: var(--space-3);
    font-size: 0.72rem;
    color: var(--color-text-tertiary);
  }

  .history-toolbar-project-label {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
  }

  /* — History screen — */
  .history-toolbar {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-6);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .history-search {
    flex: none;
    width: 360px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    padding: 0.4rem 0.6rem;
    font: inherit;
    font-size: 0.8rem;
    color: var(--color-text);
  }

  .hr-v {
    width: 2px;
    height: 22px;
    background: var(--color-border-strong);
  }

  .history-filter-chip {
    font-size: 0.7rem;
    font-weight: 800;
    padding: 5px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
  }

  .history-filter-chip.active {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .history-header-row,
  .history-row {
    display: grid;
    grid-template-columns: 70px 2fr 80px 90px 160px;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-6);
  }

  .history-header-row {
    flex: none;
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .history-rows {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .history-row {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--color-border);
    font: inherit;
    font-size: 0.78rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .history-row:hover {
    background: var(--color-bg-secondary);
  }

  .history-method {
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.04em;
  }

  .history-path {
    font-family: var(--font-mono);
    font-size: 0.74rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* — Settings screen — */
  .settings-screen-row {
    display: grid;
    grid-template-columns: 1.3fr 1.6fr;
    align-items: center;
    gap: var(--space-6);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--color-border);
  }

  .settings-screen-row-label {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 0.95rem;
    margin-bottom: 2px;
  }

  .settings-screen-row-value {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 0.9rem;
    color: var(--color-text-secondary);
  }

  .settings-screen-diagnostics-grid {
    display: grid;
    grid-template-columns: 1fr auto;
    row-gap: 4px;
    column-gap: var(--space-3);
    font-size: 0.78rem;
  }

  .settings-screen-diagnostics-grid span {
    color: var(--color-text-tertiary);
  }

  .seg {
    display: inline-flex;
    overflow: hidden;
    border: 1px solid var(--color-border-strong);
  }

  .seg-opt {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    background: transparent;
    border: none;
    border-left: 1px solid var(--color-border-strong);
    color: var(--color-text);
    font-family: inherit;
  }

  .seg-opt:first-child {
    border-left: none;
  }

  .seg-opt.active {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .seg-opt:not(.active):hover {
    background: var(--color-bg-hover);
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .shortcut-toggle-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.8rem;
    cursor: pointer;
  }

  .shortcut-label {
    flex: 1;
  }

  .shortcut-keys {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    padding: 2px 6px;
    border: 1px solid var(--color-border-strong);
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
  }

  /* — Theme screen — */
  .theme-compare {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: var(--space-6);
    gap: var(--space-6);
  }

  .theme-compare-col {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 2px solid transparent;
    padding: var(--space-3);
    margin: calc(var(--space-3) * -1);
    font: inherit;
    color: inherit;
    cursor: pointer;
  }

  .theme-compare-col:hover {
    border-color: var(--color-border);
  }

  .theme-compare-col.active {
    border-color: var(--color-accent);
  }

  .theme-active-badge {
    font-size: 0.62rem;
    font-weight: 800;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-accent-700);
    border: 1px solid var(--color-accent);
    padding: 2px 8px;
  }

  .theme-compare-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .theme-swatch {
    border: 2px solid var(--color-border-strong);
    background: var(--color-bg);
    color: var(--color-text);
  }

  .theme-swatch-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.7rem 0.9rem;
    border-bottom: 2px solid var(--color-border-strong);
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .theme-swatch-sync {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    font-size: 0.62rem;
    padding: 3px 8px;
  }

  .theme-swatch-body {
    display: flex;
  }

  .theme-swatch-side {
    width: 38%;
    flex: none;
    border-right: 1px solid var(--color-border);
    padding: 0.9rem;
    font-size: 0.78rem;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .theme-swatch-active {
    color: var(--color-accent);
    font-weight: 800;
  }

  .theme-swatch-main {
    flex: 1;
    padding: 0.9rem;
  }

  .theme-swatch-path {
    font-family: var(--font-mono);
    font-size: 0.72rem;
  }

  .theme-swatch-status {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 1.3rem;
  }

  /* — Git screen: real 3-way conflict view — */
  .conflict-3way {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-3);
  }

  .conflict-3way-col {
    flex: 1;
    min-width: 0;
  }

  .conflict-3way-pre {
    max-height: 220px;
    font-size: 0.7rem;
  }

  /* — Response screen — */
  .response-screen-body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .response-screen-side {
    width: 300px;
    flex: none;
    border-right: 2px solid var(--color-border-strong);
    overflow-y: auto;
  }

  .response-screen-stat-block {
    padding: var(--space-4) var(--space-4);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .response-screen-status {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 2rem;
  }

  .response-screen-path {
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .response-screen-metrics {
    display: grid;
    grid-template-columns: 1fr 1fr;
    border-bottom: 2px solid var(--color-border-strong);
  }

  .response-screen-metric {
    padding: var(--space-3) var(--space-4);
    border-right: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
  }

  .response-screen-metric-value {
    font-family: var(--font-heading);
    font-weight: 800;
    font-size: 1.1rem;
  }

  .response-screen-tests {
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .response-screen-test-row {
    display: flex;
    gap: var(--space-3);
    font-size: 0.78rem;
    padding: 4px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .response-screen-test-row span:first-child {
    font-weight: 800;
    width: 36px;
    flex: none;
  }

  .response-screen-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .response-screen-content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--space-4);
  }

  .screen-body-view {
    max-height: none;
  }

  /* — Command palette — */
  .palette {
    width: 620px;
    max-width: 92vw;
    background: var(--color-bg);
    border: 2px solid var(--color-border-strong);
    box-shadow: var(--shadow-lg);
  }

  .palette-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 2px solid var(--color-border-strong);
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    font: inherit;
    font-size: 1rem;
    color: var(--color-text);
    outline: none;
  }

  .palette-results {
    max-height: 50vh;
    overflow-y: auto;
  }

  .palette-item {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-3) var(--space-4);
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .palette-item:hover {
    background: var(--color-bg-secondary);
  }

  .palette-item-method {
    width: 48px;
    flex: none;
    font-size: 0.65rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    color: var(--color-accent);
  }

  .palette-item-hint {
    font-size: 0.72rem;
    color: var(--color-text-tertiary);
  }

  .palette-footer {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-2) var(--space-4);
    font-size: 0.68rem;
    color: var(--color-text-tertiary);
  }

  .palette-trigger {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 260px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    padding: 0.4rem 0.7rem;
    font: inherit;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    cursor: text;
    text-align: left;
  }

  .palette-trigger:hover {
    border-color: var(--color-border-strong);
  }

  .palette-trigger span:first-child {
    flex: 1;
  }

  .palette-kbd {
    font-size: 0.62rem;
    font-weight: 800;
    border: 1px solid var(--color-border);
    padding: 1px 5px;
  }

  /* ---------- Topbar ---------- */
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0.9rem;
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .brand {
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--color-primary);
    white-space: nowrap;
  }

  .topbar-left,
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .topbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    min-width: 0;
  }

  .env-bar {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .env-select {
    max-width: 180px;
  }

  /* ---------- Buttons & inputs ---------- */
  button {
    font-family: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    border-radius: var(--radius-sm);
    padding: 0.35rem 0.7rem;
    transition: background-color 0.12s, border-color 0.12s, color 0.12s;
  }

  button:hover {
    background: var(--color-bg-hover);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input,
  select,
  textarea {
    font-family: inherit;
    font-size: 0.8rem;
    color: var(--color-text);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.35rem 0.55rem;
  }

  input:focus,
  select:focus,
  textarea:focus,
  button:focus-visible {
    outline: 2px solid var(--color-focus);
    outline-offset: -1px;
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .btn-ghost:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .btn-secondary {
    background: transparent;
    border: 1px solid var(--color-border-strong);
    color: var(--color-text);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .btn-secondary:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .btn-xs {
    padding: 0.2rem 0.5rem;
    font-size: 0.68rem;
  }

  .btn-xs.active {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  /* Orange is the brand mark only (logo, "+"/add affordances) — the actual primary action
     color in real Postman is blue (Send, and anything analogous to it). Conflating the two
     was the biggest color mismatch against the reference. */
  .btn-primary {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-primary-contrast);
    font-weight: 600;
  }

  .btn-primary:hover {
    background: var(--color-primary-hover);
    border-color: var(--color-primary-hover);
  }

  .btn-save {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    background: transparent;
    border: 1px solid var(--color-border-strong);
    white-space: nowrap;
  }

  .btn-save:hover {
    color: var(--color-text-primary);
    border-color: var(--color-text-tertiary);
  }

  .btn-save.is-unsaved {
    color: var(--color-text-primary);
    border-color: var(--color-accent);
  }

  .btn-save.is-error {
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  .btn-send {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-accent-contrast);
    font-weight: 600;
  }

  .btn-send:hover {
    background: var(--color-accent-hover);
    border-color: var(--color-accent-hover);
  }

  .btn-cancel {
    background: var(--color-danger);
    border-color: var(--color-danger);
    color: #fff;
    font-weight: 600;
  }

  .btn-icon-add {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: #fff;
    font-weight: 700;
    padding: 0.35rem 0.6rem;
    flex-shrink: 0;
  }

  .icon-btn {
    background: transparent;
    border: none;
    padding: 0.2rem 0.35rem;
    color: var(--color-text-tertiary);
    border-radius: var(--radius-sm);
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-btn.active {
    color: var(--color-accent);
  }

  .icon-btn-ghost {
    display: none;
  }

  /* ---------- Banners ---------- */
  .success-banner,
  .error-banner,
  .warn-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.45rem 0.9rem;
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .success-banner {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .error-banner {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .warn-banner {
    background: var(--color-warn-bg);
    color: var(--color-warn);
    border-radius: var(--radius-sm);
    margin: 0.5rem 0.9rem 0;
  }

  .missing-vars-banner {
    justify-content: flex-start;
    flex-wrap: wrap;
  }

  .missing-var-chip {
    display: inline-block;
    cursor: default;
    border-bottom: 1px dotted var(--color-warn);
  }

  /* A plain CSS :hover popover nested inside either the banner or the (overflow-clipped) URL
     bar would get clipped by an ancestor eventually — the URL bar's overlay in particular
     needs real overflow-x clipping for long URLs, and CSS has no "clip X, don't clip Y" (a
     non-"visible" axis paired with "visible" silently becomes "auto", which still clips). So
     this is a single shared, JS-positioned, fixed-position portal instead — see
     showMissingVarPopover/scheduleHideMissingVarPopover — rendered once at the bottom of the
     request editor and reused by both the banner chips and the URL bar tokens.  */
  .missing-var-popover-portal {
    display: flex;
    position: fixed;
    background: var(--color-bg);
    border: 2px solid var(--color-border-strong);
    padding: var(--space-2);
    gap: 4px;
    z-index: 1000;
    color: var(--color-text);
  }

  .missing-var-popover-portal input {
    font-size: 0.8rem;
  }

  /* Highlights {{variables}} directly inside the URL bar, in place, instead of only in the
     warning banner below — an invisible-text "ghost" input sits on top of a styled overlay
     that renders the same string with each {{var}} as its own token; the overlay is
     pointer-events:none everywhere except on a missing token, so typing/clicking still goes
     to the real input underneath except when hovering a token that needs a value. */
  .url-input-shell {
    position: relative;
    flex: 1;
    display: flex;
    align-items: stretch;
    min-width: 0;
  }

  .url-token-overlay {
    position: absolute;
    inset: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    white-space: pre;
    overflow: hidden;
    pointer-events: none;
    padding: 0.35rem 0.55rem;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    color: var(--color-text);
  }

  .url-token-text {
    white-space: pre;
  }

  .url-token-var {
    white-space: pre;
    color: var(--color-accent);
  }

  .url-token-var.missing {
    pointer-events: auto;
    color: var(--color-warn);
    border-bottom: 1px dotted var(--color-warn);
    cursor: default;
  }

  .url-input-ghost {
    position: relative;
    z-index: 1;
    color: transparent;
    caret-color: var(--color-text);
  }

  .dismiss-btn {
    background: none;
    border: none;
    color: inherit;
    padding: 0 0.3rem;
  }

  .warn-inline {
    color: var(--color-warn);
    font-size: 0.75rem;
  }

  .hint {
    color: var(--color-text-tertiary);
    font-size: 0.78rem;
    padding: 0.2rem 0;
  }

  .error {
    color: var(--color-danger);
    font-size: 0.8rem;
  }

  /* ---------- Workspace layout ---------- */
  .workspace {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .sidebar {
    width: 300px;
    flex-shrink: 0;
    background: var(--color-sidebar-bg);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .sidebar-resize-handle {
    width: 5px;
    flex-shrink: 0;
    margin-left: -3px;
    cursor: col-resize;
    z-index: 1;
    background: transparent;
  }

  .sidebar-resize-handle:hover,
  .sidebar-resize-handle.resizing {
    background: var(--color-accent);
  }

  .sidebar-header {
    padding: 0.7rem 0.9rem 0.3rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sidebar-expand-btn {
    flex: none;
    width: 18px;
    align-self: flex-start;
    margin-top: 6px;
    background: transparent;
    border: none;
    border-right: 1px solid var(--color-border);
    color: var(--color-text-tertiary);
    cursor: pointer;
    font-size: 0.7rem;
  }

  .sidebar-expand-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .sidebar-title {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
  }

  .sidebar-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .project-list {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 1rem;
  }

  .project-node {
    border-bottom: 1px solid transparent;
  }

  .project-row {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.1rem 0.5rem 0.1rem 0.7rem;
    border-radius: var(--radius-sm);
    margin: 0 0.4rem;
  }

  .project-row:hover {
    background: var(--color-bg-hover);
  }

  .project-row.active {
    background: var(--color-bg-hover);
  }

  .project-link {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.4rem 0.1rem;
    font-weight: 500;
    color: var(--color-text);
    min-width: 0;
  }

  .project-link:hover {
    background: none;
  }

  .folder-icon {
    flex-shrink: 0;
  }

  .project-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-row-actions {
    display: none;
    gap: 0.1rem;
    flex-shrink: 0;
  }

  .project-row:hover .project-row-actions,
  .project-row:focus-within .project-row-actions,
  .project-row-actions.force-visible {
    display: flex;
  }

  .project-requests {
    padding: 0.3rem 0.5rem 0.6rem 1.3rem;
    border-left: 2px solid var(--color-border);
    margin: 0 0.9rem 0.4rem 1.1rem;
  }

  .project-search-box {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex: none;
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--color-border);
  }

  .project-search-input {
    flex: 1;
    width: 100%;
  }

  .request-search-box {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.4rem;
  }

  .request-search-input {
    flex: 1;
    width: 100%;
  }

  .menu-wrap {
    position: relative;
    flex: none;
  }

  .dropdown-backdrop {
    position: fixed;
    inset: 0;
    background: transparent;
    border: none;
    padding: 0;
    z-index: 55;
    cursor: default;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.2rem);
    inset-inline-end: 0;
    z-index: 56;
    min-width: 10rem;
    background: var(--color-bg);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    padding: 0.25rem;
  }

  .dropdown-menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.4rem 0.5rem;
    font-size: 0.78rem;
    color: var(--color-text);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .dropdown-menu-item:hover {
    background: var(--color-bg-hover);
  }

  .dropdown-menu-item.active {
    font-weight: 700;
    color: var(--color-accent);
  }

  .sort-dir-indicator {
    flex: none;
  }

  .request-count-badge {
    font-size: 0.7rem;
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }

  .request-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .request-item {
    display: flex;
    align-items: center;
    border-radius: var(--radius-sm);
  }

  .folder-node {
    margin-bottom: 2px;
  }

  .folder-row {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    border-radius: var(--radius-sm);
  }

  .folder-row:hover {
    background: var(--color-bg-hover);
  }

  .folder-row:hover .project-row-actions,
  .folder-row:focus-within .project-row-actions {
    display: flex;
  }

  .folder-link {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.3rem 0.1rem;
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--color-text);
    cursor: pointer;
  }

  .folder-request-list {
    padding: 0.1rem 0 0.2rem 1.1rem;
    border-left: 2px solid var(--color-border);
    margin-left: 0.5rem;
  }

  .request-item:hover {
    background: var(--color-bg-hover);
  }

  .request-item.active {
    background: var(--color-bg-hover);
    font-weight: 600;
  }

  .request-link {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.3rem 0.2rem;
    min-width: 0;
    color: var(--color-text);
  }

  .request-link:hover {
    background: none;
  }

  .request-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.78rem;
  }

  .request-item:hover .icon-btn-ghost,
  .request-item:focus-within .icon-btn-ghost {
    display: inline-block;
  }

  .request-item-wrapper {
    display: flex;
    flex-direction: column;
  }

  .tree-expand-btn {
    flex: none;
    width: 1rem;
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    font-size: 0.65rem;
  }

  .sample-tree-list {
    list-style: none;
    margin: 0;
    padding: 0.1rem 0 0.2rem 1.6rem;
    border-left: 2px solid var(--color-border);
    margin-left: 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .sample-tree-item {
    display: flex;
    align-items: center;
    border-radius: var(--radius-sm);
  }

  .sample-tree-item:hover {
    background: var(--color-bg-hover);
  }

  .sample-tree-item:hover .icon-btn-ghost,
  .sample-tree-item:focus-within .icon-btn-ghost {
    display: inline-block;
  }

  .sample-tree-link {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.3rem 0.2rem;
    min-width: 0;
    color: var(--color-text);
  }

  .sample-tree-link:hover {
    background: none;
  }

  .status-chip {
    flex: none;
    font-family: var(--font-mono);
    font-size: 0.68rem;
    font-weight: 700;
  }

  .sample-tree-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.76rem;
  }

  .request-pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.4rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .empty {
    color: var(--color-text-tertiary);
    font-size: 0.78rem;
    padding: 0.3rem 0.9rem;
    list-style: none;
  }

  /* ---------- Method badges (Postman color language) ---------- */
  .method-badge,
  .tab-method-badge,
  .method,
  .method-select {
    font-weight: 700;
    font-size: 0.68rem;
    letter-spacing: 0.02em;
  }

  /* Method shown as small colored text, never a tinted pill/badge — that's how the
     reference actually renders it in the sidebar and tab bar. */
  .method-badge,
  .tab-method-badge {
    display: inline-block;
    width: 2.8rem;
    text-align: left;
    flex-shrink: 0;
    background: none;
  }

  .tab-method-badge {
    width: auto;
  }

  .method-select {
    width: 6.5rem;
    flex-shrink: 0;
    font-weight: 700;
    background: var(--color-bg);
  }

  .method-get { color: var(--method-get); }
  .method-post { color: var(--method-post); }
  .method-put { color: var(--method-put); }
  .method-patch { color: var(--method-patch); }
  .method-delete { color: var(--method-delete); }
  .method-head { color: var(--method-head); }
  .method-options { color: var(--method-options); }
  .method-trace { color: var(--method-trace); }

  /* ---------- Main content ---------- */
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow-y: auto;
    background: var(--color-bg);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    color: var(--color-text-tertiary);
  }

  .empty-icon {
    font-size: 2.5rem;
    opacity: 0.5;
  }

  .detail {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  /* ---------- Request tabs bar ---------- */
  .request-tabs-bar {
    display: flex;
    gap: 0;
    padding: 0 0.4rem;
    overflow-x: auto;
    background: var(--color-sidebar-bg);
    border-bottom: 1px solid var(--color-border);
  }

  .request-tab-pill {
    display: flex;
    align-items: center;
    gap: 0;
    background: transparent;
    border: none;
    border-right: 1px solid var(--color-border);
    border-top: 2px solid transparent;
    padding: 0 0.15rem 0 0.6rem;
    max-width: 200px;
  }

  .request-tab-pill.active {
    background: var(--color-bg);
    border-top-color: var(--color-accent);
  }

  .tab-pill-btn {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: none;
    padding: 0.45rem 0.2rem;
    min-width: 0;
    color: var(--color-text-secondary);
  }

  .request-tab-pill.active .tab-pill-btn {
    color: var(--color-text);
  }

  .tab-pill-btn:hover {
    background: none;
    color: var(--color-text);
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.76rem;
    max-width: 110px;
  }

  .dirty-dot {
    color: var(--method-post);
    font-size: 0.9rem;
    line-height: 0;
  }

  .tab-close-btn {
    background: none;
    border: none;
    padding: 0.15rem 0.35rem;
    color: var(--color-text-tertiary);
    font-size: 0.65rem;
    opacity: 0;
  }

  .request-tab-pill:hover .tab-close-btn {
    opacity: 1;
  }

  .tab-close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-danger);
  }

  /* ---------- Request bar ---------- */
  .breadcrumb-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 0.9rem 0;
    font-size: 0.76rem;
    color: var(--color-text-tertiary);
  }

  .breadcrumb-icon {
    opacity: 0.7;
  }

  .breadcrumb-sep {
    color: var(--color-text-tertiary);
  }

  .breadcrumb-current {
    color: var(--color-text);
    font-weight: 600;
  }

  .breadcrumb-current-btn {
    background: transparent;
    border: none;
    padding: 2px 4px;
    font: inherit;
    cursor: pointer;
  }

  .breadcrumb-current-btn:hover {
    background: var(--color-bg-hover);
  }

  .breadcrumb-edit-hint {
    opacity: 0;
    font-size: 0.68rem;
  }

  .breadcrumb-current-btn:hover .breadcrumb-edit-hint {
    opacity: 0.6;
  }

  .request-bar {
    padding: 0.5rem 0.9rem 0.4rem;
    border-bottom: 1px solid var(--color-border);
  }

  .request-bar-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
  }

  .request-bar-row.secondary {
    align-items: center;
  }

  /* One seamless pill (method + url), like the reference — not two separate boxed
     controls sitting side by side. */
  .url-pill {
    flex: 1;
    display: flex;
    align-items: center;
    background: var(--color-panel-bg);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    /* No overflow:hidden — the design's radius is 0 here anyway (nothing to corner-clip), and
       hiding overflow would clip the missing-variable popover that hangs below the URL bar. */
  }

  .url-pill:focus-within {
    border-color: var(--color-focus);
  }

  .url-pill-divider {
    width: 1px;
    align-self: stretch;
    background: var(--color-border);
    margin: 0.4rem 0;
  }

  .url-pill .method-select {
    border: none;
    background: transparent;
    border-radius: 0;
  }

  .url-pill .url-input {
    border: none;
    background: transparent;
    border-radius: 0;
  }

  .url-pill .url-input:focus {
    outline: none;
  }

  .url-input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 0.82rem;
  }

  .send-action {
    display: flex;
    gap: 0.4rem;
  }

  .url-preview-bar {
    padding: 0.3rem 0.9rem;
    font-size: 0.76rem;
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .preview-label {
    color: var(--color-text-tertiary);
  }

  code {
    font-family: var(--font-mono);
    background: var(--color-bg-hover);
    padding: 0.05rem 0.3rem;
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
  }

  /* ---------- Editor tab strip ---------- */
  .editor-tabs {
    display: flex;
    gap: 1rem;
    padding: 0 0.9rem;
    border-bottom: 1px solid var(--color-border);
    overflow-x: auto;
  }

  .editor-tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    padding: 0.55rem 0.1rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 0.3rem;
    white-space: nowrap;
  }

  .editor-tab:hover {
    background: none;
    color: var(--color-text);
  }

  .editor-tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
    font-weight: 700;
  }

  .tab-badge {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
    font-size: 0.65rem;
    font-weight: 700;
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
  }

  .tab-badge-warn {
    background: var(--color-warn-bg);
    color: var(--color-warn);
    font-size: 0.65rem;
    font-weight: 700;
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
  }

  .tab-dot {
    color: var(--color-primary);
  }

  .tab-content {
    padding: 0.8rem 0.9rem;
  }

  .editor-body-row {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .editor-main-col {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }

  .bottom-bar {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex: none;
    padding: 0.3rem 0.6rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .bottom-bar-tab {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .bottom-bar-tab:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .bottom-bar-tab.active {
    background: var(--color-bg-hover);
    color: var(--color-accent);
  }

  /* Its own scroll area, generously tall (not the old 280px-wide sidebar sliver) — a code
     snippet or the info list gets real room to breathe when opened. */
  .bottom-panel {
    flex: none;
    height: 45vh;
    min-height: 260px;
    overflow-y: auto;
    padding: var(--space-4) var(--space-6);
    border-top: 2px solid var(--color-border-strong);
    background: var(--color-panel-bg);
  }

  .bottom-panel-code {
    max-height: none;
  }

  .info-list-grid {
    grid-template-columns: max-content 1fr;
    column-gap: var(--space-4);
    max-width: 480px;
  }

  .right-panel-title {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-tertiary);
    margin: 0 0 0.6rem;
    font-weight: 700;
  }

  .info-list {
    display: grid;
    grid-template-columns: auto;
    row-gap: 0.5rem;
    margin: 0;
    font-size: 0.78rem;
  }

  .info-list dt {
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    font-size: 0.68rem;
    letter-spacing: 0.03em;
    margin-bottom: 0.1rem;
  }

  .info-list dd {
    color: var(--color-text);
    margin: 0 0 0.4rem;
    word-break: break-all;
  }

  /* Defensive default: any bare heading dropped into the detail/response area (e.g. new
     feature sections not yet given their own class) inherits the same muted label style
     as the rest of the system instead of a jarring browser-default heading. */
  .detail h2,
  .detail h3 {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-tertiary);
    margin: 1rem 0.9rem 0.4rem;
    font-weight: 700;
  }


  /* ---------- Params/headers tables ---------- */
  .params-table {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .params-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .params-row-spacer {
    display: inline-block;
    width: 14px;
    flex: none;
  }

  .params-row input:not([type="checkbox"]) {
    flex: 1;
    min-width: 80px;
  }

  .radio-label,
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.8rem;
    white-space: nowrap;
  }

  .radio-row {
    display: flex;
    gap: 1rem;
  }

  .body-input {
    width: 100%;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    resize: vertical;
  }

  .body-mode-bar {
    display: flex;
    align-items: center;
    gap: 0.9rem;
  }

  .graphql-editor h4,
  .params-table h4 {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-tertiary);
    margin: 0.3rem 0 0.1rem;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(200px, 1fr));
    gap: 0.7rem;
  }

  .settings-row {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.8rem;
  }

  .settings-row span {
    color: var(--color-text-secondary);
    font-size: 0.75rem;
  }

  .var-key {
    font-weight: 600;
    min-width: 100px;
  }

  .var-val {
    flex: 1;
    color: var(--color-text-secondary);
    font-family: var(--font-mono);
    font-size: 0.76rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badge {
    font-size: 0.65rem;
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
  }

  .badge-local {
    background: color-mix(in srgb, var(--method-put) 16%, transparent);
    color: var(--method-put);
  }

  /* ---------- Response panel ---------- */
  .response {
    border-top: 1px solid var(--color-border);
    padding: 0.7rem 0.9rem 1rem;
  }

  .response-loading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 0.9rem;
    color: var(--color-text-secondary);
    font-size: 0.82rem;
    border-top: 1px solid var(--color-border);
  }

  .response-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 2rem 0.9rem;
    color: var(--color-text-tertiary);
    border-top: 1px solid var(--color-border);
  }

  .spinner {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: 2px solid var(--color-border-strong);
    border-top-color: var(--color-accent);
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .response-stat-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-bottom: 0.6rem;
  }

  .response-stat-status {
    font-weight: 700;
    font-size: 0.85rem;
  }

  .response-stat-item {
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .response-stat-label {
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    font-size: 0.66rem;
    letter-spacing: 0.03em;
    margin-right: 0.2rem;
  }

  .response-stat-spacer {
    flex: 1;
  }

  .status-ok { color: var(--color-success); }
  .status-err { color: var(--color-danger); }

  .response-subtabs {
    display: flex;
    align-items: center;
    gap: 0.1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .response-subtab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 0.4rem 0.7rem;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .response-subtab:hover {
    color: var(--color-text);
  }

  .response-subtab.active {
    color: var(--color-text);
    border-bottom-color: var(--color-primary);
    font-weight: 600;
  }

  .response-subtab-content {
    padding-top: 0.6rem;
  }

  .response-format-toggle {
    display: flex;
    gap: 2px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-sm);
    padding: 2px;
    margin-left: auto;
  }

  .btn-toggle {
    border: none;
    background: none;
    padding: 0.2rem 0.6rem;
    font-size: 0.72rem;
    border-radius: var(--radius-sm);
  }

  .btn-toggle.active {
    background: var(--color-bg);
    box-shadow: var(--shadow-sm);
    font-weight: 600;
  }

  .test-results-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .test-result-row {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    font-size: 0.8rem;
    padding: 0.25rem 0;
  }

  .test-result-icon {
    font-weight: 700;
    width: 1rem;
    flex-shrink: 0;
  }

  .test-pass .test-result-icon { color: var(--color-success); }
  .test-fail .test-result-icon { color: var(--color-danger); }

  .test-result-name {
    color: var(--color-text);
  }

  .test-result-error {
    color: var(--color-danger);
    font-family: var(--font-mono);
    font-size: 0.74rem;
  }

  .headers-list {
    padding: 0.3rem 0 0.3rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .header-line {
    font-size: 0.76rem;
    font-family: var(--font-mono);
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .body-view {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.7rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    max-height: 420px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }

  .requests {
    list-style: none;
    margin: 0.3rem 0;
    padding: 0;
  }

  .row-item {
    display: flex;
    align-items: center;
  }

  .link {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: none;
    text-align: left;
    padding: 0.25rem 0.1rem;
    color: var(--color-text);
    font-size: 0.78rem;
  }

  .link:hover {
    background: var(--color-bg-hover);
  }

  .name { font-weight: 500; }
  .url { color: var(--color-text-tertiary); }

  /* ---------- Console drawer ---------- */
  .console-drawer {
    height: 260px;
    flex-shrink: 0;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    display: flex;
    flex-direction: column;
  }

  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.4rem 0.7rem;
    border-bottom: 1px solid var(--color-border);
    flex-wrap: wrap;
  }

  .console-title-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .console-title {
    font-weight: 700;
    font-size: 0.8rem;
  }

  .console-count-badge {
    font-size: 0.7rem;
    color: var(--color-text-tertiary);
  }

  .console-toolbar {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .console-select,
  .console-search {
    font-size: 0.72rem;
    padding: 0.25rem 0.4rem;
  }

  .console-check-label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .console-btn,
  .console-close-btn {
    font-size: 0.7rem;
    padding: 0.25rem 0.5rem;
  }

  .console-body {
    flex: 1;
    overflow-y: auto;
    font-family: var(--font-mono);
  }

  .console-empty {
    padding: 1rem;
    color: var(--color-text-tertiary);
    font-size: 0.78rem;
    font-family: var(--font-sans);
  }

  .console-events-list {
    display: flex;
    flex-direction: column;
  }

  .console-row {
    border-bottom: 1px solid var(--color-border);
  }

  .console-row.error-row { background: var(--color-danger-bg); }
  .console-row.warn-row { background: var(--color-warn-bg); }

  .console-row-summary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.7rem;
    font-size: 0.74rem;
    cursor: pointer;
  }

  .evt-expander {
    width: 0.9rem;
    color: var(--color-text-tertiary);
  }

  .evt-time {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .evt-level {
    font-weight: 700;
    padding: 0 0.3rem;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .level-info { color: var(--method-put); }
  .level-warn { color: var(--color-warn); }
  .level-error { color: var(--color-danger); }
  .level-debug { color: var(--color-text-tertiary); }

  .evt-type {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .evt-cid {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .evt-msg {
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .console-row-details {
    padding: 0.4rem 0.7rem 0.7rem 2.1rem;
  }

  .details-actions {
    margin-bottom: 0.3rem;
  }

  .console-mini-btn {
    font-size: 0.68rem;
    padding: 0.15rem 0.4rem;
  }

  .console-json-view {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.5rem;
    font-size: 0.72rem;
    max-height: 200px;
    overflow: auto;
    margin: 0;
  }

  /* ---------- Footer status bar ---------- */
  .app-status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0.9rem;
    background: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    font-size: 0.72rem;
  }

  .console-toggle-btn {
    background: none;
    border: none;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.15rem 0.4rem;
    color: var(--color-text-secondary);
    font-size: 0.72rem;
  }

  .console-toggle-btn.active {
    color: var(--color-primary);
    font-weight: 600;
  }

  .status-badge-error {
    background: var(--color-danger);
    color: #fff;
    border-radius: 999px;
    padding: 0 0.35rem;
    font-size: 0.65rem;
  }

  .status-badge-warn {
    background: var(--color-warn);
    color: #fff;
    border-radius: 999px;
    padding: 0 0.35rem;
    font-size: 0.65rem;
  }

  .status-right {
    display: flex;
    gap: 1rem;
    color: var(--color-text-tertiary);
  }

  /* ---------- Modals ---------- */
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(20, 20, 25, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 2rem;
  }

  .modal {
    background: var(--color-bg);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
    padding: 1rem 1.2rem 1.2rem;
    width: 520px;
    max-width: 100%;
    max-height: 85vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .modal-wide {
    width: 720px;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.2rem;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 0.95rem;
  }

  .file-dropzone {
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.6rem;
    text-align: center;
  }

  .file-label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    cursor: pointer;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .import-report-card {
    background: var(--color-success-bg);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.8rem;
    font-size: 0.8rem;
  }

  .import-report-card h4 {
    margin: 0 0 0.3rem;
  }

  .warnings-box {
    margin-top: 0.4rem;
    font-size: 0.75rem;
  }

  .warnings-box h5 {
    margin: 0.2rem 0;
  }

  .request-form {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
    align-items: center;
  }

  /* ---------- Modal system (backdrop / container / tabs / body / footer) ---------- */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(20, 20, 25, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 2rem;
  }

  .modal-container {
    background: var(--color-bg);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
    padding: 1rem 1.2rem 1.2rem;
    width: 560px;
    max-width: 100%;
    max-height: 85vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .modal-container.modal-wide {
    width: 760px;
  }

  .modal-title-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .modal-sub {
    font-size: 0.72rem;
    color: var(--color-text-tertiary);
    font-weight: 400;
  }

  .modal-close-btn {
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    font-size: 0.85rem;
    padding: 0.15rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .modal-close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-danger);
  }

  .modal-tabs {
    display: flex;
    gap: 1rem;
    border-bottom: 1px solid var(--color-border);
    margin: 0.2rem 0 0.4rem;
    overflow-x: auto;
  }

  .modal-tab-btn {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    padding: 0.5rem 0.1rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 0.3rem;
    white-space: nowrap;
  }

  .modal-tab-btn:hover {
    background: none;
    color: var(--color-text);
  }

  .modal-tab-btn.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
    font-weight: 700;
  }

  .tab-badge-alert {
    background: var(--color-danger);
    color: #fff;
    font-size: 0.65rem;
    font-weight: 700;
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.4rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--color-border);
  }

  .action-alert {
    padding: 0.4rem 0.7rem;
    border-radius: var(--radius-md);
    font-size: 0.78rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
  }

  .action-alert.success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .action-alert.error {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .action-alert.warning {
    background: var(--color-warn-bg);
    color: var(--color-warn);
  }

  .action-alert .btn-xs {
    flex-shrink: 0;
  }

  /* ---------- Git status pill (footer) ---------- */
  .git-status-pill {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 0.1rem 0.6rem;
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .git-status-pill:hover {
    background: var(--color-bg);
  }

  .git-status-pill.git-has-conflict {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .git-status-pill.git-kind-ahead,
  .git-status-pill.git-kind-behind,
  .git-status-pill.git-kind-modified {
    border-color: var(--color-warn);
    color: var(--color-warn);
  }

  .git-status-pill.git-kind-clean {
    border-color: var(--color-success);
    color: var(--color-success);
  }

  .git-icon {
    font-weight: 700;
  }

  .git-alert {
    color: var(--color-danger);
    font-weight: 600;
  }

  .git-ahead,
  .git-behind {
    font-weight: 600;
  }

  /* ---------- Git panel content ---------- */
  .git-panel-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .git-panel-section h4 {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
  }

  .git-panel-section h5 {
    margin: 0.2rem 0;
    font-size: 0.78rem;
  }

  .form-row-stacked {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .form-row-stacked label {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .input-with-actions {
    display: flex;
    gap: 0.4rem;
  }

  .path-input {
    flex: 1;
    font-family: var(--font-mono);
  }

  .alert-box-warning {
    background: var(--color-warn-bg);
    color: var(--color-warn);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.8rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.8rem;
  }

  .git-status-card {
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.8rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .status-summary-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    flex-wrap: wrap;
  }

  .status-label {
    color: var(--color-text-tertiary);
    font-size: 0.75rem;
  }

  .status-sep {
    color: var(--color-border);
  }

  .git-badge-kind {
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.05rem 0.4rem;
    border-radius: var(--radius-sm);
    background: var(--color-bg-hover);
  }

  .git-badge-kind.kind-clean { color: var(--color-success); }
  .git-badge-kind.kind-ahead,
  .git-badge-kind.kind-behind,
  .git-badge-kind.kind-modified { color: var(--color-warn); }
  .git-badge-kind.kind-conflict { color: var(--color-danger); }

  .badge-ahead,
  .badge-behind {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-warn);
    background: var(--color-warn-bg);
    border-radius: var(--radius-sm);
    padding: 0.05rem 0.4rem;
  }

  .files-changed-summary {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .file-category {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .working-tree-clean {
    margin: 0;
    color: var(--color-success);
    font-size: 0.8rem;
  }

  .git-commit-box {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .commit-input-row {
    display: flex;
    gap: 0.4rem;
  }

  .commit-input-row input {
    flex: 1;
  }

  .quick-git-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .icon-btn-text {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    padding: 0.2rem 0.3rem;
  }

  .icon-btn-text:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .auto-sync-box {
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .clean-box {
    background: var(--color-success-bg);
    color: var(--color-success);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
  }

  .clean-box p { margin: 0; }

  .conflict-alert-box {
    background: var(--color-danger-bg);
    color: var(--color-danger);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
    font-size: 0.8rem;
  }

  .conflict-alert-box p { margin: 0; }

  .conflicts-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .conflict-item-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
  }

  .conflict-item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .conflict-filename {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 600;
  }

  .conflict-choices {
    display: flex;
    gap: 0.4rem;
  }

  .btn-choice {
    font-size: 0.72rem;
  }

  .btn-choice.local {
    border-color: var(--method-put);
    color: var(--method-put);
  }

  .btn-choice.remote {
    border-color: var(--method-get);
    color: var(--method-get);
  }

  .github-profile-card {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
  }

  .github-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
  }

  .github-profile-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 0.05rem;
  }

  .badge-success {
    background: var(--color-success-bg);
    color: var(--color-success);
    font-size: 0.68rem;
    font-weight: 600;
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
  }

  .repo-permissions-card {
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.7rem;
  }

  .perm-badges {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin: 0.3rem 0;
  }

  .perm-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .perm-badge.perm-granted {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .projectfile-options {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }

  .json-preview-box {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .json-preview-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .toolbar-actions {
    display: flex;
    gap: 0.4rem;
  }

  .code-area,
  .diff-viewer {
    width: 100%;
    font-family: var(--font-mono);
    font-size: 0.76rem;
  }

  .diff-viewer {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.7rem;
    max-height: 420px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 420px;
    overflow-y: auto;
  }

  .history-item {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.4rem 0.6rem;
  }

  .commit-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
  }

  .commit-hash {
    color: var(--color-primary);
    font-weight: 700;
  }

  .commit-author {
    color: var(--color-text-secondary);
  }

  .commit-date {
    color: var(--color-text-tertiary);
  }

  .commit-msg {
    margin: 0.2rem 0 0;
    font-size: 0.78rem;
  }

  /* ---------- Phase 08: AI & Source Intelligence Styles ---------- */
  .field-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.5rem;
    margin-bottom: 0.2rem;
  }

  .field-header-row h4,
  .field-header-row h3 {
    margin: 0;
  }

  .field-header-row .hint {
    margin: 0;
  }

  /* — Scripts tab: Pre/Post side selector instead of two long-labeled stacked textareas. — */
  .scripts-layout {
    display: flex;
    gap: var(--space-4);
    min-height: 0;
  }

  .scripts-side {
    flex: none;
    width: 100px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-right: 1px solid var(--color-border-strong);
    padding-right: var(--space-3);
  }

  .scripts-side-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    padding: var(--space-2) var(--space-2);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    cursor: pointer;
    text-align: left;
  }

  .scripts-side-item:hover {
    background: var(--color-bg-hover);
  }

  .scripts-side-item.active {
    color: var(--color-text);
    border-left-color: var(--color-accent);
    background: var(--color-bg-hover);
  }

  .scripts-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .scripts-textarea {
    flex: 1;
    min-height: 280px;
  }

  .action-feedback-inline {
    font-size: 0.75rem;
    color: var(--color-primary);
    margin: 0.2rem 0;
    font-weight: 500;
  }

  .text-success {
    color: var(--color-success, #22c55e) !important;
  }

  .sample-responses-section {
    border-top: 1px solid var(--color-border);
    padding: 0.8rem 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .sample-responses-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .sample-response-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-subtle, rgba(255, 255, 255, 0.02));
    overflow: hidden;
  }

  .sample-response-summary {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.4rem 0.6rem;
    cursor: pointer;
    user-select: none;
    font-size: 0.8rem;
  }

  .sample-name {
    font-weight: 600;
    flex: 1;
  }

  .badge-sample {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
  }

  .badge-framework {
    background: color-mix(in srgb, #3b82f6 20%, transparent);
    color: #3b82f6;
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
  }

  .badge-openapi {
    background: color-mix(in srgb, #10b981 20%, transparent);
    color: #10b981;
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
  }

  .badge-auth {
    background: color-mix(in srgb, #f59e0b 20%, transparent);
    color: #f59e0b;
    font-size: 0.68rem;
    padding: 0.1rem 0.35rem;
    border-radius: 999px;
  }

  .btn-delete-icon {
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    padding: 0.1rem 0.3rem;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
  }

  .btn-delete-icon:hover {
    color: var(--color-danger);
    background: var(--color-bg-hover);
  }

  .ai-context-options {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    background: var(--color-bg-subtle, rgba(255, 255, 255, 0.02));
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.8rem;
  }

  .ai-prompt-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .ai-preview-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.8rem;
    background: var(--color-bg-subtle, rgba(255, 255, 255, 0.03));
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .preview-title-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .preview-name {
    font-weight: 700;
    font-size: 0.9rem;
  }

  .preview-url {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--color-text-secondary);
  }

  .preview-desc {
    margin: 0;
  }

  .preview-meta-row {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .preview-body-pre {
    max-height: 180px;
    overflow: auto;
    font-size: 0.75rem;
    padding: 0.5rem;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .source-scan-bar {
    display: flex;
    gap: 0.5rem;
  }

  .source-summary-panel {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .source-badges-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .source-filter-row input {
    width: 100%;
  }

  .discovered-endpoints-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 380px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.4rem;
  }

  .discovered-endpoint-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
  }

  .discovered-endpoint-card:hover {
    background: var(--color-bg-hover);
  }

  .ep-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .ep-path {
    font-family: monospace;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .ep-meta {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    font-size: 0.72rem;
    color: var(--color-text-tertiary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ep-file {
    font-family: monospace;
  }

  .ep-summary {
    color: var(--color-text-secondary);
  }

  .btn-xs-primary {
    padding: 0.15rem 0.45rem;
    font-size: 0.72rem;
    font-weight: 600;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-xs-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .ai-settings-grid {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .settings-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .password-input-row {
    display: flex;
    gap: 0.4rem;
  }

  .password-input-row input {
    flex: 1;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--color-primary);
    cursor: pointer;
    text-decoration: underline;
    padding: 0;
    font: inherit;
  }
</style>
