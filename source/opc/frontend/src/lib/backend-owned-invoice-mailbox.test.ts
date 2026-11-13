import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const root = resolve(process.cwd(), "src");
const component = readFileSync(
  resolve(root, "components/settings/InvoiceMailboxCard.tsx"),
  "utf8",
);
const settings = readFileSync(resolve(root, "pages/Settings.tsx"), "utf8");
const contracts = readFileSync(resolve(root, "lib/api/contracts.ts"), "utf8");

describe("backend-owned XML invoice mailbox workflow", () => {
  it("exposes mailbox and review-queue endpoints through the API contract", () => {
    expect(contracts).toContain('invoiceMailboxes: "/api/v1/invoice-mailboxes"');
    expect(contracts).toContain('invoiceMailImports: "/api/v1/invoice-mail-imports"');
  });

  it("keeps the UI focused on IMAP XML attachments and explicit confirmation", () => {
    expect(component).toContain("IMAP TLS");
    expect(component).toContain("XML 附件");
    expect(component).toContain("确认入账");
    expect(component).toContain("pending_confirmation");
    expect(component).toContain("每 15 分钟自动拉取");
    expect(component).toContain("enabled: false");
    expect(component).toContain("新建邮箱默认停用");
    expect(component).not.toContain("OCR");
    expect(component).not.toContain("PWA");
    expect(component).not.toContain("camera");
  });

  it("renders mailbox settings in the settings page", () => {
    expect(settings).toContain('import { InvoiceMailboxCard }');
    expect(settings).toContain("<InvoiceMailboxCard />");
  });
});
