import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { useEffect } from "react";
import { api, cloudProviders } from "@/integrations/api/client";
import { toast } from "sonner";
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

const providerSchema = z.object({
  provider_name: z.string().min(1, "请输入服务商名称"),
  provider_type: z.string().min(1, "请选择服务商类型"),
  account_name: z.string().min(1, "请输入账户名称"),
  account_id: z.string().optional(),
  region: z.string().optional(),
  contact_person: z.string().optional(),
  contact_email: z.string().email("请输入有效的邮箱").optional().or(z.literal("")),
  contact_phone: z.string().optional(),
  notes: z.string().optional(),
});

type ProviderFormValues = z.infer<typeof providerSchema>;

const CLOUD_COST_FORM_OPTIONS_ENDPOINT = "/api/v1/cloud_cost/form_options";

interface CloudCostFormOption {
  value: string;
  label: string;
}

interface CloudCostFormOptions {
  provider_type_options: CloudCostFormOption[];
  default_values: {
    provider_type: string;
  };
}

interface CloudProviderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  initialData?: any;
}

const CloudProviderDialog = ({
  open,
  onOpenChange,
  onSuccess,
  initialData,
}: CloudProviderDialogProps) => {
  const form = useForm<ProviderFormValues>({
    resolver: zodResolver(providerSchema),
    defaultValues: {
      provider_name: "",
      provider_type: "",
      account_name: "",
      account_id: "",
      region: "",
      contact_person: "",
      contact_email: "",
      contact_phone: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["cloud-cost-form-options"],
    queryFn: () =>
      api.get<CloudCostFormOptions>(CLOUD_COST_FORM_OPTIONS_ENDPOINT),
    enabled: open,
  });
  const providerTypeOptions = formOptions?.provider_type_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      provider_name: initialData.provider_name || "",
      provider_type: initialData.provider_type || defaults.provider_type,
      account_name: initialData.account_name || "",
      account_id: initialData.account_id || "",
      region: initialData.region || "",
      contact_person: initialData.contact_person || "",
      contact_email: initialData.contact_email || "",
      contact_phone: initialData.contact_phone || "",
      notes: initialData.notes || "",
    } : {
      provider_name: "",
      provider_type: defaults.provider_type,
      account_name: "",
      account_id: "",
      region: "",
      contact_person: "",
      contact_email: "",
      contact_phone: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const onSubmit = async (data: ProviderFormValues) => {
    try {
      const payload = {
        provider_name: data.provider_name,
        provider_type: data.provider_type,
        account_name: data.account_name,
        account_id: data.account_id || null,
        region: data.region || null,
        contact_person: data.contact_person || null,
        contact_email: data.contact_email || null,
        contact_phone: data.contact_phone || null,
        notes: data.notes || null,
      };

      if (initialData?.id) {
        await cloudProviders.update(initialData.id, payload);
        toast.success("云服务商信息已更新");
      } else {
        await cloudProviders.create(payload);
        toast.success("云服务商已添加");
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
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑云服务商" : "添加云服务商"}</DialogTitle>
          <DialogDescription>
            配置云服务商账户信息
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="provider_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>服务商类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择服务商" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {providerTypeOptions.map((option) => (
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
                name="provider_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>服务商名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="如：阿里云-生产环境" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="account_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账户名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="账户别名" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="account_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账户ID</FormLabel>
                    <FormControl>
                      <Input placeholder="云服务商账户ID" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="region"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>主要区域</FormLabel>
                  <FormControl>
                    <Input placeholder="如：华东1（杭州）" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid grid-cols-3 gap-4">
              <FormField
                control={form.control}
                name="contact_person"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>联系人</FormLabel>
                    <FormControl>
                      <Input placeholder="负责人姓名" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="contact_email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>联系邮箱</FormLabel>
                    <FormControl>
                      <Input type="email" placeholder="email@example.com" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="contact_phone"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>联系电话</FormLabel>
                    <FormControl>
                      <Input placeholder="手机号" {...field} />
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

export default CloudProviderDialog;
