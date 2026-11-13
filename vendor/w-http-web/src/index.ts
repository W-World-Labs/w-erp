// @w/http-web — pure-TypeScript frontend HTTP client + ApiError superset.
//
// The JS twin of the fleet's hand-rolled frontend fetch wrappers (OneMan,
// W-DaaS/data-asset, w-maas, quantA, …). Designed as a FAITHFUL SUPERSET: every
// consumer's CURRENT request shape (method / URL / headers / body) and error
// behaviour is reproducible purely through config — defaults are the most common
// shape, divergences are absorbed as injectable callbacks (baseUrl resolver,
// token getter, envelope unwrap, onUnauthorized / onError / span hooks), NOT a
// winner-takes-all pick. Zero runtime dependencies, tree-shakeable
// (`sideEffects: false`), ESM + CJS.
//
// Per-consumer config fragments live in `presets`; see the README migration map.

/* ============================== ApiError ============================== */

/** Field bag for the superset {@link ApiError}. Every consumer's readable error
 *  surface is a subset of these fields. */
export interface ApiErrorInit {
  /** Human message. Falls back to `detail`, then `HTTP <status>`. */
  message?: string;
  /** HTTP status code (quantA / data-asset / w-maas). Default 0. */
  status?: number;
  /** HTTP status text (w-maas). Default `''`. */
  statusText?: string;
  /** Parsed response body (quantA `body`, w-maas structured body). */
  body?: unknown;
  /** Raw response text (data-asset `responseText`). Default `''`. */
  responseText?: string;
  /** Stable error code (OneMan `code`). Default `''`. */
  code?: string;
  /** Provider error code (w-maas `errorCode`). Falls back to `code`. */
  errorCode?: string;
  /** Provider error category (w-maas `errorType`). Default `''`. */
  errorType?: string;
  /** Long-form detail (w-maas `detail`). Falls back to `message`. */
  detail?: string;
  /** Remediation hint (OneMan `hint`). */
  hint?: string;
  /** Secondary detail string (OneMan `details`). */
  details?: string;
}

/**
 * Rich error superset thrown by {@link createHttpClient}. Carries the union of
 * every fleet client's error fields so each consumer's callers keep reading the
 * same property they read today (`.status`, `.body`, `.responseText`, `.code`,
 * `.errorCode`, `.errorType`, `.detail`, `.hint`, `.message`). `instanceof
 * Error` holds; `name` is always `'ApiError'`.
 */
export class ApiError extends Error {
  status: number;
  statusText: string;
  body: unknown;
  responseText: string;
  code: string;
  errorCode: string;
  errorType: string;
  detail: string;
  hint?: string;
  details?: string;

  constructor(init: ApiErrorInit = {}) {
    const message =
      init.message ??
      init.detail ??
      (init.status != null ? `HTTP ${init.status}` : 'Request failed');
    super(message);
    this.name = 'ApiError';
    this.status = init.status ?? 0;
    this.statusText = init.statusText ?? '';
    this.body = init.body;
    this.responseText = init.responseText ?? '';
    this.code = init.code ?? '';
    this.errorCode = init.errorCode ?? init.code ?? '';
    this.errorType = init.errorType ?? '';
    this.detail = init.detail ?? message;
    this.hint = init.hint;
    this.details = init.details;
  }
}

/* ============================== Config ============================== */

/** Opaque telemetry span returned by `onRequestStart`, threaded to `onRequestEnd`. */
export type SpanHandle = unknown;

/** Per-call options. A `RequestInit` plus the optional `silent401` flag some
 *  consumers (data-asset) thread through to gate the 401 redirect. */
export interface FetchOptions extends RequestInit {
  /** Suppress the `onUnauthorized` 401 hook for this call (data-asset). */
  silent401?: boolean;
}

/** Immutable context describing one in-flight request. */
export interface RequestContext {
  path: string;
  url: string;
  method: string;
  options: FetchOptions;
}

/** Context for a non-2xx response, after the body has been read once. */
export interface ErrorResponseInfo extends RequestContext {
  status: number;
  statusText: string;
  /** Raw response text (read exactly once). */
  raw: string;
  /** Tolerantly-parsed body: object | string | null. */
  parsed: unknown;
  response: Response;
}

/** Context handed to `onError` for both network and HTTP failures. */
export interface ErrorHookInfo {
  kind: 'network' | 'http';
  path: string;
  method: string;
  status?: number;
  raw?: string;
}

