import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned tax management UI", () => {
  it("loads tax summary values from the backend management endpoint", () => {
    const source = readFileSync(
      resolve(root, "pages/TaxManagement.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/tax/management-summary");
    expect(source).toContain("taxSummary.declarations");
    expect(source).toContain("taxSummary.pending_declarations");
    expect(source).toContain("taxSummary.declared_declarations");
    expect(source).toContain("taxSummary.permissions.can_delete");
    expect(source).toContain("companyInfo.taxpayer_type_label");
    expect(source).toContain("companyInfo.invoice_limit_label");
    expect(source).toContain("invoiceTax.deductible_total_label");
    expect(source).toContain("invoiceTax.output_tax_due_label");
    expect(source).toContain("declaration.declaration_type_label");
    expect(source).toContain("declaration.declaration_period_label");
    expect(source).toContain("declaration.period_label");
    expect(source).toContain("declaration.deadline_label");
    expect(source).toContain("declaration.declaration_date_label");
    expect(source).toContain("declaration.tax_amount_label");
    expect(source).toContain("declaration.declared_amount_label");
    expect(source).toContain("declaration.status_badge");
    expect(source).toContain("declaration.can_delete");
    expect(source).toContain("declaration.delete_action");
    expect(source).toContain("declaration.update_action");
    expect(source).toContain("taxSummary.declaration_create_action");
    expect(source).not.toContain("API_ENDPOINTS.companyInfo");
    expect(source).not.toContain("API_ENDPOINTS.invoices");
    expect(source).not.toContain("inputInvoices");
    expect(source).not.toContain("deductibleTotal");
    expect(source).not.toContain("outputTotal");
    expect(source).not.toContain("amount * 0.13");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("declarationTypeLabels");
    expect(source).not.toContain("declarationPeriodLabels");
    expect(source).not.toContain("statusLabels");
    expect(source).not.toContain("taxpayerTypeLabels");
    expect(source).not.toContain(
      "declarations?.filter(d => d.status === 'pending')",
    );
    expect(source).not.toContain(
      "declarations?.filter(d => d.status === 'declared')",
    );
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("${TAX_DECLARATIONS_ENDPOINT}/${declarationToDelete.id}");
    expect(source).not.toContain("确定要删除这条税务申报记录吗？此操作无法撤销。");
  });

  it("keeps tax declaration form amounts on the backend", () => {
    const source = readFileSync(
      resolve(root, "components/tax/TaxDeclarationDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("tax_amount: data.tax_amount ? parse");
    expect(source).toContain("/api/v1/tax_declarations/form_options");
    expect(source).toContain("formOptions?.declaration_type_options");
    expect(source).toContain("formOptions?.declaration_period_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.taxDeclarations}/${declaration.id}`");
    expect(source).not.toContain("api.post(API_ENDPOINTS.taxDeclarations");
    expect(source).not.toContain('<option value="">请选择</option>');
    expect(source).not.toContain("auth.getCurrentUser");
    expect(source).not.toContain("user_id: user.id");
    expect(source).not.toContain('<option value="vat">');
    expect(source).not.toContain('<option value="monthly">');
    expect(source).not.toContain("status: data.declaration_date ? 'declared' : 'pending'");
  });
});
