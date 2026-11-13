import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned asset records UI", () => {
  it("loads stats, filtering, action labels, display dates, and export content from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/Records.tsx"), "utf8");
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.assetRecordsOverview");
    expect(contracts).toContain("assetRecordsOverview");
    expect(source).not.toContain('action_options: [{ value: "all", label: "全部类型" }]');

    expect(source).not.toContain('queryKey: ["asset_records"]');
    expect(source).not.toContain("/api/asset_records");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain("getActionBadge");
    expect(source).not.toContain("getActionLabel");
    expect(source).not.toContain("new Date");
    expect(source).not.toContain("safeFormat");
  });
});
