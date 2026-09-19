// Dev-only in-memory backend used when the UI is opened in a plain browser (`vite dev`), never inside
// the desktop app. Built lazily from the git-ignored SQLite snapshot at ./browser-data.json.
import type { Environment, EnvironmentWithProject, Folder, Project, ProjectHistoryEntry, RequestFull, RequestSummary, VariableScope, VariableView, Workspace } from "./api";

// In-memory mock database for browser development mode, built from a local SQLite snapshot
// (`src/lib/browser-data.json`, git-ignored). It is loaded lazily and ONLY by the dev server when
// the page is opened outside the Tauri shell — never bundled into (or parsed by) release builds.
function buildBrowserMock(browserDataRaw: unknown) {
  const raw = browserDataRaw as unknown as {
    workspaces: Workspace[];
    projects: Project[];
    folders: Folder[];
    requestsSummary: RequestSummary[];
    fullRequests: Record<string, RequestFull>;
    environments: EnvironmentWithProject[];
    variables: VariableView[];
    projectRequestCounts: Record<string, number>;
  };

  const wsId = raw.workspaces[0]?.id || "default";
  const projId = raw.projects[0]?.id || "proj-ansari-scenarios";
  const folderId = raw.folders[0]?.id || "folder-1";
  const reqId = raw.requestsSummary[0]?.id || "req-login";
  const envId = raw.environments[0]?.id || "env-uat";

  const workspaces: Workspace[] = raw.workspaces.map(w => ({ ...w, name: "FOO-FRAMEWORK" }));
  const projects: Project[] = raw.projects;
  const folders: Folder[] = raw.folders;
  const requests: RequestSummary[] = raw.requestsSummary;
  const envs: EnvironmentWithProject[] = raw.environments;
  const mockVariables: VariableView[] = raw.variables;
  const projectRequestCounts: Record<string, number> = raw.projectRequestCounts;

  const fullRequests = new Map<string, RequestFull>();
  for (const [k, v] of Object.entries(raw.fullRequests)) {
    fullRequests.set(k, v);
  }

  function makeEnvVariables(environmentId: string, _envName: string): VariableView[] {
    return mockVariables.filter(v => v.scope === "environment" && v.environment_id === environmentId);
  }

  const defaultFullRequest: RequestFull = fullRequests.get(reqId) || {
    id: reqId,
    project_id: projId,
    folder_id: folderId,
    name: "Request",
    method: "GET",
    url: "{{host}}/api/v1/endpoint",
    query_params: [],
    headers: [
      { key: "Content-Type", value: "application/json", enabled: true, description: "" },
      { key: "Accept", value: "application/json", enabled: true, description: "" }
    ],
    body: "",
    auth: { type: "none" },
    settings: null,
    pre_request_script: null,
    post_request_script: null,
    description: "",
    sort_order: 1,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  };

  const initialResponse = {
    status: 200,
    status_text: "OK",
    duration_ms: 4700,
    body_size: 2760,
    body: '{\n  "meta": {\n    "status": "OK"\n  },\n  "response": {\n    "status": "success",\n    "message": "Registered Wallet",\n    "document_in_grace_period": false,\n    "document_expired": false,\n    "flow": "default"\n  }\n}',
    headers: [
      { key: "Content-Type", value: "application/json" },
      { key: "Date", value: new Date().toUTCString() },
      { key: "Server", value: "nginx" }
    ],
    cookies: [
      { name: "session_id", value: "ansari_sess_9a8f7c6e", domain: "alansari.ae", path: "/", expires: null, max_age: 3600, secure: true, http_only: true, same_site: "Lax" }
    ],
    console_events: [
      { level: "info", message: "POST {{host}}/api/v1/auth/login", timestamp: new Date().toISOString() },
      { level: "info", message: "PASS Status code is 200", timestamp: new Date().toISOString() },
      { level: "info", message: "PASS Response has status OK", timestamp: new Date().toISOString() }
    ],
    response_id: "resp-1",
    truncated: false
  };

  const mockHistory: ProjectHistoryEntry[] = [
    {
      id: "resp-1",
      request_id: reqId,
      request_name: "Login with Credentials",
      method: "POST",
      url: "{{host}}/api/v1/auth/login",
      status: 200,
      status_text: "OK",
      duration_ms: 245,
      body_size: 2760,
      created_at: new Date(Date.now() - 1000 * 60 * 5).toISOString()
    },
    {
      id: "resp-2",
      request_id: requests[1]?.id || reqId,
      request_name: requests[1]?.name || "Get Profile",
      method: requests[1]?.method || "GET",
      url: requests[1]?.url || "{{host}}/api/v1/profile",
      status: 200,
      status_text: "OK",
      duration_ms: 112,
      body_size: 940,
      created_at: new Date(Date.now() - 1000 * 60 * 32).toISOString()
    },
    {
      id: "resp-3",
      request_id: requests[2]?.id || reqId,
      request_name: requests[2]?.name || "Quote Remittance",
      method: requests[2]?.method || "POST",
      url: requests[2]?.url || "{{host}}/api/v1/quote",
      status: 201,
      status_text: "Created",
      duration_ms: 380,
      body_size: 1850,
      created_at: new Date(Date.now() - 1000 * 60 * 95).toISOString()
    },
    {
      id: "resp-4",
      request_id: requests[3]?.id || reqId,
      request_name: requests[3]?.name || "Validate Token",
      method: requests[3]?.method || "POST",
      url: requests[3]?.url || "{{host}}/api/v1/token/validate",
      status: 401,
      status_text: "Unauthorized",
      duration_ms: 95,
      body_size: 120,
      created_at: new Date(Date.now() - 1000 * 60 * 180).toISOString()
    }
  ];

  return { wsId, projId, folderId, reqId, envId, workspaces, projects, folders, requests, envs, mockVariables, projectRequestCounts, makeEnvVariables, fullRequest: defaultFullRequest, fullRequests, initialResponse, mockHistory };
}


