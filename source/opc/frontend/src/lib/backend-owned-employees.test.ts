import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned employees UI", () => {
  it("loads employee display labels and permissions from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Employees.tsx"), "utf8");

    expect(source).toContain("/api/employees/overview");
    expect(source).toContain("employee.status_badge");
    expect(source).toContain("employee.employment_type_badge");
    expect(source).toContain("employee.base_salary_label");
    expect(source).toContain("employee.hire_date_label");
    expect(source).toContain("employee.can_delete");
    expect(source).toContain("employee.delete_action");
    expect(source).toContain("employee.update_action");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("overview.empty_message");

    expect(source).not.toContain("setEmployeeToDelete(employee.id)");
    expect(source).not.toContain("api.delete(`/api/employees/${id}`)");
    expect(source).not.toContain('queryKey: ["employees"]');
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getEmploymentTypeBadge");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("zhCN");
  });

  it("keeps employee salary field normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "components/employees/EmployeeDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("base_salary: parse");
    expect(source).not.toContain("social_security_base: parse");
    expect(source).not.toContain("housing_fund_base: parse");
    expect(source).toContain("/api/employees/form_options");
    expect(source).toContain("formOptions?.status_options");
    expect(source).toContain("formOptions?.employment_type_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`/api/employees/${employee.id}`");
    expect(source).not.toContain("api.post('/api/employees'");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("user_id: user.id");
    expect(source).not.toContain('<SelectItem value="active">');
    expect(source).not.toContain('<SelectItem value="full_time">');
  });
});
