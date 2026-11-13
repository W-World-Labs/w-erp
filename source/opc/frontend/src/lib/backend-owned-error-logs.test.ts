import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned error logs UI", () => {
  it("uses backend-provided filtering, counts, and display labels", () => {
    const source = readFileSync(
      resolve(root, "components/settings/ErrorLogsPanel.tsx"),
      "utf8",
    );

    expect(source).toContain("params.set('error_type', filter)");
    expect(source).toContain("data?.type_options");
    expect(source).toContain("log.created_at_label");
    expect(source).toContain("log.page_path");
    expect(source).toContain("log.metadata_fields");
    expect(source).toContain("log.error_type_badge");
    expect(source).toContain("log.delete_action");
    expect(source).toContain("api.runAction(action)");

    expect(source).not.toContain("(data?.data || []).filter");
    expect(source).not.toContain("safeFormatDate");
    expect(source).not.toContain("parseISO");
    expect(source).not.toContain("isValid");
    expect(source).not.toContain("new URL(log.page_url)");
    expect(source).not.toContain("JSON.parse");
    expect(source).not.toContain("typeCounts[log.error_type]");
    expect(source).not.toContain("ERROR_TYPE_CONFIG");
    expect(source).not.toContain('<SelectItem value="render_crash">');
    expect(source).not.toContain('<SelectItem value="api_error">');
    expect(source).not.toContain("api.delete(action.endpoint)");
    expect(source).not.toContain("api.delete(`/api/v1/error_logs/${id}`)");
    expect(source).not.toContain("deleteMutation.mutate(log.id)");
  });
});
