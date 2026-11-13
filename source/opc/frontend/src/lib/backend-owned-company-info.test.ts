import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned company info UI", () => {
  it("exposes branch management as a direct frontend route", () => {
    const appSource = readFileSync(resolve(root, "App.tsx"), "utf8");
    const companyInfoSource = readFileSync(
      resolve(root, "pages/CompanyInfo.tsx"),
      "utf8",
    );

    expect(appSource).toContain('path="branches"');
    expect(appSource).toContain('initialTab="branches"');
    expect(companyInfoSource).toContain('initialTab = "info"');
    expect(companyInfoSource).toContain("<Tabs defaultValue={initialTab}");
  });

  it("uses the backend overview for company display, branches, timeline, labels, and permissions", () => {
    const source = readFileSync(resolve(root, "pages/CompanyInfo.tsx"), "utf8");

    expect(source).toContain("/api/v1/company_info/overview");
    expect(source).toContain("overview.company_create_action");
    expect(source).toContain("companyInfo.update_action");
    expect(source).toContain("overview.branch_create_action");
    expect(source).toContain("branch.update_action");
    expect(source).toContain("overview.milestone_create_action");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("API_ENDPOINTS.companyInfo");
    expect(source).not.toContain("API_ENDPOINTS.branches");
    expect(source).not.toContain("API_ENDPOINTS.employees");
    expect(source).not.toContain("API_ENDPOINTS.projects");
    expect(source).not.toContain("API_ENDPOINTS.contracts");
    expect(source).not.toContain("API_ENDPOINTS.rent");
    expect(source).not.toContain("API_ENDPOINTS.salary");
    expect(source).not.toContain("API_ENDPOINTS.licenses");
    expect(source).not.toContain("API_ENDPOINTS.budgets");
    expect(source).not.toContain("API_ENDPOINTS.milestones");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("parseISO");
    expect(source).not.toContain("isValid");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("buildTimelineEvents");
    expect(source).not.toContain("timelineEvents.filter");
    expect(source).not.toContain("branchTypeLabels");
    expect(source).not.toContain("statusLabels");
    expect(source).toContain("branch.can_activate");
    expect(source).toContain("branch.can_mark_preparing");
    expect(source).toContain("branch.can_suspend");
    expect(source).toContain("branch.can_close");
    expect(source).toContain("branch.delete_action");
    expect(source).toContain("branch.activate_action");
    expect(source).toContain("branch.mark_preparing_action");
    expect(source).toContain("branch.suspend_action");
    expect(source).toContain("branch.close_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("/activate");
    expect(source).not.toContain("/mark_preparing");
    expect(source).not.toContain("/suspend");
    expect(source).not.toContain("/close");
    expect(source).not.toContain("${BRANCHES_ENDPOINT}/${branchToDelete.id}");
    expect(source).not.toContain('确定要删除"{branchToDelete?.branch_name}"吗？此操作无法撤销。');
  });

  it("keeps company and branch form numeric normalization on the backend", () => {
    const source = [
      readFileSync(resolve(root, "components/company/CompanyInfoDialog.tsx"), "utf8"),
      readFileSync(resolve(root, "components/branches/BranchDialog.tsx"), "utf8"),
      readFileSync(resolve(root, "components/timeline/MilestoneDialog.tsx"), "utf8"),
    ].join("\n");

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("safeFormat");
    expect(source).toContain("parentBranchOptions");
    expect(source).not.toContain('queryKey: ["parent-branches"');
    expect(source).not.toContain(".neq(");
    expect(source).not.toContain("employee_count: data.employee_count ? parse");
    expect(source).toContain("/api/v1/company_info/form_options");
    expect(source).toContain("formOptions?.taxpayer_type_options");
    expect(source).toContain("formOptions?.founder_education_options");
    expect(source).toContain("/api/v1/branches/form_options");
    expect(source).toContain("formOptions?.branch_type_options");
    expect(source).toContain("formOptions?.parent_branch_options");
    expect(source).toContain("/api/v1/milestones/form_options");
    expect(source).toContain("formOptions?.milestone_type_options");
    expect(source).toContain("formOptions?.importance_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.post(API_ENDPOINTS.companyInfo");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.companyInfo}/${companyInfo.id}`");
    expect(source).not.toContain("api.post(API_ENDPOINTS.branches");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.branches}/${branch.id}`");
    expect(source).not.toContain("api.post('/api/milestones'");
    expect(source).not.toContain("api.put(`/api/milestones/${milestone.id}`");
    expect(source).not.toContain("z.enum(['general'");
    expect(source).not.toContain('<option value="small_scale">');
    expect(source).not.toContain('<option value="highschool">');
    expect(source).not.toContain("taxpayer_type: data.taxpayer_type || 'small_scale'");
    expect(source).not.toContain("const branchTypes = [");
    expect(source).not.toContain("const statusOptions = [");
    const branchDialogSource = readFileSync(
      resolve(root, "components/branches/BranchDialog.tsx"),
      "utf8",
    );
    expect(branchDialogSource).not.toContain("formOptions?.status_options");
    expect(branchDialogSource).not.toContain("default_values.status");
    expect(branchDialogSource).not.toContain("status: data.status");
    expect(branchDialogSource).not.toContain('watch("status")');
    expect(branchDialogSource).not.toContain('setValue("status"');
    expect(branchDialogSource).not.toContain('htmlFor="status"');
    expect(source).not.toContain('<SelectItem value="none">无上级机构</SelectItem>');
    expect(source).not.toContain('<SelectItem value="funding">');
    expect(source).not.toContain('<SelectItem value="medium">');
    expect(source).not.toContain('importance: milestone?.importance || "medium"');
    expect(source).not.toContain("auth.getCurrentUser");
    expect(source).not.toContain("user_id: user.id");
  });
});
