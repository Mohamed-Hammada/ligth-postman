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
  body: string | null;
  created_at: string;
}

export interface NewRequestInput {
  project_id: string;
  name: string;
  method: string;
  url: string;
  headers: HeaderEntry[];
  body: string | null;
}

/** Structured backend error (see AppError in Rust) instead of a raw string/exception. */
export interface AppError {
  kind: "Storage" | "NotFound" | "Validation";
  message: string;
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
  if (isAppError(err)) return err.message;
  if (err instanceof Error) return err.message;
  return String(err);
}

export const api = {
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  listProjects: () => invoke<Project[]>("list_projects"),
  getProject: (id: string) => invoke<Project>("get_project", { id }),

  createRequest: (input: NewRequestInput) =>
    invoke<RequestFull>("create_request", { input }),
  listRequests: (projectId: string) =>
    invoke<RequestSummary[]>("list_requests", { projectId }),
  getRequest: (id: string) => invoke<RequestFull>("get_request", { id }),
};
