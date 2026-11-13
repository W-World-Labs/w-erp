import { useState } from "react";
import { api } from "@/integrations/api/client";
import { toast } from "sonner";

export type BackupListItem = {
  file_name: string;
  size_bytes?: number;
  created_at?: string;
};

export type SettingsAction = {
  route?: string;
  method?: string;
  action_type?: string;
  list_endpoint?: string;
  confirm_message?: string;
  success_message?: string;
};

/** Settings-page backup/restore network actions (kept out of page components). */
export function useSettingsBackupActions(onDone?: () => void) {
  const [backupBusy, setBackupBusy] = useState(false);
  const [backupList, setBackupList] = useState<BackupListItem[]>([]);
  const [selectedBackup, setSelectedBackup] = useState("");
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [restoreAction, setRestoreAction] = useState<SettingsAction | null>(null);

  const handleSettingsAction = async (action?: SettingsAction) => {
    if (!action?.route) return;

    if (action.action_type === "restore_picker") {
      try {
        setBackupBusy(true);
        const res = await api.get<{ data: BackupListItem[] }>(
          action.list_endpoint || "/api/v1/settings/backups",
        );
        const items = res.data || [];
        setBackupList(items);
        setSelectedBackup(items[0]?.file_name || "");
        setRestoreAction(action);
        setRestoreOpen(true);
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : "加载备份列表失败";
        toast.error(message);
      } finally {
        setBackupBusy(false);
      }
      return { navigated: false as const };
    }

    if (
      action.action_type === "download" ||
      (action.method === "GET" && action.route.includes("/download"))
    ) {
      try {
        const resp = await fetch(action.route, {
          credentials: "include",
        });
        if (!resp.ok) {
          throw new Error((await resp.text()) || `HTTP ${resp.status}`);
        }
        const blob = await resp.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        const name =
          action.route.split("/").filter(Boolean).slice(-2, -1)[0] || "backup.tar.gz";
        a.download = name;
        a.click();
        URL.revokeObjectURL(url);
        toast.success("备份下载已开始");
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : "下载失败";
        toast.error(message);
      }
      return { navigated: false as const };
    }

    if (action.method === "POST" || action.method === "PUT" || action.method === "DELETE") {
      if (action.confirm_message && !confirm(action.confirm_message)) {
        return { navigated: false as const };
      }
      try {
        setBackupBusy(true);
        await api.runAction({
          method: action.method,
          endpoint: action.route,
        });
        toast.success(action.success_message || "操作成功");
        onDone?.();
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : "操作失败";
        toast.error(message);
      } finally {
        setBackupBusy(false);
      }
      return { navigated: false as const };
    }

    return { navigated: true as const, route: action.route };
  };

  const handleConfirmRestore = async () => {
    if (!restoreAction || !selectedBackup) {
      toast.error("请选择要恢复的备份");
      return;
    }
    if (restoreAction.confirm_message && !confirm(restoreAction.confirm_message)) {
      return;
    }
    try {
      setBackupBusy(true);
      const res = await api.post<{ message?: string }>(restoreAction.route!, {
        file_name: selectedBackup,
      });
      toast.success(res.message || restoreAction.success_message || "恢复已就绪，请重启服务");
      setRestoreOpen(false);
      onDone?.();
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "恢复失败";
      toast.error(message);
    } finally {
      setBackupBusy(false);
    }
  };

  return {
    backupBusy,
    backupList,
    selectedBackup,
    setSelectedBackup,
    restoreOpen,
    setRestoreOpen,
    restoreAction,
    handleSettingsAction,
    handleConfirmRestore,
  };
}
