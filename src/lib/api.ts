import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/** True only inside the actual Tauri desktop shell — absent when this app is opened as a plain
 * web page (e.g. `vite dev` visited directly in a browser instead of via `tauri dev`), which has
 * no Rust backend to call into. */
function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

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
  if (!import.meta.env.DEV) {
    return Promise.reject(
      new Error("This runs as a desktop app — open it via the app window, not a browser tab."),
    );
  }
  browserMockPromise ??= import("./browser-data.json").then((m) => buildBrowserMock(m.default));
  return browserMockPromise;
}

/** Same call shape as the real `invoke`, but checked up front — every one of this file's ~80
 * call sites goes through here. Outside the Tauri shell it serves the in-memory mock (so the UI
 * can be previewed in a plain browser or preview server). In Tauri runtime it calls tauriInvoke. */
function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    return loadBrowserMock().then((mock) => invokeBrowserMock<T>(mock, cmd, args));
  }
  return tauriInvoke<T>(cmd, args);
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
      case "invite_workspace_collaborator":
        return Promise.resolve({
          status: "invited",
          username: String(args?.username ?? ""),
          repo: "preview/browser-mock",
          role: args?.role === "viewer" ? "pull" : "push",
          invitation_url: null,
        } as unknown as T);
      case "list_workspace_collaborators":
        return Promise.resolve([] as unknown as T);
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
      case "get_workspace_git_settings":
        return Promise.resolve({ repo_path: "d:/hamada/postman", auto_sync: true } as unknown as T);
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

