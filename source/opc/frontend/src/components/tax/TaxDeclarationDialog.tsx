import { useState, useEffect } from "react";
import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";

const taxDeclarationSchema = z.object({
  declaration_type: z.string().min(1, "请选择申报类型"),
  declaration_period: z.string().min(1, "请选择申报周期"),
  period_start: z.string().min(1, "请选择开始日期"),
  period_end: z.string().min(1, "请选择结束日期"),
  declaration_deadline: z.string().min(1, "请选择申报截止日期"),
  declared_amount: z.string().optional(),
  tax_amount: z.string().optional(),
  declaration_date: z.string().optional(),
  notes: z.string().optional(),
});

type TaxDeclarationForm = z.infer<typeof taxDeclarationSchema>;

type TaxDeclarationOption = {
  value: string;
  label: string;
};

type TaxDeclarationFormOptions = {
  declaration_type_options: TaxDeclarationOption[];
  declaration_period_options: TaxDeclarationOption[];
  default_values: {
    declaration_type: string;
    declaration_period: string;
  };
};

interface TaxDeclarationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  declaration?: any;
  action?: BackendAction | null;
  onSuccess: () => void;
}

export function TaxDeclarationDialog({
  open,
  onOpenChange,
  declaration,
  action: createAction,
  onSuccess,
}: TaxDeclarationDialogProps) {
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<TaxDeclarationForm>({
    resolver: zodResolver(taxDeclarationSchema),
  });

  const { data: formOptions } = useQuery({
    queryKey: ["tax-declaration-form-options"],
    queryFn: () => api.get<TaxDeclarationFormOptions>("/api/v1/tax_declarations/form_options"),
    enabled: open,
  });
  const declarationTypeOptions = formOptions?.declaration_type_options ?? [];
  const declarationPeriodOptions = formOptions?.declaration_period_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (declaration) {
      reset({
        declaration_type: declaration.declaration_type || defaults.declaration_type,
        declaration_period: declaration.declaration_period || defaults.declaration_period,
        period_start: declaration.period_start || "",
        period_end: declaration.period_end || "",
        declaration_deadline: declaration.declaration_deadline || "",
        declared_amount: declaration.declared_amount?.toString() || "",
        tax_amount: declaration.tax_amount?.toString() || "",
        declaration_date: declaration.declaration_date || "",
        notes: declaration.notes || "",
      });
    } else {
      reset({
        declaration_type: defaults.declaration_type,
        declaration_period: defaults.declaration_period,
        period_start: "",
        period_end: "",
        declaration_deadline: "",
        declared_amount: "",
        tax_amount: "",
        declaration_date: "",
        notes: "",
      });
    }
  }, [declaration, reset, open, formOptions?.default_values]);

  const onSubmit = async (data: TaxDeclarationForm) => {
    setSubmitting(true);

    try {
      const declarationData = {
        declaration_type: data.declaration_type,
        declaration_period: data.declaration_period,
        period_start: data.period_start,
        period_end: data.period_end,
        declaration_deadline: data.declaration_deadline,
        declared_amount: data.declared_amount || null,
        tax_amount: data.tax_amount || null,
        declaration_date: data.declaration_date || null,
        notes: data.notes || null,
      };

      const action = declaration?.update_action ?? createAction;
      if (!action) {
        toast.error("当前无法保存税务申报");
        return;
      }

      await api.runAction(action, declarationData);
      toast.success(action.success_message || "税务申报已保存");

      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error("Error saving tax declaration:", error);
      toast.error("保存失败，请重试");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {declaration ? "编辑税务申报" : "添加税务申报"}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="declaration_type">
                申报类型 <span className="text-destructive">*</span>
              </Label>
              <select
                id="declaration_type"
                {...register("declaration_type")}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {declarationTypeOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
              {errors.declaration_type && (
                <p className="text-sm text-destructive">
                  {errors.declaration_type.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="declaration_period">
                申报周期 <span className="text-destructive">*</span>
              </Label>
              <select
                id="declaration_period"
                {...register("declaration_period")}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {declarationPeriodOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
              {errors.declaration_period && (
                <p className="text-sm text-destructive">
                  {errors.declaration_period.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="period_start">
                开始日期 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="period_start"
                type="date"
                {...register("period_start")}
              />
              {errors.period_start && (
                <p className="text-sm text-destructive">
                  {errors.period_start.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="period_end">
                结束日期 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="period_end"
                type="date"
                {...register("period_end")}
              />
              {errors.period_end && (
                <p className="text-sm text-destructive">
                  {errors.period_end.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="declaration_deadline">
                申报截止日期 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="declaration_deadline"
                type="date"
                {...register("declaration_deadline")}
              />
              {errors.declaration_deadline && (
                <p className="text-sm text-destructive">
                  {errors.declaration_deadline.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="declared_amount">申报金额（元）</Label>
              <Input
                id="declared_amount"
                type="number"
                step="0.01"
                {...register("declared_amount")}
                placeholder="请输入申报金额"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="tax_amount">应纳税额（元）</Label>
              <Input
                id="tax_amount"
                type="number"
                step="0.01"
                {...register("tax_amount")}
                placeholder="请输入应纳税额"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="declaration_date">实际申报日期</Label>
              <Input
                id="declaration_date"
                type="date"
                {...register("declaration_date")}
              />
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="notes">备注</Label>
              <Textarea
                id="notes"
                {...register("notes")}
                placeholder="请输入备注信息"
                rows={2}
              />
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={submitting}
            >
              取消
            </Button>
            <Button type="submit" disabled={submitting}>
              {submitting ? "保存中..." : "保存"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
