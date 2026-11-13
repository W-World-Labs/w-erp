import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned salary UI", () => {
  it("loads salary rows, totals, trends, and cost analysis from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Salary.tsx"), "utf8");
    const analysis = readFileSync(
      resolve(root, "components/salary/SalaryCostAnalysis.tsx"),
      "utf8",
    );
    const dialog = readFileSync(
      resolve(root, "components/salary/SalaryDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/salary/overview");
    expect(source).toContain("search=");
    expect(source).toContain("salaryOverview.create_action");
    expect(source).toContain("selectedSalary.update_action");
    expect(source).toContain("salary.mark_paid_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.salary");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.salary}/${selectedSalary.id}`");
    expect(source).not.toContain("/mark_paid");
    expect(source).not.toContain("API_ENDPOINTS.salary}/${id}/mark_paid");
    expect(source).toContain("salary.delete_action");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.salary}/${id}`)",
    );
    expect(source).not.toContain("deleteMutation.mutate(salaryToDelete)");
    expect(source).not.toContain("handleDelete(salary.id)");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("API_ENDPOINTS.employees");
    expect(source).not.toContain("salaries?.filter");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("format(new Date()");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("isAdmin");
    expect(source).not.toContain("new Map((employees");
    expect(source).not.toContain("const monthlyTrend = (() =>");
    expect(source).not.toContain("const stats = {");
    expect(source).not.toContain("salaries?.reduce");
    expect(source).not.toContain("subMonths");
    expect(source).not.toContain('status: "paid"');

    expect(analysis).not.toContain("subMonths");
    expect(analysis).not.toContain("const monthlyTrend = (() =>");
    expect(analysis).not.toContain("const departmentCosts = (() =>");
    expect(analysis).not.toContain("monthOverMonth");
    expect(analysis).not.toContain("toLocaleString");
    expect(analysis).not.toContain("toFixed");
    expect(analysis).not.toContain("employee_cost / stats.total_cost");

    expect(dialog).not.toContain("total_salary:");
    expect(dialog).not.toContain("actual_salary:");
    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("baseSalary");
  });
});
