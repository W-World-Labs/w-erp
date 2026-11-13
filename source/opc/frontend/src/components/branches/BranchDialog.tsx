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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";

const branchSchema = z.object({
  branch_name: z.string().min(1, "请输入分支机构名称"),
  branch_type: z.string().min(1, "请选择机构类型"),
  branch_code: z.string().optional(),
  legal_representative: z.string().optional(),
  unified_social_credit_code: z.string().optional(),
  registration_date: z.string().optional(),
  registered_capital: z.string().optional(),
  branch_address: z.string().optional(),
  contact_person: z.string().optional(),
  contact_phone: z.string().optional(),
  contact_email: z.string().email("请输入有效的邮箱地址").optional().or(z.literal("")),
  business_scope: z.string().optional(),
  employee_count: z.string().optional(),
  parent_branch_id: z.string().optional(),
  notes: z.string().optional(),
});

type BranchForm = z.infer<typeof branchSchema>;

type BranchOption = {
  value: string;
  label: string;
};

type BranchFormOptions = {
  branch_type_options: BranchOption[];
  parent_branch_options: BranchOption[];
  default_values: {
    branch_type: string;
    employee_count: string;
    parent_branch_id: string;
  };
};

interface BranchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  branch?: any;
  action?: BackendAction | null;
  parentBranchOptions?: BranchOption[];
  onSuccess: () => void;
}

export function BranchDialog({
  open,
  onOpenChange,
  branch,
  action,
  parentBranchOptions = [],
  onSuccess,
}: BranchDialogProps) {
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    setValue,
    watch,
    formState: { errors },
  } = useForm<BranchForm>({
    resolver: zodResolver(branchSchema),
    defaultValues: {
      branch_type: "",
    },
  });

  const branchType = watch("branch_type");
  const { data: formOptions } = useQuery({
    queryKey: ["branch-form-options", branch?.id],
    queryFn: () =>
      api.get<BranchFormOptions>(
        `/api/v1/branches/form_options?exclude_id=${encodeURIComponent(branch?.id || "")}`,
      ),
    enabled: open,
  });
  const branchTypes = formOptions?.branch_type_options ?? [];
  const parentOptions = formOptions?.parent_branch_options ?? parentBranchOptions;
  const parentNoneValue = formOptions?.default_values.parent_branch_id ?? "";

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (branch) {
      reset({
        branch_name: branch.branch_name || "",
        branch_type: branch.branch_type || defaults.branch_type,
        branch_code: branch.branch_code || "",
        legal_representative: branch.legal_representative || "",
        unified_social_credit_code: branch.unified_social_credit_code || "",
        registration_date: branch.registration_date || "",
        registered_capital: branch.registered_capital?.toString() || "",
        branch_address: branch.branch_address || "",
        contact_person: branch.contact_person || "",
        contact_phone: branch.contact_phone || "",
        contact_email: branch.contact_email || "",
        business_scope: branch.business_scope || "",
        employee_count: branch.employee_count?.toString() || "",
        parent_branch_id: branch.parent_branch_id || "",
        notes: branch.notes || "",
      });
    } else {
      reset({
        branch_name: "",
        branch_type: defaults.branch_type,
        branch_code: "",
        legal_representative: "",
        unified_social_credit_code: "",
        registration_date: "",
        registered_capital: "",
        branch_address: "",
        contact_person: "",
        contact_phone: "",
        contact_email: "",
        business_scope: "",
        employee_count: defaults.employee_count,
        parent_branch_id: "",
        notes: "",
      });
    }
  }, [branch, reset, open, formOptions?.default_values]);

  const onSubmit = async (data: BranchForm) => {
    setSubmitting(true);

    try {
      if (!action) {
        toast.error("当前无法保存分支机构");
        return;
      }
      const branchData = {
        branch_name: data.branch_name,
        branch_type: data.branch_type,
        branch_code: data.branch_code || null,
        legal_representative: data.legal_representative || null,
        unified_social_credit_code: data.unified_social_credit_code || null,
        registration_date: data.registration_date || null,
        registered_capital: data.registered_capital || null,
        branch_address: data.branch_address || null,
        contact_person: data.contact_person || null,
        contact_phone: data.contact_phone || null,
        contact_email: data.contact_email || null,
        business_scope: data.business_scope || null,
        employee_count: data.employee_count || formOptions?.default_values.employee_count,
        parent_branch_id: data.parent_branch_id || null,
        notes: data.notes || null,
      };

      await api.runAction(action, branchData);
      toast.success(action.success_message || "分支机构已保存");

      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error("Error saving branch:", error);
      toast.error("保存失败，请重试");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {branch ? "编辑分支机构" : "添加分支机构"}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="branch_name">
                机构名称 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="branch_name"
                {...register("branch_name")}
                placeholder="请输入分支机构名称"
              />
              {errors.branch_name && (
                <p className="text-sm text-destructive">
                  {errors.branch_name.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="branch_type">
                机构类型 <span className="text-destructive">*</span>
              </Label>
              <Select
                value={branchType}
                onValueChange={(value) => setValue("branch_type", value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="选择机构类型" />
                </SelectTrigger>
                <SelectContent>
                  {branchTypes.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.branch_type && (
                <p className="text-sm text-destructive">
                  {errors.branch_type.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="branch_code">机构编码</Label>
              <Input
                id="branch_code"
                {...register("branch_code")}
                placeholder="如：BJ-001"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="parent_branch_id">上级机构</Label>
              <Select
                value={watch("parent_branch_id") || parentNoneValue}
                onValueChange={(value) =>
                  setValue(
                    "parent_branch_id",
                    value === parentNoneValue ? "" : value,
                  )
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder="选择上级机构（可选）" />
                </SelectTrigger>
                <SelectContent>
                  {parentOptions.map((parent) => (
                    <SelectItem key={parent.value} value={parent.value}>
                      {parent.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="legal_representative">负责人/法定代表人</Label>
              <Input
                id="legal_representative"
                {...register("legal_representative")}
                placeholder="请输入负责人姓名"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="unified_social_credit_code">
                统一社会信用代码
              </Label>
              <Input
                id="unified_social_credit_code"
                {...register("unified_social_credit_code")}
                placeholder="请输入统一社会信用代码"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="registration_date">成立/注册日期</Label>
              <Input
                id="registration_date"
                type="date"
                {...register("registration_date")}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="registered_capital">注册资本（元）</Label>
              <Input
                id="registered_capital"
                type="number"
                step="0.01"
                {...register("registered_capital")}
                placeholder="请输入注册资本"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="employee_count">员工人数</Label>
              <Input
                id="employee_count"
                type="number"
                {...register("employee_count")}
                placeholder="请输入员工人数"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="contact_person">联系人</Label>
              <Input
                id="contact_person"
                {...register("contact_person")}
                placeholder="请输入联系人姓名"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="contact_phone">联系电话</Label>
              <Input
                id="contact_phone"
                {...register("contact_phone")}
                placeholder="请输入联系电话"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="contact_email">联系邮箱</Label>
              <Input
                id="contact_email"
                type="email"
                {...register("contact_email")}
                placeholder="请输入联系邮箱"
              />
              {errors.contact_email && (
                <p className="text-sm text-destructive">
                  {errors.contact_email.message}
                </p>
              )}
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="branch_address">机构地址</Label>
              <Input
                id="branch_address"
                {...register("branch_address")}
                placeholder="请输入机构地址"
              />
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="business_scope">经营范围</Label>
              <Textarea
                id="business_scope"
                {...register("business_scope")}
                placeholder="请输入经营范围"
                rows={2}
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
