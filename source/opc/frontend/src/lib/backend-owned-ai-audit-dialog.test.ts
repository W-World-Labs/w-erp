import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned AI audit dialog", () => {
  it("loads audit copy from the backend and posts a unified audit payload", () => {
    const source = readFileSync(
      resolve(root, "components/applications/AIAuditDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/ai/audit-context");
    expect(source).toContain("auditContext.heading");
    expect(source).toContain("auditContext.description");
    expect(source).toContain("auditContext.feature_items");
    expect(source).toContain("auditContext.audit_action");
    expect(source).toContain("api.runAction<{ analysis: string }>");
    expect(source).toContain("target_id: targetId");

    expect(source).not.toContain("api.post");
    expect(source).not.toContain("Promise.resolve({ data: { session");
    expect(source).not.toContain("auditType ===");
    expect(source).not.toContain("application_id: applicationId");
    expect(source).not.toContain("contract_id: contractId");
    expect(source).not.toContain("employee_id: employeeId");
    expect(source).not.toContain("budget_id: budgetId");
    expect(source).not.toContain("project_id: projectId");
    expect(source).not.toContain("全面项目评估");
    expect(source).not.toContain("合同风险审核");
  });
});
