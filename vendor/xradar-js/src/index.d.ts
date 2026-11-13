export interface XRadarConfig {
    code?: string | null;
    endpoint?: string;
    serviceName?: string;
    release?: string | null;
    environment?: string | null;
    clientKind?: string;
    runtime?: string;
    platform?: string;
    sdkVersion?: string;
    slowTraceMs?: number;
}

export interface TraceContext {
    traceId: string;
    parentSpanId?: string | null;
    spanId: string;
}

export interface Span {
    traceId: string;
    spanId: string;
    parentSpanId: string | null;
    spanName: string;
    spanKind: string;
    startTimeMs: number;
    startedAt: number;
    durationMs?: number;
}

export interface SpanOptions {
    traceId?: string;
    spanId?: string;
    parentSpanId?: string | null;
    parent?: TraceContext | null;
    spanKind?: string;
    startTimeMs?: number;
    durationMs?: number;
}

export interface DeliveryOptions {
    fetcher?: typeof fetch;
    timeoutMs?: number;
    [key: string]: unknown;
}

export interface SpanBatchItem {
    span: Span;
    status?: string;
    data?: Record<string, unknown>;
}

export interface ErrorOptions extends DeliveryOptions {
    exceptionType?: string;
    exception_type?: string;
    stack?: string | null;
    handled?: boolean;
    severity?: string;
    url?: string | null;
    screen?: string | null;
    breadcrumbs?: unknown[];
    tags?: Record<string, unknown>;
    context?: Record<string, unknown>;
}

export function configFromEnv(env?: Record<string, string | undefined>): XRadarConfig;
export function startSpan(spanName: string, options?: SpanOptions): Span;
export function startChildSpan(parent: TraceContext | Span | null | undefined, spanName: string, options?: SpanOptions): Span;
export function traceparent(span: Span): string;
export function buildSpanPayload(
    config: XRadarConfig,
    span: Span,
    status?: string,
    extraData?: Record<string, unknown>,
): Record<string, unknown>;
export function buildSpanBatchPayload(
    config: XRadarConfig,
    spanItems: Array<Span | SpanBatchItem>,
): Record<string, unknown>;
export function buildErrorPayload(
    config: XRadarConfig,
    error: unknown,
    options?: ErrorOptions,
): Record<string, unknown>;
export function recordSpan(
    config: XRadarConfig,
    span: Span,
    status?: string,
    extraData?: Record<string, unknown>,
    options?: DeliveryOptions,
): Promise<void>;
export function recordSpans(
    config: XRadarConfig,
    spanItems: Array<Span | SpanBatchItem>,
    options?: DeliveryOptions,
): Promise<void>;
export function captureError(config: XRadarConfig, error: unknown, options?: ErrorOptions): Promise<void>;
export function withWorkerTelemetry<TEnv = Record<string, string | undefined>, TContext = unknown>(
    handler: (request: Request, env: TEnv, ctx: TContext) => Response | Promise<Response>,
    options?: DeliveryOptions & {
        clientKind?: string;
        runtime?: string;
        platform?: string;
    },
): (request: Request, env: TEnv, ctx: TContext) => Promise<Response>;
export interface TracedFetchOptions {
    baseFetch?: typeof fetch;
    parentSpan?: Span | null;
}
export function tracedFetch(
    config: XRadarConfig,
    options?: TracedFetchOptions,
): typeof fetch;
export function parseTraceparent(value?: string | null): TraceContext | null;
export function normalizePath(path: string): string;
export function shouldSkipPath(path: string): boolean;

// ============================================================
// Phase-2 capability self-report
// ============================================================

export const SDK_VERSION: string;
export const CAPABILITIES_HEARTBEAT_SECS: number;

export interface TenancyState {
    mirror_installed: boolean;
    auth_callback_wires_mirror: boolean;
    principal_can_access_uses: number;
    user_owned_tables: string[];
}

export interface GitState {
    branch: string | null;
    sha: string | null;
    dirty: boolean;
}

export interface ProcessInfo {
    pid: number;
    started_at: number;
    uptime_seconds: number;
}

export interface Capabilities {
    project_id: string | null;
    instance_id: string | null;
    host: string | null;
    runtime: 'node' | 'cloudflare-workers' | 'browser' | string;
    sdks: Record<string, string>;
    tenancy: TenancyState | null;
    git: GitState | null;
    process: ProcessInfo;
    reported_at: number;
    sdk_version: string;
}

export interface BuildCapabilitiesOptions {
    projectId?: string | null;
    instanceId?: string | null;
    host?: string | null;
    runtime?: string;
    sdks?: Record<string, string>;
    tenancy?: TenancyState | null;
    git?: GitState | null;
    process?: Partial<ProcessInfo>;
    reportedAt?: number;
    sdkVersion?: string;
    env?: Record<string, string | undefined>;
}

export interface ReportCapabilitiesOptions {
    fetcher?: typeof fetch;
    timeoutMs?: number;
    env?: Record<string, string | undefined>;
    apiKey?: string;
}

export interface HeartbeatHandle {
    disabled: boolean;
    stop(): void;
}

export interface HeartbeatOptions extends ReportCapabilitiesOptions {
    intervalSeconds?: number;
}

export function buildCapabilities(opts?: BuildCapabilitiesOptions): Capabilities;
export function buildCapabilitiesPayload(caps: Capabilities): Record<string, unknown>;
export function reportCapabilities(
    config: { endpoint?: string },
    caps: Capabilities,
    opts?: ReportCapabilitiesOptions,
): Promise<boolean>;
export function spawnCapabilitiesHeartbeat(
    config: { endpoint?: string },
    caps: Capabilities,
    opts?: HeartbeatOptions,
): HeartbeatHandle;
