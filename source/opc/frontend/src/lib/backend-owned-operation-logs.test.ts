import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned operation logs UI", () => {
  it("loads filtering, stats, operation labels, table labels, timestamps, and export content from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/OperationLogs.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.operationLogsOverview");
    expect(contracts).toContain("operationLogsOverview");
    expect(source).toContain("overview.permissions");
    expect(source).not.toContain('type_options: [{ value: "all", label: "全部类型" }]');
    expect(source).not.toContain('table_options: [{ value: "all", label: "全部表" }]');
    expect(source).not.toContain("useAuth()");
    expect(source).not.toContain("isAdmin");

    expect(source).not.toContain('queryKey: ["operation_logs"]');
    expect(source).not.toContain("/api/logs");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain("getOperationBadge");
    expect(source).not.toContain("getTableLabel");
    expect(source).not.toContain("new Date");
    expect(source).not.toContain("safeFormat");
  });
});
