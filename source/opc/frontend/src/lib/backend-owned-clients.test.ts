import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned clients UI", () => {
  it("loads client CRM filtering, labels, detail contacts, and interactions from the backend overview endpoint", () => {
    const source = readFileSync(
      resolve(root, "pages/ClientManagement.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/clients/overview");
    expect(source).toContain("selected_client_id");
    expect(source).toContain("overview.detail_contacts");
    expect(source).toContain("overview.detail_interactions");
    expect(source).toContain("overview.type_options");
    expect(source).toContain("overview.source_options");
    expect(source).toContain("client.type_badge");
    expect(source).toContain("client.annual_revenue_label");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("editingClient.update_action");
    expect(source).toContain("currentDetailClient.interaction_create_action");
    expect(source).toContain("client.delete_action");
    expect(source).toContain("currentDetailClient.contact_create_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("clientStats.total_revenue_label");
    expect(source).toContain("overview.empty_message");

    expect(source).not.toContain('confirm("确定删除?")');
    expect(source).not.toContain("deleteClient.mutate(client.id)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.clients");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.clients}/${id}`");
    expect(source).not.toContain("api.post(API_ENDPOINTS.interactions");
    expect(source).not.toContain("client_id: currentDetailClient.id");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.clients}/${id}`)",
    );
    expect(source).not.toContain('queryKey: ["clients"]');
    expect(source).not.toContain('queryKey: ["contacts"]');
    expect(source).not.toContain(
      'queryKey: ["interactions", detailClient?.id]',
    );
    expect(source).not.toContain("filteredClients");
    expect(source).not.toContain("clients.filter");
    expect(source).not.toContain("contacts.filter");
    expect(source).not.toContain("interactions.filter");
    expect(source).not.toContain("new Date().toISOString");
    expect(source).not.toContain("values.interaction_date =");
    expect(source).not.toContain("getTypeBadge");
    expect(source).not.toContain("getInteractionIcon");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain('<SelectItem value="referral">');
    expect(source).not.toContain('<SelectItem value="website">');
    expect(source).not.toContain("const stats = {");
    expect(source).not.toContain(
      'clients.filter((c: any) => c.client_type === "active"',
    );
    expect(source).not.toContain(
      'clients.filter((c: any) => c.client_type === "prospect"',
    );
    expect(source).not.toContain("clients.reduce((sum: number, c: any)");
    expect(source).not.toContain("prompt(");
    expect(source).not.toContain("API_ENDPOINTS.contacts");
  });
});
