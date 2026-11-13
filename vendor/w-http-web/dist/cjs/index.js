"use strict";
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.presets = exports.fetchJSON = exports.HANG_FOREVER = exports.ApiError = void 0;
exports.createHttpClient = createHttpClient;
exports.smartJoinUrl = smartJoinUrl;
/**
 * Rich error superset thrown by {@link createHttpClient}. Carries the union of
 * every fleet client's error fields so each consumer's callers keep reading the
 * same property they read today (`.status`, `.body`, `.responseText`, `.code`,
 * `.errorCode`, `.errorType`, `.detail`, `.hint`, `.message`). `instanceof
 * Error` holds; `name` is always `'ApiError'`.
 */
class ApiError extends Error {
    constructor(init = {}) {
        const message = init.message ??
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
exports.ApiError = ApiError;
/**
 * Sentinel returned from `onUnauthorized` to make `fetchJSON` resolve to a
 * never-settling promise (data-asset hard-redirects then "hangs" so the caller
 * never sees a value or an error). Returning `undefined` instead falls through
 * to normal error construction + throw.
 */
exports.HANG_FOREVER = Symbol('@w/http-web:hang');
/* ============================== Helpers ============================== */
function isRecord(value) {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}
/** Empty → `null`; else `JSON.parse`, falling back to the raw string. Used for
 *  ALL error bodies (every consumer tolerates a non-JSON error body). */
function tolerantParse(raw) {
    if (raw === '')
        return null;
    try {
        return JSON.parse(raw);
    }
    catch {
        return raw;
    }
}
/** Normalise `HeadersInit` (Headers | [k,v][] | record | undefined) to a record. */
function toHeaderRecord(init) {
    const out = {};
    if (!init)
        return out;
    if (typeof Headers !== 'undefined' && init instanceof Headers) {
        init.forEach((v, k) => {
            out[k] = v;
        });
        return out;
    }
    if (Array.isArray(init)) {
        for (const [k, v] of init)
            out[k] = v;
        return out;
    }
    return { ...init };
}
function hasHeader(record, name) {
    const lower = name.toLowerCase();
    return Object.keys(record).some((k) => k.toLowerCase() === lower);
}
function defaultErrorMessage(info) {
    const p = info.parsed;
    if (isRecord(p)) {
        const e = p.error;
        if (isRecord(e) && typeof e.message === 'string' && e.message)
            return e.message;
        if (typeof e === 'string' && e)
            return e;
        if (typeof p.message === 'string' && p.message)
            return p.message;
        if (typeof p.detail === 'string' && p.detail)
            return p.detail;
    }
    return info.raw || `HTTP ${info.status}`;
}
function buildHeaders(config, options) {
    const out = {};
    const dh = typeof config.defaultHeaders === 'function' ? config.defaultHeaders() : config.defaultHeaders;
    Object.assign(out, dh ?? {});
    if (config.accept)
        out['Accept'] = config.accept;
    const body = options.body;
    const isFormData = typeof FormData !== 'undefined' && body instanceof FormData;
    const mode = config.contentType ?? 'non-formdata';
    const callHeaders = toHeaderRecord(options.headers);
    let setCT;
    if (mode === 'always')
        setCT = true;
    else if (mode === 'non-formdata')
        setCT = !isFormData;
    else if (mode === 'string-only')
        setCT = typeof body === 'string' && !hasHeader(callHeaders, 'content-type');
    else
        setCT = false;
    if (setCT)
        out['Content-Type'] = config.contentTypeValue ?? 'application/json';
    const token = config.getToken?.();
    const authHeader = config.authHeader ?? 'Authorization';
    const authValue = token ? `${config.authScheme ?? 'Bearer'} ${token}` : undefined;
    if (config.authOverridesHeaders) {
        Object.assign(out, callHeaders);
        if (authValue)
            out[authHeader] = authValue;
    }
    else {
        if (authValue)
            out[authHeader] = authValue;
        Object.assign(out, callHeaders);
    }
    return out;
}
function encodeBody(body) {
    if (body === undefined || body === null)
        return undefined;
    if (typeof body === 'string')
        return body;
    if (typeof FormData !== 'undefined' && body instanceof FormData)
        return body;
    if (typeof Blob !== 'undefined' && body instanceof Blob)
        return body;
    if (body instanceof ArrayBuffer || ArrayBuffer.isView(body))
        return body;
    return JSON.stringify(body);
}
/**
 * Build a configured HTTP client. Each fleet consumer is reproduced by supplying
 * the matching {@link HttpClientConfig} (see `presets`); zero config yields a
 * sane same-origin JSON client that throws {@link ApiError}.
 */
function createHttpClient(config = {}) {
    const fetchImpl = config.fetchImpl ?? globalThis.fetch;
    async function fetchJSON(path, options = {}) {
        const base = typeof config.baseUrl === 'function' ? config.baseUrl() : config.baseUrl ?? '';
        const url = config.buildUrl ? config.buildUrl(base, path) : `${base}${path}`;
        const method = options.method ?? 'GET';
        const headers = buildHeaders(config, options);
        const ctx = { path, url, method, options };
        const span = config.onRequestStart?.(ctx);
        const init = { ...options, headers };
        const credentials = options.credentials ?? config.credentials;
        if (credentials != null)
            init.credentials = credentials;
        let res;
        try {
            res = await fetchImpl(url, init);
        }
        catch (networkErr) {
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
            const info = {
                ...ctx,
                status: res.status,
                statusText: res.statusText,
                raw,
                parsed,
                response: res,
            };
            if (res.status === 401 && config.onUnauthorized && !options.silent401) {
                const result = config.onUnauthorized(info);
                if (result === exports.HANG_FOREVER) {
                    config.onRequestEnd?.(span, 'error', {
                        'http.method': method,
                        'http.route': path,
                        'http.status_code': 401,
                    });
                    return new Promise(() => { });
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
        if (raw === '')
            return config.emptyBody;
        const parsed = config.parse === 'safe' ? tolerantParse(raw) : JSON.parse(raw);
        const value = config.unwrap ? config.unwrap(parsed, { ...ctx, response: res }) : parsed;
        return value;
    }
    const post = (path, body, options = {}) => fetchJSON(path, { ...options, method: 'POST', body: encodeBody(body) });
    const put = (path, body, options = {}) => fetchJSON(path, { ...options, method: 'PUT', body: encodeBody(body) });
    const patch = (path, body, options = {}) => fetchJSON(path, { ...options, method: 'PATCH', body: encodeBody(body) });
    return {
        fetchJSON,
        request: fetchJSON,
        get: (path, options = {}) => fetchJSON(path, { ...options, method: options.method ?? 'GET' }),
        post,
        put,
        patch,
        del: (path, options = {}) => fetchJSON(path, { ...options, method: 'DELETE' }),
        config,
    };
}
/** Zero-config same-origin JSON client (`Bearer`-less, throws {@link ApiError}). */
const defaultClient = createHttpClient();
/** Standalone bound {@link createHttpClient}().fetchJSON for the zero-config case. */
exports.fetchJSON = defaultClient.fetchJSON;
/* ============================== Presets ============================== */
/** quantA-style smart join: empty base → path; relative path → `base/path`. */
function smartJoinUrl(base, path) {
    if (!base)
        return path;
    if (!path)
        return base;
    return path.startsWith('/') ? `${base}${path}` : `${base}/${path}`;
}
/**
 * Behaviour-preserving config fragments, one per consuming frontend. Spread into
 * `createHttpClient({ ...presets.<name>, baseUrl, getToken, … })`; the static
 * knobs (content-type / parse / error extraction) live here, the app-specific
 * dynamic hooks (`baseUrl` resolver, `getToken`, `onUnauthorized`, `onError`,
 * span) are supplied at the call site. See the README migration map.
 */
exports.presets = {
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
        errorMessage: (i) => isRecord(i.parsed) && typeof i.parsed.error === 'string'
            ? i.parsed.error
            : i.raw || `HTTP ${i.status}`,
        errorFactory: (i, message) => new ApiError({ message, status: i.status, responseText: i.raw, body: i.raw }),
    },
    /** w-maas `web/src/lib/api.ts`: instance Bearer token (auth overrides per-call
     *  headers), always-JSON, `credentials: 'include'`, structured
     *  `{error:{message,type,code}}` body, `ApiError(status,statusText,body)`. */
    wMaas: {
        contentType: 'always',
        authScheme: 'Bearer',
        authOverridesHeaders: true,
        credentials: 'include',
        emptyBody: undefined,
        parse: 'json',
        errorMessage: (i) => {
            const b = i.parsed;
            if (isRecord(b)) {
                const e = b.error;
                if (isRecord(e) && typeof e.message === 'string')
                    return e.message;
                if (typeof b.message === 'string')
                    return b.message;
                if (typeof b.detail === 'string')
                    return b.detail;
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
        errorMessage: (i) => isRecord(i.parsed) && typeof i.parsed.error === 'string' ? i.parsed.error : `HTTP ${i.status}`,
        errorFactory: (i, message) => new ApiError({ message, status: i.status, body: i.parsed }),
    },
};
