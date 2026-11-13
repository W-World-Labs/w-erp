import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned salary insurance preview", () => {
  it("loads the current social insurance config from the salary overview", () => {
    const source = readFileSync(resolve(root, "pages/Salary.tsx"), "utf8");
    const dialog = readFileSync(
      resolve(root, "components/salary/SocialInsuranceConfigDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("current_insurance_config");
    expect(source).toContain("insurance_config_create_action");
    expect(source).toContain("insurance_preview_action");
    expect(source).toContain("currentInsuranceConfig?.update_action");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain('queryKey: ["social-insurance-config"]');
    expect(source).not.toContain("const { data: insuranceConfig }");
    expect(source).not.toContain("Array.isArray(insuranceConfig)");
    expect(source).not.toContain("insuranceConfig[0]");
    expect(source).not.toContain("API_ENDPOINTS.socialInsuranceConfig");

    expect(dialog).toContain("social_security_base_min_input");
    expect(dialog).not.toContain("?.toString()");
  });

  it("delegates insurance and housing fund calculation to the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/salary/SalaryDialog.tsx"),
      "utf8",
    );
    expect(dialog).toContain("insurancePreviewAction");
    expect(dialog).toContain("api.runAction<SalaryInsurancePreview>");
    expect(dialog).toContain("preview.social_security_input");

    expect(dialog).not.toContain("API_ENDPOINTS.salaryInsurancePreview");
    expect(dialog).not.toContain("api.post");
    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("toFixed");
    expect(dialog).not.toContain("format(new Date()");
    expect(dialog).not.toContain('period: format(new Date(), "yyyy-MM")');
    expect(dialog).not.toContain("insuranceConfig");
    expect(dialog).not.toContain("employees.find");
    expect(dialog).not.toContain("Math.max(");
    expect(dialog).not.toContain("Math.min(");
    expect(dialog).not.toContain("pension_personal_rate");
    expect(dialog).not.toContain("housing_fund_personal_rate");
    expect(dialog).not.toContain("social_security_base_min");
  });

  it("keeps social insurance config numeric normalization on the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/salary/SocialInsuranceConfigDialog.tsx"),
      "utf8",
    );

    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("social_security_base_min: parse");
    expect(dialog).not.toContain("housing_fund_company_rate: parse");
  });
});
