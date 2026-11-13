import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Bell, Loader2, Plus, Trash2, Send } from "lucide-react";
import { useNotificationChannels } from "@/hooks/useNotificationChannels";
import { toast } from "sonner";

export function NotificationChannelsCard() {
  const s = useNotificationChannels();
  const [saving, setSaving] = useState(false);
  const [testingId, setTestingId] = useState<string | null>(null);

  const handleAdd = async () => {
    if (!s.url.trim()) {
      toast.error("请填写 Webhook URL");
      return;
    }
    setSaving(true);
    try {
      await s.add();
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("确定删除？")) return;
    await s.remove(id);
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    try {
      await s.test(id);
    } finally {
      setTestingId(null);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Bell className="h-5 w-5 text-primary" />
          <div>
            <CardTitle>通知渠道</CardTitle>
            <CardDescription>
              粘贴飞书/企微/钉钉/Telegram/Slack Webhook，提醒自动送达
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2">
            <Label>名称</Label>
            <Input value={s.name} onChange={(e) => s.setName(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>模式</Label>
            <Select value={s.mode} onValueChange={s.setMode}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="digest">每日摘要 (09:00)</SelectItem>
                <SelectItem value="instant">即时推送</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label>Webhook URL</Label>
            <Input value={s.url} onChange={(e) => s.setUrl(e.target.value)} />
          </div>
        </div>
        <Button onClick={() => void handleAdd()} disabled={saving} size="sm">
          {saving ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Plus className="mr-2 h-4 w-4" />
          )}
          添加渠道
        </Button>
        {s.loading ? (
          <p className="text-sm text-muted-foreground">加载中…</p>
        ) : s.channels.length === 0 ? (
          <p className="text-sm text-muted-foreground">还没有通知渠道</p>
        ) : (
          <div className="space-y-2">
            {s.channels.map((ch) => (
              <div
                key={ch.id}
                className="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-muted p-3"
              >
                <div className="min-w-0 flex-1">
                  <div className="font-medium">{ch.name || "未命名"}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {ch.mode} · {ch.channel_url}
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => void handleTest(ch.id)}
                    disabled={testingId === ch.id}
                  >
                    {testingId === ch.id ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Send className="h-4 w-4" />
                    )}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => void handleDelete(ch.id)}>
                    <Trash2 className="h-4 w-4 text-destructive" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
