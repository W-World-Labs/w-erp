import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned user management UI", () => {
  it("loads member filtering, stats, permissions, role labels, and assignable roles from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/UserManagement.tsx"),
      "utf8",
    );
    const client = readFileSync(
      resolve(root, "integrations/api/client.ts"),
      "utf8",
    );

    expect(source).toContain("companies.members.overview");
    expect(client).toContain("overview: (companyId: string");

    expect(source).not.toContain("ROLE_CONFIG");
    expect(source).not.toContain("ASSIGNABLE_ROLES");
    expect(source).not.toContain("getRoleBadge");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain("members?.");
    expect(source).not.toContain("myRole ===");
    expect(source).not.toContain("member.role !==");
  });

  it("keeps company switcher role display owned by the backend", () => {
    const source = readFileSync(
      resolve(root, "components/CompanySwitcher.tsx"),
      "utf8",
    );
    const client = readFileSync(
      resolve(root, "integrations/api/client.ts"),
      "utf8",
    );

    expect(source).toContain("company.role_badge");
    expect(source).toContain("currentCompany?.role_badge?.label");
    expect(client).toContain("role_badge?: CompanyMemberRoleBadge");
    expect(source).not.toContain("roleMap");
    expect(source).not.toContain("getRoleBadge");
    expect(source).not.toContain("currentCompany?.role ===");
  });

  it("initializes the switcher from the backend current company marker", () => {
    const source = readFileSync(
      resolve(root, "components/CompanySwitcher.tsx"),
      "utf8",
    );
    const client = readFileSync(
      resolve(root, "integrations/api/client.ts"),
      "utf8",
    );

    expect(client).toContain("is_current?: boolean");
    expect(source).toContain("companiesData.find(c => c.is_current)");
    expect(source).toContain("currentCompanyFromBackend");
  });

  it("links company settings to an existing admin route", () => {
    const source = readFileSync(
      resolve(root, "components/CompanySwitcher.tsx"),
      "utf8",
    );

    expect(source).toContain("import { Link } from 'react-router-dom'");
    expect(source).toContain('<Link to="/settings">');
    expect(source).not.toContain('/settings/companies');
  });
});
