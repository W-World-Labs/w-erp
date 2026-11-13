import { useCallback, useEffect, useState } from "react";
import { api } from "@/integrations/api/client";
import { toast } from "sonner";

export type NotificationChannel = {
  id: string;
  name?: string;
  channel_url: string;
  mode: string;
  enabled: boolean;
};

/** Backend-owned notification channel CRUD (mutations live here, not in components). */
export function useNotificationChannels() {
  const [loading, setLoading] = useState(true);
  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [mode, setMode] = useState("digest");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.get<NotificationChannel[] | { data: NotificationChannel[] }>(
        "/api/v1/notification_channels",
      );
      const list = Array.isArray(data) ? data : (data as { data: NotificationChannel[] }).data;
      setChannels(list || []);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "加载失败";
      toast.error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const add = async () => {
    try {
      await api.post("/api/v1/notification_channels", {
        name: name.trim() || "默认渠道",
        channel_url: url.trim(),
        mode,
        enabled: true,
      });
      setName("");
      setUrl("");
      toast.success("已添加");
      await load();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "添加失败";
      toast.error(message);
    }
  };

  const remove = async (id: string) => {
    try {
      await api.delete(`/api/v1/notification_channels/${id}`);
      toast.success("已删除");
      await load();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "删除失败";
      toast.error(message);
    }
  };

  const test = async (id: string) => {
    try {
      const res = await api.post<{ ok: boolean; message: string }>(
        `/api/v1/settings/notifications/${id}/test`,
        {},
      );
      if (res.ok) toast.success(res.message || "测试成功");
      else toast.error(res.message || "测试失败");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "测试失败";
      toast.error(message);
    }
  };

  return {
    loading,
    channels,
    name,
    setName,
    url,
    setUrl,
    mode,
    setMode,
    add,
    remove,
    test,
    reload: load,
  };
}