/** How a non-empty success body is parsed. */
export type ParseMode =
  /** `JSON.parse` — throws on invalid JSON (mirrors `res.json()`). */
  | 'json'
  /** Tolerant — invalid JSON is returned verbatim as a string (quantA). */
  | 'safe';

/** Content-Type policy for the request body. */
export type ContentTypeMode =
  /** Always set (overridable per-call). w-maas. */
  | 'always'
  /** Set unless the body is `FormData`. OneMan. */
  | 'non-formdata'
  /** Set only when the body is a string and no per-call CT is present. data-asset. */
  | 'string-only'
  /** Never set by the core (the caller / a method wrapper sets it). quantA. */
  | 'none';

type HeadersRecord = Record<string, string>;

/**
 * Sentinel returned from `onUnauthorized` to make `fetchJSON` resolve to a
 * never-settling promise (data-asset hard-redirects then "hangs" so the caller
 * never sees a value or an error). Returning `undefined` instead falls through
 * to normal error construction + throw.
 */
export const HANG_FOREVER: unique symbol = Symbol('@w/http-web:hang');

export interface HttpClientConfig {
  // ── URL ──────────────────────────────────────────────────────────────────
  /** Base URL, or a resolver (re-read per request). Default `''` (same-origin). */
  baseUrl?: string | (() => string);
  /** Join `base`+`path`. Default `base + path` (raw concat). quantA overrides. */
  buildUrl?: (base: string, path: string) => string;
  /** Injected `fetch` (tests / non-browser). Default `globalThis.fetch`. */
  fetchImpl?: typeof fetch;

  // ── Auth + headers ─────────────────────────────────────────────────────────
  /** Token getter; when truthy, sets `<authHeader>: <authScheme> <token>`. */
  getToken?: () => string | null | undefined;
  /** Auth header name. Default `'Authorization'`. */
  authHeader?: string;
  /** Auth scheme prefix. Default `'Bearer'`. */
  authScheme?: string;
  /** Auth header wins over per-call headers (w-maas). Default `false`. */
  authOverridesHeaders?: boolean;
  /** Static `Accept` header (quantA: `'application/json'`). */
  accept?: string;
  /** Content-Type policy. Default `'non-formdata'`. */
  contentType?: ContentTypeMode;
  /** Content-Type value. Default `'application/json'`. */
  contentTypeValue?: string;
  /** Base headers merged first (lowest precedence). */
  defaultHeaders?: HeadersRecord | (() => HeadersRecord);
  /** `credentials` mode (w-maas: `'include'`). Per-call `credentials` wins. */
  credentials?: RequestCredentials;

  // ── Success parsing ─────────────────────────────────────────────────────────
  /** Non-empty success body parse mode. Default `'json'`. */
  parse?: ParseMode;
  /** Value for an empty success body. Default `undefined` (data-asset `{}`, quantA `null`). */
  emptyBody?: unknown;
  /** Unwrap an envelope on success ({data}/{ok,data}/…). Default identity. */
  unwrap?: (parsed: unknown, ctx: RequestContext & { response: Response }) => unknown;

  // ── Errors ───────────────────────────────────────────────────────────────────
  /** Extract the error message from a non-2xx response. Default: best-effort
   *  `parsed.error.message | parsed.error | parsed.message | parsed.detail | raw | HTTP <status>`. */
  errorMessage?: (info: ErrorResponseInfo) => string;
  /** Build the thrown error. Default: {@link ApiError} carrying status/body/raw. */
  errorFactory?: (info: ErrorResponseInfo, message: string) => Error;
  /** Invoked on a 401 before error construction. May redirect / clear token,
   *  then: throw (OneMan), return {@link HANG_FOREVER} (data-asset), or return
   *  `undefined` to fall through to the normal error path. */
  onUnauthorized?: (info: ErrorResponseInfo) => void | typeof HANG_FOREVER;
  /** Side-channel error reporter (telemetry / error_logs). */
  onError?: (err: unknown, info: ErrorHookInfo) => void;

  // ── Telemetry span ───────────────────────────────────────────────────────────
  /** Start a span before `fetch`; the returned handle is threaded to `onRequestEnd`. */
  onRequestStart?: (ctx: RequestContext) => SpanHandle;
  /** Finish the span. `attrs` use OpenTelemetry HTTP semantic-convention keys
   *  (`http.method` / `http.route` / `http.status_code` / `error.type` / `error.message`). */
  onRequestEnd?: (span: SpanHandle, outcome: 'ok' | 'error', attrs: Record<string, unknown>) => void;
}

