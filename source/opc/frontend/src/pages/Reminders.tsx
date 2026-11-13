import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AlertCircle, Bell, CheckCircle2, XCircle } from "lucide-react";
import { toast } from "sonner";
import { API_ENDPOINTS } from "@/lib/api/contracts";
import { cn } from "@/lib/utils";
import { QueryStateGate } from "@/components/common/QueryStateGate";

const badgeIcons = {
  Bell,
  CheckCircle2,
  XCircle,
};

type BadgeData = {
  label?: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
  class_name?: string;
};

const defaultOverview = {
  stats: {
    pending: 0,
    completed: 0,
    dismissed: 0,
  },
  pending_reminders: [] as any[],
  history_reminders: [] as any[],
  check_asset_expiry_action: null as BackendAction | null,
  check_operating_deadlines_action: null as BackendAction | null,
};

type ReminderRow = {
  id: string;
  title: string;
  description?: string | null;
  type_label?: string;
  priority_badge?: BadgeData;
  status_badge?: BadgeData;
  date_badge?: BadgeData;
  remind_date_label?: string;
  history_remind_date_label?: string;
  card_class?: string;
  date_text_class?: string;
  show_alert_icon?: boolean;
  complete_action?: BackendAction | null;
  dismiss_action?: BackendAction | null;
};

const renderBadge = (badge?: BadgeData, className?: string) => {
  if (!badge?.label) {
    return null;
  }
  const Icon = badgeIcons[badge.icon as keyof typeof badgeIcons];
  return (
    <Badge
      variant={badge.variant || "outline"}
      className={cn("gap-1", badge.class_name, className)}
    >
      {Icon && <Icon className="h-3 w-3" />}
      {badge.label}
    </Badge>
  );
};

export default function Reminders() {
  const {
    data: overview = defaultOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["reminders-overview"],
    queryFn: async () =>
      api.get<typeof defaultOverview>(API_ENDPOINTS.remindersOverview),
  });

  const runReminderAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "提醒状态已更新");
      refetch();
    } catch (err) {
      toast.error("操作失败");
    }
  };

  const runReminderCreationAction = async (
    action: BackendAction | null,
    missingMessage: string,
  ) => {
    try {
      if (!action) {
        throw new Error(missingMessage);
      }
      const result = await api.runAction<{ reminders_created?: number }>(action);
      toast.success(
        `${action.success_message || "检查完成"}：创建了 ${
          result?.reminders_created ?? 0
        } 条提醒`,
      );
      refetch();
    } catch (error: any) {
      console.error("检查资产到期失败:", error);
      toast.error("检查失败，请重试");
    }
  };

  const checkOperatingDeadlines = async () => {
    await runReminderCreationAction(
      overview.check_operating_deadlines_action,
      "后端未返回运营到期检查动作",
    );
  };

  const checkAssetExpiry = async () => {
    await runReminderCreationAction(
      overview.check_asset_expiry_action,
      "后端未返回资产到期检查动作",
    );
  };

  const pendingReminders = overview.pending_reminders as ReminderRow[];
  const historyReminders = overview.history_reminders as ReminderRow[];

  return (
    <QueryStateGate isLoading={isLoading} isError={isError} refetch={refetch}>
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">提醒中心</h1>
          <p className="text-muted-foreground mt-2">
            管理所有重要事项提醒，不错过任何截止日期
          </p>
        </div>
        <div className="flex gap-2">
          <Badge variant="outline" className="text-lg px-4 py-2">
            待处理：{overview.stats.pending}
          </Badge>
          <Button onClick={checkOperatingDeadlines}>
            <Bell className="h-4 w-4 mr-2" />
            检查运营到期
          </Button>
          <Button onClick={checkAssetExpiry} variant="outline">
            <Bell className="h-4 w-4 mr-2" />
            检查资产到期
          </Button>
        </div>
      </div>

      <div className="space-y-4">
        <h2 className="text-xl font-semibold">待处理提醒</h2>
        {pendingReminders.length === 0 ? (
          <Card className="p-8 text-center">
            <CheckCircle2 className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">
              太好了！没有待处理的提醒
            </h3>
            <p className="text-muted-foreground">
              系统会自动为证照到期、合同到期等事项创建提醒
            </p>
          </Card>
        ) : (
          <div className="grid gap-4">
            {pendingReminders.map((reminder) => (
              <Card
                key={reminder.id}
                className={cn("p-6", reminder.card_class)}
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 space-y-2">
                    <div className="flex items-center gap-2">
                      {reminder.show_alert_icon && (
                        <AlertCircle className="h-5 w-5 text-destructive" />
                      )}
                      <h3 className="text-lg font-semibold">
                        {reminder.title}
                      </h3>
                      <Badge variant="outline">{reminder.type_label}</Badge>
                      {renderBadge(reminder.priority_badge)}
                      {renderBadge(reminder.status_badge)}
                    </div>

                    {reminder.description && (
                      <p className="text-muted-foreground">
                        {reminder.description}
                      </p>
                    )}

                    <div className="flex items-center gap-2 text-sm">
                      <span
                        className={cn("font-medium", reminder.date_text_class)}
                      >
                        提醒日期：{reminder.remind_date_label}
                      </span>
                      {renderBadge(reminder.date_badge)}
                    </div>
                  </div>

                  <div className="flex gap-2">
                    {reminder.complete_action && (
                      <Button
                        size="sm"
                        onClick={() =>
                          runReminderAction(reminder.complete_action)
                        }
                      >
                        <CheckCircle2 className="h-4 w-4 mr-1" />
                        {reminder.complete_action.label || "完成"}
                      </Button>
                    )}
                    {reminder.dismiss_action && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                          runReminderAction(reminder.dismiss_action)
                        }
                      >
                        <XCircle className="h-4 w-4 mr-1" />
                        {reminder.dismiss_action.label || "忽略"}
                      </Button>
                    )}
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      {historyReminders.length > 0 && (
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">历史记录</h2>
          <div className="grid gap-3">
            {historyReminders.map((reminder) => (
              <Card key={reminder.id} className="p-4 opacity-60">
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{reminder.title}</span>
                      <Badge variant="outline" className="text-xs">
                        {reminder.type_label}
                      </Badge>
                      {renderBadge(reminder.status_badge)}
                    </div>
                    <div className="text-sm text-muted-foreground mt-1">
                      提醒日期：{reminder.history_remind_date_label}
                    </div>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </div>
      )}
    </div>
    </QueryStateGate>
  );
}
