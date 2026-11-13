import { useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useForm } from "react-hook-form";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Switch } from "@/components/ui/switch";

interface EmailConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  config?: {
    id: string;
    name: string;
    api_key: string;
    from_email: string;
    from_name: string | null;
    provider?: "smtp" | "w-mail";
    smtp_host?: string | null;
    smtp_port?: number | null;
    smtp_user?: string | null;
    is_default: boolean;
    update_action?: BackendAction | null;
  } | null;
  wMailAvailable?: boolean;
  createAction?: BackendAction | null;
  onSuccess: () => void;
}

interface FormData {
  provider: "smtp" | "w-mail";
  name: string;
  api_key: string;
  from_email: string;
  from_name: string;
  smtp_port: number;
  smtp_user: string;
  is_default: boolean;
}

export function EmailConfigDialog({ open, onOpenChange, config, createAction, wMailAvailable = false, onSuccess }: EmailConfigDialogProps) {
  const { register, handleSubmit, formState: { errors, isSubmitting }, reset, watch, setValue } = useForm<FormData>();
  useEffect(() => {
    if (open) reset({
      provider: config?.provider ?? "smtp",
      name: config?.smtp_host ?? config?.name ?? "",
      api_key: config?.api_key ?? "",
      from_email: config?.from_email ?? "",
      from_name: config?.from_name ?? "",
      smtp_port: config?.smtp_port ?? 587,
      smtp_user: config?.smtp_user ?? "",
      is_default: config?.is_default ?? false,
    });
  }, [config, open, reset]);
  const provider = watch("provider") ?? "smtp";

  const onSubmit = async (data: FormData) => {
    try {
      const action = config?.update_action ?? createAction ?? null;
      if (!action) throw new Error("后端未返回邮件配置保存动作");
      const payload = data.provider === "w-mail" ? {
        provider: data.provider,
        enabled: data.is_default ? 1 : 0,
        ...(!config ? { smtp_host: "W-Mail" } : {}),
      } : {
        provider: data.provider,
        smtp_host: data.name,
        smtp_port: Number(data.smtp_port),
        smtp_user: data.smtp_user,
        smtp_password: data.api_key,
        sender_email: data.from_email,
        sender_name: data.from_name,
        enabled: data.is_default ? 1 : 0,
      };
      await api.runAction(action, payload);
      toast.success(action.success_message || "邮件配置已保存");
      reset();
      onSuccess();
      onOpenChange(false);
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : "保存失败");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{config ? "编辑" : "添加"} 邮件配置</DialogTitle>
          <DialogDescription>选择邮件渠道。切换到 W-Mail 会保留已有 SMTP 配置。</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email-provider">邮件渠道</Label>
            <select id="email-provider" className="flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm" {...register("provider")}>
              <option value="smtp">SMTP 配置（当前未实现发送）</option>
              <option value="w-mail" disabled={!wMailAvailable}>W-Mail{wMailAvailable ? "" : "（服务尚未配置）"}</option>
            </select>
          </div>
          {provider === "w-mail" ? (
            <p className="text-sm text-muted-foreground">通过系统管理的项目业务邮箱发送，凭据仅由服务端使用。投递记录会区分上游受理、实际递送、失败与结果未知。</p>
          ) : (
            <>
              <p role="status" className="text-sm text-muted-foreground">SMTP 配置可以保存；当前版本没有 SMTP 发送实现，发送请求会明确报错，不会显示为已发送。</p>
              <div className="space-y-2">
                <Label htmlFor="name">SMTP 服务器 *</Label>
                <Input id="name" {...register("name", { required: provider === "smtp" ? "请输入 SMTP 服务器" : false })} placeholder="smtp.example.com" />
                {errors.name && <p className="text-sm text-destructive">{errors.name.message}</p>}
              </div>
              <div className="space-y-2">
                <Label htmlFor="smtp_port">SMTP 端口</Label>
                <Input id="smtp_port" type="number" {...register("smtp_port")} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="smtp_user">SMTP 用户名</Label>
                <Input id="smtp_user" {...register("smtp_user")} autoComplete="off" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="api_key">SMTP 密码</Label>
                <Input id="api_key" type="password" {...register("api_key")} autoComplete="off" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="from_email">发件人邮箱</Label>
                <Input id="from_email" type="email" {...register("from_email")} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="from_name">发件人名称</Label>
                <Input id="from_name" {...register("from_name")} />
              </div>
            </>
          )}
          <div className="flex items-center justify-between p-4 bg-muted rounded-lg">
            <Label htmlFor="is_default">启用此配置</Label>
            <Switch id="is_default" checked={watch("is_default") ?? false} onCheckedChange={(checked) => setValue("is_default", checked)} />
          </div>
          <div className="flex justify-end gap-2 pt-4">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
            <Button type="submit" disabled={isSubmitting || !(config?.update_action ?? createAction)}>{isSubmitting ? "保存中..." : "保存"}</Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
