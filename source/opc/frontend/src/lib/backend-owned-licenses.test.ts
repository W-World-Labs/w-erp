import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned licenses UI", () => {
  it("renders license display rows, badges, dates, and permissions from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/Licenses.tsx"), "utf8");

    expect(source).toContain("/api/v1/licenses/overview");
    expect(source).toContain("license.status_badge");
    expect(source).toContain("license.license_type_label");
    expect(source).toContain("license.issue_date_label");
    expect(source).toContain("license.expiry_date_label");
    expect(source).toContain("license.can_delete");
    expect(source).toContain("license.delete_action");
    expect(source).toContain("license.update_action");
    expect(source).toContain("overview.create_action");
    expect(source).not.toContain("setLicenseToDelete(license.id)");
    expect(source).not.toContain("api.delete(`/api/v1/licenses/${id}`)");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("licenseTypeLabels");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("format(");
    expect(source).not.toContain("isAdmin");
  });

  it("stores license uploads as backend file ids without frontend path generation", () => {
    const source = readFileSync(
      resolve(root, "components/licenses/LicenseDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("const uploadResult = await files.upload(file);");
    expect(source).toContain("path: uploadResult.id");
    expect(source).toContain("name: uploadResult.original_name");
    expect(source).toContain("/api/v1/licenses/form_options");
    expect(source).toContain("formOptions?.license_type_options");
    expect(source).toContain("formOptions?.status_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`/api/licenses/${license.id}`");
    expect(source).not.toContain("api.post('/api/licenses'");
    expect(source).not.toContain("Date.now()");
    expect(source).not.toContain("fileExt");
    expect(source).not.toContain("user.id");
    expect(source).not.toContain("if (!user)");
    expect(source).not.toContain("uploadError");
    expect(source).not.toContain("if (error)");
    expect(source).not.toContain('<SelectItem value="business_license">');
    expect(source).not.toContain('<SelectItem value="valid">');
  });
});
