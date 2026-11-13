# @w/http-web

Configurable TypeScript HTTP client, JSON helpers and structured API errors. Zero runtime dependencies; ESM, CommonJS and TypeScript declarations.

Build this source package before installing it by local path:

```sh
npm ci
npm run build
npm test
# From a consuming project, use its path to this checkout:
npm install /path/to/w-radar/sdk/js/http-web
```

The public release includes source and built files. No private npm registry is needed.

## API
| Export | Signature | Notes |
|---|---|---|
| `createHttpClient` | `(config?: HttpClientConfig) => HttpClient` | Build a configured client (`fetchJSON` + `request`/`get`/`post`/`put`/`patch`/`del`). |
| `fetchJSON` | `<T>(path, opts?: FetchOptions) => Promise<T>` | Zero-config same-origin bound client (uses `globalThis.fetch`). |
| `ApiError` | `new ApiError(init: ApiErrorInit)` | Rich superset error; `extends Error`, `name = 'ApiError'`. |
| `presets` | `{ oneman, dataAsset, wMaas, quantA }` | Per-consumer `Partial<HttpClientConfig>` fragments. |
| `smartJoinUrl` | `(base, path) => string` | quantA URL join (empty base → path; relative path → `base/path`). |
| `HANG_FOREVER` | `unique symbol` | `onUnauthorized` return value → `fetchJSON` resolves to a never-settling promise (hard-redirect-then-hang). |

### `HttpClientConfig` (all optional)

`baseUrl` (`string | () => string`) · `buildUrl` (`(base,path)=>string`) ·
`fetchImpl` · `getToken` · `authHeader` (`'Authorization'`) · `authScheme`
(`'Bearer'`) · `authOverridesHeaders` (`false`) · `accept` · `contentType`
(`'always' | 'non-formdata' | 'string-only' | 'none'`, default `'non-formdata'`)
· `contentTypeValue` (`'application/json'`) · `defaultHeaders` · `credentials` ·
`parse` (`'json' | 'safe'`) · `emptyBody` · `unwrap` · `errorMessage` ·
`errorFactory` · `onUnauthorized` · `onError` · `onRequestStart` /
`onRequestEnd` (OpenTelemetry `http.*` attrs).

**Request pipeline:** resolve URL (`baseUrl` + `buildUrl`) → build headers
(`defaultHeaders` → `accept` → content-type policy → auth, with per-call headers
merged by `authOverridesHeaders`) → `onRequestStart` span → `fetch` → on network
throw: `onRequestEnd('error')` + `onError('network')` + rethrow raw → on `!ok`:
read body once, tolerant-parse, `401`→`onUnauthorized` (throw / `HANG_FOREVER` /
fall through), else `errorMessage`→`errorFactory`→`onRequestEnd('error')`→
`onError('http')`→throw → on ok: `onRequestEnd('ok')`, empty→`emptyBody`, else
`parse` then `unwrap`.
