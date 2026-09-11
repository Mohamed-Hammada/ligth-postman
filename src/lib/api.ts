import { invoke } from "@tauri-apps/api/core";

export interface Project {
  id: string;
  name: string;
  default_environment_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface HeaderEntry {
  key: string;
  value: string;
  enabled: boolean;
  description?: string | null;
}

/** Never baked into `url` — appended at send time so a disabled param, or one whose value
 * comes from a variable, never requires rewriting the stored request. */
export interface QueryParam {
  key: string;
  value: string;
  enabled: boolean;
  description?: string | null;
}

/** One multipart/form-data row. `file_path` is an absolute path on disk — the backend reads
 * and streams its bytes at send time via `reqwest::multipart::Form`, so the frontend never
 * loads file contents into memory itself. Matches Rust `models::FormDataPart`. */
export interface FormDataPart {
  key: string;
  value: string;
  enabled: boolean;
  description?: string | null;
  is_file: boolean;
  file_path?: string | null;
}

/** One application/x-www-form-urlencoded row. Matches Rust `models::UrlEncodedItem`. */
export interface UrlEncodedItem {
  key: string;
  value: string;
  enabled: boolean;
  description?: string | null;
}

/**
 * The stored `body` column is a plain string, but for structured body types it's actually one
 * of these shapes serialized as JSON (matches Rust `models::RequestBody`, `#[serde(tag =
 * "type")]`). `describeBodyForEditing`/`serializeBodyForStorage` in +page.svelte convert
 * between this and the editor's per-type state.
 */
export type StructuredRequestBody =
  | { type: "form_data"; items: FormDataPart[] }
  | { type: "url_encoded"; items: UrlEncodedItem[] }
  | { type: "binary"; file_path?: string | null };

/** LP-0107. `token`/`username`/`password`/`key`/`value` are raw templates — `{{var}}` is
 * resolved at send/snippet time, never baked into storage. No `inherit`/OAuth2 variant yet:
 * there is no Collection/Folder to inherit from and no OAuth2 flow implemented. */
export type Auth =
  | { type: "none" }
  | { type: "bearer"; token: string }
  | { type: "basic"; username: string; password: string }
  | { type: "api_key"; key: string; value: string; location: "header" | "query" };

/** Lightweight row for lists — never carries headers/body. */
export interface RequestSummary {
  id: string;
  project_id: string;
  folder_id: string | null;
  name: string;
  method: string;
  url: string;
  updated_at: string;
}

export interface Folder {
  id: string;
  project_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface NewFolderInput {
  project_id: string;
  name: string;
}

export interface UpdateFolderInput {
  id: string;
  name?: string;
}

export interface RequestSettings {
  timeout_ms?: number | null;
  follow_redirects?: boolean | null;
  max_redirects?: number | null;
  verify_ssl?: boolean | null;
  proxy_url?: string | null;
  http_version?: string | null;
}

/** Fully hydrated request — fetch only when a tab is actually opened. */
export interface RequestFull extends RequestSummary {
  headers: HeaderEntry[];
  query_params: QueryParam[];
  auth: Auth;
  body: string | null;
  description?: string | null;
  settings?: RequestSettings | null;
  pre_request_script?: string | null;
  post_request_script?: string | null;
  created_at: string;
}

export interface NewRequestInput {
  project_id: string;
  folder_id?: string | null;
  name: string;
  method: string;
  url: string;
  headers: HeaderEntry[];
  query_params: QueryParam[];
  auth: Auth;
  body: string | null;
  description?: string | null;
  settings?: RequestSettings | null;
  pre_request_script?: string | null;
  post_request_script?: string | null;
}

/** `undefined`/omitted fields are left unchanged server-side — only send what actually changed. */
export interface UpdateProjectInput {
  id: string;
  name?: string;
  default_environment_id?: string;
  clear_default_environment_id?: boolean;
}

export interface UpdateRequestInput {
  id: string;
  name?: string;
  method?: string;
  url?: string;
  headers?: HeaderEntry[];
  query_params?: QueryParam[];
  auth?: Auth;
  body?: string;
  clear_body?: boolean;
  description?: string;
  clear_description?: boolean;
  settings?: RequestSettings;
  clear_settings?: boolean;
  pre_request_script?: string;
  clear_pre_request_script?: boolean;
  post_request_script?: string;
  clear_post_request_script?: boolean;
  folder_id?: string;
  clear_folder_id?: boolean;
}

export type VariableScope = "global" | "environment" | "request";

export interface VariableView {
  id: string;
  scope: VariableScope;
  project_id?: string | null;
  environment_id?: string | null;
  request_id?: string | null;
  key: string;
  value: string;
  enabled: boolean;
  is_secret: boolean;
  is_local: boolean;
  description?: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewVariableInput {
  scope: VariableScope;
  project_id?: string | null;
  environment_id?: string | null;
  request_id?: string | null;
  key: string;
  value?: string;
  enabled?: boolean;
  is_secret?: boolean;
  is_local?: boolean;
  description?: string | null;
}

export interface UpdateVariableInput {
  id: string;
  key?: string;
  value?: string;
  enabled?: boolean;
  is_secret?: boolean;
  is_local?: boolean;
  description?: string | null;
  clear_description?: boolean;
}

export interface Environment {
  id: string;
  project_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface EnvironmentWithProject {
  id: string;
  project_id: string;
  project_name: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface ResolvedTemplate {
  resolved: string;
  missing: string[];
}

/** Structured backend error (see AppError in Rust) instead of a raw string/exception. */
export interface AppError {
  kind: "Storage" | "NotFound" | "Validation" | "Network" | "Cancelled" | "Ai";
  message: string;
  hint: string;
}

export interface ResponseSummary {
  id: string;
  request_id: string;
  status: number;
  status_text: string;
  duration_ms: number;
  body_size: number;
  created_at: string;
}

/** One row of the project-wide History screen. Matches Rust `models::ProjectHistoryEntry`. */
export interface ProjectHistoryEntry {
  id: string;
  request_id: string;
  request_name: string;
  method: string;
  url: string;
  status: number;
  status_text: string;
  duration_ms: number;
  body_size: number;
  created_at: string;
}

export interface ResponseCookie {
  name: string;
  value: string;
  domain?: string | null;
  path?: string | null;
  expires?: string | null;
  http_only?: boolean | null;
  secure?: boolean | null;
  same_site?: string | null;
}

export interface ResponseMeta extends ResponseSummary {
  headers: HeaderEntry[];
  content_type?: string | null;
  cookies?: ResponseCookie[];
}

export interface SampleResponse {
  id: string;
  request_id: string;
  name: string;
  status: number;
  headers: HeaderEntry[];
  body?: string | null;
  created_at: string;
}

export interface NewSampleResponseInput {
  request_id: string;
  name: string;
  status: number;
  headers: HeaderEntry[];
  body?: string | null;
}

export interface UpdateSampleResponseInput {
  id: string;
  name: string;
}

export interface CookieEntry {
  id: string;
  project_id: string;
  domain: string;
  path: string;
  name: string;
  value: string;
  expires_at?: string | null;
  http_only: boolean;
  secure: boolean;
  created_at: string;
}

export interface NewCookieInput {
  project_id: string;
  domain: string;
  path?: string | null;
  name: string;
  value: string;
  expires_at?: string | null;
  http_only?: boolean;
  secure?: boolean;
}

export interface ParsedCurlRequest {
  method: string;
  url: string;
  headers: HeaderEntry[];
  query_params: QueryParam[];
  auth: Auth;
  body: string | null;
}

export interface RequestDiagnostics {
  all_missing: string[];
  url_missing: string[];
  headers_missing: string[];
  query_params_missing: string[];
  auth_missing: string[];
  body_missing: string[];
}

export interface ResponseBodyPayload {
  text: string;
  truncated: boolean;
}

export type ConsoleLevel = "info" | "debug" | "warn" | "error";

export interface ConsoleEvent {
  id: string;
  correlation_id: string;
  request_id?: string | null;
  timestamp: string;
  level: ConsoleLevel;
  event_type: string;
  message: string;
  details?: Record<string, unknown> | null;
}

/** Matches Rust `diagnostics::SystemDiagnostics` exactly — every field here is a real
 * measurement (RSS via `sysinfo`, SQLite/WAL file sizes, real `COUNT(*)` totals), never a
 * placeholder. */
export interface SystemDiagnostics {
  process_rss_bytes: number;
  db_size_bytes: number;
  db_wal_size_bytes: number;
  total_projects: number;
  total_requests: number;
  total_environments: number;
  total_variables: number;
  total_responses: number;
  total_sample_responses: number;
  console_events_count: number;
  ai_configured: boolean;
  uptime_seconds: number;
}

export type SnippetMode = "placeholder" | "resolved";
export type SnippetTarget = "bash" | "powershell" | "windows_cmd" | "python" | "javascript";

/** Structured output from the AI generation command — preview before "Add to Project". */
export interface GeneratedApiDefinition {
  name: string;
  method: string;
  url: string;
  headers: HeaderEntry[];
  query_params: QueryParam[];
  body: string | null;
  description: string | null;
}

export function isAppError(err: unknown): err is AppError {
  return (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    "message" in err
  );
}

/** Short, human summary per error kind — shown instead of the raw backend message. */
const FRIENDLY_ERROR_SUMMARY: Record<AppError["kind"], string> = {
  Storage: "A local storage error occurred.",
  NotFound: "That item couldn't be found.",
  Validation: "Please check the highlighted fields.",
  Network: "Unable to reach the server.",
  Cancelled: "Request cancelled.",
  Ai: "The AI request failed.",
};

/**
 * Turns a raw AppError into an actionable, human sentence: a short summary plus the backend's
 * remediation hint (e.g. "Unable to reach the server. Verify target URL, ensure server is
 * active, and inspect proxy or firewall settings.") instead of a technical string like
 * "network error: connection refused (os error 10061)".
 */
export function describeError(err: unknown): string {
  if (isAppError(err)) {
    if (err.kind === "Cancelled") return FRIENDLY_ERROR_SUMMARY.Cancelled;
    const summary = FRIENDLY_ERROR_SUMMARY[err.kind] ?? err.message;
    return err.hint ? `${summary} ${err.hint}` : summary;
  }
  if (err instanceof Error) return err.message;
  return String(err);
}

export const api = {
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  listProjects: () => invoke<Project[]>("list_projects"),
  getProject: (id: string) => invoke<Project>("get_project", { id }),
  getProjectRequestCounts: () => invoke<Record<string, number>>("get_project_request_counts"),
  updateProject: (input: UpdateProjectInput) =>
    invoke<Project>("update_project", { input }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  createRequest: (input: NewRequestInput) =>
    invoke<RequestFull>("create_request", { input }),
  listRequests: (projectId: string) =>
    invoke<RequestSummary[]>("list_requests", { projectId }),
  getRequest: (id: string) => invoke<RequestFull>("get_request", { id }),
  updateRequest: (input: UpdateRequestInput) =>
    invoke<RequestFull>("update_request", { input }),
  deleteRequest: (id: string) => invoke<void>("delete_request", { id }),

  createFolder: (input: NewFolderInput) => invoke<Folder>("create_folder", { input }),
  listFolders: (projectId: string) => invoke<Folder[]>("list_folders", { projectId }),
  updateFolder: (input: UpdateFolderInput) => invoke<Folder>("update_folder", { input }),
  deleteFolder: (id: string) => invoke<void>("delete_folder", { id }),

  createEnvironment: (projectId: string, name: string) =>
    invoke<Environment>("create_environment", { input: { project_id: projectId, name } }),
  listEnvironments: (projectId: string) =>
    invoke<Environment[]>("list_environments", { projectId }),
  listAllEnvironments: () =>
    invoke<EnvironmentWithProject[]>("list_all_environments"),
  updateEnvironment: (input: { id: string; name?: string }) =>
    invoke<Environment>("update_environment", { input }),
  deleteEnvironment: (id: string) => invoke<void>("delete_environment", { id }),

  createVariable: (input: NewVariableInput) =>
    invoke<VariableView>("create_variable", { input }),
  listVariablesForScope: (scope: VariableScope, scopeRef: string) =>
    invoke<VariableView[]>("list_variables_for_scope", { scope, scopeRef }),
  updateVariable: (input: UpdateVariableInput) =>
    invoke<VariableView>("update_variable", { input }),
  deleteVariable: (id: string) => invoke<void>("delete_variable", { id }),
  revealVariableValue: (id: string) =>
    invoke<string>("reveal_variable_value", { id }),

  resolvePreview: (
    projectId: string,
    environmentId: string | null,
    requestId: string | null,
    template: string,
  ) =>
    invoke<ResolvedTemplate>("resolve_preview", {
      projectId,
      environmentId,
      requestId,
      template,
    }),

  diagnoseRequest: (
    requestId: string,
    environmentId: string | null,
  ) =>
    invoke<RequestDiagnostics>("diagnose_request", {
      requestId,
      environmentId,
    }),

  sendRequest: (requestId: string, environmentId: string | null, timeoutMs?: number) =>
    invoke<ResponseMeta>("send_request", { requestId, environmentId, timeoutMs }),
  cancelSend: (requestId: string) => invoke<void>("cancel_send", { requestId }),
  listResponseSummaries: (requestId: string) =>
    invoke<ResponseSummary[]>("list_response_summaries", { requestId }),
  listProjectHistory: (projectId: string, limit?: number) =>
    invoke<ProjectHistoryEntry[]>("list_project_history", { projectId, limit: limit ?? null }),
  getResponse: (id: string) => invoke<ResponseMeta>("get_response", { id }),
  getResponseBody: (id: string) => invoke<ResponseBodyPayload>("get_response_body", { id }),
  deleteResponse: (id: string) => invoke<void>("delete_response", { id }),

  createSampleResponse: (input: NewSampleResponseInput) =>
    invoke<SampleResponse>("create_sample_response", { input }),
  listSampleResponses: (requestId: string) =>
    invoke<SampleResponse[]>("list_sample_responses", { requestId }),
  deleteSampleResponse: (id: string) =>
    invoke<void>("delete_sample_response", { id }),
  updateSampleResponse: (input: UpdateSampleResponseInput) =>
    invoke<SampleResponse>("update_sample_response", { input }),

  createCookie: (input: NewCookieInput) =>
    invoke<CookieEntry>("create_cookie", { input }),
  listCookiesForProject: (projectId: string) =>
    invoke<CookieEntry[]>("list_cookies_for_project", { projectId }),
  deleteCookie: (id: string) =>
    invoke<void>("delete_cookie", { id }),

  importCurl: (curlCommand: string) =>
    invoke<ParsedCurlRequest>("import_curl", { curlCommand }),

  isAiConfigured: () => invoke<boolean>("is_ai_configured"),
  generateApiWithAi: (prompt: string) =>
    invoke<GeneratedApiDefinition>("generate_api_with_ai", { prompt }),

  generateCurlSnippet: (
    requestId: string,
    environmentId: string | null,
    mode: SnippetMode,
    target?: SnippetTarget,
  ) =>
    invoke<string>("generate_curl_snippet", { requestId, environmentId, mode, target }),

  importPostmanCollection: (collectionJson: string, targetProjectId?: string | null) =>
    invoke<CollectionImportReport>("import_postman_collection", {
      collectionJson,
      targetProjectId: targetProjectId ?? null,
    }),

  importPostmanEnvironment: (environmentJson: string, targetProjectId: string) =>
    invoke<EnvironmentImportReport>("import_postman_environment", {
      environmentJson,
      targetProjectId,
    }),

  importLocalPostmanWorkspace: (rootPath: string) =>
    invoke<LocalWorkspaceImportReport>("import_local_postman_workspace", { rootPath }),

  exportPostmanCollection: (projectId: string) =>
    invoke<string>("export_postman_collection", { projectId }),

  exportPostmanEnvironment: (environmentId: string) =>
    invoke<string>("export_postman_environment", { environmentId }),

  getConsoleEvents: (limit?: number, level?: string, requestId?: string) =>
    invoke<ConsoleEvent[]>("get_console_events", {
      limit: limit ?? null,
      level: level ?? null,
      requestId: requestId ?? null,
    }),

  clearConsoleEvents: () => invoke<void>("clear_console_events"),

  exportConsoleEvents: () => invoke<string>("export_console_events"),

  getSystemDiagnostics: () => invoke<SystemDiagnostics>("get_system_diagnostics"),

  exportProjectFile: (projectId: string, includeSecrets?: boolean) =>
    invoke<string>("export_project_file", {
      projectId,
      includeSecrets: includeSecrets ?? false,
    }),

  importProjectFile: (fileContent: string, targetProjectId?: string | null) =>
    invoke<Project>("import_project_file", {
      fileContent,
      targetProjectId: targetProjectId ?? null,
    }),

  saveProjectToRepo: (
    projectId: string,
    directory: string,
    includeSecrets?: boolean,
  ) =>
    invoke<string>("save_project_to_repo", {
      projectId,
      directory,
      includeSecrets: includeSecrets ?? false,
    }),

  loadProjectFromRepo: (
    directory: string,
    targetProjectId?: string | null,
  ) =>
    invoke<Project>("load_project_from_repo", {
      directory,
      targetProjectId: targetProjectId ?? null,
    }),

  getGitStatus: (directory: string) =>
    invoke<GitStatus>("get_git_status", { directory }),

  gitInitRepository: (directory: string) =>
    invoke<void>("git_init_repository", { directory }),

  gitCommitChanges: (directory: string, message: string) =>
    invoke<string>("git_commit_changes", { directory, message }),

  gitGetDiff: (directory: string) =>
    invoke<string>("git_get_diff", { directory }),

  gitGetLog: (directory: string, limit?: number) =>
    invoke<GitCommit[]>("git_get_log", {
      directory,
      limit: limit ?? null,
    }),

  gitPullRepository: (
    directory: string,
    remote?: string,
    branch?: string,
  ) =>
    invoke<string>("git_pull_repository", {
      directory,
      remote: remote ?? null,
      branch: branch ?? null,
    }),

  gitPushRepository: (
    directory: string,
    remote?: string,
    branch?: string,
  ) =>
    invoke<string>("git_push_repository", {
      directory,
      remote: remote ?? null,
      branch: branch ?? null,
    }),

  gitResolveConflict: (directory: string, file: string, choice: string) =>
    invoke<void>("git_resolve_conflict", { directory, file, choice }),

  gitGetConflictVersions: (directory: string, file: string) =>
    invoke<ConflictVersions>("git_get_conflict_versions", { directory, file }),

  getProjectGitSettings: (projectId: string) =>
    invoke<ProjectGitSettings | null>("get_project_git_settings", {
      projectId,
    }),

  saveProjectGitSettings: (settings: ProjectGitSettings) =>
    invoke<void>("save_project_git_settings", { settings }),

  verifyGitHubToken: (token: string) =>
    invoke<GitHubUser>("verify_github_token", { token }),

  getGitHubRepoInfo: (token: string, owner: string, repo: string) =>
    invoke<GitHubRepoInfo>("get_github_repo_info", { token, owner, repo }),

  getAiSettings: () => invoke<AiSettings>("get_ai_settings"),

  saveAiSettings: (input: UpdateAiSettingsInput) =>
    invoke<void>("save_ai_settings", { input }),

  testAiConnection: () => invoke<string>("test_ai_connection"),

  generateApiWithProjectContext: (
    projectId: string,
    prompt: string,
    includeExistingRequests?: boolean,
    includeVariableNames?: boolean,
  ) =>
    invoke<GeneratedApiDefinition>("generate_api_with_project_context", {
      projectId,
      prompt,
      includeExistingRequests: includeExistingRequests ?? true,
      includeVariableNames: includeVariableNames ?? true,
    }),

  generateSampleResponseWithAi: (requestId: string) =>
    invoke<SampleResponse>("generate_sample_response_with_ai", { requestId }),

  generateTestsAndDocsWithAi: (requestId: string) =>
    invoke<GeneratedTestsAndDocs>("generate_tests_and_docs_with_ai", { requestId }),

  scanSourceProject: (directory: string) =>
    invoke<SourceProjectReport>("scan_source_project", { directory }),

  getProjectSourceDirectory: (projectId: string) =>
    invoke<string | null>("get_project_source_directory", { projectId }),

  setProjectSourceDirectory: (
    projectId: string,
    directory: string,
    framework?: string | null,
  ) =>
    invoke<void>("set_project_source_directory", {
      projectId,
      directory,
      framework: framework ?? null,
    }),

  importDiscoveredEndpoint: (
    projectId: string,
    endpoint: DiscoveredEndpoint,
  ) =>
    invoke<RequestSummary>("import_discovered_endpoint", {
      projectId,
      endpoint,
    }),
};

export interface GitStatus {
  is_repo: boolean;
  branch: string;
  status_kind: string;
  ahead: number;
  behind: number;
  staged_files: string[];
  unstaged_files: string[];
  untracked_files: string[];
  has_conflicts: boolean;
  conflict_files: string[];
}

/** The three sides of a real merge conflict, read from Git's index stages. Any side can be
 * `null` (e.g. an add/add conflict has no common ancestor) — never fabricated. Matches Rust
 * `git_sync::ConflictVersions`. */
export interface ConflictVersions {
  base: string | null;
  local: string | null;
  remote: string | null;
}

export interface GitCommit {
  hash: string;
  author: string;
  date: string;
  message: string;
}

export interface ProjectGitSettings {
  project_id: string;
  repo_path?: string | null;
  remote_url?: string | null;
  branch: string;
  auto_sync: boolean;
  github_token?: string | null;
  last_sync_at?: string | null;
}

export interface GitHubUser {
  login: string;
  id: number;
  name?: string | null;
  avatar_url?: string | null;
  email?: string | null;
}

export interface GitHubRepoPermissions {
  admin: boolean;
  push: boolean;
  pull: boolean;
}

export interface GitHubRepoInfo {
  full_name: string;
  private: boolean;
  default_branch: string;
  permissions?: GitHubRepoPermissions | null;
}

export interface CollectionImportReport {
  project_id: string;
  project_name: string;
  requests_count: number;
  variables_count: number;
  sample_responses_count: number;
  warnings: string[];
}

export interface EnvironmentImportReport {
  environment_id: string;
  environment_name: string;
  variables_count: number;
  warnings: string[];
}

export interface LocalWorkspaceImportReport {
  projects_created: number;
  folders_created: number;
  requests_imported: number;
  samples_imported: number;
  environments_imported: number;
  variables_imported: number;
  warnings: string[];
}

export interface AiSettings {
  api_key?: string | null;
  model: string;
  base_url?: string | null;
  is_configured: boolean;
}

export interface UpdateAiSettingsInput {
  api_key?: string | null;
  model?: string | null;
  base_url?: string | null;
}

export interface GeneratedTestsAndDocs {
  tests_script: string;
  documentation: string;
}

export interface DiscoveredEndpoint {
  name: string;
  method: string;
  path: string;
  description?: string | null;
  source_file: string;
  line_number?: number | null;
  auth_hint?: string | null;
  framework: string;
}

export interface SourceProjectReport {
  directory: string;
  project_type: string;
  frameworks: string[];
  has_openapi: boolean;
  openapi_path?: string | null;
  endpoints: DiscoveredEndpoint[];
  scanned_files_count: number;
  warnings: string[];
}


