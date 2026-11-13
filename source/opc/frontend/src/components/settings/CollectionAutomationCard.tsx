import { useEffect, useState } from "react";
import { BellRing } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  defaultCollectionSettings,
  type CollectionSettings,
  useCollectionAutomation,
} from "@/hooks/useCollectionAutomation";

export function CollectionAutomationCard() {
  const { settings, saveSettings, isSaving } = useCollectionAutomation();
  const [form, setForm] = useState<CollectionSettings>(
    defaultCollectionSettings,
  );

  useEffect(() => setForm(settings), [settings]);

  const handleSave = async () => {
    try {
      await saveSettings(form);
      toast.success("催收自动化设置已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "保存失败");
    }
  };

  const setDays = (
    key: "friendly_days" | "formal_days" | "final_days" | "bad_debt_days",
    value: string,
  ) => setForm((current) => ({ ...current, [key]: Number(value) || 0 }));

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <BellRing className="h-5 w-5 text-primary" />
          <CardTitle>逾期催收自动化</CardTitle>
        </div>
        <CardDescription>
          账龄达到设定天数时，复用已启用的即时或每日摘要通知渠道自动触达。
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <Label htmlFor="collection-enabled">启用自动催收</Label>
            <p className="text-sm text-muted-foreground">
              关闭后调度器不会创建或发送任何新催收事件。
            </p>
          </div>
          <Switch
            id="collection-enabled"
            checked={form.enabled}
            onCheckedChange={(enabled) =>
              setForm((current) => ({ ...current, enabled }))
            }
          />
        </div>

        <div className="grid gap-3 sm:grid-cols-4">
          {([
            ["friendly_days", "友好提醒"],
            ["formal_days", "正式催款"],
            ["final_days", "最后通知"],
            ["bad_debt_days", "坏账风险"],
          ] as const).map(([key, label]) => (
            <div key={key} className="space-y-1.5">
              <Label htmlFor={key}>{label}</Label>
              <div className="flex items-center gap-2">
                <Input
                  id={key}
                  type="number"
                  min={1}
                  max={3650}
                  value={form[key]}
                  onChange={(event) => setDays(key, event.target.value)}
                />
                <span className="text-sm text-muted-foreground">天</span>
              </div>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between gap-4">
          <p className="text-xs text-muted-foreground">
            {settings.delivery_help ||
              "即时渠道在计费周期当场发送；摘要渠道在每日 09:00 合并发送。"}
          </p>
          <Button onClick={() => void handleSave()} disabled={isSaving}>
            {isSaving ? "保存中..." : "保存催收设置"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
