import { invoke } from "@tauri-apps/api/core";

export interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface HeaderEntry {
  key: string;
  value: string;
  enabled: boolean;
}

/** Never baked into `url` — appended at send time so a disabled param, or one whose value
 * comes from a variable, never requires rewriting the stored request. */
export interface QueryParam {
  key: string;
  value: string;
  enabled: boolean;
  description?: string | null;
}

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
  name: string;
  method: string;
  url: string;
  updated_at: string;
}

/** Fully hydrated request — fetch only when a tab is actually opened. */
export interface RequestFull extends RequestSummary {
  headers: HeaderEntry[];
  query_params: QueryParam[];
  auth: Auth;
  body: string | null;
  created_at: string;
}

export interface NewRequestInput {
  project_id: string;
  name: string;
  method: string;
  url: string;
  headers: HeaderEntry[];
  query_params: QueryParam[];
  auth: Auth;
  body: string | null;
}

/** `undefined`/omitted fields are left unchanged server-side — only send what actually changed. */
export interface UpdateProjectInput {
  id: string;
  name?: string;
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
}

export interface Environment {
  id: string;
  project_id: string;
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

export interface ResponseMeta extends ResponseSummary {
  headers: HeaderEntry[];
}

export interface ResponseBodyPayload {
  text: string;
  truncated: boolean;
}

export type SnippetMode = "placeholder" | "resolved";

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

export function describeError(err: unknown): string {
  // AppError::Cancelled is a unit variant, so serde's adjacently-tagged encoding omits
  // `message` entirely for it (`{ kind: "Cancelled" }` with no content key) — everything
  // else always carries one.
  if (isAppError(err)) return err.message ?? "Request cancelled";
  if (err instanceof Error) return err.message;
  return String(err);
}

export const api = {
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  listProjects: () => invoke<Project[]>("list_projects"),
  getProject: (id: string) => invoke<Project>("get_project", { id }),
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

  createEnvironment: (projectId: string, name: string) =>
    invoke<Environment>("create_environment", { input: { project_id: projectId, name } }),
  listEnvironments: (projectId: string) =>
    invoke<Environment[]>("list_environments", { projectId }),
  deleteEnvironment: (id: string) => invoke<void>("delete_environment", { id }),

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

  sendRequest: (requestId: string, environmentId: string | null, timeoutMs?: number) =>
    invoke<ResponseMeta>("send_request", { requestId, environmentId, timeoutMs }),
  cancelSend: (requestId: string) => invoke<void>("cancel_send", { requestId }),
  listResponseSummaries: (requestId: string) =>
    invoke<ResponseSummary[]>("list_response_summaries", { requestId }),
  getResponse: (id: string) => invoke<ResponseMeta>("get_response", { id }),
  getResponseBody: (id: string) => invoke<ResponseBodyPayload>("get_response_body", { id }),
  deleteResponse: (id: string) => invoke<void>("delete_response", { id }),

  isAiConfigured: () => invoke<boolean>("is_ai_configured"),
  generateApiWithAi: (prompt: string) =>
    invoke<GeneratedApiDefinition>("generate_api_with_ai", { prompt }),

  generateCurlSnippet: (requestId: string, environmentId: string | null, mode: SnippetMode) =>
    invoke<string>("generate_curl_snippet", { requestId, environmentId, mode }),
};
