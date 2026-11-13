import { toast } from "sonner";

/**
 * Extract a user-facing message from axios/fetch/ApiError-like failures.
 * Aligns with `@/lib/api/errors` codes and the raw Error strings thrown by
 * `@/integrations/api/client` (`Unauthorized`, JSON body text, `HTTP <status>`).
 */
export function apiErrorMessage(err: unknown): string {
  if (err == null) {
    return "发生未知错误，请稍后重试";
  }

  const code =
    typeof err === "object" && err !== null && "code" in err
      ? String((err as { code?: unknown }).code || "")
      : "";

  const codeMessages: Record<string, string> = {
    UNAUTHORIZED: "登录已过期，请重新登录",
    NOT_FOUND: "数据不存在",
    NETWORK_ERROR: "网络连接失败，请检查网络",
    TIMEOUT: "请求超时，请稍后重试",
    CONFLICT: "记录已存在，请检查是否重复",
  };
  if (code && codeMessages[code]) {
    return codeMessages[code];
  }

  let raw = "";
  if (err instanceof Error) {
    raw = err.message;
  } else if (typeof err === "string") {
    raw = err;
  } else if (typeof err === "object" && err !== null && "message" in err) {
    raw = String((err as { message?: unknown }).message || "");
  } else {
    raw = String(err);
  }

  const lower = raw.toLowerCase();
  if (
    lower === "unauthorized" ||
    lower.includes("401") ||
    /登录已过期|未授权/.test(raw)
  ) {
    return "登录已过期，请重新登录";
  }

  if (
    /failed to fetch|networkerror|network request failed|load failed|econnrefused|err_network/i.test(
      raw,
    )
  ) {
    return "网络连接失败，请检查网络";
  }

  // Backend often returns JSON text as Error.message
  const trimmed = raw.trim();
  if (
    (trimmed.startsWith("{") && trimmed.endsWith("}")) ||
    (trimmed.startsWith("[") && trimmed.endsWith("]"))
  ) {
    try {
      const parsed = JSON.parse(trimmed) as {
        message?: unknown;
        error?: unknown;
        detail?: unknown;
      };
      if (typeof parsed.message === "string" && parsed.message.trim()) {
        return parsed.message;
      }
      if (typeof parsed.error === "string" && parsed.error.trim()) {
        return parsed.error;
      }
      if (
        parsed.error &&
        typeof parsed.error === "object" &&
        parsed.error !== null &&
        "message" in parsed.error &&
        typeof (parsed.error as { message?: unknown }).message === "string"
      ) {
        return String((parsed.error as { message: string }).message);
      }
      if (typeof parsed.detail === "string" && parsed.detail.trim()) {
        return parsed.detail;
      }
    } catch {
      // fall through
    }
  }

  const statusMatch = raw.match(/HTTP\s+(\d{3})/i);
  if (statusMatch) {
    const status = Number(statusMatch[1]);
    if (status === 401 || status === 403) {
      return "登录已过期，请重新登录";
    }
    if (status >= 500) {
      return "服务器错误，请稍后重试";
    }
    if (status === 404) {
      return "数据不存在";
    }
  }

  if (raw && raw !== "[object Object]") {
    return raw;
  }

  return "发生未知错误，请稍后重试";
}

const recentToasts = new Map<string, number>();
const TOAST_DEDUP_MS = 2000;

/** Deduplicate identical toasts within a short window (multi-query pages). */
export function toastOnce(
  message: string,
  kind: "error" | "success" | "info" = "error",
): void {
  const now = Date.now();
  const last = recentToasts.get(message);
  if (last !== undefined && now - last < TOAST_DEDUP_MS) {
    return;
  }
  recentToasts.set(message, now);

  // Prune stale entries occasionally
  if (recentToasts.size > 50) {
    for (const [key, ts] of recentToasts) {
      if (now - ts >= TOAST_DEDUP_MS) {
        recentToasts.delete(key);
      }
    }
  }

  if (kind === "success") {
    toast.success(message);
  } else if (kind === "info") {
    toast.message(message);
  } else {
    toast.error(message);
  }
}

/** Reset dedup state (tests only). */
export function __resetToastOnceForTests(): void {
  recentToasts.clear();
}
