import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned payments UI", () => {
  it("loads payment filtering, labels, badges, and permissions from the backend overview endpoint", () => {
    const source = [
      readFileSync(resolve(root, "pages/Payments.tsx"), "utf8"),
      readFileSync(
        resolve(root, "components/payments/InvoiceReminderDialog.tsx"),
        "utf8",
      ),
      readFileSync(
        resolve(root, "components/payments/AccountsReceivableAgingPanel.tsx"),
        "utf8",
      ),
    ].join("\n");

    expect(source).toContain("/api/payments/overview");
    expect(source).toContain("/api/v1/ar/aging");
    expect(source).toContain("AccountsReceivableAgingPanel");
    expect(source).toContain("bucket_sections");
    expect(source).toContain("bucket.risk_tone");
    expect(source).toContain("invoice.collection_action");
    expect(source).toContain("handleCollectionReminder");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("queryKey: [\"accounts-receivable-aging\"]");
    expect(source).toContain("invoice.days_overdue_label");
    expect(source).toContain("invoice.amount_label");
    expect(source).toContain("arAging.total_receivable_label");
    expect(source).toContain("arAging.total_overdue_label");
    expect(source).toContain("payment_type");
    expect(source).toContain("payment_method");
    expect(source).toContain("invoice_status");
    expect(source).toContain("payment.payment_type_badge");
    expect(source).toContain("payment.invoice_badge");
    expect(source).toContain("payment.status_badge");
    expect(source).toContain("payment.signed_amount_label");
    expect(source).toContain("payment.payment_date_label");
    expect(source).toContain("payment.payment_method_label");
    expect(source).toContain("payment.can_delete");
    expect(source).toContain("payment.delete_action");
    expect(source).toContain("payment.update_action");
    expect(source).toContain("paymentOverview.create_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("paymentOverview.payment_type_options");
    expect(source).toContain("paymentOverview.payment_method_options");
    expect(source).toContain("paymentOverview.invoice_status_options");
    expect(source).toContain("paymentOverview.empty_message");
    expect(source).toContain("invoice_reminder_stats");
    expect(source).toContain("pending_invoice_payments");
    expect(source).toContain("payment.urgency_badge");
    expect(source).toContain("payment.invoice_deadline_label");
    expect(source).not.toContain('<SelectItem value="all">全部</SelectItem>');
    expect(source).not.toContain('<SelectItem value="all">全部方式</SelectItem>');
    expect(source).not.toContain('<SelectItem value="all">全部发票</SelectItem>');
    expect(source).not.toContain("handleDelete(payment.id)");
    expect(source).not.toContain("deleteMutation.mutate(paymentToDelete)");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.payments}/${id}`)",
    );
    expect(source).not.toContain("api.post(API_ENDPOINTS.payments");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.payments}/${id}`");

    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("PAYMENT_METHOD_LABELS");
    expect(source).not.toContain("filteredPayments");
    expect(source).not.toContain("payments.filter");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getInvoiceBadge");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain("const stats = {");
    expect(source).not.toContain("payments.filter((p)");
    expect(source).not.toContain(".reduce((sum, p)");
    expect(source).not.toContain("new Date(payment.invoice_deadline)");
    expect(source).not.toContain("isOverdue");
    expect(source).not.toContain("differenceInDays");
    expect(source).not.toContain("safeDiffDays");
    expect(source).not.toContain("getUrgencyBadge");
    expect(source).not.toContain("collection-reminder`");
    expect(source).not.toContain("付款提醒：");
    expect(source).not.toContain("已为发票");
  });

  it("keeps payment form normalization on the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/payments/PaymentDialog.tsx"),
      "utf8",
    );

    expect(dialog).toContain("/api/payments/form_options");
    expect(dialog).toContain("formOptions?.payment_type_options");
    expect(dialog).toContain("formOptions?.payment_method_options");
    expect(dialog).toContain("formOptions?.category_options");
    expect(dialog).toContain("formOptions?.status_options");
    expect(dialog).toContain("formOptions?.default_values");
    expect(dialog).toContain("payment_date: payment.payment_date ? new Date(payment.payment_date) : undefined");
    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("amount: parse");
    expect(dialog).not.toContain("format(data.payment_date");
    expect(dialog).not.toContain("format(data.invoice_deadline");
    expect(dialog).not.toContain("payment_date: payment?.payment_date ? new Date(payment.payment_date) : new Date()");
    expect(dialog).not.toContain("payment_date: new Date(payment.payment_date)");
    expect(dialog).not.toContain("PAYMENT_METHODS");
    expect(dialog).not.toContain("PAYMENT_CATEGORIES");
    expect(dialog).not.toContain("<SelectItem value=\"expense\">");
    expect(dialog).not.toContain("<SelectItem value=\"completed\">");
  });
});
