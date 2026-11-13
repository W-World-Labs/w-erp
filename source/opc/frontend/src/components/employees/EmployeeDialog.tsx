import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { EmployeeDocuments } from "./EmployeeDocuments";
import { useDocumentParser } from "@/hooks/useDocumentParser";
import { Sparkles, Loader2, Upload, X, FileText, Image as ImageIcon } from "lucide-react";

interface EmployeeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  employee?: any;
  action?: BackendAction | null;
  onSuccess: () => void;
}

type EmployeeFormOption = {
  value: string;
  label: string;
};

type EmployeeFormOptions = {
  status_options: EmployeeFormOption[];
  employment_type_options: EmployeeFormOption[];
  default_values: {
    status: string;
    employment_type: string;
  };
};

export function EmployeeDialog({ open, onOpenChange, employee, action: createAction, onSuccess }: EmployeeDialogProps) {
  const { register, handleSubmit, reset, setValue, watch } = useForm();
  const [submitting, setSubmitting] = useState(false);
  const [currentEmployeeId, setCurrentEmployeeId] = useState<string | undefined>(employee?.id);
  const [aiFile, setAiFile] = useState<File | null>(null);
  const { parseFile, isParsing, error: parseError, enabled: parserEnabled } = useDocumentParser('employee');
  const { data: formOptions } = useQuery({
    queryKey: ["employee-form-options"],
    queryFn: () => api.get<EmployeeFormOptions>("/api/employees/form_options"),
    enabled: open,
  });
  const employmentTypeOptions = formOptions?.employment_type_options ?? [];
  const statusOptions = formOptions?.status_options ?? [];

  const fillableFields = [
    'full_name', 'id_number', 'phone', 'email',
    'department', 'position', 'bank_account', 'bank_name',
    'emergency_contact', 'emergency_phone', 'hire_date',
    'contract_start_date', 'contract_end_date', 'base_salary', 'notes',
  ];

  const handleAiFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > 50 * 1024 * 1024) {
      toast.error("文件大小不能超过 50MB");
      return;
    }
    setAiFile(file);
  };

  const handleAutoParse = async () => {
    if (!aiFile) return;
    const fields = await parseFile(aiFile);
    if (fields) {
      let filled = 0;
      for (const [key, value] of Object.entries(fields)) {
        if (value && fillableFields.includes(key)) {
          setValue(key, value);
          filled++;
        }
      }

      toast.success(filled > 0
        ? `AI 识别完成，已填入 ${filled} 个字段，请核对后提交`
        : '未能从文件中识别到信息，请尝试其他文件');
    }
  };

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (employee) {
      Object.keys(employee).forEach((key) => {
        setValue(key, employee[key]);
      });
      setValue("employment_type", employee.employment_type || defaults.employment_type);
      setValue("status", employee.status || defaults.status);
      setCurrentEmployeeId(employee.id);
    } else {
      reset();
      setValue("employment_type", defaults.employment_type);
      setValue("status", defaults.status);
      setCurrentEmployeeId(undefined);
    }
    setAiFile(null);
  }, [employee, setValue, reset, open, formOptions?.default_values]);

  const onSubmit = async (data: any) => {
    setSubmitting(true);

    try {
      const employeeData = {
        ...data,
        name: data.name || data.full_name,
        base_salary: data.base_salary || 0,
        social_security_base: data.social_security_base || 0,
        housing_fund_base: data.housing_fund_base || 0,
      };

      const action = employee?.update_action ?? createAction;
      if (!action) {
        toast.error("当前无法保存员工");
        return;
      }

      const result = (await api.runAction(action, employeeData)) as any;
      if (!employee) {
        // 设置新员工ID以便上传文档
        const newEmployee = result.data || result;
        setCurrentEmployeeId(newEmployee.id);
        toast.success(action.success_message || "添加成功，您可以继续上传文档");
        return; // 不关闭对话框，让用户可以上传文档
      }

      toast.success(action.success_message || "员工更新成功");
      onSuccess();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{employee ? "编辑员工" : "添加员工"}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
          {/* AI 智能识别区域 */}
          {parserEnabled && (
          <div className="bg-gradient-to-r from-primary/5 to-primary/10 rounded-lg p-4 space-y-3">
            <h3 className="font-semibold flex items-center gap-2">
              <Sparkles className="h-4 w-4" />
              智能识别
              <span className="text-xs font-normal text-muted-foreground">上传简历、身份证、学历证书等，AI 自动填入信息</span>
            </h3>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="flex-shrink-0"
                onClick={() => document.getElementById('ai-employee-upload')?.click()}
                disabled={isParsing}
              >
                <Upload className="mr-2 h-4 w-4" />
                选择文件
              </Button>
              <input
                id="ai-employee-upload"
                type="file"
                accept="image/jpeg,image/jpg,image/png,application/pdf"
                className="hidden"
                onChange={handleAiFileChange}
              />
              {aiFile && (
                <>
                  <div className="flex items-center gap-1 px-2 py-1 bg-background rounded border text-sm flex-1 min-w-0">
                    {aiFile.type === 'application/pdf' ? (
                      <FileText className="h-4 w-4 text-red-500 flex-shrink-0" />
                    ) : (
                      <ImageIcon className="h-4 w-4 text-blue-500 flex-shrink-0" />
                    )}
                    <span className="truncate">{aiFile.name}</span>
                    <Button type="button" variant="ghost" size="sm" className="h-5 w-5 p-0 ml-auto flex-shrink-0" onClick={() => setAiFile(null)}>
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    onClick={handleAutoParse}
                    disabled={isParsing}
                    className="flex-shrink-0"
                  >
                    {isParsing ? (
                      <><Loader2 className="mr-2 h-4 w-4 animate-spin" />识别中...</>
                    ) : (
                      <><Sparkles className="mr-2 h-4 w-4" />AI 识别</>
                    )}
                  </Button>
                </>
              )}
            </div>
            {parseError && <p className="text-xs text-destructive">{parseError}</p>}
          </div>
          )}

          <div className="space-y-4">
            <h3 className="font-semibold">基本信息</h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label>员工编号 *</Label>
                <Input {...register("employee_number", { required: true })} placeholder="如：EMP001" />
              </div>

              <div className="space-y-2">
                <Label>姓名 *</Label>
                <Input {...register("full_name", { required: true })} placeholder="员工姓名" />
              </div>

              <div className="space-y-2">
                <Label>身份证号</Label>
                <Input {...register("id_number")} placeholder="身份证号" />
              </div>

              <div className="space-y-2">
                <Label>联系电话</Label>
                <Input {...register("phone")} placeholder="手机号" />
              </div>

              <div className="space-y-2">
                <Label>邮箱</Label>
                <Input type="email" {...register("email")} placeholder="邮箱地址" />
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="font-semibold">岗位信息</h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label>部门</Label>
                <Input {...register("department")} placeholder="所属部门" />
              </div>

              <div className="space-y-2">
                <Label>职位</Label>
                <Input {...register("position")} placeholder="职位名称" />
              </div>

              <div className="space-y-2">
                <Label>雇佣类型 *</Label>
                <Select onValueChange={(value) => setValue("employment_type", value)} value={watch("employment_type")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {employmentTypeOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>入职日期 *</Label>
                <Input type="date" {...register("hire_date", { required: true })} />
              </div>

              <div className="space-y-2">
                <Label>合同开始日期</Label>
                <Input type="date" {...register("contract_start_date")} />
              </div>

              <div className="space-y-2">
                <Label>合同结束日期</Label>
                <Input type="date" {...register("contract_end_date")} />
              </div>

              <div className="space-y-2">
                <Label>状态</Label>
                <Select onValueChange={(value) => setValue("status", value)} value={watch("status")}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {statusOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>基本工资</Label>
                <Input type="number" step="0.01" {...register("base_salary")} placeholder="0.00" />
              </div>

              <div className="space-y-2">
                <Label>社保基数</Label>
                <Input type="number" step="0.01" {...register("social_security_base")} placeholder="0.00" />
              </div>

              <div className="space-y-2">
                <Label>公积金基数</Label>
                <Input type="number" step="0.01" {...register("housing_fund_base")} placeholder="0.00" />
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="font-semibold">银行信息</h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>银行账号</Label>
                <Input {...register("bank_account")} placeholder="银行账号" />
              </div>

              <div className="space-y-2">
                <Label>开户行</Label>
                <Input {...register("bank_name")} placeholder="开户行名称" />
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="font-semibold">紧急联系人</h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>紧急联系人</Label>
                <Input {...register("emergency_contact")} placeholder="姓名" />
              </div>

              <div className="space-y-2">
                <Label>紧急联系电话</Label>
                <Input {...register("emergency_phone")} placeholder="电话号码" />
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <Label>备注</Label>
            <Textarea {...register("notes")} placeholder="备注信息" rows={3} />
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-semibold">员工文档</h3>
            <EmployeeDocuments
              employeeId={currentEmployeeId}
              disabled={submitting}
            />
          </div>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                onOpenChange(false);
                if (!employee) {
                  // 如果是新添加的员工，关闭时刷新列表
                  onSuccess();
                }
              }}
              disabled={submitting}
            >
              {currentEmployeeId && !employee ? "完成" : "取消"}
            </Button>
            <Button type="submit" disabled={submitting}>
              {submitting ? "处理中..." : employee ? "更新" : "添加"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
