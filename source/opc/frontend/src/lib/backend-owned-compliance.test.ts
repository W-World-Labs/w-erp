import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned compliance UI", () => {
  it("loads insurance and pension stats from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/ComplianceManagement.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.complianceOverview");
    expect(source).toContain("complianceStats.monthly_compliance_cost_label");
    expect(source).toContain("i.insurance_type_label");
    expect(source).toContain("i.premium_amount_label");
    expect(source).toContain("i.coverage_amount_label");
    expect(source).toContain("i.coverage_period_label");
    expect(source).toContain("i.status_badge");
    expect(source).toContain("i.update_action");
    expect(source).toContain("i.delete_action");
    expect(source).toContain("overview?.insurance_create_action");
    expect(source).toContain("p.plan_type_label");
    expect(source).toContain("p.country_label");
    expect(source).toContain("p.contribution_employer_label");
    expect(source).toContain("p.contribution_employee_label");
    expect(source).toContain("p.current_value_label");
    expect(source).toContain("p.status_badge");
    expect(source).toContain("p.update_action");
    expect(source).toContain("p.delete_action");
    expect(source).toContain("overview?.pension_create_action");
    expect(source).toContain("api.runAction(action");
    expect(contracts).toContain("complianceOverview");
    expect(source).not.toContain('queryKey: ["insurance_policies"]');
    expect(source).not.toContain('queryKey: ["pension_plans"]');
    expect(source).not.toContain("insurances.filter");
    expect(source).not.toContain("pensions.filter");
    expect(source).not.toContain(".reduce(");
    expect(source).not.toContain("new Date().toISOString");
    expect(source).not.toContain("insTypeLabel");
    expect(source).not.toContain("penTypeLabel");
    expect(source).not.toContain("statusBadge");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("Number(");
    expect(source).not.toContain('confirm("确定删除?")');
    expect(source).not.toContain("deleteIns.mutate(i.id)");
    expect(source).not.toContain("deletePen.mutate(p.id)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.insurancePolicies}/${id}`)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.pensionPlans}/${id}`)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.insurancePolicies");
    expect(source).not.toContain(
      "api.put(`${API_ENDPOINTS.insurancePolicies}/${id}`",
    );
    expect(source).not.toContain("api.post(API_ENDPOINTS.pensionPlans");
    expect(source).not.toContain(
      "api.put(`${API_ENDPOINTS.pensionPlans}/${id}`",
    );
  });

  it("keeps compliance form numeric normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "pages/ComplianceManagement.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("vesting_period_months = parse");
  });
});