type BrowserMock = ReturnType<typeof buildBrowserMock>;
let browserMockPromise: Promise<BrowserMock> | null = null;
function loadBrowserMock(): Promise<BrowserMock> {
  browserMockPromise ??= import("./browser-data.json").then((m) => buildBrowserMock(m.default));
  return browserMockPromise;
}

export function invokeDevMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return loadBrowserMock().then((mock) => invokeBrowserMock<T>(mock, cmd, args));
}

function invokeBrowserMock<T>(mock: BrowserMock, cmd: string, args?: Record<string, unknown>): Promise<T> {
  {
    switch (cmd) {
      case "list_workspaces":
        return Promise.resolve(mock.workspaces as unknown as T);
      case "list_projects": {
        const ws = (args?.workspaceId as string) || (args?.workspace_id as string);
        if (ws && ws !== "default" && ws !== "ws-foo") {
          return Promise.resolve(mock.projects.filter(p => p.workspace_id === ws) as unknown as T);
        }
        return Promise.resolve(mock.projects as unknown as T);
      }
      case "list_folders": {
        const pid = (args?.projectId as string) || (args?.project_id as string);
        if (pid) {
          return Promise.resolve(mock.folders.filter(f => f.project_id === pid) as unknown as T);
        }
        return Promise.resolve(mock.folders as unknown as T);
      }
      case "list_requests": {
        const pid = (args?.projectId as string) || (args?.project_id as string);
        if (pid) {
          return Promise.resolve(mock.requests.filter(r => r.project_id === pid) as unknown as T);
        }
        return Promise.resolve(mock.requests as unknown as T);
      }
      case "list_all_environments":
        return Promise.resolve(mock.envs as unknown as T);
      case "get_request": {
        const id = (args?.id as string) || mock.reqId;
        const found = mock.fullRequests.get(id);
        if (found) return Promise.resolve(found as unknown as T);
        const summary = mock.requests.find(r => r.id === id);
        const fallback: RequestFull = {
          id,
          project_id: summary?.project_id || mock.projId,
          folder_id: summary?.folder_id || mock.folderId,
          name: summary?.name || "Request",
          method: summary?.method || "GET",
          url: summary?.url || "{{host}}/api/v1/endpoint",
          query_params: [],
          headers: [
            { key: "Content-Type", value: "application/json", enabled: true, description: "" },
            { key: "Accept", value: "application/json", enabled: true, description: "" }
          ],
          body: '{\n  "status": "active"\n}',
          auth: { type: "none" },
          settings: { timeout_ms: null, follow_redirects: true, max_redirects: 10, verify_ssl: true, proxy_url: null, http_version: null },
          pre_request_script: null,
          post_request_script: null,
          description: summary?.name || "",
          sort_order: 1,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };
        mock.fullRequests.set(id, fallback);
        return Promise.resolve(fallback as unknown as T);
      }
      case "update_request":
      case "save_request": {
        const input = (args?.input as any) || {};
        const id = (args?.id as string) || input.id || mock.reqId;
        const current = mock.fullRequests.get(id) || mock.fullRequest;
        const updated: RequestFull = {
          ...current,
          ...input,
          body: input.clear_body ? null : (input.body !== undefined ? input.body : current.body),
          description: input.clear_description ? null : (input.description !== undefined ? input.description : current.description),
          pre_request_script: input.clear_pre_request_script ? null : (input.pre_request_script !== undefined ? input.pre_request_script : current.pre_request_script),
          post_request_script: input.clear_post_request_script ? null : (input.post_request_script !== undefined ? input.post_request_script : current.post_request_script),
        };
        mock.fullRequests.set(id, updated);
        const reqSummary = mock.requests.find(r => r.id === id);
        if (reqSummary) {
          if (input.name) reqSummary.name = input.name;
          if (input.method) reqSummary.method = input.method;
          if (input.url) reqSummary.url = input.url;
        }
        return Promise.resolve(updated as unknown as T);
      }
      case "create_request": {
        const input = (args?.input as any) || {};
        const newId = `req-custom-${Date.now()}`;
        const newSummary: RequestSummary = {
          id: newId,
          project_id: input.project_id || mock.projId,
          folder_id: input.folder_id || null,
          name: input.name || "New Request",
          method: input.method || "GET",
          url: input.url || "",
          sort_order: mock.requests.length + 1,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };
        mock.requests.push(newSummary);
        const newFull: RequestFull = {
          ...newSummary,
          query_params: input.query_params || [],
          headers: input.headers || [{ key: "Accept", value: "*/*", enabled: true, description: "" }],
          body: input.body ?? "",
          auth: input.auth || { type: "none" },
          settings: { timeout_ms: null, follow_redirects: true, max_redirects: 10, verify_ssl: true, proxy_url: null, http_version: null },
          pre_request_script: null,
          post_request_script: null,
          description: ""
        };
        mock.fullRequests.set(newId, newFull);
        return Promise.resolve(newFull as unknown as T);
      }
      case "duplicate_request": {
        const id = args?.id as string;
        const src = mock.fullRequests.get(id);
        const dupId = `req-${Date.now()}`;
        const dupSummary: RequestSummary = {
          id: dupId,
          project_id: src?.project_id || mock.projId,
          folder_id: src?.folder_id || null,
          name: `${src?.name || "Request"} Copy`,
          method: src?.method || "GET",
          url: src?.url || "",
          sort_order: mock.requests.length + 1,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };
        mock.requests.push(dupSummary);
        const dupFull: RequestFull = { ...(src || mock.fullRequest), ...dupSummary };
        mock.fullRequests.set(dupId, dupFull);
        return Promise.resolve(dupFull as unknown as T);
      }
      case "delete_request": {
        const id = args?.id as string;
        mock.requests = mock.requests.filter(r => r.id !== id);
        mock.fullRequests.delete(id);
        return Promise.resolve(null as unknown as T);
      }
      case "get_environment": {
        const targetId = (args?.id as string) || (args?.environmentId as string) || mock.envs[0].id;
        const env = mock.envs.find(e => e.id === targetId) || mock.envs[0];
        const envVars = mock.mockVariables.filter(v => v.scope === "environment" && (v.environment_id === targetId || (targetId === mock.envs[0].id && v.environment_id === "env-1")));
        return Promise.resolve({
          id: env.id,
          project_id: env.project_id,
          name: env.name,
          variables: envVars.length > 0 ? envVars.map(v => ({ key: v.key, value: v.value, enabled: v.enabled, is_secret: v.is_secret })) : [
            { key: "host", value: "https://ansari-remittance-dev.alansari.com", enabled: true, is_secret: false },
            { key: "device", value: "c8118037-c7cb-49d7-832f-a681320fa290", enabled: true, is_secret: false },
            { key: "phone", value: "+971501234567", enabled: true, is_secret: false }
          ],
          is_active: true,
          created_at: env.created_at,
          updated_at: env.updated_at
        } as unknown as T);
      }
      case "list_variables_for_scope": {
        const scope = args?.scope as VariableScope;
        const scopeRef = args?.scopeRef as string;
        if (scope === "global") {
          return Promise.resolve(mock.mockVariables.filter(v => v.scope === "global") as unknown as T);
        }
        if (scope === "environment") {
          let list = mock.mockVariables.filter(v => v.scope === "environment" && v.environment_id === scopeRef);
          if (list.length === 0 && scopeRef) {
            const envObj = mock.envs.find(e => e.id === scopeRef);
            const envName = envObj?.name || "";
            const newVars = mock.makeEnvVariables(scopeRef, envName);
            mock.mockVariables.push(...newVars);
            list = newVars;
          }
          return Promise.resolve(list as unknown as T);
        }
        return Promise.resolve([] as unknown as T);
      }
      case "create_variable": {
        const input = (args?.input as any) || {};
        const newVar: VariableView = {
          id: `var-${Date.now()}`,
          scope: input.scope || "global",
          project_id: input.project_id || mock.projId,
          environment_id: input.environment_id || null,
          request_id: null,
          key: input.key || "",
          value: input.value || "",
          enabled: input.enabled !== false,
          is_secret: Boolean(input.is_secret),
          is_local: Boolean(input.is_local),
          description: input.description || null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };
        mock.mockVariables.push(newVar);
        return Promise.resolve(newVar as unknown as T);
      }
      case "update_variable": {
        const input = (args?.input as any) || {};
        const found = mock.mockVariables.find(v => v.id === input.id);
        if (found) {
          if (input.key !== undefined) found.key = input.key;
          if (input.value !== undefined) found.value = input.value;
          if (input.enabled !== undefined) found.enabled = input.enabled;
          if (input.is_secret !== undefined) found.is_secret = input.is_secret;
          if (input.is_local !== undefined) found.is_local = input.is_local;
          found.updated_at = new Date().toISOString();
          return Promise.resolve(found as unknown as T);
        }
        return Promise.resolve(null as unknown as T);
      }
      case "delete_variable": {
        const id = args?.id as string;
        mock.mockVariables = mock.mockVariables.filter(v => v.id !== id);
        return Promise.resolve(null as unknown as T);
      }
      case "reveal_variable_value": {
        const id = args?.id as string;
        const found = mock.mockVariables.find(v => v.id === id);
        return Promise.resolve((found?.value ?? "") as unknown as T);
      }
      case "resolve_template":
      case "resolve_preview": {
        const template = (args?.template as string) || "";
        const envId = (args?.environmentId as string) || (args?.scopeRef as string) || null;
        const envVars = envId ? mock.mockVariables.filter(v => v.scope === "environment" && v.environment_id === envId) : [];
        const globVars = mock.mockVariables.filter(v => v.scope === "global");
        let resolved = template;
        const unresolved: string[] = [];
        const regex = /\{\{([^}]+)\}\}/g;
        let m;
        while ((m = regex.exec(template)) !== null) {
          const key = m[1].trim();
          const found = envVars.find(v => v.key === key && v.enabled) || globVars.find(v => v.key === key && v.enabled);
          if (found) {
            resolved = resolved.replace(m[0], found.value);
          } else {
            unresolved.push(key);
          }
        }
        return Promise.resolve({
          resolved,
          unresolved_variables: unresolved,
          missing: unresolved
        } as unknown as T);
      }
      case "diagnose_request":
        return Promise.resolve({
          has_unresolved: false,
          all_missing: [],
          variable_sources: { host: "environment" }
        } as unknown as T);
      case "get_workspace_git_settings":
        // Preview-only sample so the Git-dependent UI (Invite dialog, sync panel) can be exercised in a browser.
        return Promise.resolve({
          workspace_id: String(args?.workspaceId ?? "default"),
          repo_path: "C:/preview/api-specs",
          remote_url: "https://github.com/acme/api-specs.git",
          branch: "main",
          auto_sync: false,
          github_token: "ghp_preview_token",
          last_sync_at: null,
        } as unknown as T);
      case "invite_workspace_collaborator":
        return Promise.resolve({
          status: "invited",
          username: String(args?.username ?? ""),
          repo: "preview/browser-mock",
          role: args?.role === "viewer" ? "pull" : "push",
          invitation_url: null,
        } as unknown as T);
      case "list_workspace_collaborators":
        return Promise.resolve([
          { login: "octocat", avatar_url: null, role: "admin", pending: false },
          { login: "hubot", avatar_url: null, role: "read", pending: true },
        ] as unknown as T);
      case "get_project_request_counts":
        return Promise.resolve(mock.projectRequestCounts as unknown as T);
      case "get_sample_responses":
        return Promise.resolve([] as unknown as T);
      case "execute_request":
        return Promise.resolve(mock.initialResponse as unknown as T);
      case "list_response_summaries":
        return Promise.resolve([
          { id: "resp-1", request_id: mock.reqId, status: 200, status_text: "OK", duration_ms: 245, body_size: 2760, created_at: new Date().toISOString() }
        ] as unknown as T);
      case "list_project_history":
        return Promise.resolve(mock.mockHistory as unknown as T);
      case "clear_project_history":
        mock.mockHistory = [];
        return Promise.resolve(null as unknown as T);
      case "get_response":
        return Promise.resolve({
          id: (args?.id as string) || "resp-1",
          request_id: mock.reqId,
          status: 200,
          status_text: "OK",
          duration_ms: 245,
          body_size: 2760,
          headers: [
            { key: "Content-Type", value: "application/json", enabled: true, description: "" },
            { key: "Date", value: new Date().toUTCString(), enabled: true, description: "" },
            { key: "Server", value: "nginx", enabled: true, description: "" }
          ],
          content_type: "application/json",
          cookies: [
            { name: "session_id", value: "ansari_sess_9a8f7c6e", domain: "alansari.ae", path: "/", expires: null, secure: true, http_only: true, same_site: "Lax" }
          ],
          created_at: new Date().toISOString()
        } as unknown as T);
      case "send_request": {
        const reqId = (args?.requestId as string) || mock.reqId;
        const req = mock.fullRequests.get(reqId) || mock.requests.find(r => r.id === reqId) || mock.fullRequest;
        const newHistId = `resp-${Date.now()}`;
        const newEntry: ProjectHistoryEntry = {
          id: newHistId,
          request_id: reqId,
          request_name: (req as any).name || "Request",
          method: (req as any).method || "GET",
          url: (req as any).url || "{{host}}/api/v1/endpoint",
          status: 200,
          status_text: "OK",
          duration_ms: Math.floor(Math.random() * 180) + 45,
          body_size: 2760,
          created_at: new Date().toISOString()
        };
        mock.mockHistory.unshift(newEntry);
        return Promise.resolve({
          id: newHistId,
          request_id: reqId,
          status: 200,
          status_text: "OK",
          duration_ms: newEntry.duration_ms,
          body_size: 2760,
          headers: [
            { key: "Content-Type", value: "application/json", enabled: true, description: "" },
            { key: "Date", value: new Date().toUTCString(), enabled: true, description: "" },
            { key: "Server", value: "nginx", enabled: true, description: "" }
          ],
          content_type: "application/json",
          cookies: [
            { name: "session_id", value: "ansari_sess_9a8f7c6e", domain: "alansari.ae", path: "/", expires: null, secure: true, http_only: true, same_site: "Lax" }
          ],
          created_at: new Date().toISOString()
        } as unknown as T);
      }
      case "get_response_body":
        return Promise.resolve({
          text: JSON.stringify({
            meta: { status: "OK" },
            response: {
              status: "success",
              message: "Registered Wallet",
              document_in_grace_period: false,
              document_expired: false,
              flow: "default"
            }
          }, null, 2),
          truncated: false
        } as unknown as T);
      case "is_ai_configured":
        return Promise.resolve(true as unknown as T);
      case "get_ai_settings":
        return Promise.resolve({
          provider: "openai",
          api_key_masked: "sk-••••••••••••",
          model: "gpt-4o",
          endpoint_url: null,
          has_key: true
        } as unknown as T);
      case "get_console_events":
        return Promise.resolve([
          { id: "e1", level: "error", message: "Failed to verify certificate for legacy endpoint", timestamp: new Date().toISOString() },
          { id: "e2", level: "warn", message: "Deprecated auth header detected", timestamp: new Date().toISOString() }
        ] as unknown as T);
      case "get_system_diagnostics":
        return Promise.resolve({
          process_rss_bytes: 42 * 1024 * 1024,
          db_size_bytes: 140 * 1024,
          db_wal_size_bytes: 0,
          total_projects: 6,
          total_requests: 11,
          total_environments: 1,
          total_variables: 3,
          total_history_entries: 4
        } as unknown as T);
      case "get_git_status":
        return Promise.resolve({
          branch: "main",
          status_kind: "clean",
          ahead: 0,
          behind: 0,
          staged: [],
          unstaged: [],
          untracked: [],
          conflict_files: [],
          has_conflicts: false
        } as unknown as T);
      case "create_environment": {
        const input = (args as any)?.input;
        const newEnv: Environment = {
          id: `env-${Date.now()}`,
          project_id: input?.project_id || "proj-default",
          name: input?.name || "New Environment",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        (mock.envs as any).push(newEnv);
        return Promise.resolve(newEnv as unknown as T);
      }
      case "generate_curl_snippet": {
        const reqId = (args?.requestId as string) || mock.reqId;
        const target = (args?.target as string) || "bash";
        const mode = (args?.mode as string) || "placeholder";
        const req = mock.fullRequests.get(reqId) || mock.requests.find(r => r.id === reqId) || mock.fullRequest;
        const method = (req as any).method || "POST";
        const rawUrl = (req as any).url || "{{host}}/api/v1/auth/login";
        const url = (mode === "resolved" && rawUrl.includes("{{host}}"))
          ? rawUrl.replace("{{host}}", "https://uat.alansari.ae")
          : rawUrl;

        const bodyContent = (req as any).body || JSON.stringify({ phone: "{{phone}}", pin: "{{pin}}" }, null, 2);

        if (target === "power_shell") {
          return Promise.resolve(`$headers = @{\n  "Content-Type" = "application/json"\n  "Accept" = "application/json"\n}\n$body = @'\n${bodyContent}\n'@\n$response = Invoke-RestMethod -Uri "${url}" -Method ${method} -Headers $headers -Body $body` as unknown as T);
        }
        if (target === "java_script_fetch") {
          return Promise.resolve(`const myHeaders = new Headers();\nmyHeaders.append("Content-Type", "application/json");\n\nconst raw = JSON.stringify(${bodyContent});\n\nconst requestOptions = {\n  method: "${method}",\n  headers: myHeaders,\n  body: raw,\n  redirect: "follow"\n};\n\nfetch("${url}", requestOptions)\n  .then((response) => response.text())\n  .then((result) => console.log(result))\n  .catch((error) => console.error(error));` as unknown as T);
        }
        if (target === "node_fetch") {
          return Promise.resolve(`const fetch = (...args) => import('node-fetch').then(({default: fetch}) => fetch(...args));\n\nconst response = await fetch("${url}", {\n  method: "${method}",\n  headers: {\n    "Content-Type": "application/json"\n  },\n  body: JSON.stringify(${bodyContent})\n});\nconst data = await response.json();\nconsole.log(data);` as unknown as T);
        }
        if (target === "python_requests") {
          return Promise.resolve(`import requests\nimport json\n\nurl = "${url}"\npayload = json.dumps(${bodyContent})\nheaders = {\n  'Content-Type': 'application/json'\n}\n\nresponse = requests.request("${method}", url, headers=headers, data=payload)\nprint(response.text)` as unknown as T);
        }
        if (target === "preload") {
          return Promise.resolve(`<link rel="preload" href="${url}" as="fetch" crossorigin="anonymous">` as unknown as T);
        }
        if (target === "har") {
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
                  postData: { mimeType: "application/json", text: bodyContent }
                }
              }]
            }
          };
          return Promise.resolve(JSON.stringify(harObj, null, 2) as unknown as T);
        }

        const isCmd = target === "windows_cmd";
        const sep = isCmd ? " ^\n" : " \\\n";
        const quote = isCmd ? '"' : "'";

        const lines = [
          `curl --location${method !== 'GET' ? ` --request ${method}` : ''} ${quote}${url}${quote}`,
          `--header ${quote}Content-Type: application/json${quote}`,
          `--header ${quote}Accept: application/json${quote}`,
          `--data-raw ${quote}${bodyContent}${quote}`
        ];
        return Promise.resolve(lines.join(sep) as unknown as T);
      }
      default:
        return Promise.resolve(null as unknown as T);
    }
  }
}

