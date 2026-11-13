import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned audit center UI", () => {
  it("loads audit modules, stats, quick items, and capability copy from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/AuditCenter.tsx"), "utf8");
    const contracts = readFileSync(resolve(root, "lib/api/contracts.ts"), "utf8");

    expect(contracts).toContain("auditCenterOverview");
    expect(source).toContain("API_ENDPOINTS.auditCenterOverview");
    expect(source).toContain("overview.stats.total_auditable_label");
    expect(source).toContain("overview.stats.module_count_label");
    expect(source).toContain("overview.stats.analysis_label");
    expect(source).toContain("overview.modules");
    expect(source).toContain("module.quick_items");
    expect(source).toContain("module.empty_message");
    expect(source).toContain("overview.capability_sections");
    expect(source).toContain("item.audit_title");
    expect(source).toContain("item.audit_request");

    expect(source).not.toContain('queryKey: ["audit-applications"]');
    expect(source).not.toContain('queryKey: ["audit-contracts"]');
    expect(source).not.toContain('queryKey: ["audit-employees"]');
    expect(source).not.toContain('queryKey: ["audit-budgets"]');
    expect(source).not.toContain('queryKey: ["audit-projects"]');
    expect(source).not.toContain("api.get<{ data: any[]; count: number }>('/api/applications')");
    expect(source).not.toContain("api.get<{ data: any[]; count: number }>('/api/contracts')");
    expect(source).not.toContain("applications.length + contracts.length");
    expect(source).not.toContain("const auditModules = [");
    expect(source).not.toContain("module.data.slice");
    expect(source).not.toContain("module.getTitle");
    expect(source).not.toContain("auditConfig.type ===");
  });
});
