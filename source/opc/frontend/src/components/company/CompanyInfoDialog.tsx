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

const companyInfoSchema = z.object({
  company_name: z.string().min(1, "请输入公司名称"),
  legal_representative: z.string().optional(),
  unified_social_credit_code: z.string().optional(),
  registration_date: z.string().optional(),
  registered_capital: z.string().optional(),
  business_scope: z.string().optional(),
  company_address: z.string().optional(),
  contact_phone: z.string().optional(),
  contact_email: z.string().email("请输入有效的邮箱地址").optional().or(z.literal("")),
  notes: z.string().optional(),
  taxpayer_type: z.string().optional(),
  vat_rate: z.string().optional(),
  invoice_limit: z.string().optional(),
  tax_registration_number: z.string().optional(),
  tax_authority: z.string().optional(),
  cash_flow_threshold: z.string().optional(),
  // OPC Founder Profile
  founder_education: z.string().optional(),
  founder_is_expat: z.boolean().default(false).optional(),
  founder_is_overseas_returnee: z.boolean().default(false).optional(),
});

type CompanyInfoForm = z.infer<typeof companyInfoSchema>;

type CompanyInfoOption = {
  value: string;
  label: string;
};

type CompanyInfoFormOptions = {
  taxpayer_type_options: CompanyInfoOption[];
  founder_education_options: CompanyInfoOption[];
  default_values: {
    taxpayer_type: string;
    vat_rate: string;
    invoice_limit: string;
    cash_flow_threshold: string;
    founder_education: string;
    founder_is_expat: boolean;
    founder_is_overseas_returnee: boolean;
  };
};

interface CompanyInfoDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  companyInfo?: any;
  action?: BackendAction | null;
  onSuccess: () => void;
}