export interface Project {
  id: string;
  name: string;
  default_environment_id: string | null;
  workspace_id: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Workspace {
  id: string;
  name: string;
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
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export type SearchField = "name" | "url" | "body";

export interface RequestSearchResult {
  id: string;
  project_id: string;
  project_name: string;
  folder_id: string | null;
  name: string;
  method: string;
  url: string;
}

export interface Folder {
  id: string;
  project_id: string;
  name: string;
  parent_folder_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface NewFolderInput {
  project_id: string;
  name: string;
  parent_folder_id?: string | null;
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
  status_text: string;
  headers: HeaderEntry[];
  body?: string | null;
  content_type?: string | null;
  created_at: string;
}

export interface NewSampleResponseInput {
  request_id: string;
  name: string;
  status: number;
  status_text: string;
  headers: HeaderEntry[];
  body?: string | null;
  content_type?: string | null;
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
  settings: RequestSettings;
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

export type SnippetTarget =
  | "bash"
  | "power_shell"
  | "windows_cmd"
  | "python_requests"
  | "java_script_fetch"
  | "node_fetch"
  | "preload"
  | "har";

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
  createProject: (name: string, workspaceId: string) =>
    invoke<Project>("create_project", { name, workspaceId }),
  reorderProjects: (workspaceId: string, orderedIds: string[]) =>
    invoke<void>("reorder_projects", { workspaceId, orderedIds }),
  listProjects: (workspaceId: string) => invoke<Project[]>("list_projects", { workspaceId }),
  getProject: (id: string) => invoke<Project>("get_project", { id }),
  createWorkspace: (name: string) => invoke<Workspace>("create_workspace", { name }),
  listWorkspaces: () => invoke<Workspace[]>("list_workspaces"),
  updateWorkspace: (id: string, name: string) =>
    invoke<Workspace>("update_workspace", { input: { id, name } }),
  deleteWorkspace: (id: string) => invoke<void>("delete_workspace", { id }),
  getProjectRequestCounts: () => invoke<Record<string, number>>("get_project_request_counts"),
  updateProject: (input: UpdateProjectInput) =>
    invoke<Project>("update_project", { input }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  createRequest: (input: NewRequestInput) =>
    invoke<RequestFull>("create_request", { input }),
  listRequests: (projectId: string) =>
    invoke<RequestSummary[]>("list_requests", { projectId }),
  searchRequestsInWorkspace: (workspaceId: string, query: string, fields: SearchField[]) =>
    invoke<RequestSearchResult[]>("search_requests_in_workspace", { workspaceId, query, fields }),
  reorderRequests: (projectId: string, folderId: string | null, orderedIds: string[]) =>
    invoke<void>("reorder_requests", { projectId, folderId, orderedIds }),
  getRequest: (id: string) => invoke<RequestFull>("get_request", { id }),
  updateRequest: (input: UpdateRequestInput) =>
    invoke<RequestFull>("update_request", { input }),
  deleteRequest: (id: string) => invoke<void>("delete_request", { id }),

  createFolder: (input: NewFolderInput) => invoke<Folder>("create_folder", { input }),
  reorderFolders: (projectId: string, parentFolderId: string | null, orderedIds: string[]) =>
    invoke<void>("reorder_folders", { projectId, parentFolderId, orderedIds }),
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
  clearProjectHistory: (projectId: string) =>
    invoke<void>("clear_project_history", { projectId }),
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
    invoke<ParsedCurlRequest>("import_curl", { command: curlCommand }),

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

  importPostmanCollection: (collectionJson: string, targetProjectId: string | null | undefined, workspaceId: string) =>
    invoke<CollectionImportReport>("import_postman_collection", {
      collectionJson,
      targetProjectId: targetProjectId ?? null,
      workspaceId,
    }),

  importPostmanEnvironment: (environmentJson: string, targetProjectId: string) =>
    invoke<EnvironmentImportReport>("import_postman_environment", {
      environmentJson,
      targetProjectId,
    }),

  importLocalPostmanWorkspace: (rootPath: string, workspaceId: string) =>
    invoke<LocalWorkspaceImportReport>("import_local_postman_workspace", { rootPath, workspaceId }),

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

  importProjectFile: (fileContent: string, targetProjectId: string | null | undefined, workspaceId: string) =>
    invoke<Project>("import_project_file", {
      fileContent,
      targetProjectId: targetProjectId ?? null,
      workspaceId,
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
    targetProjectId: string | null | undefined,
    workspaceId: string,
  ) =>
    invoke<Project>("load_project_from_repo", {
      directory,
      targetProjectId: targetProjectId ?? null,
      workspaceId,
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

  saveWorkspaceToRepo: (
    workspaceId: string,
    directory: string,
    includeSecrets?: boolean,
  ) =>
    invoke<string[]>("save_workspace_to_repo", {
      workspaceId,
      directory,
      includeSecrets: includeSecrets ?? false,
    }),

  loadWorkspaceFromRepo: (workspaceId: string, directory: string) =>
    invoke<WorkspaceImportReport>("load_workspace_from_repo", { workspaceId, directory }),

  getWorkspaceGitSettings: (workspaceId: string) =>
    invoke<WorkspaceGitSettings | null>("get_workspace_git_settings", { workspaceId }),

  saveWorkspaceGitSettings: (settings: WorkspaceGitSettings) =>
    invoke<void>("save_workspace_git_settings", { settings }),

  findLegacyGitSettingsForWorkspace: (workspaceId: string) =>
    invoke<LegacyGitSettingsCandidate[]>("find_legacy_git_settings_for_workspace", { workspaceId }),

  verifyGitHubToken: (token: string) =>
    invoke<GitHubUser>("verify_github_token", { token }),
  inviteWorkspaceCollaborator: (workspaceId: string, username: string, role: CollaboratorRole) =>
    invoke<InviteResult>("invite_workspace_collaborator", { workspaceId, username, role }),
  listWorkspaceCollaborators: (workspaceId: string) =>
    invoke<GitHubCollaborator[]>("list_workspace_collaborators", { workspaceId }),

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

/** Same shape as ProjectGitSettings, keyed by workspace — one repo covers every project in it. */
export interface WorkspaceGitSettings {
  workspace_id: string;
  repo_path?: string | null;
  remote_url?: string | null;
  branch: string;
  auto_sync: boolean;
  github_token?: string | null;
  last_sync_at?: string | null;
}

export interface WorkspaceImportReport {
  updated_projects: Project[];
  created_projects: Project[];
  warnings: string[];
}

/** A leftover per-project git config found in a workspace with no settings of its own yet —
 * offered to the user to adopt, never applied automatically. */
export interface LegacyGitSettingsCandidate {
  project_id: string;
  project_name: string;
  settings: ProjectGitSettings;
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

/** Access level granted to an invited GitHub user: viewer = read-only, collaborator = read + write. */
export type CollaboratorRole = "viewer" | "collaborator" | "admin";

export interface InviteResult {
  /** "invited" = pending acceptance by the invitee, "updated" = they already had access. */
  status: "invited" | "updated";
  username: string;
  repo: string;
  role: string;
  invitation_url?: string | null;
}

export interface GitHubCollaborator {
  login: string;
  avatar_url?: string | null;
  role: string;
  pending: boolean;
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
  project_ids: string[];
}

export type AiProviderKind = "anthropic" | "openai" | "google" | "custom";

export interface AiSettings {
  provider: AiProviderKind;
  api_key?: string | null;
  model: string;
  base_url?: string | null;
  is_configured: boolean;
}

export interface UpdateAiSettingsInput {
  provider?: AiProviderKind | null;
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


