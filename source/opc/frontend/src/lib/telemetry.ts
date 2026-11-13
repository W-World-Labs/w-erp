import {
  configFromEnv,
  captureError,
  startSpan,
  recordSpan,
  type XRadarConfig,
  type Span,
} from '@w/xradar'

// Additive @w/xradar instrumentation. The existing local errorReporter.ts
// (POST /api/v1/error_logs) is preserved; this mirrors crashes to Radar too.
//
// Telemetry is opt-in. Configure your own tracking code and endpoint at build time.
const RADAR_TRACKING_CODE =
  (import.meta.env.VITE_W_RADAR_CODE as string | undefined)?.trim() ||
  '';

const config: XRadarConfig = {
  ...configFromEnv({
    W_RADAR_CODE: RADAR_TRACKING_CODE,
    W_RADAR_ENDPOINT:
      (import.meta.env.VITE_W_RADAR_ENDPOINT as string | undefined)?.trim() ||
      undefined,
    SERVICE_NAME: 'w-opc-frontend',
    APP_ENV: location.hostname === 'localhost' ? 'development' : 'production',
    XRADAR_CLIENT_KIND: 'browser',
    XRADAR_RUNTIME: 'browser',
    XRADAR_PLATFORM: 'browser',
  }),
}

export function reportError(error: unknown, context?: Record<string, unknown>) {
  captureError(config, error instanceof Error ? error : new Error(String(error)), {
    handled: true,
    severity: 'error',
    url: location.href,
    tags: { page: location.pathname },
    context: context ?? {},
  })
}

export function reportUnhandledError(error: unknown) {
  captureError(config, error instanceof Error ? error : new Error(String(error)), {
    handled: false,
    severity: 'fatal',
    url: location.href,
    tags: { page: location.pathname },
  })
}

export function traceApiCall(method: string, path: string): Span {
  return startSpan(`http.client ${method} ${path}`, { spanKind: 'client' })
}

export async function finishApiSpan(
  span: Span,
  status: 'ok' | 'error',
  data: Record<string, unknown> = {},
) {
  await recordSpan(config, span, status, data)
}

export function initGlobalErrorHandlers() {
  const bootstrapUrl = (import.meta.env.VITE_W_RADAR_BOOTSTRAP_URL as string | undefined)?.trim();
  const workspaceId = (import.meta.env.VITE_W_RADAR_WORKSPACE_ID as string | undefined)?.trim();
  const projectId = (import.meta.env.VITE_W_RADAR_PROJECT_ID as string | undefined)?.trim();
  if (RADAR_TRACKING_CODE && bootstrapUrl && workspaceId && projectId) {
    const script = document.createElement('script');
    script.async = true;
    script.src = bootstrapUrl;
    script.dataset.clientKind = 'paas-platform';
    script.dataset.workspaceId = workspaceId;
    script.dataset.projectId = projectId;
    script.dataset.code = RADAR_TRACKING_CODE;
    document.head.append(script);
  }

  window.addEventListener('error', (event) => {
    reportUnhandledError(event.error ?? event.message)
  })
  window.addEventListener('unhandledrejection', (event) => {
    reportUnhandledError(event.reason)
  })
}
