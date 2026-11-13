import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned recurring invoices UI", () => {
  it("loads recurring invoice stats from the backend overview endpoint", () => {
    const source = readFileSync(
      resolve(root, "pages/RecurringInvoices.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/recurring_invoices/overview");
    expect(source).toContain("total_monthly_label");
    expect(source).toContain("r.amount_label");
    expect(source).toContain("r.frequency_label");
    expect(source).toContain("r.status_badge");
    expect(source).toContain("r.toggle_action");
    expect(source).toContain("r.update_action");
    expect(source).toContain("r.delete_action");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("overview.reminder_create_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("r.total_generated_label");
    expect(source).toContain("r.next_invoice_date_label");
    expect(source).toContain("r.reminder_type_label");
    expect(source).toContain("r.scheduled_date_label");
    expect(source).toContain("r.status_badge");
    expect(source).toContain("r.invoice_display");
    expect(source).not.toContain("const stats = {");
    expect(source).not.toContain("recurring.filter((r: any) => r.status ===");
    expect(source).not.toContain("reminders.filter((r: any) => r.status ===");
    expect(source).not.toContain(".reduce((s: number, r: any)");
    expect(source).not.toContain('queryKey: ["recurring_invoices"]');
    expect(source).not.toContain('queryKey: ["invoice_reminders"]');
    expect(source).not.toContain("getFreqLabel");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getReminderBadge");
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("Number(r.amount");
    expect(source).not.toContain("toLocaleString()");
    expect(source).not.toContain('r.status === "active"');
    expect(source).not.toContain('confirm("确定删除?")');
    expect(source).not.toContain("deleteRecurring.mutate(r.id)");
    expect(source).not.toContain("deleteReminder.mutate(r.id)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.recurringInvoices");
    expect(source).not.toContain(
      "api.put(`${API_ENDPOINTS.recurringInvoices}/${id}`",
    );
    expect(source).not.toContain("api.post(API_ENDPOINTS.invoiceReminders");
  });

  it("keeps recurring invoice form normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "pages/RecurringInvoices.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/recurring_invoices/form_options");
    expect(source).toContain("formOptions?.frequency_options");
    expect(source).toContain("formOptions?.reminder_type_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("amount) values.amount");
    expect(source).not.toContain("day_of_month = parse");
    expect(source).not.toContain("<SelectItem value=\"monthly\">");
    expect(source).not.toContain("<SelectItem value=\"before_due\">");
    expect(source).not.toContain("payment_terms_days || 30");
  });
});
