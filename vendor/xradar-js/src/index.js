const DEFAULT_ENDPOINT = 'https://radar.region-a.example.invalid';

// Carried in the Capabilities `sdk_version` wire field so the Radar
// receiver can spot SDK skew across deployed binaries.
export const SDK_VERSION = '0.2.1';

// 15 min default heartbeat for the capability pusher.
export const CAPABILITIES_HEARTBEAT_SECS = 15 * 60;

function envValue(env, key) {
  const value = env?.[key] ?? globalThis.process?.env?.[key];
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function randomHex(bytes) {
  const data = new Uint8Array(bytes);
  globalThis.crypto.getRandomValues(data);
  return [...data].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

function nowMs() {
  return Date.now();
}

function traceSessionId(code, traceId) {
  let namespace = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(String(code || ''))) {
    namespace = BigInt.asUintN(64, (namespace ^ BigInt(byte)) * 0x100000001b3n);
  }
  return `trace_${namespace.toString(16).padStart(16, '0')}_${traceId || 'unknown'}`;
}

export function configFromEnv(env = {}) {
  const code = envValue(env, 'W_RADAR_CODE');
  return {
    code,
    endpoint: (envValue(env, 'W_RADAR_ENDPOINT') || envValue(env, 'W_RADAR_URL') || envValue(env, 'W_RADAR_ORIGIN') || DEFAULT_ENDPOINT).replace(/\/+$/, ''),
    serviceName: envValue(env, 'SERVICE_NAME') || 'javascript',
    release: envValue(env, 'APP_RELEASE') || envValue(env, 'RELEASE'),
    environment: envValue(env, 'APP_ENV') || envValue(env, 'ENVIRONMENT'),
    clientKind: envValue(env, 'XRADAR_CLIENT_KIND') || 'server',
    runtime: envValue(env, 'XRADAR_RUNTIME') || 'javascript',
    platform: envValue(env, 'XRADAR_PLATFORM') || 'server',
    slowTraceMs: Number(envValue(env, 'W_RADAR_SLOW_TRACE_MS') || envValue(env, 'SLOW_TRACE_MS') || 1000),
  };
}

export function startSpan(spanName, options = {}) {
  return {
    traceId: options.traceId || options.parent?.traceId || randomHex(16),
    spanId: options.spanId || randomHex(8),
    parentSpanId: options.parentSpanId ?? options.parent?.spanId ?? null,
    spanName,
    spanKind: options.spanKind || 'internal',
    startTimeMs: options.startTimeMs || nowMs(),
    startedAt: globalThis.performance?.now?.() ?? nowMs(),
    durationMs: options.durationMs,
  };
}

export function startChildSpan(parent, spanName, options = {}) {
  return startSpan(spanName, {
    ...options,
    traceId: options.traceId || parent?.traceId,
    parentSpanId: options.parentSpanId ?? parent?.spanId ?? null,
  });
}

export function traceparent(span) {
  return `00-${span.traceId}-${span.spanId}-01`;
}

export function buildSpanPayload(config, span, status = 'ok', extraData = {}) {
  return buildSpanBatchPayload(config, [{ span, status, data: extraData }]);
}

export function buildSpanBatchPayload(config, spanItems = []) {
  const entries = spanItems
    .map((item) => item?.span ? item : { span: item, status: 'ok', data: {} })
    .filter((item) => item.span);
  const firstSpan = entries[0]?.span;
  const serviceName = config.serviceName || 'javascript';

  return {
    code: config.code,
    session_id: traceSessionId(config.code, firstSpan?.traceId),
    visitor_id: `service:${serviceName}`,
    events: entries.map((item) => buildSpanEvent(config, item.span, item.status || 'ok', item.data || {})),
    session_info: {
      client_kind: config.clientKind || 'server',
      client_version: config.release || null,
    },
  };
}

function buildSpanEvent(config, span, status = 'ok', extraData = {}) {
  const durationMs = span.durationMs ?? Math.round((globalThis.performance?.now?.() ?? nowMs()) - span.startedAt);
  const serviceName = config.serviceName || 'javascript';
  const data = {
    trace_id: span.traceId,
    span_id: span.spanId,
    parent_span_id: span.parentSpanId,
    traceparent: traceparent(span),
    span_name: span.spanName,
    span_kind: span.spanKind,
    duration_ms: durationMs,
    status,
    service_name: serviceName,
    runtime: config.runtime || 'javascript',
    platform: config.platform || 'server',
    release: config.release || null,
    environment: config.environment || null,
    'telemetry.sdk.name': 'xradar',
    'telemetry.sdk.language': 'javascript',
    'telemetry.sdk.version': config.sdkVersion || SDK_VERSION,
    'service.name': serviceName,
    'service.version': config.release || null,
    'deployment.environment': config.environment || null,
    slow_trace: durationMs >= (config.slowTraceMs ?? 1000),
    ...extraData,
  };

  return {
    type: 'performance_span',
    name: span.spanName,
    url: data['http.route'] || data.url || null,
    data,
    ts: span.startTimeMs,
  };
}

export function buildErrorPayload(config, error, options = {}) {
  const errorObject = error instanceof Error ? error : new Error(String(error || 'Application error'));
  return {
    code: config.code,
    platform: config.platform || 'server',
    runtime: config.runtime || 'javascript',
    release: config.release || null,
    environment: config.environment || null,
    message: errorObject.message,
    exception_type: options.exceptionType || options.exception_type || errorObject.name || 'Error',
    stack: options.stack ?? errorObject.stack ?? null,
    handled: options.handled ?? false,
    severity: options.severity || 'error',
    url: options.url || null,
    screen: options.screen || null,
    breadcrumbs: Array.isArray(options.breadcrumbs) ? options.breadcrumbs : [],
    tags: {
      service_name: config.serviceName || 'javascript',
      ...(options.tags || {}),
    },
    context: options.context || {},
    timestamp: nowMs(),
  };
}

export async function recordSpan(config, span, status = 'ok', extraData = {}, options = {}) {
  return recordSpans(config, [{ span, status, data: extraData }], options);
}

export async function recordSpans(config, spanItems = [], options = {}) {
  if (!config?.code) return;
  const fetcher = options.fetcher || globalThis.fetch;
  if (!fetcher) return;
  const payload = buildSpanBatchPayload(config, spanItems);
  if (!payload.events.length) return;
  try {
    await fetcher(`${(config.endpoint || DEFAULT_ENDPOINT).replace(/\/$/, '')}/collect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: timeoutSignal(options.timeoutMs ?? 2000),
    });
  } catch {
    // Telemetry delivery must never break the application path.
  }
}

export async function captureError(config, error, options = {}) {
  if (!config?.code) return;
  const fetcher = options.fetcher || globalThis.fetch;
  if (!fetcher) return;
  const payload = buildErrorPayload(config, error, options);
  try {
    await fetcher(`${(config.endpoint || DEFAULT_ENDPOINT).replace(/\/$/, '')}/errors/capture`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: timeoutSignal(options.timeoutMs ?? 2000),
    });
  } catch {
    // Error telemetry delivery must never break the application path.
  }
}

export function withWorkerTelemetry(handler, options = {}) {
  return async function xradarWorkerTelemetry(request, env, ctx) {
    const config = {
      ...configFromEnv(env),
      clientKind: options.clientKind || 'worker',
      runtime: options.runtime || 'cloudflare-worker',
      platform: options.platform || 'worker',
    };
    const url = new URL(request.url);
    if (shouldSkipPath(url.pathname)) {
      return handler(request, env, ctx);
    }

    const span = startSpan(`http.server ${request.method} ${normalizePath(url.pathname)}`, {
      spanKind: 'server',
      parent: parseTraceparent(request.headers.get('traceparent')),
    });

    try {
      const response = await handler(request, env, ctx);
      const status = response.status >= 500 ? 'error' : 'ok';
      const promise = recordSpan(config, span, status, {
        'http.method': request.method,
        'http.route': normalizePath(url.pathname),
        'http.status_code': response.status,
        'http.target': url.pathname,
      }, options);
      ctx?.waitUntil?.(promise);
      const headers = new Headers(response.headers);
      headers.set('traceparent', traceparent(span));
      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers,
      });
    } catch (error) {
      const spanPromise = recordSpan(config, span, 'error', {
        'http.method': request.method,
        'http.route': normalizePath(url.pathname),
        'error.type': error instanceof Error ? error.name : 'Error',
      }, options);
      const errorPromise = captureError(config, error, {
        ...options,
        handled: false,
        url: request.url,
        context: { method: request.method, path: url.pathname },
      });
      ctx?.waitUntil?.(Promise.all([spanPromise, errorPromise]));
      throw error;
    }
  };
}

export function parseTraceparent(value) {
  if (!value) return null;
  const parts = value.split('-');
  if (
    parts.length !== 4 ||
    parts[0] !== '00' ||
    !/^[a-fA-F0-9]{32}$/.test(parts[1]) ||
    !/^[a-fA-F0-9]{16}$/.test(parts[2]) ||
    !/^[a-fA-F0-9]{2}$/.test(parts[3])
  ) {
    return null;
  }
  return {
    traceId: parts[1].toLowerCase(),
    parentSpanId: parts[2].toLowerCase(),
    spanId: parts[2].toLowerCase(),
  };
}

export function normalizePath(path) {
  const cleanPath = (path || '/').split('?')[0];
  if (cleanPath === '/') return '/';
  return cleanPath
    .split('/')
    .map((segment) => {
      if (!segment) return '';
      return highCardinalitySegment(segment) ? ':id' : segment;
    })
    .join('/');
}

export function shouldSkipPath(path) {
  const cleanPath = (path || '/').split('?')[0];
  return cleanPath === '/health' ||
    cleanPath === '/healthz' ||
    cleanPath === '/ready' ||
    cleanPath === '/metrics' ||
    cleanPath.startsWith('/health/') ||
    cleanPath === '/collect' ||
    cleanPath.startsWith('/collect/') ||
    cleanPath === '/errors/capture' ||
    cleanPath.startsWith('/errors/capture/') ||
    cleanPath.startsWith('/pixel/') ||
    cleanPath.startsWith('/assets/') ||
    cleanPath.startsWith('/static/') ||
    ['.js', '.css', '.ico', '.svg', '.png', '.jpg', '.jpeg', '.gif', '.webp', '.woff', '.woff2']
      .some((suffix) => cleanPath.toLowerCase().endsWith(suffix));
}

function highCardinalitySegment(segment) {
  return /^\d+$/.test(segment) ||
    /^[a-fA-F0-9]{16,}$/.test(segment) ||
    (segment.length >= 16 && /^[a-zA-Z0-9_-]+$/.test(segment)) ||
    (segment.length >= 20 && (segment.match(/-/g) || []).length >= 2);
}

/**
 * Wrap globalThis.fetch to propagate traceparent headers and record HTTP
 * client spans. Returns a fetch-compatible function.
 *
 * Usage:
 *   const config = configFromEnv(process.env);
 *   const fetch = tracedFetch(config);
 *   const resp = await fetch('https://api.example.com/data');
 */
export function tracedFetch(config, options = {}) {
  const baseFetch = options.baseFetch || globalThis.fetch;
  const parentSpan = options.parentSpan || null;

  return async function (input, init = {}) {
    if (!config?.code || !baseFetch) return baseFetch(input, init);

    const url = typeof input === 'string' ? input : input?.url || String(input);
    const method = init.method || (input?.method) || 'GET';
    const route = normalizePath(new URL(url, 'http://localhost').pathname);

    const span = startSpan(`http.client ${method} ${route}`, {
      spanKind: 'client',
      parent: parentSpan,
    });

    const headers = new Headers(init.headers);
    headers.set('traceparent', traceparent(span));
    const mergedInit = { ...init, headers };

    try {
      const response = await baseFetch(input, mergedInit);
      const status = response.status >= 400 ? 'error' : 'ok';
      recordSpan(config, span, status, {
        'http.method': method,
        'http.route': route,
        'http.status_code': response.status,
        'http.url': url,
        'span_kind': 'network.request',
      }).catch(() => {});
      return response;
    } catch (error) {
      recordSpan(config, span, 'error', {
        'http.method': method,
        'http.route': route,
        'http.url': url,
        'span_kind': 'network.request',
        'error.type': error instanceof Error ? error.name : 'Error',
        'error.message': error instanceof Error ? error.message : String(error),
      }).catch(() => {});
      throw error;
    }
  };
}

function timeoutSignal(timeoutMs) {
  if (typeof AbortSignal !== 'undefined' && typeof AbortSignal.timeout === 'function') {
    return AbortSignal.timeout(timeoutMs);
  }
  return undefined;
}

// ============================================================
// Phase-2 capability self-report
// ============================================================
//
// Mirrors sdk/rust/xradar (canonical) + sdk/python/xradar. Wire format must
// stay byte-identical across all three SDKs.

function detectRuntime() {
  if (typeof globalThis.WebSocketPair !== 'undefined') return 'cloudflare-workers';
  if (typeof globalThis.process !== 'undefined' && globalThis.process.versions?.node) return 'node';
  if (typeof globalThis.window !== 'undefined') return 'browser';
  return 'node';
}

function detectHost(env = {}) {
  return (
    env.HOSTNAME ||
    globalThis.process?.env?.HOSTNAME ||
    null
  );
}

function detectProcessInfo() {
  const startedAt = Math.floor(Date.now() / 1000);
  const pid = globalThis.process?.pid ?? 0;
  return { pid, started_at: startedAt, uptime_seconds: 0 };
}

function readGitFromEnv(env = {}) {
  const sha =
    (env.XRADAR_HOST_GIT_SHA ?? globalThis.process?.env?.XRADAR_HOST_GIT_SHA ?? globalThis.XRADAR_HOST_GIT_SHA) || null;
  const branch =
    (env.XRADAR_HOST_GIT_BRANCH ?? globalThis.process?.env?.XRADAR_HOST_GIT_BRANCH ?? globalThis.XRADAR_HOST_GIT_BRANCH) || null;
  const dirtyRaw =
    env.XRADAR_HOST_GIT_DIRTY ?? globalThis.process?.env?.XRADAR_HOST_GIT_DIRTY ?? globalThis.XRADAR_HOST_GIT_DIRTY;
  const dirty = dirtyRaw === '1' || dirtyRaw === 'true' || dirtyRaw === 'yes';
  return { branch: branch || null, sha: sha || null, dirty };
}

/**
 * Build a Capabilities object with sensible defaults. The host overlays
 * extra detail via the opts: projectId / instanceId / host / runtime /
 * sdks / tenancy / git / process / reportedAt. Unset fields fall through
 * to introspected defaults. Pass `env` to seed git/host vars from a Workers
 * env binding rather than `process.env`.
 */
export function buildCapabilities(opts = {}) {
  const env = opts.env || {};
  const process = opts.process || detectProcessInfo();
  const reported_at = opts.reportedAt ?? process.started_at ?? Math.floor(Date.now() / 1000);
  return {
    project_id: opts.projectId ?? null,
    instance_id: opts.instanceId ?? null,
    host: opts.host ?? detectHost(env) ?? null,
    runtime: opts.runtime ?? detectRuntime(),
    sdks: { xradar: SDK_VERSION, ...(opts.sdks || {}) },
    tenancy: opts.tenancy ?? null,
    git: opts.git ?? readGitFromEnv(env),
    process: {
      pid: Number(process.pid ?? 0),
      started_at: Number(process.started_at ?? 0),
      uptime_seconds: Number(process.uptime_seconds ?? 0),
    },
    reported_at: Number(reported_at),
    sdk_version: opts.sdkVersion ?? SDK_VERSION,
  };
}

/**
 * Serialize a Capabilities into the canonical wire shape. Optional branches
 * (tenancy/git) are dropped when null so the Radar deserializer can tell
 * "host didn't provide" from "host said false".
 */
export function buildCapabilitiesPayload(caps) {
  const payload = {
    project_id: caps.project_id ?? null,
    instance_id: caps.instance_id ?? null,
    host: caps.host ?? null,
    runtime: caps.runtime,
    sdks: { ...(caps.sdks || {}) },
    process: {
      pid: Number(caps.process?.pid ?? 0),
      started_at: Number(caps.process?.started_at ?? 0),
      uptime_seconds: Number(caps.process?.uptime_seconds ?? 0),
    },
    reported_at: Number(caps.reported_at ?? Math.floor(Date.now() / 1000)),
    sdk_version: caps.sdk_version || SDK_VERSION,
  };
  if (caps.tenancy) {
    payload.tenancy = {
      mirror_installed: Boolean(caps.tenancy.mirror_installed),
      auth_callback_wires_mirror: Boolean(caps.tenancy.auth_callback_wires_mirror),
      principal_can_access_uses: Number(caps.tenancy.principal_can_access_uses || 0),
      user_owned_tables: Array.isArray(caps.tenancy.user_owned_tables)
        ? [...caps.tenancy.user_owned_tables]
        : [],
    };
  }
  if (caps.git) {
    payload.git = {
      branch: caps.git.branch ?? null,
      sha: caps.git.sha ?? null,
      dirty: Boolean(caps.git.dirty),
    };
  }
  return payload;
}

/**
 * POST a capability snapshot to Radar's /api/platform/runtime/capabilities.
 * Returns true on 2xx, false on transport/non-2xx/disabled. Caller can
 * inject a `fetcher` (CF Workers) or rely on globalThis.fetch (Node 18+).
 * Honors `XRADAR_DISABLE_CAPABILITIES=1` as a kill switch via `opts.env`
 * or `globalThis.process.env`.
 */
export async function reportCapabilities(config, caps, opts = {}) {
  const env = opts.env || {};
  const disabled =
    env.XRADAR_DISABLE_CAPABILITIES === '1' ||
    globalThis.process?.env?.XRADAR_DISABLE_CAPABILITIES === '1';
  if (disabled) return false;

  const fetcher = opts.fetcher || globalThis.fetch;
  if (!fetcher) return false;
  const endpoint = (config?.endpoint || DEFAULT_ENDPOINT).replace(/\/+$/, '');
  const payload = buildCapabilitiesPayload(caps);
  const headers = { 'content-type': 'application/json' };
  const token =
    opts.apiKey ||
    env.W_RADAR_API_KEY ||
    globalThis.process?.env?.W_RADAR_API_KEY;
  if (token) headers.authorization = `Bearer ${token}`;

  try {
    const resp = await fetcher(`${endpoint}/api/platform/runtime/capabilities`, {
      method: 'POST',
      headers,
      body: JSON.stringify(payload),
      signal: timeoutSignal(opts.timeoutMs ?? 5000),
    });
    return Boolean(resp && resp.ok);
  } catch {
    return false;
  }
}

/**
 * Spawn a background heartbeat that posts once on start and then every
 * `intervalSeconds` (default 900s = 15min). Returns a handle the host can
 * call `.stop()` on for graceful shutdown. When disabled via env, the
 * returned handle has `disabled=true` and `.stop()` is a no-op.
 *
 * Cloudflare Workers don't have a long-lived process — for that runtime
 * the host should call `reportCapabilities` directly inside the cold-start
 * path and pass the promise to `ctx.waitUntil()`.
 */
export function spawnCapabilitiesHeartbeat(config, caps, opts = {}) {
  const env = opts.env || {};
  const disabled =
    env.XRADAR_DISABLE_CAPABILITIES === '1' ||
    globalThis.process?.env?.XRADAR_DISABLE_CAPABILITIES === '1';
  if (disabled) {
    return { disabled: true, stop: () => {} };
  }

  const requestedInterval = opts.intervalSeconds ?? CAPABILITIES_HEARTBEAT_SECS;
  const intervalMs = Number.isFinite(requestedInterval) && requestedInterval > 0 && requestedInterval <= 2147483
    ? requestedInterval * 1000
    : CAPABILITIES_HEARTBEAT_SECS * 1000;
  let stopped = false;
  let timer = null;

  async function pushOnce() {
    if (stopped) return;
    const now = Math.floor(Date.now() / 1000);
    const elapsed = Math.max(0, now - Number(caps.process?.started_at || now));
    const snapshot = {
      ...caps,
      process: { ...caps.process, uptime_seconds: elapsed },
      reported_at: now,
    };
    try {
      await reportCapabilities(config, snapshot, opts);
    } catch {
      // Telemetry must never break the host.
    }
    if (!stopped) {
      timer = setTimeout(pushOnce, intervalMs);
      timer.unref?.();
    }
  }

  // Kick off immediately.
  timer = setTimeout(pushOnce, 0);
  timer.unref?.();

  return {
    disabled: false,
    stop() {
      stopped = true;
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    },
  };
}
