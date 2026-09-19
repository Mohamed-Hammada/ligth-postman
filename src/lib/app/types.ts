// Types shared by the app controller and the view components (hoisted out of the old +page.svelte script).
import type {
  Auth,
  CollectionImportReport,
  Environment,
  EnvironmentWithProject,
  EnvironmentImportReport,
  Folder,
  GeneratedApiDefinition,
  FormDataPart,
  HeaderEntry,
  LocalWorkspaceImportReport,
  Project,
  QueryParam,
  UrlEncodedItem,
  RequestFull,
  RequestDiagnostics,
  RequestSettings,
  RequestSummary,
  RequestSearchResult,
  SearchField,
  ResolvedTemplate,
  ResponseMeta,
  ResponseSummary,
  SnippetMode,
  SnippetTarget,
  VariableView,
  ConsoleEvent,
  ConsoleLevel,
  GitStatus,
  GitCommit,
  WorkspaceGitSettings,
  LegacyGitSettingsCandidate,
  WorkspaceImportReport,
  GitHubUser,
  GitHubRepoInfo,
  AiSettings,
  AiProviderKind,
  DiscoveredEndpoint,
  GeneratedTestsAndDocs,
  SampleResponse,
  SourceProjectReport,
  UpdateAiSettingsInput,
  SystemDiagnostics,
  Workspace,
  ProjectHistoryEntry,
  ConflictVersions,
} from "$lib/api";
import type { Locale } from "$lib/i18n";

export type ScreenId = "workspace" | "environments" | "globals" | "git" | "launcher" | "history" | "settings";

export type ThemeMode = "light" | "dark" | "terminal" | "blueprint";

export type PaletteScope = "all" | "projects" | "apis";

export type ShortcutId = "commandPalette" | "sendRequest" | "saveRequest" | "newRequest" | "nextTab" | "prevTab" | "closeTab";

export type PaletteItem = { kind: "project" | "request"; label: string; method?: string; hint: string; project?: string; onSelect: () => void };

export interface RequestTab {
    id: string;
    name: string;
    method: string;
    tabType?: "request" | "env" | "doc" | "spec" | "mock" | "dataset" | "flow";
    envId?: string;
    data?: any;
  }

export interface RequestDraft {
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

export type SidebarSearchScope = "all" | "projects" | "apis";

export type ProjectSortField = "name" | "created" | "updated" | "custom";

export type RequestSortField = "name" | "method" | "created" | "updated" | "custom";

export type SortDir = "asc" | "desc";

export interface DetailHeaderRow {
    key: string;
    value: string;
    enabled?: boolean;
  }

export interface DetailCookieRow {
    name: string;
    value: string;
    domain: string;
    path: string;
    http_only: boolean;
    secure: boolean;
  }

export type UrlToken = { type: "text"; text: string } | { type: "var"; name: string; raw: string };

export interface AutocompleteItem {
    insertText: string;
    label: string;
    detail?: string;
  }

export interface AutocompleteState {
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
