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
export declare class ApiError extends Error {
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
    constructor(init?: ApiErrorInit);
}
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
'json'
/** Tolerant — invalid JSON is returned verbatim as a string (quantA). */
 | 'safe';
/** Content-Type policy for the request body. */
export type ContentTypeMode = 
/** Always set (overridable per-call). w-maas. */
'always'
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
export declare const HANG_FOREVER: unique symbol;
export interface HttpClientConfig {
    /** Base URL, or a resolver (re-read per request). Default `''` (same-origin). */
    baseUrl?: string | (() => string);
    /** Join `base`+`path`. Default `base + path` (raw concat). quantA overrides. */
    buildUrl?: (base: string, path: string) => string;
    /** Injected `fetch` (tests / non-browser). Default `globalThis.fetch`. */
    fetchImpl?: typeof fetch;
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
    /** Non-empty success body parse mode. Default `'json'`. */
    parse?: ParseMode;
    /** Value for an empty success body. Default `undefined` (data-asset `{}`, quantA `null`). */
    emptyBody?: unknown;
    /** Unwrap an envelope on success ({data}/{ok,data}/…). Default identity. */
    unwrap?: (parsed: unknown, ctx: RequestContext & {
        response: Response;
    }) => unknown;
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
    /** Start a span before `fetch`; the returned handle is threaded to `onRequestEnd`. */
    onRequestStart?: (ctx: RequestContext) => SpanHandle;
    /** Finish the span. `attrs` use OpenTelemetry HTTP semantic-convention keys
     *  (`http.method` / `http.route` / `http.status_code` / `error.type` / `error.message`). */
    onRequestEnd?: (span: SpanHandle, outcome: 'ok' | 'error', attrs: Record<string, unknown>) => void;
}
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
/**
 * Build a configured HTTP client. Each fleet consumer is reproduced by supplying
 * the matching {@link HttpClientConfig} (see `presets`); zero config yields a
 * sane same-origin JSON client that throws {@link ApiError}.
 */
export declare function createHttpClient(config?: HttpClientConfig): HttpClient;
/** Standalone bound {@link createHttpClient}().fetchJSON for the zero-config case. */
export declare const fetchJSON: <T = unknown>(path: string, options?: FetchOptions) => Promise<T>;
/** quantA-style smart join: empty base → path; relative path → `base/path`. */
export declare function smartJoinUrl(base: string, path: string): string;
/**
 * Behaviour-preserving config fragments, one per consuming frontend. Spread into
 * `createHttpClient({ ...presets.<name>, baseUrl, getToken, … })`; the static
 * knobs (content-type / parse / error extraction) live here, the app-specific
 * dynamic hooks (`baseUrl` resolver, `getToken`, `onUnauthorized`, `onError`,
 * span) are supplied at the call site. See the README migration map.
 */
export declare const presets: {
    /** OneMan `integrations/api/client.ts`: Bearer session, JSON-unless-FormData,
     *  plain-text error message, throws a bare `Error` (its `errors.ts` `ApiError`
     *  {code,details,hint} is modelled by the superset {@link ApiError}). 401 is
     *  handled by an injected `onUnauthorized` (clear session + redirect + throw). */
    oneman: {
        contentType: "non-formdata";
        authScheme: string;
        emptyBody: undefined;
        parse: "json";
        errorMessage: (i: ErrorResponseInfo) => string;
        errorFactory: (_i: ErrorResponseInfo, message: string) => Error;
    };
    /** W-DaaS / data-asset `frontend/src/api/client.ts`: same-origin, no token,
     *  CT only for string bodies, `{}` for empty body, `ApiError(status,
     *  responseText)`; couples a telemetry span (`onRequestStart`/`onRequestEnd`)
     *  + `onError` + a gated 401 redirect (`onUnauthorized` → {@link HANG_FOREVER}). */
    dataAsset: {
        contentType: "string-only";
        emptyBody: {};
        parse: "json";
        errorMessage: (i: ErrorResponseInfo) => string;
        errorFactory: (i: ErrorResponseInfo, message: string) => ApiError;
    };
    /** w-maas `web/src/lib/api.ts`: instance Bearer token (auth overrides per-call
     *  headers), always-JSON, `credentials: 'include'`, structured
     *  `{error:{message,type,code}}` body, `ApiError(status,statusText,body)`. */
    wMaas: {
        contentType: "always";
        authScheme: string;
        authOverridesHeaders: true;
        credentials: RequestCredentials;
        emptyBody: undefined;
        parse: "json";
        errorMessage: (i: ErrorResponseInfo) => string;
        errorFactory: (i: ErrorResponseInfo, message: string) => ApiError;
    };
    /** quantA `frontend/src/lib/api.ts`: `NEXT_PUBLIC_*` base, smart URL join,
     *  `Accept: application/json`, no core CT, tolerant parse, `null` empty body,
     *  `ApiError(message,status,body)` with `error`-string message. (Its
     *  GET-cache TTL layer stays app-local — see KEEP-LOCAL in the README.) */
    quantA: {
        accept: string;
        contentType: "none";
        emptyBody: null;
        parse: "safe";
        buildUrl: typeof smartJoinUrl;
        errorMessage: (i: ErrorResponseInfo) => string;
        errorFactory: (i: ErrorResponseInfo, message: string) => ApiError;
    };
};
export {};
