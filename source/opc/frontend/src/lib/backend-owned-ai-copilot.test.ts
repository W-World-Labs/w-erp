import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned AI copilot", () => {
  it("sends the current path and leaves page context interpretation to the backend", () => {
    const source = readFileSync(
      resolve(root, "components/ai/AICopilot.tsx"),
      "utf8",
    );

    expect(source).toContain("path: location.pathname");
    expect(source).toContain("/api/ai/chat-context");
    expect(source).toContain("chatContext.chat_action");
    expect(source).toContain("api.runAction<{ reply: string; success: boolean }>");
    expect(source).toContain("chatContext?.quick_actions");
    expect(source).not.toContain("api.post");
    expect(source).not.toContain("contextMap");
    expect(source).not.toContain("context: getContext()");
    expect(source).not.toContain("用户正在查看仪表盘首页");
    expect(source).not.toContain("用户正在管理合同");
  });
});