/* ============================== Helpers ============================== */

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** Empty → `null`; else `JSON.parse`, falling back to the raw string. Used for
 *  ALL error bodies (every consumer tolerates a non-JSON error body). */
function tolerantParse(raw: string): unknown {
  if (raw === '') return null;
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

/** Normalise `HeadersInit` (Headers | [k,v][] | record | undefined) to a record. */
function toHeaderRecord(init: HeadersInit | undefined): HeadersRecord {
  const out: HeadersRecord = {};
  if (!init) return out;
  if (typeof Headers !== 'undefined' && init instanceof Headers) {
    init.forEach((v, k) => {
      out[k] = v;
    });
    return out;
  }
  if (Array.isArray(init)) {
    for (const [k, v] of init) out[k] = v as string;
    return out;
  }
  return { ...(init as HeadersRecord) };
}

function hasHeader(record: HeadersRecord, name: string): boolean {
  const lower = name.toLowerCase();
  return Object.keys(record).some((k) => k.toLowerCase() === lower);
}

function defaultErrorMessage(info: ErrorResponseInfo): string {
  const p = info.parsed;
  if (isRecord(p)) {
    const e = p.error;
    if (isRecord(e) && typeof e.message === 'string' && e.message) return e.message;
    if (typeof e === 'string' && e) return e;
    if (typeof p.message === 'string' && p.message) return p.message;
    if (typeof p.detail === 'string' && p.detail) return p.detail;
  }
  return info.raw || `HTTP ${info.status}`;
}

function buildHeaders(config: HttpClientConfig, options: FetchOptions): HeadersRecord {
  const out: HeadersRecord = {};

  const dh =
    typeof config.defaultHeaders === 'function' ? config.defaultHeaders() : config.defaultHeaders;
  Object.assign(out, dh ?? {});

  if (config.accept) out['Accept'] = config.accept;

  const body = options.body;
  const isFormData = typeof FormData !== 'undefined' && body instanceof FormData;
  const mode = config.contentType ?? 'non-formdata';
  const callHeaders = toHeaderRecord(options.headers);

  let setCT: boolean;
  if (mode === 'always') setCT = true;
  else if (mode === 'non-formdata') setCT = !isFormData;
  else if (mode === 'string-only') setCT = typeof body === 'string' && !hasHeader(callHeaders, 'content-type');
  else setCT = false;
  if (setCT) out['Content-Type'] = config.contentTypeValue ?? 'application/json';

  const token = config.getToken?.();
  const authHeader = config.authHeader ?? 'Authorization';
  const authValue = token ? `${config.authScheme ?? 'Bearer'} ${token}` : undefined;

  if (config.authOverridesHeaders) {
    Object.assign(out, callHeaders);
    if (authValue) out[authHeader] = authValue;
  } else {
    if (authValue) out[authHeader] = authValue;
    Object.assign(out, callHeaders);
  }
  return out;
}

/* ============================== Client ============================== */

export interface HttpClient {
  /** Core: resolve URL, build headers, fetch, parse / unwrap, or throw. */
  fetchJSON: <T = unknown>(path: string, options?: FetchOptions) => Promise<T>;
  /** Alias of {@link fetchJSON} (consumers name it `request`). */
  request: <T = unknown>(path: string, options?: FetchOptions) => Promise<T>;
  get: <T = unknown>(path: string, options?: FetchOptions) => Promise<T>;
  /** Body is `JSON.stringify`'d unless it is `FormData` / a string. */
  post: <T = unknown>(path: string, body?: unknown, options?: FetchOptions) => Promise<T>;
  put: <T = unknown>(path: string, body?: unknown, options?: FetchOptions) => Promise<T>;
  patch: <T = unknown>(path: string, body?: unknown, options?: FetchOptions) => Promise<T>;
  del: <T = unknown>(path: string, options?: FetchOptions) => Promise<T>;
  config: HttpClientConfig;
}

function encodeBody(body: unknown): BodyInit | undefined {
  if (body === undefined || body === null) return undefined;
  if (typeof body === 'string') return body;
  if (typeof FormData !== 'undefined' && body instanceof FormData) return body;
  if (typeof Blob !== 'undefined' && body instanceof Blob) return body;
  if (body instanceof ArrayBuffer || ArrayBuffer.isView(body)) return body as BodyInit;
  return JSON.stringify(body);
}

/**
 * Build a configured HTTP client. Each fleet consumer is reproduced by supplying
 * the matching {@link HttpClientConfig} (see `presets`); zero config yields a
 * sane same-origin JSON client that throws {@link ApiError}.
 */
export function createHttpClient(config: HttpClientConfig = {}): HttpClient {
  const fetchImpl = config.fetchImpl ?? (globalThis.fetch as typeof fetch);

  async function fetchJSON<T = unknown>(path: string, options: FetchOptions = {}): Promise<T> {
    const base = typeof config.baseUrl === 'function' ? config.baseUrl() : config.baseUrl ?? '';
    const url = config.buildUrl ? config.buildUrl(base, path) : `${base}${path}`;
    const method = options.method ?? 'GET';
    const headers = buildHeaders(config, options);
    const ctx: RequestContext = { path, url, method, options };

    const span = config.onRequestStart?.(ctx);

    const init: RequestInit = { ...options, headers };
    const credentials = options.credentials ?? config.credentials;
    if (credentials != null) init.credentials = credentials;

    let res: Response;
    try {
      res = await fetchImpl(url, init);
    } catch (networkErr) {
      config.onRequestEnd?.(span, 'error', {
        'http.method': method,
        'http.route': path,
        'error.type': 'NetworkError',
      });
      config.onError?.(networkErr, { kind: 'network', path, method });
      throw networkErr;
    }

    if (!res.ok) {
      const raw = await res.text().catch(() => '');
      const parsed = tolerantParse(raw);
      const info: ErrorResponseInfo = {
        ...ctx,
        status: res.status,
        statusText: res.statusText,
        raw,
        parsed,
        response: res,
      };

      if (res.status === 401 && config.onUnauthorized && !options.silent401) {
        const result = config.onUnauthorized(info);
        if (result === HANG_FOREVER) {
          config.onRequestEnd?.(span, 'error', {
            'http.method': method,
            'http.route': path,
            'http.status_code': 401,
          });
          return new Promise<never>(() => {});
        }
      }

      const message = config.errorMessage ? config.errorMessage(info) : defaultErrorMessage(info);
      const err = config.errorFactory
        ? config.errorFactory(info, message)
        : new ApiError({
            message,
            status: res.status,
            statusText: res.statusText,
            body: parsed,
            responseText: raw,
          });

      config.onRequestEnd?.(span, 'error', {
        'http.method': method,
        'http.route': path,
        'http.status_code': res.status,
        'error.message': err.message,
      });
      config.onError?.(err, { kind: 'http', path, method, status: res.status, raw });
      throw err;
    }

    const raw = await res.text();
    config.onRequestEnd?.(span, 'ok', {
      'http.method': method,
      'http.route': path,
      'http.status_code': res.status,
    });

    if (raw === '') return config.emptyBody as T;
    const parsed = config.parse === 'safe' ? tolerantParse(raw) : JSON.parse(raw);
    const value = config.unwrap ? config.unwrap(parsed, { ...ctx, response: res }) : parsed;
    return value as T;
  }

  const post = <T = unknown>(path: string, body?: unknown, options: FetchOptions = {}) =>
    fetchJSON<T>(path, { ...options, method: 'POST', body: encodeBody(body) });
  const put = <T = unknown>(path: string, body?: unknown, options: FetchOptions = {}) =>
    fetchJSON<T>(path, { ...options, method: 'PUT', body: encodeBody(body) });
  const patch = <T = unknown>(path: string, body?: unknown, options: FetchOptions = {}) =>
    fetchJSON<T>(path, { ...options, method: 'PATCH', body: encodeBody(body) });

  return {
    fetchJSON,
    request: fetchJSON,
    get: <T = unknown>(path: string, options: FetchOptions = {}) =>
      fetchJSON<T>(path, { ...options, method: options.method ?? 'GET' }),
    post,
    put,
    patch,
    del: <T = unknown>(path: string, options: FetchOptions = {}) =>
      fetchJSON<T>(path, { ...options, method: 'DELETE' }),
    config,
  };
}

/** Zero-config same-origin JSON client (`Bearer`-less, throws {@link ApiError}). */
const defaultClient = createHttpClient();

/** Standalone bound {@link createHttpClient}().fetchJSON for the zero-config case. */
export const fetchJSON = defaultClient.fetchJSON;

/* ============================== Presets ============================== */

/** quantA-style smart join: empty base → path; relative path → `base/path`. */
export function smartJoinUrl(base: string, path: string): string {
  if (!base) return path;
  if (!path) return base;
  return path.startsWith('/') ? `${base}${path}` : `${base}/${path}`;
}

/**
 * Behaviour-preserving config fragments, one per consuming frontend. Spread into
 * `createHttpClient({ ...presets.<name>, baseUrl, getToken, … })`; the static
 * knobs (content-type / parse / error extraction) live here, the app-specific
 * dynamic hooks (`baseUrl` resolver, `getToken`, `onUnauthorized`, `onError`,
 * span) are supplied at the call site. See the README migration map.
 */
export const presets = {
  /** OneMan `integrations/api/client.ts`: Bearer session, JSON-unless-FormData,
   *  plain-text error message, throws a bare `Error` (its `errors.ts` `ApiError`
   *  {code,details,hint} is modelled by the superset {@link ApiError}). 401 is
   *  handled by an injected `onUnauthorized` (clear session + redirect + throw). */
  oneman: {
    contentType: 'non-formdata',
    authScheme: 'Bearer',
    emptyBody: undefined,
    parse: 'json',
    errorMessage: (i) => i.raw || `HTTP ${i.status}`,
    errorFactory: (_i, message) => new Error(message),
  },
  /** W-DaaS / data-asset `frontend/src/api/client.ts`: same-origin, no token,
   *  CT only for string bodies, `{}` for empty body, `ApiError(status,
   *  responseText)`; couples a telemetry span (`onRequestStart`/`onRequestEnd`)
   *  + `onError` + a gated 401 redirect (`onUnauthorized` → {@link HANG_FOREVER}). */
  dataAsset: {
    contentType: 'string-only',
    emptyBody: {},
    parse: 'json',
    errorMessage: (i) =>
      isRecord(i.parsed) && typeof i.parsed.error === 'string'
        ? i.parsed.error
        : i.raw || `HTTP ${i.status}`,
    errorFactory: (i, message) =>
      new ApiError({ message, status: i.status, responseText: i.raw, body: i.raw }),
  },
  /** w-maas `web/src/lib/api.ts`: instance Bearer token (auth overrides per-call
   *  headers), always-JSON, `credentials: 'include'`, structured
   *  `{error:{message,type,code}}` body, `ApiError(status,statusText,body)`. */
  wMaas: {
    contentType: 'always',
    authScheme: 'Bearer',
    authOverridesHeaders: true,
    credentials: 'include' as RequestCredentials,
    emptyBody: undefined,
    parse: 'json',
    errorMessage: (i) => {
      const b = i.parsed;
      if (isRecord(b)) {
        const e = b.error;
        if (isRecord(e) && typeof e.message === 'string') return e.message;
        if (typeof b.message === 'string') return b.message;
        if (typeof b.detail === 'string') return b.detail;
      }
      return `HTTP ${i.status} ${i.statusText}`;
    },
    errorFactory: (i, message) => {
      const e = isRecord(i.parsed) ? i.parsed.error : undefined;
      return new ApiError({
        message,
        status: i.status,
        statusText: i.statusText,
        body: i.parsed,
        errorType: isRecord(e) && typeof e.type === 'string' ? e.type : '',
        errorCode: isRecord(e) && typeof e.code === 'string' ? e.code : '',
        detail: message,
      });
    },
  },
  /** quantA `frontend/src/lib/api.ts`: `NEXT_PUBLIC_*` base, smart URL join,
   *  `Accept: application/json`, no core CT, tolerant parse, `null` empty body,
   *  `ApiError(message,status,body)` with `error`-string message. (Its
   *  GET-cache TTL layer stays app-local — see KEEP-LOCAL in the README.) */
  quantA: {
    accept: 'application/json',
    contentType: 'none',
    emptyBody: null,
    parse: 'safe',
    buildUrl: smartJoinUrl,
    errorMessage: (i) =>
      isRecord(i.parsed) && typeof i.parsed.error === 'string' ? i.parsed.error : `HTTP ${i.status}`,
    errorFactory: (i, message) => new ApiError({ message, status: i.status, body: i.parsed }),
  },
} satisfies Record<string, Partial<HttpClientConfig>>;