export function CompanyInfoDialog({
  open,
  onOpenChange,
  companyInfo,
  action,
  onSuccess,
}: CompanyInfoDialogProps) {
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CompanyInfoForm>({
    resolver: zodResolver(companyInfoSchema),
  });

  const { data: formOptions } = useQuery({
    queryKey: ["company-info-form-options"],
    queryFn: () => api.get<CompanyInfoFormOptions>("/api/v1/company_info/form_options"),
    enabled: open,
  });
  const taxpayerTypeOptions = formOptions?.taxpayer_type_options ?? [];
  const founderEducationOptions = formOptions?.founder_education_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (companyInfo) {
      reset({
        company_name: companyInfo.company_name || "",
        legal_representative: companyInfo.legal_representative || "",
        unified_social_credit_code: companyInfo.unified_social_credit_code || "",
        registration_date: companyInfo.registration_date || "",
        registered_capital: companyInfo.registered_capital?.toString() || "",
        business_scope: companyInfo.business_scope || "",
        company_address: companyInfo.company_address || "",
        contact_phone: companyInfo.contact_phone || "",
        contact_email: companyInfo.contact_email || "",
        notes: companyInfo.notes || "",
        taxpayer_type: companyInfo.taxpayer_type || defaults.taxpayer_type,
        vat_rate: companyInfo.vat_rate?.toString() || defaults.vat_rate,
        invoice_limit: companyInfo.invoice_limit?.toString() || defaults.invoice_limit,
        tax_registration_number: companyInfo.tax_registration_number || "",
        tax_authority: companyInfo.tax_authority || "",
        cash_flow_threshold: companyInfo.cash_flow_threshold?.toString() || defaults.cash_flow_threshold,
        founder_education: companyInfo.founder_education || defaults.founder_education,
        founder_is_expat: companyInfo.founder_is_expat === 1,
        founder_is_overseas_returnee: companyInfo.founder_is_overseas_returnee === 1,
      });
    } else {
      reset({
        company_name: "",
        legal_representative: "",
        unified_social_credit_code: "",
        registration_date: "",
        registered_capital: "",
        business_scope: "",
        company_address: "",
        contact_phone: "",
        contact_email: "",
        notes: "",
        taxpayer_type: defaults.taxpayer_type,
        vat_rate: defaults.vat_rate,
        invoice_limit: defaults.invoice_limit,
        tax_registration_number: "",
        tax_authority: "",
        cash_flow_threshold: defaults.cash_flow_threshold,
        founder_education: defaults.founder_education,
        founder_is_expat: defaults.founder_is_expat,
        founder_is_overseas_returnee: defaults.founder_is_overseas_returnee,
      });
    }
  }, [companyInfo, reset, open, formOptions?.default_values]);

  const onSubmit = async (data: CompanyInfoForm) => {
    setSubmitting(true);

    try {
      if (!action) {
        toast.error("当前无法保存公司信息");
        return;
      }
      const companyData = {
        company_name: data.company_name,
        legal_representative: data.legal_representative || null,
        unified_social_credit_code: data.unified_social_credit_code || null,
        registration_date: data.registration_date || null,
        registered_capital: data.registered_capital || null,
        business_scope: data.business_scope || null,
        company_address: data.company_address || null,
        contact_phone: data.contact_phone || null,
        contact_email: data.contact_email || null,
        notes: data.notes || null,
        taxpayer_type: data.taxpayer_type || formOptions?.default_values.taxpayer_type,
        vat_rate: data.vat_rate || null,
        invoice_limit: data.invoice_limit || null,
        tax_registration_number: data.tax_registration_number || null,
        tax_authority: data.tax_authority || null,
        cash_flow_threshold: data.cash_flow_threshold || null,
        founder_education: data.founder_education || null,
        founder_is_expat: data.founder_is_expat ? 1 : 0,
        founder_is_overseas_returnee: data.founder_is_overseas_returnee ? 1 : 0,
      };

      await api.runAction(action, companyData);
      toast.success(action.success_message || "公司信息已保存");

      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error("Error saving company info:", error);
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
            {companyInfo ? "编辑公司信息" : "添加公司信息"}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="company_name">
                公司名称 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="company_name"
                {...register("company_name")}
                placeholder="请输入公司名称"
              />
              {errors.company_name && (
                <p className="text-sm text-destructive">
                  {errors.company_name.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="legal_representative">法定代表人</Label>
              <Input
                id="legal_representative"
                {...register("legal_representative")}
                placeholder="请输入法定代表人"
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
              <Label htmlFor="registration_date">注册日期</Label>
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
              <Label htmlFor="company_address">公司地址</Label>
              <Input
                id="company_address"
                {...register("company_address")}
                placeholder="请输入公司地址"
              />
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="business_scope">经营范围</Label>
              <Textarea
                id="business_scope"
                {...register("business_scope")}
                placeholder="请输入经营范围"
                rows={3}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="taxpayer_type">纳税人类型</Label>
              <select
                id="taxpayer_type"
                {...register("taxpayer_type")}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {taxpayerTypeOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="vat_rate">增值税税率（%）</Label>
              <Input
                id="vat_rate"
                type="number"
                step="0.01"
                {...register("vat_rate")}
                placeholder="例如：3、6、13"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="invoice_limit">单张发票开具限额（元）</Label>
              <Input
                id="invoice_limit"
                type="number"
                step="0.01"
                {...register("invoice_limit")}
                placeholder="例如：100000"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="tax_registration_number">税务登记号</Label>
              <Input
                id="tax_registration_number"
                {...register("tax_registration_number")}
                placeholder="请输入税务登记号"
              />
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="tax_authority">主管税务机关</Label>
              <Input
                id="tax_authority"
                {...register("tax_authority")}
                placeholder="请输入主管税务机关"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="cash_flow_threshold">现金流安全阈值（元）</Label>
              <Input
                id="cash_flow_threshold"
                type="number"
                step="0.01"
                {...register("cash_flow_threshold")}
                placeholder="例如：50000"
              />
              <p className="text-xs text-muted-foreground">
                当预测现金流低于此阈值时，系统将发送预警通知
              </p>
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

            {/* Founder Profile Section */}
            <div className="md:col-span-2 pt-4 border-t mt-4">
              <h3 className="text-lg font-semibold mb-4">创始人画像 (OPC 匹配政策)</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="founder_education">最高学历</Label>
                  <select
                    id="founder_education"
                    {...register("founder_education")}
                    className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    {founderEducationOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </div>

                <div className="flex flex-col gap-4 mt-8">
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="founder_is_expat"
                      {...register("founder_is_expat")}
                      className="h-4 w-4 rounded border-gray-300"
                    />
                    <Label htmlFor="founder_is_expat" className="font-normal cursor-pointer">
                      外籍高层次人才 (Expat)
                    </Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="founder_is_overseas_returnee"
                      {...register("founder_is_overseas_returnee")}
                      className="h-4 w-4 rounded border-gray-300"
                    />
                    <Label htmlFor="founder_is_overseas_returnee" className="font-normal cursor-pointer">
                      海外归国人员 (海归/留学生)
                    </Label>
                  </div>
                </div>
              </div>
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
