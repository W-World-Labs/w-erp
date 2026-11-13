import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useForm } from "react-hook-form";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Switch } from "@/components/ui/switch";

interface AIConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  config?: {
    id: string;
    name: string;
    api_url: string;
    api_key: string;
    model: string;
    temperature: number;
    max_tokens: number;
    is_default: boolean;
    update_action?: BackendAction | null;
  } | null;
  createAction?: BackendAction | null;
  onSuccess: () => void;
}

interface FormData {
  name: string;
  api_url: string;
  api_key: string;
  model: string;
  temperature: number;
  max_tokens: number;
  is_default: boolean;
}

export function AIConfigDialog({ open, onOpenChange, config, createAction, onSuccess }: AIConfigDialogProps) {
  const { register, handleSubmit, formState: { errors, isSubmitting }, reset } = useForm<FormData>({
    defaultValues: config || {
      name: "",
      api_url: "https://api.deepseek.com/v1/chat/completions",
      api_key: "",
      model: "deepseek-v4-flash",
      temperature: 0.7,
      max_tokens: 2000,
      is_default: false,
    },
  });

  const onSubmit = async (data: FormData) => {
    try {
      const action = config?.update_action ?? createAction ?? null;
      if (!action) {
        throw new Error("后端未返回 AI 配置保存动作");
      }
      await api.runAction(action, {
        provider: data.name,
        endpoint: data.api_url,
        api_key: data.api_key,
        model: data.model,
        enabled: data.is_default ? 1 : 0,
      });
      toast.success(action.success_message || "AI 配置已保存");

      reset();
      onSuccess();
      onOpenChange(false);
    } catch (error: any) {
      console.error("Error saving AI config:", error);
      toast.error(error.message || "保存失败");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{config ? "编辑" : "添加"} AI 配置</DialogTitle>
          <DialogDescription>
            配置 OpenAI 兼容的 API 端点和模型参数
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">配置名称 *</Label>
            <Input
              id="name"
              {...register("name", { required: "请输入配置名称" })}
              placeholder="例如：DeepSeek"
            />
            {errors.name && (
              <p className="text-sm text-destructive">{errors.name.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="api_url">API URL *</Label>
            <Input
              id="api_url"
              {...register("api_url", { required: "请输入 API URL" })}
              placeholder="https://api.deepseek.com/v1/chat/completions"
            />
            {errors.api_url && (
              <p className="text-sm text-destructive">{errors.api_url.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="api_key">API Key *</Label>
            <Input
              id="api_key"
              type="password"
              {...register("api_key", { required: "请输入 API Key" })}
              placeholder="sk-..."
            />
            {errors.api_key && (
              <p className="text-sm text-destructive">{errors.api_key.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="model">模型名称 *</Label>
            <Input
              id="model"
              {...register("model", { required: "请输入模型名称" })}
              placeholder="deepseek-v4-flash"
            />
            {errors.model && (
              <p className="text-sm text-destructive">{errors.model.message}</p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="temperature">Temperature (0-2)</Label>
              <Input
                id="temperature"
                type="number"
                step="0.1"
                min="0"
                max="2"
                {...register("temperature", { 
                  valueAsNumber: true,
                  min: { value: 0, message: "最小值为 0" },
                  max: { value: 2, message: "最大值为 2" }
                })}
              />
              {errors.temperature && (
                <p className="text-sm text-destructive">{errors.temperature.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="max_tokens">Max Tokens</Label>
              <Input
                id="max_tokens"
                type="number"
                {...register("max_tokens", { 
                  valueAsNumber: true,
                  min: { value: 1, message: "最小值为 1" }
                })}
              />
              {errors.max_tokens && (
                <p className="text-sm text-destructive">{errors.max_tokens.message}</p>
              )}
            </div>
          </div>

          <div className="flex items-center justify-between p-4 bg-muted rounded-lg">
            <div className="space-y-0.5">
              <Label htmlFor="is_default">设为默认配置</Label>
              <p className="text-sm text-muted-foreground">
                将此配置设为默认的 AI 服务
              </p>
            </div>
            <Switch
              id="is_default"
              {...register("is_default")}
            />
          </div>

          <div className="flex justify-end gap-2 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              取消
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting || !(config?.update_action ?? createAction)}
            >
              {isSubmitting ? "保存中..." : "保存"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
