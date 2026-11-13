import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned projects UI", () => {
  it("loads project statistics, filters, and R&D cost grouping from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/Projects.tsx"), "utf8");
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.projectsOverview");
    expect(contracts).toContain("projectsOverview");
    expect(source).toContain("overview.status_options");
    expect(source).not.toContain('status_options: [{ value: "all", label: "全部状态" }]');
    expect(source).toContain("project.status_badge");
    expect(source).toContain("project.budget_label");
    expect(source).toMatch(/project\.rd_cost_summary\s*\.total_label/);
    expect(source).toContain("cost.amount_label");
    expect(source).toContain("project.can_delete");
    expect(source).toContain("project.delete_action");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("selectedProject.update_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("cost.can_delete");
    expect(source).toContain("cost.delete_action");
    expect(source).toContain("cost.update_action");
    expect(source).toContain("project.rd_cost_create_action");

    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("isAdmin");
    expect(source).not.toContain("useMemo");
    expect(source).not.toContain('queryKey: ["rd_costs"]');
    expect(source).not.toContain("allRdCosts.reduce");
    expect(source).not.toContain("projects.reduce");
    expect(source).not.toContain("projects.filter");
    expect(source).not.toContain("rdCostsByProject =");
    expect(source).not.toContain("costTypeLabels");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("statusConfig");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("仅管理员可执行删除操作");
    expect(source).not.toContain("handleDelete(project.id)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.projects");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.projects}/${id}`");
    expect(source).not.toContain("setRdCostToDelete(cost.id)");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.projects}/${id}`)",
    );
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.rdCosts}/${id}`)",
    );
    expect(source).not.toContain("deleteMutation.mutate(projectToDelete)");
    expect(source).not.toContain(
      "deleteRdCostMutation.mutate(rdCostToDelete)",
    );
    expect(source).not.toContain(
      '<SelectItem value="planning">规划中</SelectItem>',
    );
  });

  it("keeps project form numeric normalization on the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/projects/ProjectDialog.tsx"),
      "utf8",
    );

    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("parseInt");
    expect(dialog).not.toContain("budget: parse");
    expect(dialog).not.toContain("team_members: parse");
    expect(dialog).not.toContain("format(new Date()");
    expect(dialog).not.toContain("start_date: format(new Date()");
    expect(dialog).toContain("/api/v1/projects/form_options");
    expect(dialog).toContain("formOptions?.project_type_options");
    expect(dialog).toContain("formOptions?.default_values");
    expect(dialog).not.toContain('<SelectItem value="tech">');
  });

  it("keeps R&D cost amount normalization on the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/projects/RdCostDialog.tsx"),
      "utf8",
    );

    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("amount: parse");
    expect(dialog).not.toContain("cost_date: new Date()");
    expect(dialog).not.toContain("date > new Date()");
    expect(dialog).not.toContain("format(data.cost_date");
    expect(dialog).not.toContain("assets.filter");
    expect(dialog).toContain("rdCostTypeOptions");
    expect(dialog).toContain("employeeOptions");
    expect(dialog).toContain("assetOptions");
    expect(dialog).toContain("api.runAction(action");
    expect(dialog).not.toContain("api.put(`/api/rd_costs/${cost.id}`");
    expect(dialog).not.toContain("api.post('/api/rd_costs'");
    expect(dialog).not.toContain("costTypeOptions = [");
    expect(dialog).not.toContain("/api/employees");
    expect(dialog).not.toContain("/api/assets/overview?asset_type=physical");
    expect(dialog).not.toContain("api.get<{ data: any[]; count: number }>");

    const page = readFileSync(resolve(root, "pages/Projects.tsx"), "utf8");
    expect(page).toContain("overview.rd_cost_type_options");
    expect(page).toContain("overview.employee_options");
    expect(page).toContain("overview.asset_options");
  });
});
