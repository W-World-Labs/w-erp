import { describe, expect, it } from "vitest";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned document parser normalization", () => {
  it("keeps AI extracted field normalization on the backend", () => {
    const parser = readFileSync(resolve(root, "hooks/useDocumentParser.ts"), "utf8");
    const sources = [
      "components/invoices/InvoiceDialog.tsx",
      "components/contracts/ContractDialog.tsx",
      "components/vendors/VendorDialog.tsx",
      "components/employees/EmployeeDialog.tsx",
      "components/licenses/LicenseDialog.tsx",
    ]
      .map((path) => readFileSync(resolve(root, path), "utf8"))
      .join("\n");

    expect(parser).toContain("/api/ai/parse-document");
    expect(parser).toContain("/api/ai/capabilities");
    expect(parser).toContain("document_parser_enabled");
    expect(parser).toContain("文档识别未配置");
    // #14: image preprocess + structured invoice formats + filename for format-route
    expect(parser).toContain("preprocessImageForOcr");
    expect(parser).toContain("filename");
    expect(parser).toContain(".ofd");
    expect(parser).toContain("imageOrientation");
    expect(sources).not.toContain("@/lib/fieldNormalizer");
    expect(sources).not.toContain("@/lib/dateUtils");
    expect(sources).not.toContain("normalizeFields");
    expect(sources).not.toContain("normalizeDate");
    expect(existsSync(resolve(root, "lib/fieldNormalizer.ts"))).toBe(false);
  });
});
