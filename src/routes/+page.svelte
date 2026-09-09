<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    describeError,
    type Auth,
    type CollectionImportReport,
    type Environment,
    type EnvironmentImportReport,
    type GeneratedApiDefinition,
    type HeaderEntry,
    type Project,
    type QueryParam,
    type RequestFull,
    type RequestDiagnostics,
    type RequestSettings,
    type RequestSummary,
    type ResolvedTemplate,
    type ResponseMeta,
    type ResponseSummary,
    type SnippetMode,
    type SnippetTarget,
    type VariableScope,
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
  } from "$lib/api";

  let projects = $state<Project[]>([]);
  let selectedProjectId = $state<string | null>(null);

  let requests = $state<RequestSummary[]>([]);
  let selectedRequest = $state<RequestFull | null>(null);

  let environments = $state<Environment[]>([]);
  let selectedEnvironmentId = $state<string | null>(null);
  let newEnvironmentName = $state("");
  let urlPreview = $state<ResolvedTemplate | null>(null);

  let showVariables = $state(false);
  let projectVariables = $state<VariableView[]>([]);
  let environmentVariables = $state<VariableView[]>([]);
  let newVarKey = $state("");
  let newVarValue = $state("");
  let newVarIsSecret = $state(false);
  let newVarIsLocal = $state(false);
  let newVarScope = $state<VariableScope>("global");
  let revealedSecrets = $state<Record<string, string>>({});
  let requestDiagnostics = $state<RequestDiagnostics | null>(null);
  let copyFeedback = $state("");

  let newProjectName = $state("");
  let newRequestName = $state("");
  let newRequestMethod = $state("GET");
  let newRequestUrl = $state("");

  let renamingProjectId = $state<string | null>(null);
  let renameProjectValue = $state("");

  // Editor tab selection (LP-0406)
  let activeEditorTab = $state<"params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "code">("params");

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
  let editBodyType = $state<"raw" | "form-data" | "x-www-form-urlencoded" | "graphql">("raw");
  let editGraphqlQuery = $state("");
  let editGraphqlVariables = $state("");

  // cURL importer (LP-0608, LP-0609)
  let showCurlImport = $state(false);
  let curlImportText = $state("");
  let curlImportName = $state("");
  let curlImportError = $state("");

  // Postman compatibility import/export (LP-0501 - LP-0507, LP-0212)
  let showCollectionImport = $state(false);
  let collectionImportText = $state("");
  let collectionImportTarget = $state<"new" | "current">("new");
  let collectionImportLoading = $state(false);
  let collectionImportError = $state("");
  let collectionImportReport = $state<CollectionImportReport | null>(null);

  let showEnvironmentImport = $state(false);
  let environmentImportText = $state("");
  let environmentImportLoading = $state(false);
  let environmentImportError = $state("");
  let environmentImportReport = $state<EnvironmentImportReport | null>(null);

  let exportFeedback = $state("");

  let snippetMode = $state<SnippetMode>("placeholder");
  let snippetTarget = $state<SnippetTarget>("bash");
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

  let aiConfigured = $state(false);
  let showAiPanel = $state(false);
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
  let sampleResponses = $state<SampleResponse[]>([]);
  let generatingTestsDocs = $state(false);
  let testsDocsFeedback = $state("");

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
    editBodyType: "raw" | "form-data" | "x-www-form-urlencoded" | "graphql";
    editGraphqlQuery: string;
    editGraphqlVariables: string;
    activeEditorTab: "params" | "headers" | "auth" | "body" | "scripts" | "settings" | "docs" | "code";
    activeResponse: ResponseMeta | null;
    activeResponseBody: string;
    activeResponseTruncated: boolean;
  }

  let openTabs = $state<RequestTab[]>([]);
  const tabDrafts = new Map<string, RequestDraft>();

  // Request search & windowing (LP-0409, LP-0410)
  let requestSearchQuery = $state("");
  let filteredRequests = $derived.by(() => {
    const q = requestSearchQuery.trim().toLowerCase();
    if (!q) return requests;
    return requests.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.method.toLowerCase().includes(q) ||
        r.url.toLowerCase().includes(q),
    );
  });
  let requestPageSize = $state(50);
  let requestPage = $state(0);
  let visibleRequests = $derived.by(() => {
    if (filteredRequests.length <= 100) return filteredRequests;
    const start = requestPage * requestPageSize;
    return filteredRequests.slice(start, start + requestPageSize);
  });
  let totalRequestPages = $derived(Math.ceil(filteredRequests.length / requestPageSize));

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
  let showGitModal = $state(false);
  let showDiffModal = $state(false);
  let showHistoryModal = $state(false);
  let showProjectFileModal = $state(false);
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

  $effect(() => {
    if (selectedRequest) {
      if (!tabDrafts.has(selectedRequest.id)) {
        editName = selectedRequest.name;
        editMethod = selectedRequest.method;
        editUrl = selectedRequest.url;
        editHeaders = selectedRequest.headers.map((h) => ({ ...h }));
        editQueryParams = selectedRequest.query_params.map((p) => ({ ...p }));
        editBody = selectedRequest.body ?? "";
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
      }
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
  $effect(() => {
    const projectId = selectedProjectId;
    const requestId = selectedRequest?.id ?? null;
    const template = editUrl;
    if (!projectId || !template) {
      urlPreview = null;
      return;
    }
    api
      .resolvePreview(projectId, selectedEnvironmentId, requestId, template)
      .then((result) => (urlPreview = result))
      .catch(() => (urlPreview = null));
  });

  onMount(() => {
    loadProjects();
    loadAiSettings();
    api.isAiConfigured().then((configured) => (aiConfigured = configured));
    refreshConsoleEvents();
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
    activeEditorTab = draft.activeEditorTab;
    activeResponse = draft.activeResponse;
    activeResponseBody = draft.activeResponseBody;
    activeResponseTruncated = draft.activeResponseTruncated;
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
      sampleResponses = await api.listSampleResponses(requestId);
    } catch (err) {
      console.error("Failed to load sample responses", err);
    }
  }

  async function deleteSampleResponseAction(id: string) {
    if (!selectedRequest) return;
    try {
      await api.deleteSampleResponse(id);
      await loadSampleResponses(selectedRequest.id);
    } catch (err) {
      errorMessage = describeError(err);
    }
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
        { id: request.id, project_id: request.project_id, name: request.name, method: request.method, url: request.url, updated_at: request.updated_at },
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

  async function createProject(event: Event) {
    event.preventDefault();
    if (!newProjectName.trim()) return;
    try {
      const project = await api.createProject(newProjectName.trim());
      newProjectName = "";
      projects = [project, ...projects];
      await selectProject(project.id);
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
      environments = await api.listEnvironments(id);
      await loadVariables();
      await loadGitSettings(id);
      await loadSourceAssociation(id);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      loadingRequests = false;
    }
  }

  async function createEnvironment(event: Event) {
    event.preventDefault();
    if (!selectedProjectId || !newEnvironmentName.trim()) return;
    try {
      const env = await api.createEnvironment(selectedProjectId, newEnvironmentName.trim());
      newEnvironmentName = "";
      environments = [...environments, env];
      selectedEnvironmentId = env.id;
      await loadVariables();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function createRequest(event: Event) {
    event.preventDefault();
    if (!selectedProjectId || !newRequestName.trim() || !newRequestUrl.trim()) return;
    try {
      const request = await api.createRequest({
        project_id: selectedProjectId,
        name: newRequestName.trim(),
        method: newRequestMethod,
        url: newRequestUrl.trim(),
        headers: [],
        query_params: [],
        auth: { type: "none" },
        body: null,
      });
      newRequestName = "";
      newRequestUrl = "";
      requests = [
        {
          id: request.id,
          project_id: request.project_id,
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
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = [];
    } catch (err) {
      errorMessage = describeError(err);
    }
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
    const requestId = selectedRequest.id;
    sending = true;
    try {
      const meta = await api.sendRequest(requestId, selectedEnvironmentId);
      activeResponse = meta;
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

  // Only sends fields that actually differ from the hydrated request —
  // the backend preserves anything omitted, but there's no reason to send it either.
  async function saveRequest() {
    if (!selectedRequest) return;
    const original = selectedRequest;
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
        ...(JSON.stringify(editQueryParams) !== JSON.stringify(original.query_params)
          ? { query_params: editQueryParams }
          : {}),
        ...(JSON.stringify(editHeaders) !== JSON.stringify(original.headers)
          ? { headers: editHeaders }
          : {}),
        ...(JSON.stringify(auth) !== JSON.stringify(original.auth) ? { auth } : {}),
        ...(() => {
          if (editBody !== (original.body ?? "")) {
            return editBody.trim().length === 0
              ? { clear_body: true }
              : { body: editBody };
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
          ? { id: updated.id, project_id: updated.project_id, name: updated.name, method: updated.method, url: updated.url, updated_at: updated.updated_at }
          : r,
      );
    } catch (err) {
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
      showCurlImport = false;
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

  function addHeader() {
    editHeaders = [...editHeaders, { key: "", value: "", enabled: true, description: "" }];
  }

  function removeHeader(index: number) {
    editHeaders = editHeaders.filter((_, i) => i !== index);
  }

  function addQueryParam() {
    editQueryParams = [...editQueryParams, { key: "", value: "", enabled: true }];
  }

  function removeQueryParam(index: number) {
    editQueryParams = editQueryParams.filter((_, i) => i !== index);
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

  async function createVariable(event: Event) {
    event.preventDefault();
    if (!selectedProjectId || !newVarKey.trim()) return;
    try {
      if (newVarScope === "global") {
        await api.createVariable({
          scope: "global",
          project_id: selectedProjectId,
          key: newVarKey.trim(),
          value: newVarValue,
          is_secret: newVarIsSecret,
          is_local: newVarIsLocal,
          enabled: true,
        });
      } else if (newVarScope === "environment" && selectedEnvironmentId) {
        await api.createVariable({
          scope: "environment",
          environment_id: selectedEnvironmentId,
          key: newVarKey.trim(),
          value: newVarValue,
          is_secret: newVarIsSecret,
          is_local: newVarIsLocal,
          enabled: true,
        });
      }
      newVarKey = "";
      newVarValue = "";
      newVarIsSecret = false;
      newVarIsLocal = false;
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
      showProjectFileModal = true;
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
      }, 60000);
    }
    return () => {
      if (autoSyncTimer) {
        clearInterval(autoSyncTimer);
        autoSyncTimer = null;
      }
    };
  });
</script>


<div class="app">
  <header class="topbar">
    <div class="topbar-left">
      <span class="brand">⚡ Light Postman</span>
    </div>
    <div class="topbar-center">
      {#if selectedProjectId}
        <div class="env-bar">
          <select bind:value={selectedEnvironmentId} onchange={loadVariables} class="env-select">
            <option value={null}>No Environment</option>
            {#each environments as env (env.id)}
              <option value={env.id}>{env.name}</option>
            {/each}
          </select>
          <form class="env-new-form" onsubmit={createEnvironment}>
            <input placeholder="+ new environment" bind:value={newEnvironmentName} class="env-new-input" />
          </form>
          <button
            type="button"
            class="icon-btn"
            title="Manage Variables"
            onclick={() => { showVariables = true; loadVariables(); }}
          >
            👁
          </button>
          {#if selectedEnvironmentId}
            <button
              type="button"
              class="icon-btn"
              title="Export selected environment as Postman Environment JSON"
              onclick={exportPostmanEnvironmentAction}
            >
              ⤓
            </button>
          {/if}
        </div>
      {/if}
    </div>
    <div class="topbar-right">
      {#if aiConfigured}
        <button type="button" class="btn-ghost" onclick={() => (showAiPanel = true)}>✨ Ask AI</button>
      {/if}
      <button type="button" class="btn-ghost" onclick={() => { showCollectionImport = true; collectionImportReport = null; collectionImportError = ""; }}>
        📥 Import Collection
      </button>
      <button type="button" class="btn-ghost" onclick={() => (showCurlImport = true)}>🔗 Import cURL</button>
    </div>
  </header>

  {#if exportFeedback}
    <div class="success-banner">{exportFeedback}</div>
  {/if}
  {#if errorMessage}
    <div class="error-banner">
      {errorMessage}
      <button type="button" class="dismiss-btn" onclick={() => (errorMessage = "")}>✕</button>
    </div>
  {/if}

  <div class="workspace">
    <aside class="sidebar">
      <div class="sidebar-header">
        <span class="sidebar-title">Projects</span>
      </div>
      <form class="new-project-form" onsubmit={createProject}>
        <input placeholder="New project name" bind:value={newProjectName} />
        <button type="submit" class="btn-icon-add" title="Create project">+</button>
      </form>

      <div class="project-list">
        {#each projects as project (project.id)}
          <div class="project-node">
            <div class="project-row" class:active={project.id === selectedProjectId}>
              {#if renamingProjectId === project.id}
                <form class="inline-form" onsubmit={submitRenameProject}>
                  <input bind:value={renameProjectValue} />
                  <button type="submit" title="Save">✓</button>
                  <button type="button" title="Cancel" onclick={() => (renamingProjectId = null)}>✕</button>
                </form>
              {:else}
                <button type="button" class="project-link" onclick={() => selectProject(project.id)}>
                  <span class="folder-icon">{project.id === selectedProjectId ? "📂" : "📁"}</span>
                  <span class="project-name">{project.name}</span>
                </button>
                <div class="project-row-actions">
                  <button class="icon-btn" title="Export Postman Collection v2.1" onclick={() => exportPostmanCollectionAction(project.id)}>⤓</button>
                  <button class="icon-btn" title="Rename" onclick={() => startRenameProject(project)}>✎</button>
                  <button class="icon-btn" title="Delete" onclick={() => deleteProject(project.id)}>🗑</button>
                </div>
              {/if}
            </div>

            {#if project.id === selectedProjectId}
              <div class="project-requests">
                <div class="request-search-box">
                  <input
                    type="search"
                    placeholder="Search requests..."
                    bind:value={requestSearchQuery}
                    class="request-search-input"
                  />
                  {#if requestSearchQuery}
                    <span class="request-count-badge">{filteredRequests.length}/{requests.length}</span>
                  {/if}
                </div>

                <form class="new-request-form" onsubmit={createRequest}>
                  <select bind:value={newRequestMethod} class="method-select-sm">
                    <option>GET</option>
                    <option>POST</option>
                    <option>PUT</option>
                    <option>PATCH</option>
                    <option>DELETE</option>
                    <option>HEAD</option>
                    <option>OPTIONS</option>
                  </select>
                  <input placeholder="Name" bind:value={newRequestName} />
                  <input placeholder="URL" bind:value={newRequestUrl} />
                  <button type="submit" class="btn-icon-add" title="Add request">+</button>
                </form>

                {#if loadingRequests}
                  <p class="hint">Loading…</p>
                {:else}
                  <ul class="request-list">
                    {#each visibleRequests as req (req.id)}
                      <li class="request-item" class:active={req.id === selectedRequest?.id}>
                        <button type="button" class="request-link" onclick={() => openRequest(req.id)}>
                          <span class="method-badge method-{req.method.toLowerCase()}">{req.method}</span>
                          <span class="request-name">{req.name}</span>
                        </button>
                        <button class="icon-btn icon-btn-ghost" title="Delete" onclick={() => deleteRequest(req.id)}>🗑</button>
                      </li>
                    {:else}
                      <li class="empty">{requestSearchQuery ? "No matching requests." : "No requests yet."}</li>
                    {/each}
                  </ul>

                  {#if totalRequestPages > 1}
                    <div class="request-pagination">
                      <button type="button" disabled={requestPage === 0} onclick={() => (requestPage = Math.max(0, requestPage - 1))}>◀</button>
                      <span>{requestPage + 1} / {totalRequestPages}</span>
                      <button type="button" disabled={requestPage >= totalRequestPages - 1} onclick={() => (requestPage = Math.min(totalRequestPages - 1, requestPage + 1))}>▶</button>
                    </div>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <p class="empty">No projects yet.</p>
        {/each}
      </div>
    </aside>

    <main class="main">
      {#if !selectedProjectId}
        <div class="empty-state">
          <div class="empty-icon">📁</div>
          <p>Select or create a project to get started.</p>
        </div>
      {:else if !selectedRequest}
        <div class="empty-state">
          <div class="empty-icon">📨</div>
          <p>Select or create a request to begin.</p>
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
                      <span class="dirty-dot" title="Unsaved changes">•</span>
                    {/if}
                  </button>
                  <button
                    type="button"
                    class="tab-close-btn"
                    title="Close tab"
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
            <span class="breadcrumb-current">{selectedRequest.name}</span>
          </div>

          <form class="request-bar" onsubmit={(e) => { e.preventDefault(); saveRequest(); }}>
            <div class="request-bar-row">
              <div class="url-pill">
                <select bind:value={editMethod} class="method-select method-{editMethod.toLowerCase()}">
                  <option>GET</option>
                  <option>POST</option>
                  <option>PUT</option>
                  <option>PATCH</option>
                  <option>DELETE</option>
                  <option>HEAD</option>
                  <option>OPTIONS</option>
                </select>
                <span class="url-pill-divider"></span>
                <input placeholder="Enter request URL" bind:value={editUrl} class="url-input" />
              </div>
              <div class="send-action">
                {#if sending}
                  <button type="button" class="btn-cancel" onclick={cancelCurrentSend}>Cancel</button>
                {:else}
                  <button type="button" class="btn-send" onclick={sendCurrentRequest}>Send</button>
                {/if}
              </div>
            </div>
            <div class="request-bar-row secondary">
              <input placeholder="Request name" bind:value={editName} class="name-input" />
              <button type="submit" class="btn-save" title="Save changes">💾 Save</button>
              <button type="button" class="icon-btn" title="Delete request" onclick={() => deleteRequest(selectedRequest!.id)}>🗑</button>
            </div>
          </form>

          {#if urlPreview}
            <div class="url-preview-bar">
              <span class="preview-label">Resolves to:</span> <code>{urlPreview.resolved}</code>
              {#if urlPreview.missing.length}
                <span class="warn-inline">(missing: {urlPreview.missing.join(", ")})</span>
              {/if}
            </div>
          {/if}

          {#if requestDiagnostics?.all_missing?.length}
            <div class="warn-banner">
              ⚠️ Unresolved variables: <strong>{requestDiagnostics.all_missing.join(", ")}</strong>
            </div>
          {/if}

          <div class="editor-tabs">
            <button type="button" class="editor-tab" class:active={activeEditorTab === "params"} onclick={() => (activeEditorTab = "params")}>
              Params
              {#if requestDiagnostics?.query_params_missing?.length}
                <span class="tab-badge-warn" title="Missing in Params: {requestDiagnostics.query_params_missing.join(', ')}">⚠ {requestDiagnostics.query_params_missing.length}</span>
              {:else if editQueryParams.length}
                <span class="tab-badge">{editQueryParams.length}</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "headers"} onclick={() => (activeEditorTab = "headers")}>
              Headers
              {#if requestDiagnostics?.headers_missing?.length}
                <span class="tab-badge-warn" title="Missing in Headers: {requestDiagnostics.headers_missing.join(', ')}">⚠ {requestDiagnostics.headers_missing.length}</span>
              {:else if editHeaders.length}
                <span class="tab-badge">{editHeaders.length}</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "auth"} onclick={() => (activeEditorTab = "auth")}>
              Auth
              {#if requestDiagnostics?.auth_missing?.length}
                <span class="tab-badge-warn" title="Missing in Auth: {requestDiagnostics.auth_missing.join(', ')}">⚠ {requestDiagnostics.auth_missing.length}</span>
              {:else if editAuthType !== "none"}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "body"} onclick={() => (activeEditorTab = "body")}>
              Body
              {#if requestDiagnostics?.body_missing?.length}
                <span class="tab-badge-warn" title="Missing in Body: {requestDiagnostics.body_missing.join(', ')}">⚠ {requestDiagnostics.body_missing.length}</span>
              {:else if editBody}
                <span class="tab-dot">•</span>
              {/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "scripts"} onclick={() => (activeEditorTab = "scripts")}>
              Scripts {#if editPreScript || editPostScript}<span class="tab-dot">•</span>{/if}
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "settings"} onclick={() => (activeEditorTab = "settings")}>
              Settings
            </button>
            <button type="button" class="editor-tab" class:active={activeEditorTab === "docs"} onclick={() => (activeEditorTab = "docs")}>
              Docs {#if editDescription}<span class="tab-dot">•</span>{/if}
            </button>
          </div>

          <div class="editor-body-row">
          <div class="editor-main-col">
          <div class="tab-content">
            {#if activeEditorTab === "params"}
              <div class="params-table">
                {#each editQueryParams as param, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={param.enabled} title="Enabled" />
                    <input placeholder="Key" bind:value={param.key} />
                    <input placeholder="Value" bind:value={param.value} />
                    <button type="button" class="icon-btn" title="Remove" onclick={() => removeQueryParam(i)}>🗑</button>
                  </div>
                {:else}
                  <p class="hint">No query parameters.</p>
                {/each}
                <div class="params-row">
                  <button type="button" onclick={addQueryParam}>Add param</button>
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save params</button>
                </div>
              </div>

            {:else if activeEditorTab === "headers"}
              <div class="params-table">
                {#each editHeaders as header, i (i)}
                  <div class="params-row">
                    <input type="checkbox" bind:checked={header.enabled} title="Enabled" />
                    <input placeholder="Key" bind:value={header.key} />
                    <input placeholder="Value" bind:value={header.value} />
                    <input placeholder="Description (optional)" bind:value={header.description} />
                    <button type="button" class="icon-btn" title="Remove" onclick={() => removeHeader(i)}>🗑</button>
                  </div>
                {:else}
                  <p class="hint">No headers.</p>
                {/each}
                <div class="params-row">
                  <button type="button" onclick={addHeader}>Add header</button>
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save headers</button>
                </div>
              </div>

            {:else if activeEditorTab === "auth"}
              <div class="params-table">
                <select bind:value={editAuthType}>
                  <option value="none">No Auth</option>
                  <option value="bearer">Bearer Token</option>
                  <option value="basic">Basic Auth</option>
                  <option value="api_key">API Key</option>
                </select>
                {#if editAuthType === "bearer"}
                  <div class="params-row">
                    <input placeholder="Token" bind:value={editAuthBearerToken} />
                  </div>
                {:else if editAuthType === "basic"}
                  <div class="params-row">
                    <input placeholder="Username" bind:value={editAuthBasicUsername} />
                    <input placeholder="Password" type="password" bind:value={editAuthBasicPassword} />
                  </div>
                {:else if editAuthType === "api_key"}
                  <div class="params-row">
                    <input placeholder="Key" bind:value={editAuthApiKeyKey} />
                    <input placeholder="Value" bind:value={editAuthApiKeyValue} />
                    <select bind:value={editAuthApiKeyLocation}>
                      <option value="header">Header</option>
                      <option value="query">Query Param</option>
                    </select>
                  </div>
                {/if}
                <div class="params-row">
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save auth</button>
                </div>
              </div>

            {:else if activeEditorTab === "body"}
              <div class="params-table">
                <div class="body-mode-bar">
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="raw" /> Raw
                  </label>
                  <label class="radio-label">
                    <input type="radio" bind:group={editBodyType} value="graphql" /> GraphQL
                  </label>
                  {#if editBodyType === "raw"}
                    <button type="button" onclick={() => { if (!editBody) editBody = "{\n  \n}"; }}>JSON template</button>
                    <button type="button" onclick={() => { editBody = ""; }}>Clear body</button>
                  {/if}
                </div>

                {#if editBodyType === "raw"}
                  <textarea
                    placeholder="Request body (JSON, text, XML, etc.)"
                    bind:value={editBody}
                    class="body-input"
                    rows="8"
                  ></textarea>
                {:else if editBodyType === "graphql"}
                  <div class="graphql-editor">
                    <h4>Query</h4>
                    <textarea
                      placeholder={"query MyQuery {\n  ...\n}"}
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
                      }}
                    ></textarea>
                    <h4>Variables (JSON)</h4>
                    <textarea
                      placeholder={"{\n  \"key\": \"value\"\n}"}
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
                      }}
                    ></textarea>
                  </div>
                {/if}

                <div class="params-row">
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save body</button>
                </div>
              </div>

            {:else if activeEditorTab === "scripts"}
              <div class="params-table">
                <h4>Pre-request Script (executed before sending)</h4>
                <textarea
                  placeholder="// e.g. pm.environment.set('timestamp', Date.now());"
                  bind:value={editPreScript}
                  class="body-input"
                  rows="5"
                ></textarea>

                <div class="field-header-row">
                  <h4>Post-request / Tests Script (executed after receiving response)</h4>
                  <button
                    type="button"
                    class="btn-ghost btn-xs"
                    disabled={generatingTestsDocs}
                    onclick={() => generateTestsAndDocsWithAiAction("tests")}
                    title="Generate pm.test assertions using Claude AI"
                  >
                    {generatingTestsDocs ? "Generating…" : "✨ Generate Tests with AI"}
                  </button>
                </div>
                {#if testsDocsFeedback}
                  <p class="action-feedback-inline">{testsDocsFeedback}</p>
                {/if}
                <textarea
                  placeholder="// e.g. pm.test('Status is 200', () => pm.response.to.have.status(200));"
                  bind:value={editPostScript}
                  class="body-input"
                  rows="5"
                ></textarea>

                <div class="params-row">
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save scripts</button>
                </div>
              </div>

            {:else if activeEditorTab === "settings"}
              <div class="params-table settings-grid">
                <label class="settings-row">
                  <span>Timeout (ms):</span>
                  <input
                    type="number"
                    placeholder="None (use default)"
                    value={editTimeoutMs ?? ""}
                    oninput={(e) => {
                      const val = (e.target as HTMLInputElement).value;
                      editTimeoutMs = val ? parseInt(val, 10) : null;
                    }}
                  />
                </label>
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={editFollowRedirects} />
                  Follow HTTP Redirects
                </label>
                <label class="settings-row">
                  <span>Max Redirects:</span>
                  <input type="number" bind:value={editMaxRedirects} min="0" max="50" />
                </label>
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={editVerifySsl} />
                  Verify SSL / TLS Certificates
                </label>
                <label class="settings-row">
                  <span>Proxy URL:</span>
                  <input placeholder="http://127.0.0.1:8080" bind:value={editProxyUrl} />
                </label>
                <label class="settings-row">
                  <span>HTTP Version:</span>
                  <select bind:value={editHttpVersion}>
                    <option value="">Default (HTTP/1.1 or HTTP/2)</option>
                    <option value="HTTP/1.1">HTTP/1.1</option>
                    <option value="HTTP/2">HTTP/2</option>
                  </select>
                </label>
                <div class="params-row">
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save settings</button>
                </div>
              </div>

            {:else if activeEditorTab === "docs"}
              <div class="params-table">
                <div class="field-header-row">
                  <h4>Documentation & Notes</h4>
                  <button
                    type="button"
                    class="btn-ghost btn-xs"
                    disabled={generatingTestsDocs}
                    onclick={() => generateTestsAndDocsWithAiAction("docs")}
                    title="Generate Markdown documentation using Claude AI"
                  >
                    {generatingTestsDocs ? "Generating…" : "✨ Generate Docs with AI"}
                  </button>
                </div>
                {#if testsDocsFeedback}
                  <p class="action-feedback-inline">{testsDocsFeedback}</p>
                {/if}
                <textarea
                  placeholder="Documentation, notes, or endpoint description (Markdown supported)..."
                  bind:value={editDescription}
                  class="body-input"
                  rows="8"
                ></textarea>
                <div class="params-row">
                  <button type="button" class="btn-primary" onclick={saveRequest}>Save docs</button>
                </div>
              </div>

            {/if}
          </div>

          {#if activeResponse}
            <div class="response">
              <div class="response-header-bar">
                <p class="response-status">
                  <strong class:status-ok={activeResponse.status < 400} class:status-err={activeResponse.status >= 400}>
                    {activeResponse.status} {activeResponse.status_text}
                  </strong>
                  · {activeResponse.duration_ms} ms · {activeResponse.body_size} bytes
                </p>
                <div class="response-format-toggle">
                  <button type="button" class="btn-toggle" class:active={responseViewMode === "pretty"} onclick={() => (responseViewMode = "pretty")}>Pretty</button>
                  <button type="button" class="btn-toggle" class:active={responseViewMode === "raw"} onclick={() => (responseViewMode = "raw")}>Raw</button>
                </div>
              </div>

              <div class="response-actions">
                <button type="button" onclick={copyResponseBody}>Copy response body</button>
                <button type="button" onclick={downloadResponseBody}>Download response</button>
                {#if copyFeedback}
                  <span class="hint">{copyFeedback}</span>
                {/if}
              </div>

              {#if activeResponse.headers && activeResponse.headers.length}
                <details class="response-headers-details">
                  <summary>Response Headers ({activeResponse.headers.length})</summary>
                  <div class="headers-list">
                    {#each activeResponse.headers as h}
                      <div class="header-line">
                        <strong>{h.key}:</strong> {h.value}
                      </div>
                    {/each}
                  </div>
                </details>
              {/if}

              {#if activeResponse.cookies && activeResponse.cookies.length}
                <details class="response-headers-details">
                  <summary>Response Cookies ({activeResponse.cookies.length})</summary>
                  <div class="headers-list">
                    {#each activeResponse.cookies as c}
                      <div class="header-line">
                        <strong>{c.name}:</strong> {c.value}
                        {#if c.domain}<span class="hint">domain: {c.domain}</span>{/if}
                        {#if c.path}<span class="hint">path: {c.path}</span>{/if}
                        {#if c.http_only}<span class="badge">HttpOnly</span>{/if}
                        {#if c.secure}<span class="badge">Secure</span>{/if}
                      </div>
                    {/each}
                  </div>
                </details>
              {/if}

              <pre class="body-view">{prettyResponseBody}</pre>
              {#if activeResponseTruncated}
                <p class="hint">(truncated — body is larger than the preview cap)</p>
              {/if}
            </div>
          {/if}

          <div class="sample-responses-section">
            <div class="field-header-row">
              <h3>Sample / Mock Responses ({sampleResponses.length})</h3>
              <button
                type="button"
                class="btn-ghost btn-xs"
                disabled={generatingSample}
                onclick={generateSampleResponseWithAiAction}
                title="Synthesize a realistic sample response using Claude AI"
              >
                {generatingSample ? "Generating…" : "✨ Generate Sample Response with AI"}
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
                      <span class="badge badge-sample">SAMPLE / MOCK</span>
                      <strong class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>
                        {sr.status}
                      </strong>
                      <span class="sample-name">{sr.name}</span>
                      <span class="hint">{new Date(sr.created_at).toLocaleTimeString()}</span>
                      <button
                        type="button"
                        class="btn-delete-icon"
                        onclick={(e) => { e.stopPropagation(); deleteSampleResponseAction(sr.id); }}
                        title="Delete sample response"
                      >✕</button>
                    </summary>
                    <pre class="body-view">{sr.body ?? ""}</pre>
                  </details>
                {/each}
              </div>
            {/if}
          </div>

          {#if responseHistory.length}
            <h2>History</h2>
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
          {/if}
        </section>
      {/if}
    </main>
  </div>

  {#if showConsole}
    <div class="console-drawer">
      <div class="console-header">
        <div class="console-title-group">
          <span class="console-title">Developer Console</span>
          <span class="console-count-badge">{filteredConsoleEvents.length} events</span>
        </div>

        <div class="console-toolbar">
          <select bind:value={consoleLevelFilter} class="console-select">
            <option value="all">All Levels</option>
            <option value="info">Info</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
            <option value="debug">Debug</option>
          </select>

          <label class="console-check-label" title="Show only events for current request">
            <input type="checkbox" bind:checked={consoleActiveRequestOnly} />
            Active request only
          </label>

          <input
            type="search"
            placeholder="Filter console..."
            bind:value={consoleSearchFilter}
            class="console-search"
          />

          <button type="button" class="console-btn" onclick={refreshConsoleEvents} title="Refresh events">↻ Refresh</button>
          <button type="button" class="console-btn" onclick={clearConsole} title="Clear event buffer">Clear</button>
          <button type="button" class="console-btn" onclick={copyConsoleLog} title="Copy all visible logs to clipboard">Copy</button>
          <button type="button" class="console-btn" onclick={exportConsoleJson} title="Export events as JSON">Export JSON</button>
          <button type="button" class="console-close-btn" onclick={() => (showConsole = false)} title="Close console">✕</button>
        </div>
      </div>

      <div class="console-body">
        {#if filteredConsoleEvents.length === 0}
          <div class="console-empty">No console events logged yet. Send a request to see lifecycle diagnostics.</div>
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
                  <span class="evt-cid" title="Correlation ID: {evt.correlation_id}">#{evt.correlation_id.slice(0, 8)}</span>
                  <span class="evt-msg">{evt.message}</span>
                </div>

                {#if expandedEventIds.has(evt.id) && evt.details}
                  <div class="console-row-details">
                    <div class="details-actions">
                      <button type="button" class="console-mini-btn" onclick={() => copyEventDetails(evt)}>Copy Details JSON</button>
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
        <span>Developer Console</span>
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
          title="Git & Remote Sync (Click to open)"
          onclick={() => {
            showGitModal = true;
            if (gitRepoPathInput) refreshGitStatus();
          }}
        >
          <span class="git-icon">⎇</span>
          {#if !gitSettings?.repo_path}
            <span>Git: Not Configured</span>
          {:else if gitStatusLoading}
            <span>Git: Checking…</span>
          {:else if gitStatus?.has_conflicts}
            <span class="git-alert">⚠ Conflict ({gitStatus.conflict_files.length})</span>
          {:else if gitStatus}
            <span>{gitStatus.branch}: {gitStatus.status_kind}</span>
            {#if gitStatus.ahead > 0}<span class="git-ahead">↑{gitStatus.ahead}</span>{/if}
            {#if gitStatus.behind > 0}<span class="git-behind">↓{gitStatus.behind}</span>{/if}
          {:else}
            <span>Git: {gitBranchInput}</span>
          {/if}
        </button>
        <span class="status-info">Project: {projects.find((p) => p.id === selectedProjectId)?.name ?? selectedProjectId}</span>
      {/if}
      {#if selectedEnvironmentId}
        <span class="status-info">Env: {environments.find((e) => e.id === selectedEnvironmentId)?.name ?? selectedEnvironmentId}</span>
      {/if}
    </div>
  </footer>

  {#if showCollectionImport}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showCollectionImport = false; }} onkeydown={(e) => { if (e.key === "Escape") showCollectionImport = false; }} role="dialog" aria-modal="true" tabindex="0">
      <div class="modal-container">
        <div class="modal-header">
          <h3>Import Postman Collection</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showCollectionImport = false)}>✕</button>
        </div>
        <p class="hint">Upload or paste Postman Collection v2.0 or v2.1 JSON.</p>

        <div class="file-dropzone">
          <label class="file-label">
            <span>Choose JSON file</span>
            <input type="file" accept=".json,application/json" onchange={handleCollectionFileUpload} />
          </label>
        </div>

        <textarea
          placeholder="Or paste Postman Collection JSON here..."
          bind:value={collectionImportText}
          rows="4"
          class="body-input"
        ></textarea>

        {#if selectedProjectId}
          <div class="radio-row">
            <label class="radio-label">
              <input type="radio" name="collectionTarget" value="new" bind:group={collectionImportTarget} />
              New Project
            </label>
            <label class="radio-label">
              <input type="radio" name="collectionTarget" value="current" bind:group={collectionImportTarget} />
              Current Project
            </label>
          </div>
        {/if}

        {#if collectionImportError}
          <p class="error">{collectionImportError}</p>
        {/if}

        {#if collectionImportReport}
          <div class="import-report-card">
            <h4>Import Complete!</h4>
            <p><strong>Project:</strong> {collectionImportReport.project_name}</p>
            <p><strong>Requests:</strong> {collectionImportReport.requests_count}</p>
            <p><strong>Variables:</strong> {collectionImportReport.variables_count}</p>
            <p><strong>Sample responses:</strong> {collectionImportReport.sample_responses_count}</p>
            {#if collectionImportReport.warnings.length > 0}
              <div class="warnings-box">
                <h5>Compatibility Notes:</h5>
                <ul>
                  {#each collectionImportReport.warnings as warn}
                    <li>{warn}</li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {/if}

        <div class="params-row">
          <button
            type="button"
            class="btn-primary"
            disabled={!collectionImportText.trim() || collectionImportLoading}
            onclick={importPostmanCollectionAction}
          >
            {collectionImportLoading ? "Importing…" : "Import"}
          </button>
          <button type="button" onclick={() => { showCollectionImport = false; }}>
            {collectionImportReport ? "Done" : "Cancel"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showEnvironmentImport}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showEnvironmentImport = false; }} onkeydown={(e) => { if (e.key === "Escape") showEnvironmentImport = false; }} role="dialog" aria-modal="true" tabindex="0">
      <div class="modal-container">
        <div class="modal-header">
          <h3>Import Postman Environment</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showEnvironmentImport = false)}>✕</button>
        </div>
        <p class="hint">Upload or paste a Postman Environment JSON file.</p>

        <div class="file-dropzone">
          <label class="file-label">
            <span>Choose JSON file</span>
            <input type="file" accept=".json,application/json" onchange={handleEnvironmentFileUpload} />
          </label>
        </div>

        <textarea
          placeholder="Or paste Postman Environment JSON here..."
          bind:value={environmentImportText}
          rows="3"
          class="body-input"
        ></textarea>

        {#if environmentImportError}
          <p class="error">{environmentImportError}</p>
        {/if}

        {#if environmentImportReport}
          <div class="import-report-card">
            <h4>Environment Imported!</h4>
            <p><strong>Environment:</strong> {environmentImportReport.environment_name}</p>
            <p><strong>Variables:</strong> {environmentImportReport.variables_count}</p>
            {#if environmentImportReport.warnings.length > 0}
              <div class="warnings-box">
                <h5>Compatibility Notes:</h5>
                <ul>
                  {#each environmentImportReport.warnings as warn}
                    <li>{warn}</li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {/if}

        <div class="params-row">
          <button
            type="button"
            class="btn-primary"
            disabled={!environmentImportText.trim() || environmentImportLoading}
            onclick={importPostmanEnvironmentAction}
          >
            {environmentImportLoading ? "Importing…" : "Import Environment"}
          </button>
          <button type="button" onclick={() => { showEnvironmentImport = false; }}>
            {environmentImportReport ? "Done" : "Cancel"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showCurlImport}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showCurlImport = false; }} onkeydown={(e) => { if (e.key === "Escape") showCurlImport = false; }} role="dialog" aria-modal="true" tabindex="0">
      <div class="modal-container">
        <div class="modal-header">
          <h3>Import from cURL</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showCurlImport = false)}>✕</button>
        </div>
        <form onsubmit={importCurlCommand}>
          <input placeholder="Request Name (optional)" bind:value={curlImportName} />
          <textarea
            placeholder="Paste cURL command here (e.g. curl -X POST https://api.example.com/users -H 'Content-Type: application/json' -d 'data')"
            bind:value={curlImportText}
            rows="4"
            class="body-input"
          ></textarea>
          {#if curlImportError}
            <p class="error">{curlImportError}</p>
          {/if}
          <div class="params-row">
            <button type="submit" class="btn-primary">Import Request</button>
            <button type="button" onclick={() => (showCurlImport = false)}>Cancel</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

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
            <h3>✨ AI Assistant & Source Intelligence</h3>
            <span class="modal-sub">Generate requests with project context, discover codebase routes, or configure Claude AI</span>
          </div>
          <button type="button" class="modal-close-btn" onclick={() => (showAiPanel = false)}>✕</button>
        </div>

        <div class="modal-tabs">
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "generate"}
            onclick={() => (aiActiveTab = "generate")}
          >
            ✨ Ask AI (Project Context)
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "source"}
            onclick={() => (aiActiveTab = "source")}
          >
            🔍 Source Discovery {#if sourceReport}<span class="badge badge-framework">{sourceReport.endpoints.length}</span>{/if}
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={aiActiveTab === "settings"}
            onclick={() => (aiActiveTab = "settings")}
          >
            ⚙️ AI Settings {#if !aiConfigured}<span class="tab-badge-alert">!</span>{/if}
          </button>
        </div>

        {#if aiActiveTab === "generate"}
          <div class="modal-body">
            {#if !aiConfigured}
              <div class="action-alert warning">
                <span>⚠️ Claude AI API key is not configured. Add your key in the <button type="button" class="link-btn" onclick={() => (aiActiveTab = "settings")}>AI Settings</button> tab.</span>
              </div>
            {/if}

            <div class="ai-context-options">
              <label class="checkbox-label" title="Send existing project request names/methods as context for high fidelity generation">
                <input type="checkbox" bind:checked={aiIncludeExistingRequests} />
                Include existing project requests as context
              </label>
              <label class="checkbox-label" title="Send variable template keys only (e.g. baseUrl, token) — values and secrets are strictly redacted">
                <input type="checkbox" bind:checked={aiIncludeVariables} />
                Include variable names (keys only, values never sent)
              </label>
            </div>

            <form onsubmit={generateWithAi} class="ai-prompt-form">
              <textarea
                placeholder="Describe the API request you want, e.g. 'Get user profile by ID with Authorization bearer token' or 'Create a checkout order with items array and shipping address'"
                bind:value={aiPrompt}
                class="body-input"
                rows="3"
              ></textarea>
              <div class="params-row">
                <button type="submit" class="btn-primary" disabled={aiGenerating || !aiPrompt.trim()}>
                  {aiGenerating ? "Generating API Definition…" : "Generate Request"}
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
                  <span>Headers: <strong>{aiPreview.headers.length}</strong></span>
                  <span>Query params: <strong>{aiPreview.query_params.length}</strong></span>
                  {#if aiPreview.body}<span>Has Body</span>{/if}
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
                    Add to Project
                  </button>
                  <button type="button" onclick={() => (aiPreview = null)}>Discard</button>
                </div>
              </div>
            {/if}
          </div>

        {:else if aiActiveTab === "source"}
          <div class="modal-body">
            <p class="hint">
              Analyze a local backend project (Node.js, Express, FastAPI, Flask, Django, Spring Boot, Go Gin/Chi, Laravel) to discover endpoints and OpenAPI specifications. <em>Strictly read-only inspection.</em>
            </p>

            <div class="source-scan-bar">
              <input
                type="text"
                placeholder="Path to backend codebase (e.g. C:/projects/my-api or ../backend)"
                bind:value={sourceDirectoryInput}
                class="url-input"
              />
              <button
                type="button"
                class="btn-primary"
                disabled={sourceScanning || !sourceDirectoryInput.trim()}
                onclick={scanSourceProjectAction}
              >
                {sourceScanning ? "Scanning…" : "Scan Codebase"}
              </button>
            </div>

            {#if sourceActionFeedback}
              <p class="action-feedback-inline">{sourceActionFeedback}</p>
            {/if}

            {#if sourceReport}
              <div class="source-summary-panel">
                <div class="source-badges-row">
                  <span class="badge">Type: {sourceReport.project_type}</span>
                  <span class="badge">Scanned: {sourceReport.scanned_files_count} files</span>
                  <span class="badge badge-success">Found: {sourceReport.endpoints.length} routes</span>
                  {#each sourceReport.frameworks as fw}
                    <span class="badge badge-framework">{fw}</span>
                  {/each}
                  {#if sourceReport.has_openapi}
                    <span class="badge badge-openapi">OpenAPI: {sourceReport.openapi_path ?? "Spec detected"}</span>
                  {/if}
                </div>

                {#if sourceReport.endpoints.length > 0}
                  <div class="source-filter-row">
                    <input
                      type="text"
                      placeholder="Filter routes by path, method, or file..."
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
                            <span class="badge badge-auth" title="Authentication detected">🔒 {ep.auth_hint}</span>
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
                          title="Import this endpoint as a request in the active project"
                        >
                          + Import Request
                        </button>
                      </div>
                    {/each}
                    {#if filteredSourceEndpoints.length === 0}
                      <p class="hint text-center">No routes match filter "{sourceFilter}".</p>
                    {/if}
                  </div>
                {:else}
                  <p class="hint">No API routes or OpenAPI specs discovered in the scanned files.</p>
                {/if}
              </div>
            {/if}
          </div>

        {:else if aiActiveTab === "settings"}
          <div class="modal-body">
            <div class="ai-settings-grid">
              <div class="settings-field">
                <label for="ai-api-key-input">
                  <strong>Anthropic Claude API Key:</strong>
                  {#if aiConfigured}
                    <span class="badge badge-success">Configured</span>
                  {:else}
                    <span class="badge">Not configured</span>
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
                    {aiShowKey ? "Hide" : "Show"}
                  </button>
                </div>
                <span class="hint">Stored locally in SQLite or falls back to ANTHROPIC_API_KEY environment variable. Never sent anywhere except directly to Anthropic.</span>
              </div>

              <div class="settings-field">
                <label for="ai-model-select"><strong>Model:</strong></label>
                <select id="ai-model-select" bind:value={aiModelInput} class="url-input">
                  <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet (Recommended)</option>
                  <option value="claude-3-5-haiku-20241022">Claude 3.5 Haiku (Fastest)</option>
                  <option value="claude-3-opus-20240229">Claude 3 Opus (Most Powerful)</option>
                </select>
              </div>

              <div class="settings-field">
                <label for="ai-base-url-input"><strong>Custom Base URL (Optional):</strong></label>
                <input
                  id="ai-base-url-input"
                  type="text"
                  placeholder="https://api.anthropic.com (default)"
                  bind:value={aiBaseUrlInput}
                  class="url-input"
                />
              </div>

              <div class="params-row">
                <button type="button" class="btn-primary" onclick={saveAiSettingsAction}>Save Settings</button>
                <button type="button" class="btn-secondary" disabled={aiTesting} onclick={testAiConnectionAction}>
                  {aiTesting ? "Testing…" : "Test Connection"}
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
          <button type="button" onclick={() => (showAiPanel = false)}>Close</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showVariables}
    <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showVariables = false; }} onkeydown={(e) => { if (e.key === "Escape") showVariables = false; }} role="dialog" aria-modal="true" tabindex="0">
      <div class="modal-container modal-wide">
        <div class="modal-header">
          <h3>Environment & Variables</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showVariables = false)}>✕</button>
        </div>

        <div class="variables-panel">
          <h4>Add Variable</h4>
          <form class="request-form" onsubmit={createVariable}>
            <select bind:value={newVarScope}>
              <option value="global">Global (Project)</option>
              {#if selectedEnvironmentId}
                <option value="environment">Environment</option>
              {/if}
            </select>
            <input placeholder="Key" bind:value={newVarKey} />
            <input placeholder="Value" bind:value={newVarValue} />
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={newVarIsSecret} /> Secret
            </label>
            <label class="checkbox-label" title="Keep local on this device; excluded from git & exports">
              <input type="checkbox" bind:checked={newVarIsLocal} /> Local
            </label>
            <button type="submit" class="btn-primary">Add</button>
          </form>

          <h4>Global Variables ({projectVariables.length})</h4>
          <div class="params-table">
            {#each projectVariables as v (v.id)}
              <div class="params-row">
                <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title="Enabled" />
                <span class="var-key">{v.key}</span>
                <span class="var-val">{revealedSecrets[v.id] ?? v.value}</span>
                {#if v.is_local}
                  <span class="badge badge-local" title="Local only — excluded from git & exports">local</span>
                {/if}
                {#if v.is_secret}
                  <span class="badge">secret</span>
                  {#if !revealedSecrets[v.id]}
                    <button type="button" class="icon-btn" title="Reveal secret" onclick={() => revealSecret(v.id)}>👁</button>
                  {/if}
                {/if}
                <button type="button" class="icon-btn" title={v.is_local ? "Make Shared" : "Make Local Only"} onclick={() => toggleVariableLocal(v)}>{v.is_local ? "💻" : "🌐"}</button>
                <button type="button" class="icon-btn" title="Toggle Secret" onclick={() => toggleVariableSecret(v)}>🔒</button>
                <button type="button" class="icon-btn" title="Delete" onclick={() => deleteVariable(v.id)}>🗑</button>
              </div>
            {:else}
              <p class="hint">No global variables defined.</p>
            {/each}
          </div>

          {#if selectedEnvironmentId}
            <h4>Environment Variables ({environmentVariables.length})</h4>
            <div class="params-table">
              {#each environmentVariables as v (v.id)}
                <div class="params-row">
                  <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} title="Enabled" />
                  <span class="var-key">{v.key}</span>
                  <span class="var-val">{revealedSecrets[v.id] ?? v.value}</span>
                  {#if v.is_local}
                    <span class="badge badge-local" title="Local only — excluded from git & exports">local</span>
                  {/if}
                  {#if v.is_secret}
                    <span class="badge">secret</span>
                    {#if !revealedSecrets[v.id]}
                      <button type="button" class="icon-btn" title="Reveal secret" onclick={() => revealSecret(v.id)}>👁</button>
                    {/if}
                  {/if}
                  <button type="button" class="icon-btn" title={v.is_local ? "Make Shared" : "Make Local Only"} onclick={() => toggleVariableLocal(v)}>{v.is_local ? "💻" : "🌐"}</button>
                  <button type="button" class="icon-btn" title="Toggle Secret" onclick={() => toggleVariableSecret(v)}>🔒</button>
                  <button type="button" class="icon-btn" title="Delete" onclick={() => deleteVariable(v.id)}>🗑</button>
                </div>
              {:else}
                <p class="hint">No variables defined in this environment.</p>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if showGitModal}
    <div
      class="modal-backdrop"
      onclick={(e) => { if (e.target === e.currentTarget) showGitModal = false; }}
      onkeydown={(e) => { if (e.key === "Escape") showGitModal = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="0"
    >
      <div class="modal-container">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <h3>⎇ Git & Collaboration Sync</h3>
            {#if selectedProjectId}
              <span class="modal-sub">Project: {projects.find((p) => p.id === selectedProjectId)?.name ?? selectedProjectId}</span>
            {/if}
          </div>
          <button type="button" class="modal-close-btn" onclick={() => (showGitModal = false)}>✕</button>
        </div>

        <div class="modal-tabs">
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "sync"}
            onclick={() => (gitActiveTab = "sync")}
          >
            Repository & Sync
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "conflicts"}
            onclick={() => (gitActiveTab = "conflicts")}
          >
            Conflicts
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
            GitHub Auth & Team
          </button>
          <button
            type="button"
            class="modal-tab-btn"
            class:active={gitActiveTab === "projectfile"}
            onclick={() => (gitActiveTab = "projectfile")}
          >
            Project File
          </button>
        </div>

        <div class="modal-body">
          {#if gitActionFeedback}
            <div class="action-alert success">{gitActionFeedback}</div>
          {/if}
          {#if gitActionError}
            <div class="action-alert error">{gitActionError}</div>
          {/if}

          {#if gitActiveTab === "sync"}
            <div class="git-panel-section">
              <h4>Local Repository Settings</h4>
              <div class="form-row-stacked">
                <label for="git-repo-path-input">Repository Directory Path:</label>
                <div class="input-with-actions">
                  <input
                    id="git-repo-path-input"
                    type="text"
                    placeholder="e.g. C:/Projects/my-api-repo or /home/user/my-api-repo"
                    bind:value={gitRepoPathInput}
                    class="path-input"
                  />
                  <button type="button" onclick={saveGitSettingsAction}>Save Path</button>
                  <button type="button" onclick={() => refreshGitStatus()} disabled={!gitRepoPathInput.trim() || gitStatusLoading}>
                    {gitStatusLoading ? "Checking…" : "↻ Check Status"}
                  </button>
                </div>
                <p class="hint">Absolute path to local Git repository folder containing <code>light-postman.json</code>.</p>
              </div>

              {#if !gitStatus || !gitStatus.is_repo}
                <div class="alert-box-warning">
                  <p><strong>Not a Git Repository:</strong> The specified folder is not yet initialized as a Git repository.</p>
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={!gitRepoPathInput.trim() || gitLoading}
                    onclick={initializeGitRepoAction}
                  >
                    {gitLoading ? "Initializing…" : "Initialize Git Repository"}
                  </button>
                </div>
              {:else}
                <div class="git-status-card">
                  <div class="status-summary-row">
                    <span class="status-label">Branch:</span>
                    <strong>{gitStatus.branch}</strong>
                    <span class="status-sep">|</span>
                    <span class="status-label">Status:</span>
                    <span class="git-badge-kind kind-{gitStatus.status_kind}">{gitStatus.status_kind.toUpperCase()}</span>
                    {#if gitStatus.ahead > 0}
                      <span class="badge-ahead">↑ {gitStatus.ahead} unpushed</span>
                    {/if}
                    {#if gitStatus.behind > 0}
                      <span class="badge-behind">↓ {gitStatus.behind} unpulled</span>
                    {/if}
                  </div>

                  {#if gitStatus.staged_files.length > 0 || gitStatus.unstaged_files.length > 0 || gitStatus.untracked_files.length > 0}
                    <div class="files-changed-summary">
                      {#if gitStatus.staged_files.length > 0}
                        <p class="file-category">Staged: <code>{gitStatus.staged_files.join(", ")}</code></p>
                      {/if}
                      {#if gitStatus.unstaged_files.length > 0}
                        <p class="file-category">Modified: <code>{gitStatus.unstaged_files.join(", ")}</code></p>
                      {/if}
                      {#if gitStatus.untracked_files.length > 0}
                        <p class="file-category">Untracked: <code>{gitStatus.untracked_files.join(", ")}</code></p>
                      {/if}
                    </div>
                  {:else}
                    <p class="working-tree-clean">Working directory is clean.</p>
                  {/if}
                </div>

                <div class="git-commit-box">
                  <h4>Manual Sync & Commit</h4>
                  <div class="commit-input-row">
                    <input
                      type="text"
                      placeholder="Commit message (e.g. 'Add authentication endpoints')"
                      bind:value={gitCommitMessage}
                    />
                    <button
                      type="button"
                      class="btn-primary"
                      disabled={gitLoading}
                      onclick={commitAndPushAction}
                    >
                      {gitLoading ? "Syncing…" : "Commit & Push"}
                    </button>
                    <button
                      type="button"
                      disabled={gitLoading}
                      onclick={pullRepositoryAction}
                    >
                      {gitLoading ? "Pulling…" : "Pull Remote"}
                    </button>
                  </div>
                  <div class="quick-git-actions">
                    <button type="button" class="icon-btn-text" onclick={saveProjectToRepoAction} disabled={gitLoading}>
                      💾 Save light-postman.json
                    </button>
                    <button type="button" class="icon-btn-text" onclick={viewDiffAction} disabled={gitLoading}>
                      📄 View Diff
                    </button>
                    <button type="button" class="icon-btn-text" onclick={viewHistoryAction} disabled={gitLoading}>
                      📜 Commit History
                    </button>
                  </div>
                </div>

                <div class="auto-sync-box">
                  <label class="checkbox-label">
                    <input type="checkbox" bind:checked={gitAutoSyncInput} onchange={saveGitSettingsAction} />
                    <strong>Enable Automatic Background Sync (LP-0708)</strong>
                  </label>
                  <p class="hint">When enabled, synchronizes local commits with remote repository periodically in the background.</p>
                  {#if gitSettings?.last_sync_at}
                    <p class="hint">Last synchronized: {new Date(gitSettings.last_sync_at).toLocaleString()}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "conflicts"}
            <div class="git-panel-section">
              <h4>Merge Conflict Detection & Resolution (LP-0710, LP-0711)</h4>
              {#if !gitStatus?.has_conflicts || gitStatus.conflict_files.length === 0}
                <div class="clean-box">
                  <p>✔ No merge conflicts detected. All files are in sync.</p>
                </div>
              {:else}
                <div class="conflict-alert-box">
                  <p><strong>⚠️ Conflicts Detected:</strong> The following files have conflicting modifications between your local project and remote repository. Choose how to resolve each file:</p>
                </div>
                <div class="conflicts-list">
                  {#each gitStatus.conflict_files as file}
                    <div class="conflict-item-card">
                      <div class="conflict-item-header">
                        <span class="conflict-filename">📄 {file}</span>
                        <div class="conflict-choices">
                          <button
                            type="button"
                            class="btn-choice local"
                            title="Keep your local changes (--ours)"
                            onclick={() => resolveConflictAction(file, "ours")}
                            disabled={gitLoading}
                          >
                            Keep Local (Ours)
                          </button>
                          <button
                            type="button"
                            class="btn-choice remote"
                            title="Accept incoming remote changes (--theirs)"
                            onclick={() => resolveConflictAction(file, "theirs")}
                            disabled={gitLoading}
                          >
                            Keep Remote (Theirs)
                          </button>
                        </div>
                      </div>
                      <p class="hint">Decide whether to preserve your local modifications or replace them with remote version.</p>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "github"}
            <div class="git-panel-section">
              <h4>GitHub Collaboration & Permissions (LP-0703, LP-0704)</h4>
              <p class="hint">Connect your GitHub Personal Access Token (PAT with 'repo' scope) to authenticate remote Git operations and verify team permissions.</p>

              <div class="form-row-stacked">
                <label for="github-pat-input">Personal Access Token (PAT):</label>
                <div class="input-with-actions">
                  <input
                    id="github-pat-input"
                    type={githubShowToken ? "text" : "password"}
                    placeholder="ghp_..."
                    bind:value={githubTokenInput}
                  />
                  <button type="button" onclick={() => (githubShowToken = !githubShowToken)}>
                    {githubShowToken ? "Hide" : "Show"}
                  </button>
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={!githubTokenInput.trim() || githubValidating}
                    onclick={() => { saveGitSettingsAction(); verifyGitHubTokenAction(true); }}
                  >
                    {githubValidating ? "Verifying…" : "Verify Token"}
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
                  <span class="badge badge-success">Authenticated</span>
                </div>
              {/if}

              <div class="form-row-stacked">
                <label for="git-remote-url-input">Remote Git Repository URL:</label>
                <div class="input-with-actions">
                  <input
                    id="git-remote-url-input"
                    type="text"
                    placeholder="https://github.com/owner/repository.git"
                    bind:value={gitRemoteUrlInput}
                  />
                  <button type="button" onclick={saveGitSettingsAction}>Save Remote</button>
                  <button
                    type="button"
                    disabled={!githubTokenInput.trim() || !gitRemoteUrlInput.trim() || githubValidating}
                    onclick={checkGitHubRepoAction}
                  >
                    Check Permissions
                  </button>
                </div>
              </div>

              {#if githubRepoInfo}
                <div class="repo-permissions-card">
                  <h5>Repository: {githubRepoInfo.full_name}</h5>
                  <div class="perm-badges">
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.pull}>
                      Read / Pull: {githubRepoInfo.permissions?.pull ? "✓ Granted" : "✗ Denied"}
                    </span>
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.push}>
                      Write / Push: {githubRepoInfo.permissions?.push ? "✓ Granted" : "✗ Denied"}
                    </span>
                    <span class="perm-badge" class:perm-granted={githubRepoInfo.permissions?.admin}>
                      Admin: {githubRepoInfo.permissions?.admin ? "✓ Granted" : "✗ Denied"}
                    </span>
                  </div>
                  <p class="hint">Default branch: <code>{githubRepoInfo.default_branch}</code> · {githubRepoInfo.private ? "Private" : "Public"} repository</p>
                </div>
              {/if}
            </div>
          {:else if gitActiveTab === "projectfile"}
            <div class="git-panel-section">
              <h4>Canonical Project File Format (LP-0701)</h4>
              <p class="hint">
                The canonical <code>light-postman.json</code> format is human-readable, deterministically ordered, and Git merge-friendly.
                Local-only variables are strictly omitted to protect personal credentials.
              </p>

              <div class="projectfile-options">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={projectFileMaskSecrets} />
                  Mask secrets as <code>[SECRET]</code> in export
                </label>
                <button type="button" class="btn-primary" onclick={exportProjectFileAction}>
                  Generate Project JSON
                </button>
              </div>

              {#if projectFileJson}
                <div class="json-preview-box">
                  <div class="json-preview-toolbar">
                    <span>light-postman.json</span>
                    <div class="toolbar-actions">
                      <button type="button" onclick={downloadProjectFile}>Download File</button>
                      <button type="button" onclick={importProjectFileAction}>Import Into Project</button>
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

        <div class="modal-footer">
          <button type="button" onclick={() => (showGitModal = false)}>Close</button>
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
          <h3>Git Diff Preview</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showDiffModal = false)}>✕</button>
        </div>
        <div class="modal-body">
          {#if !gitDiffContent.trim()}
            <p class="hint">No uncommitted diff detected.</p>
          {:else}
            <pre class="diff-viewer">{gitDiffContent}</pre>
          {/if}
        </div>
        <div class="modal-footer">
          <button type="button" onclick={() => (showDiffModal = false)}>Close</button>
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
          <h3>Git Commit History</h3>
          <button type="button" class="modal-close-btn" onclick={() => (showHistoryModal = false)}>✕</button>
        </div>
        <div class="modal-body">
          {#if gitHistory.length === 0}
            <p class="hint">No commit history found.</p>
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
          <button type="button" onclick={() => (showHistoryModal = false)}>Close</button>
        </div>
      </div>
    </div>
  {/if}

</div>

<style>
  /* Dark is the app's own identity, not an OS-follow — same as real Postman, which
     defaults to a dark theme regardless of the host OS setting. A light override could be
     added later behind an explicit in-app toggle, but there is no such toggle today, so
     shipping a "light-unless-dark-OS" default would just be wrong most of the time. */
  :root {
    --color-bg: #1a1a1d;
    --color-bg-secondary: #232326;
    --color-bg-tertiary: #29292d;
    --color-bg-hover: #2f2f34;
    --color-sidebar-bg: #202023;
    --color-panel-bg: #1e1e21;
    --color-border: #2e2e33;
    --color-border-strong: #3a3a41;
    --color-text: #e8e8ea;
    --color-text-secondary: #9a9aa3;
    --color-text-tertiary: #6b6b72;
    --color-primary: #ff6c37;
    --color-primary-hover: #ff8257;
    --color-primary-contrast: #ffffff;
    --color-accent: #4c8ef9;
    --color-accent-hover: #6ba1fa;
    --color-accent-contrast: #ffffff;
    --color-success: #3fd68a;
    --color-success-bg: rgba(63, 214, 138, 0.14);
    --color-danger: #ff5c5c;
    --color-danger-bg: rgba(255, 92, 92, 0.14);
    --color-warn: #f2a93c;
    --color-warn-bg: rgba(242, 169, 60, 0.14);
    --color-focus: #4c8ef9;

    --method-get: #3fd68a;
    --method-post: #f0a020;
    --method-put: #4c8ef9;
    --method-patch: #b57bf6;
    --method-delete: #ff5c5c;
    --method-head: #3fc7d6;
    --method-options: #9a9aa3;

    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 10px;
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 12px 32px rgba(0, 0, 0, 0.5);
    --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    --font-mono: "SF Mono", Monaco, Menlo, Consolas, "Liberation Mono", monospace;

    color-scheme: dark;
    color: var(--color-text);
    background: var(--color-bg);
    font-family: var(--font-sans);
    font-size: 12.5px;
  }

  :global(body) {
    margin: 0;
    background: var(--color-bg);
  }

  * {
    box-sizing: border-box;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: var(--font-sans);
    color: var(--color-text);
    background: var(--color-bg);
    overflow: hidden;
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

  .env-new-form {
    display: flex;
  }

  .env-new-input {
    width: 140px;
    font-size: 0.78rem;
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

  /* Orange is the brand mark only (logo, "+"/add affordances) — the actual primary action
     color in real Postman is blue (Send, and anything analogous to it). Conflating the two
     was the biggest color mismatch against the reference. */
  .btn-primary,
  .btn-save {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-primary-contrast);
    font-weight: 600;
  }

  .btn-primary:hover,
  .btn-save:hover {
    background: var(--color-primary-hover);
    border-color: var(--color-primary-hover);
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

  .icon-btn-ghost {
    opacity: 0;
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

  .sidebar-header {
    padding: 0.7rem 0.9rem 0.3rem;
  }

  .sidebar-title {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
  }

  .new-project-form {
    display: flex;
    gap: 0.35rem;
    padding: 0.4rem 0.9rem 0.6rem;
  }

  .new-project-form input {
    flex: 1;
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
    display: flex;
    gap: 0.1rem;
    opacity: 0;
    flex-shrink: 0;
  }

  .project-row:hover .project-row-actions {
    opacity: 1;
  }

  .project-requests {
    padding: 0.3rem 0.5rem 0.6rem 1.3rem;
    border-left: 2px solid var(--color-border);
    margin: 0 0.9rem 0.4rem 1.1rem;
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

  .request-count-badge {
    font-size: 0.7rem;
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }

  .new-request-form {
    display: flex;
    gap: 0.3rem;
    margin-bottom: 0.5rem;
    flex-wrap: wrap;
  }

  .new-request-form input {
    flex: 1;
    min-width: 60px;
  }

  .method-select-sm {
    font-size: 0.7rem;
    padding: 0.35rem 0.3rem;
    width: 5.2rem;
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

  .request-item:hover .icon-btn-ghost {
    opacity: 1;
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
    overflow: hidden;
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

  .name-input {
    flex: 1;
    color: var(--color-text-secondary);
  }

  .send-action {
    display: flex;
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

  .response-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.4rem;
  }

  .response-status {
    margin: 0;
    font-size: 0.85rem;
  }

  .status-ok { color: var(--color-success); }
  .status-err { color: var(--color-danger); }

  .response-format-toggle {
    display: flex;
    gap: 2px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-sm);
    padding: 2px;
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

  .response-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .response-headers-details {
    margin-bottom: 0.5rem;
    font-size: 0.78rem;
  }

  .response-headers-details summary {
    cursor: pointer;
    color: var(--color-text-secondary);
    padding: 0.2rem 0;
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

  .variables-panel {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .variables-panel h4 {
    margin: 0.4rem 0 0.1rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--color-text-tertiary);
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
  }

  .action-alert.success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .action-alert.error {
    background: var(--color-danger-bg);
    color: var(--color-danger);
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
