import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Inbox, Loader2, Pencil, RefreshCw, Save, Trash2, X } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";

type Mailbox = {
  id: string;
  name: string;
  imap_host: string;
  imap_port: number;
  imap_username: string;
  folder: string;
  enabled: boolean;
  has_password: boolean;
  last_uid: number;
  last_sync_at?: string | null;
  last_sync_status?: string | null;
  last_sync_error?: string | null;
};

type MailImport = {
  id: string;
  mailbox_name: string;
  attachment_name: string;
  invoice_number?: string | null;
  status: string;
  status_label: string;
  error?: string | null;
  invoice?: {
    supplier?: string;
    amount_label?: string;
    invoice_date?: string;
  } | null;
};

type FormState = {
  name: string;
  imap_host: string;
  imap_port: string;
  imap_username: string;
  imap_password: string;
  folder: string;
  enabled: boolean;
};

const emptyForm: FormState = {
  name: "电子发票邮箱",
  imap_host: "",
  imap_port: "993",
  imap_username: "",
  imap_password: "",
  folder: "INBOX",
  enabled: false,
};

export function InvoiceMailboxCard() {
  const [form, setForm] = useState<FormState>(emptyForm);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const mailboxes = useQuery({
    queryKey: ["invoice-mailboxes"],
    queryFn: () => api.get<{ data: Mailbox[] }>(API_ENDPOINTS.invoiceMailboxes),
  });
  const imports = useQuery({
    queryKey: ["invoice-mail-imports"],
    queryFn: () => api.get<{ data: MailImport[]; stats: Record<string, number> }>(API_ENDPOINTS.invoiceMailImports),
  });

  const refresh = async () => {
    await Promise.all([mailboxes.refetch(), imports.refetch()]);
  };

  const update = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm((current) => ({ ...current, [key]: value }));
  };

  const edit = (mailbox: Mailbox) => {
    setEditingId(mailbox.id);
    setForm({
      name: mailbox.name,
      imap_host: mailbox.imap_host,
      imap_port: String(mailbox.imap_port),
      imap_username: mailbox.imap_username,
      imap_password: "",
      folder: mailbox.folder,
      enabled: mailbox.enabled,
    });
  };

  const reset = () => {
    setEditingId(null);
    setForm(emptyForm);
  };

  const save = async () => {
    if (!form.name.trim() || !form.imap_host.trim() || !form.imap_username.trim()) {
      toast.error("请填写名称、IMAP 地址和邮箱账号");
      return;
    }
    if (!editingId && !form.imap_password) {
      toast.error("首次配置需要填写邮箱密码或应用专用密码");
      return;
    }
    setBusy("save");
    const payload = {
      ...form,
      imap_port: Number(form.imap_port) || 993,
      imap_password: form.imap_password || null,
    };
    try {
      if (editingId) {
        await api.runAction({
          method: "PUT",
          endpoint: `${API_ENDPOINTS.invoiceMailboxes}/${editingId}`,
          label: "保存邮箱",
        }, payload);
      } else {
        await api.runAction({
          method: "POST",
          endpoint: API_ENDPOINTS.invoiceMailboxes,
          label: "添加邮箱",
        }, payload);
      }
      toast.success(editingId ? "发票邮箱已更新" : "发票邮箱已创建");
      reset();
      await refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "邮箱配置保存失败");
    } finally {
      setBusy(null);
    }
  };

  const sync = async (mailbox: Mailbox) => {
    setBusy(`sync:${mailbox.id}`);
    try {
      const result = await api.runAction<{
        pending_confirmation: number;
        duplicate: number;
        failed: number;
      }>({
        method: "POST",
        endpoint: `${API_ENDPOINTS.invoiceMailboxes}/${mailbox.id}/sync`,
        label: "立即拉取 XML 发票",
      });
      toast.success(
        `拉取完成：待确认 ${result.pending_confirmation}，重复 ${result.duplicate}，失败 ${result.failed}`,
      );
      await refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "邮箱拉取失败");
      await mailboxes.refetch();
    } finally {
      setBusy(null);
    }
  };

  const remove = async (mailbox: Mailbox) => {
    if (!confirm(`确定删除“${mailbox.name}”吗？`)) return;
    setBusy(`delete:${mailbox.id}`);
    try {
      await api.runAction({
        method: "DELETE",
        endpoint: `${API_ENDPOINTS.invoiceMailboxes}/${mailbox.id}`,
        label: "删除邮箱",
      });
      toast.success("发票邮箱已删除");
      await refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "删除失败");
    } finally {
      setBusy(null);
    }
  };

  const decide = async (item: MailImport, decision: "confirm" | "reject") => {
    setBusy(`${decision}:${item.id}`);
    try {
      await api.runAction({
        method: "POST",
        endpoint: `${API_ENDPOINTS.invoiceMailImports}/${item.id}/${decision}`,
        label: decision === "confirm" ? "确认入账" : "忽略",
      });
      toast.success(decision === "confirm" ? "发票已确认入账" : "附件已忽略");
      await imports.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "操作失败");
    } finally {
      setBusy(null);
    }
  };

  const pendingCount = imports.data?.stats.pending_confirmation || 0;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Inbox className="h-5 w-5 text-primary" />
          <div>
            <CardTitle>电子发票邮箱导入</CardTitle>
            <CardDescription>
              通过 IMAP TLS 拉取 XML 附件；新建邮箱默认停用，确认后再手动启用自动拉取。解析后先进入待确认队列，不会直接入账
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2">
            <Label>配置名称</Label>
            <Input value={form.name} onChange={(event) => update("name", event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>IMAP 地址</Label>
            <Input placeholder="imap.example.com" value={form.imap_host} onChange={(event) => update("imap_host", event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>端口（TLS）</Label>
            <Input type="number" value={form.imap_port} onChange={(event) => update("imap_port", event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>邮箱账号</Label>
            <Input value={form.imap_username} onChange={(event) => update("imap_username", event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>{editingId ? "新密码（留空则不修改）" : "密码 / 应用专用密码"}</Label>
            <Input type="password" value={form.imap_password} onChange={(event) => update("imap_password", event.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>邮箱目录</Label>
            <Input value={form.folder} onChange={(event) => update("folder", event.target.value)} />
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <Switch checked={form.enabled} onCheckedChange={(checked) => update("enabled", checked)} />
          <span className="text-sm">启用每 15 分钟自动拉取</span>
          <Button size="sm" onClick={() => void save()} disabled={busy === "save"}>
            {busy === "save" ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Save className="mr-2 h-4 w-4" />}
            {editingId ? "保存修改" : "添加邮箱"}
          </Button>
          {editingId && (
            <Button size="sm" variant="ghost" onClick={reset}>
              <X className="mr-2 h-4 w-4" />取消编辑
            </Button>
          )}
        </div>

        <div className="space-y-2">
          {(mailboxes.data?.data || []).map((mailbox) => (
            <div key={mailbox.id} className="flex flex-wrap items-center justify-between gap-3 rounded-lg bg-muted p-3">
              <div className="min-w-0 flex-1 text-sm">
                <div className="font-medium">{mailbox.name} · {mailbox.enabled ? "已启用" : "已停用"}</div>
                <div className="truncate text-xs text-muted-foreground">
                  {mailbox.imap_username} @ {mailbox.imap_host}:{mailbox.imap_port}/{mailbox.folder} · UID {mailbox.last_uid}
                </div>
                <div className="text-xs text-muted-foreground">
                  最近同步：{mailbox.last_sync_at || "尚未同步"} {mailbox.last_sync_status ? `(${mailbox.last_sync_status})` : ""}
                  {mailbox.last_sync_error ? ` · ${mailbox.last_sync_error}` : ""}
                </div>
              </div>
              <div className="flex gap-1">
                <Button size="sm" variant="outline" disabled={!mailbox.enabled || busy === `sync:${mailbox.id}`} onClick={() => void sync(mailbox)}>
                  {busy === `sync:${mailbox.id}` ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
                </Button>
                <Button size="sm" variant="outline" onClick={() => edit(mailbox)}><Pencil className="h-4 w-4" /></Button>
                <Button size="sm" variant="ghost" disabled={busy === `delete:${mailbox.id}`} onClick={() => void remove(mailbox)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
              </div>
            </div>
          ))}
          {!mailboxes.isLoading && (mailboxes.data?.data.length || 0) === 0 && (
            <p className="text-sm text-muted-foreground">还没有发票邮箱，请使用应用专用密码添加 IMAP 配置。</p>
          )}
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="font-medium">待确认发票（{pendingCount}）</div>
          {(imports.data?.data || []).map((item) => (
            <div key={item.id} className="flex flex-wrap items-center justify-between gap-3 rounded-lg border p-3 text-sm">
              <div className="min-w-0 flex-1">
                <div className="font-medium">{item.invoice_number || item.attachment_name} · {item.status_label}</div>
                <div className="text-xs text-muted-foreground">
                  {item.mailbox_name} · {item.invoice?.supplier || "未识别供应商"} · {item.invoice?.amount_label || "-"} · {item.invoice?.invoice_date || "-"}
                </div>
                {item.error && <div className="text-xs text-destructive">{item.error}</div>}
              </div>
              {item.status === "pending_confirmation" && (
                <div className="flex gap-2">
                  <Button size="sm" disabled={busy === `confirm:${item.id}`} onClick={() => void decide(item, "confirm")}>确认入账</Button>
                  <Button size="sm" variant="outline" disabled={busy === `reject:${item.id}`} onClick={() => void decide(item, "reject")}>忽略</Button>
                </div>
              )}
            </div>
          ))}
          {!imports.isLoading && (imports.data?.data.length || 0) === 0 && (
            <p className="text-sm text-muted-foreground">暂未收到 XML 发票附件。</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
