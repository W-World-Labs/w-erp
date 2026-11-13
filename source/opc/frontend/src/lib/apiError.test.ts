import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  apiErrorMessage,
  toastOnce,
  __resetToastOnceForTests,
} from "./apiError";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
    message: vi.fn(),
  },
}));

import { toast } from "sonner";

describe("apiErrorMessage", () => {
  it("maps ApiError-like codes", () => {
    expect(apiErrorMessage({ code: "UNAUTHORIZED", message: "x" })).toBe(
      "登录已过期，请重新登录",
    );
    expect(apiErrorMessage({ code: "NETWORK_ERROR", message: "x" })).toBe(
      "网络连接失败，请检查网络",
    );
  });

  it("parses JSON body message from Error", () => {
    expect(
      apiErrorMessage(new Error(JSON.stringify({ message: "余额不足" }))),
    ).toBe("余额不足");
  });

  it("maps Unauthorized and HTTP statuses", () => {
    expect(apiErrorMessage(new Error("Unauthorized"))).toBe(
      "登录已过期，请重新登录",
    );
    expect(apiErrorMessage(new Error("HTTP 503"))).toBe(
      "服务器错误，请稍后重试",
    );
  });

  it("maps network failures", () => {
    expect(apiErrorMessage(new Error("Failed to fetch"))).toBe(
      "网络连接失败，请检查网络",
    );
  });
});

describe("toastOnce", () => {
  beforeEach(() => {
    __resetToastOnceForTests();
    vi.mocked(toast.error).mockClear();
  });

  it("dedupes identical messages within the window", () => {
    toastOnce("数据加载失败:网络连接失败，请检查网络");
    toastOnce("数据加载失败:网络连接失败，请检查网络");
    expect(toast.error).toHaveBeenCalledTimes(1);
  });
});
