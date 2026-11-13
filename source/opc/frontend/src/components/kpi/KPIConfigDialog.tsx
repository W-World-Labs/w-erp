import { useState } from "react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage, FormDescription } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";

const kpiFormSchema = z.object({
  name: z.string().min(1, "请输入指标名称"),
  category: z.string().min(1, "请选择指标分类"),
  description: z.string().optional(),
  unit: z.string().optional(),
  target_value: z.string().optional(),
  warning_threshold: z.string().optional(),
  danger_threshold: z.string().optional(),
});

interface KPIFormOption {
  value: string;
  label: string;
}

interface KPIFormOptions {
  category_options: KPIFormOption[];
  unit_options: KPIFormOption[];
  create_action: BackendAction | null;
  default_values: {
    category: string;
    unit: string;
    is_active: boolean;
  };
}

interface KPIConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export const KPIConfigDialog = ({ open, onOpenChange, onSuccess }: KPIConfigDialogProps) => {
  const [isSubmitting, setIsSubmitting] = useState(false);

  const form = useForm<z.infer<typeof kpiFormSchema>>({
    resolver: zodResolver(kpiFormSchema),
    defaultValues: {
      name: "",
      category: "",
      description: "",
      unit: "",
      target_value: "",
      warning_threshold: "",
      danger_threshold: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["kpi-config-form-options"],
    queryFn: () => api.get<KPIFormOptions>("/api/v1/kpi_configs/form_options"),
    enabled: open,
  });
  const categoryOptions = formOptions?.category_options ?? [];
  const unitOptions = formOptions?.unit_options ?? [];
  const action = formOptions?.create_action ?? null;

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset({
      name: "",
      category: defaults.category,
      description: "",
      unit: defaults.unit,
      target_value: "",
      warning_threshold: "",
      danger_threshold: "",
    });
  }, [open, formOptions?.default_values, form]);

  const onSubmit = async (values: z.infer<typeof kpiFormSchema>) => {
    setIsSubmitting(true);
    try {
      if (!action) {
        throw new Error("后端未返回 KPI 创建动作");
      }
      await api.runAction(action, {
        name: values.name,
        category: values.category,
        description: values.description || null,
        unit: values.unit || null,
        target_value: values.target_value || null,
        warning_threshold: values.warning_threshold || null,
        danger_threshold: values.danger_threshold || null,
        is_active: true,
      });

      toast.success("KPI指标创建成功");
      form.reset();
      onOpenChange(false);
      onSuccess?.();
    } catch (error: any) {
      toast.error("创建失败: " + error.message);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>新增KPI指标</DialogTitle>
          <DialogDescription>
            配置企业关键经营指标，系统将自动追踪和预警
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>指标名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="如：月度营收" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="category"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>指标分类 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {categoryOptions.map((option) => (
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

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>指标说明</FormLabel>
                  <FormControl>
                    <Textarea 
                      placeholder="描述该指标的计算方式和业务意义"
                      rows={2}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="unit"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>单位</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {unitOptions.map((option) => (
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
                name="target_value"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>目标值</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="设定目标值" {...field} />
                    </FormControl>
                    <FormDescription className="text-xs">
                      用于对比实际表现
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="warning_threshold"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>警告阈值</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="低于此值显示警告" {...field} />
                    </FormControl>
                    <FormDescription className="text-xs">
                      黄色预警提醒
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="danger_threshold"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>危险阈值</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="低于此值显示危险" {...field} />
                    </FormControl>
                    <FormDescription className="text-xs">
                      红色危险提醒
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="flex justify-end gap-2 pt-4">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isSubmitting}
              >
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                  {isSubmitting ? "创建中..." : "创建指标"}
                </Button>
            </div>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
};
