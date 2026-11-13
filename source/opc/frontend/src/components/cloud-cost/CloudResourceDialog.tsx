import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { api, cloudResources } from "@/integrations/api/client";
import { toast } from "sonner";
import { useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";

const resourceSchema = z.object({
  provider_id: z.string().min(1, "请选择云服务商"),
  resource_type: z.string().min(1, "请选择资源类型"),
  resource_name: z.string().min(1, "请输入资源名称"),
  instance_id: z.string().optional(),
  specification: z.string().optional(),
  region: z.string().optional(),
  purchase_type: z.string(),
  start_date: z.string().optional(),
  end_date: z.string().optional(),
  monthly_cost: z.string().optional(),
  notes: z.string().optional(),
});

type ResourceFormValues = z.infer<typeof resourceSchema>;

const CLOUD_COST_FORM_OPTIONS_ENDPOINT = "/api/v1/cloud_cost/form_options";

interface CloudCostFormOption {
  value: string;
  label: string;
}

interface CloudCostFormOptions {
  provider_options: CloudCostFormOption[];
  resource_type_options: CloudCostFormOption[];
  purchase_type_options: CloudCostFormOption[];
  default_values: {
    resource_type: string;
    purchase_type: string;
  };
}

interface CloudResourceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  initialData?: any;
}

const CloudResourceDialog = ({
  open,
  onOpenChange,
  onSuccess,
  initialData,
}: CloudResourceDialogProps) => {
  const form = useForm<ResourceFormValues>({
    resolver: zodResolver(resourceSchema),
    defaultValues: {
      provider_id: "",
      resource_type: "",
      resource_name: "",
      instance_id: "",
      specification: "",
      region: "",
      purchase_type: "",
      start_date: "",
      end_date: "",
      monthly_cost: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["cloud-cost-form-options"],
    queryFn: () =>
      api.get<CloudCostFormOptions>(CLOUD_COST_FORM_OPTIONS_ENDPOINT),
    enabled: open,
  });
  const providerOptions = formOptions?.provider_options ?? [];
  const resourceTypeOptions = formOptions?.resource_type_options ?? [];
  const purchaseTypeOptions = formOptions?.purchase_type_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      provider_id: initialData.provider_id || "",
      resource_type: initialData.resource_type || defaults.resource_type,
      resource_name: initialData.resource_name || "",
      instance_id: initialData.instance_id || "",
      specification: initialData.specification || "",
      region: initialData.region || "",
      purchase_type: initialData.purchase_type || defaults.purchase_type,
      start_date: initialData.start_date || "",
      end_date: initialData.end_date || "",
      monthly_cost: initialData.monthly_cost?.toString() || "",
      notes: initialData.notes || "",
    } : {
      provider_id: "",
      resource_type: defaults.resource_type,
      resource_name: "",
      instance_id: "",
      specification: "",
      region: "",
      purchase_type: defaults.purchase_type,
      start_date: "",
      end_date: "",
      monthly_cost: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const onSubmit = async (data: ResourceFormValues) => {
    try {
      const payload = {
        provider_id: data.provider_id,
        resource_type: data.resource_type,
        resource_name: data.resource_name,
        instance_id: data.instance_id || null,
        specification: data.specification || null,
        region: data.region || null,
        purchase_type: data.purchase_type,
        start_date: data.start_date || undefined,
        end_date: data.end_date || null,
        monthly_cost: Number(data.monthly_cost || 0),
        notes: data.notes || null,
      };

      if (initialData?.id) {
        await cloudResources.update(initialData.id, payload);
        toast.success("云资源信息已更新");
      } else {
        await cloudResources.create(payload);
        toast.success("云资源已添加");
      }

      form.reset();
      onOpenChange(false);
      onSuccess?.();
    } catch (error: any) {
      toast.error(error.message || "操作失败");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑云资源" : "添加云资源"}</DialogTitle>
          <DialogDescription>
            配置云资源实例信息
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="provider_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>云服务商 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择云服务商" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {providerOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="resource_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>资源类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {resourceTypeOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="resource_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>资源名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="资源名称或别名" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="instance_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>实例ID</FormLabel>
                    <FormControl>
                      <Input placeholder="云平台实例ID" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="specification"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>规格配置</FormLabel>
                    <FormControl>
                      <Input placeholder="如：2核4G" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="region"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>所在区域</FormLabel>
                    <FormControl>
                      <Input placeholder="如：华东1（杭州）" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="purchase_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>购买类型</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {purchaseTypeOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="monthly_cost"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>月费用（元）</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="start_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>开始日期</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="end_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>到期日期</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="notes"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>备注</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="其他说明信息"
                      className="resize-none"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                {initialData ? "保存" : "添加"}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
};

export default CloudResourceDialog;
