import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned profile UI", () => {
  it("loads profile display labels and avatar storage from backend results", () => {
    const source = readFileSync(resolve(root, "pages/Profile.tsx"), "utf8");
    const userMenu = readFileSync(resolve(root, "components/UserMenu.tsx"), "utf8");

    expect(source).toContain("/api/profiles/current");
    expect(userMenu).toContain("/api/profiles/current");
    expect(source).not.toContain("result.data?.[0]");
    expect(userMenu).not.toContain("result.data?.[0]");
    expect(userMenu).not.toContain("return result.data || result");
    expect(source).toContain("profile?.created_at_label");
    expect(source).toContain("profile?.updated_at_label");
    expect(source).toContain("profile.update_action");
    expect(source).toContain("api.runAction(profile.update_action");
    expect(source).toContain("const uploadResult = await files.upload(file);");
    expect(source).toContain("avatar_url: uploadResult.url");
    expect(source).toContain("display_name: values.full_name");

    expect(source).not.toContain("api.put(`/api/profiles/${profile.id}`");
    expect(source).not.toContain("/api/users/profile");
    expect(source).not.toContain("Date.now()");
    expect(source).not.toContain("fileExt");
    expect(source).not.toContain("/api/files/download");
    expect(source).not.toContain("new Date(profile");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("profile.avatar_url.split");
  });
});
