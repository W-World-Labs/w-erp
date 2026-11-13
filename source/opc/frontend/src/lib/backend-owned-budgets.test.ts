import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned budget execution UI", () => {
  it("loads budget execution from the backend overview endpoint", () => {
    const source = readFileSync(
      resolve(root, "pages/BudgetManagement.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/budgets/overview");
    expect(source).toContain("overview.budget_type_options");
    expect(source).toContain("stats.total_budget_label");
    expect(source).toContain("stats.total_actual_label");
    expect(source).toContain("stats.saved_amount_label");
    expect(source).toContain("stats.utilization_label");
    expect(source).toContain("budget.category_label");
    expect(source).toContain("budget.category_icon");
    expect(source).toContain("budget.status_badge");
    expect(source).toContain("budget.budget_amount_label");
    expect(source).toContain("budget.actual_amount_label");
    expect(source).toContain("budget.usage_label");
    expect(source).toContain("budget.remaining_label");
    expect(source).toContain("budget.progress_value");
    expect(source).toContain("budget.progress_class");
    expect(source).toContain("budget.can_delete");
    expect(source).toContain("budget.delete_action");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("selectedBudget.update_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("point.period_label");
    expect(source).toContain("payload.budget_label");
    expect(source).toContain("payload.actual_label");
    expect(source).toContain("overview.empty_message");
    expect(source).not.toContain("calculateActualExpense");
    expect(source).not.toContain("API_ENDPOINTS.salary");
    expect(source).not.toContain("API_ENDPOINTS.invoices");
    expect(source).not.toContain("API_ENDPOINTS.rent");
    expect(source).not.toContain("API_ENDPOINTS.utilities");
    expect(source).not.toContain("actual_amount: actualAmount");
    expect(source).not.toContain("status = actualAmount");
    expect(source).not.toContain("startOfQuarter");
    expect(source).not.toContain("endOfQuarter");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("CATEGORY_LABELS");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain("Math.min");
    expect(source).not.toContain("format(new Date()");
    expect(source).not.toContain("(overview?.trend || []).map");
    expect(source).not.toContain("setBudgetToDelete(budget.id)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.budgets");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.budgets}/${selectedBudget.id}`");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.budgets}/${id}`)",
    );
    expect(source).not.toContain("deleteMutation.mutate(budgetToDelete)");
  });

  it("keeps budget form amount normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "components/budget/BudgetDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("budget_amount: parse");
    expect(source).toContain("/api/budgets/form_options");
    expect(source).toContain("formOptions?.budget_type_options");
    expect(source).toContain("formOptions?.category_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("z.enum(['monthly'");
    expect(source).not.toContain("z.enum(['salary'");
    expect(source).not.toContain('<SelectItem value="monthly">');
    expect(source).not.toContain('<SelectItem value="salary">');
  });
});
