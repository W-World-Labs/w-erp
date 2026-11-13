import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned invoices UI", () => {
  it("loads invoice direction groups and statistics from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Invoices.tsx"), "utf8");
    const detail = readFileSync(
      resolve(root, "components/invoices/InvoiceDetailDialog.tsx"),
      "utf8",
    );
    const combined = `${source}\n${detail}`;

    expect(source).toContain("/api/invoices/overview");
    expect(source).toContain("status_options");
    expect(source).not.toContain('status_options: [{ value: "all", label: "全部状态" }]');
    expect(source).toContain("status_badge");
    expect(source).toContain("can_mark_sent");
    expect(source).toContain("can_mark_paid");
    expect(source).toContain("mark_sent_action");
    expect(source).toContain("mark_paid_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("/mark_sent");
    expect(source).not.toContain("/mark_paid");
    expect(source).not.toContain("API_ENDPOINTS.invoices}/${id}/mark");
    expect(combined).toContain("amount_label");
    expect(combined).toContain("tax_amount_label");
    expect(source).toContain("can_delete");
    expect(source).toContain("invoice.delete_action");
    expect(source).toContain("invoice.update_action");
    expect(source).toContain("invoiceOverview.create_action");
    expect(source).not.toContain("setInvoiceToDelete(invoice.id)");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.invoices}/${invoiceToDelete}`)",
    );

    expect(combined).not.toContain("useAuth");
    expect(combined).not.toContain("isAdmin");
    expect(source).not.toContain("useMemo");
    expect(source).not.toContain("const outputStats =");
    expect(source).not.toContain("const inputStats =");
    expect(source).not.toContain("invoices.filter(inv => inv.invoice_type");
    expect(source).not.toContain("filteredInvoices =");
    expect(source).not.toContain(".reduce((sum, inv)");
    expect(combined).not.toContain("statusMap");
    expect(source).not.toContain("Object.entries(statusMap)");
    expect(combined).not.toContain("toLocaleString");
    expect(combined).not.toContain("safeFormat");
    expect(combined).not.toContain("仅管理员可执行删除操作");
  });

  it("delegates invoice import parsing validation duplicates and display metadata to backend", () => {
    const source = readFileSync(
      resolve(root, "components/invoices/InvoiceImportDialog.tsx"),
      "utf8",
    );
    const page = readFileSync(resolve(root, "pages/Invoices.tsx"), "utf8");

    expect(page).toContain("import_preview_action");
    expect(page).toContain("import_commit_action");
    expect(source).toContain("previewAction");
    expect(source).toContain("commitAction");
    expect(source).toContain("api.runAction<InvoiceImportPreview>");
    expect(source).toContain("api.runAction<InvoiceImportCommit>");
    expect(source).toContain("preview.stats");
    expect(source).toContain("row.status_badge");
    expect(source).toContain("row.amount_label");
    expect(source).toContain("preview.import_button_label");
    expect(source).toContain("preview.can_import");

    expect(source).not.toContain("DOMParser");
    expect(source).not.toContain("XLSX.read");
    expect(source).not.toContain("sheet_to_json");
    expect(source).not.toContain("parseXML");
    expect(source).not.toContain("parseCSV");
    expect(source).not.toContain("validateInvoice");
    expect(source).not.toContain("checkDuplicates");
    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("new Set");
    expect(source).not.toContain("parsedInvoices.filter");
    expect(source).not.toContain("API_ENDPOINTS.invoiceImportPreview");
    expect(source).not.toContain("API_ENDPOINTS.invoiceImportCommit");
    expect(source).not.toContain("api.post<InvoiceImportPreview>");
    expect(source).not.toContain("api.post<InvoiceImportCommit>");
    expect(source).not.toContain("invoice.status ===");
    expect(source).not.toContain("invoice.amount.toFixed");
    expect(source).not.toContain("api.get<{ data: any[]; count: number }>");
    expect(source).not.toContain("api.post('/api/invoices'");
  });

  it("keeps invoice form amount and date normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "components/invoices/InvoiceDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/invoices/form_options");
    expect(source).toContain("formOptions?.invoice_type_options");
    expect(source).toContain("formOptions?.default_invoice_type");
    expect(source).toContain("formOptions?.client_options");
    expect(source).toContain('name="client_id"');
    expect(source).toContain("invoiceData.client_id");
    expect(source).toContain("normalizeInvoiceTypeUi");
    expect(source).not.toContain("formOptions?.status_options");
    expect(source).not.toContain("formOptions?.default_status");

    expect(source).not.toContain("parseFloat");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`/api/invoices/${invoice.id}`");
    expect(source).not.toContain("api.post<{ id: string }>('/api/invoices'");
    expect(source).not.toContain("amount: parse");
    expect(source).not.toContain("tax_amount: data.tax_amount ? parse");
    expect(source).not.toContain("format(data.invoice_date");
    expect(source).not.toContain("format(data.due_date");
    expect(source).not.toContain("status: data.status");
    expect(source).not.toContain('name="status"');
    expect(source).not.toContain("const statusOptions = [");
    expect(source).not.toContain("z.enum(['pending'");
    expect(source).not.toContain("z.enum(['input'");
  });

  it("does not fetch optional payment lists inside the invoice form", () => {
    const source = readFileSync(
      resolve(root, "components/invoices/InvoiceDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("'/api/payments'");
    expect(source).not.toContain("setPayments");
    expect(source).not.toContain("const [payments");
    expect(source).not.toContain("Fetch payments for selection");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("请先登录");
  });

  it("stores and displays invoice attachments from backend-owned file metadata", () => {
    const dialog = readFileSync(
      resolve(root, "components/invoices/InvoiceDialog.tsx"),
      "utf8",
    );
    const detail = readFileSync(
      resolve(root, "components/invoices/InvoiceDetailDialog.tsx"),
      "utf8",
    );
    const page = readFileSync(resolve(root, "pages/Invoices.tsx"), "utf8");

    expect(dialog).toContain("const uploadResult = await files.upload(file);");
    expect(dialog).toContain("return uploadResult.id;");
    expect(dialog).toContain("file_path: filePath");
    expect(page).toContain("file_name: string | null");
    expect(detail).toContain("invoice.file_name");
    expect(detail).toContain("files.download(invoice.file_path)");
    expect(detail).not.toContain("TODO: implement file download");
    expect(detail).not.toContain("split(\"/\")");
    expect(detail).not.toContain("split('/')");
    expect(dialog).not.toContain("fileExt");
    expect(dialog).not.toContain("${user.id}/${fileName}");
  });
});
