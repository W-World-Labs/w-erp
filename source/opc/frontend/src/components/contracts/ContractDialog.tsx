import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Upload, Loader2, FileText } from "lucide-react";
import { useEffect, useState, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { useDocumentParser } from "@/hooks/useDocumentParser";
import { AutoParseButton } from "@/components/common/AutoParseButton";
import { useToast } from "@/hooks/use-toast";
import { api } from "@/integrations/api/client";

const contractSchema = z.object({
  contract_name: z.string().min(1, "合同标题不能为空"),
  contract_number: z.string().optional(),
  contract_type: z.string().min(1, "请选择合同类型"),
  counterparty_type: z.string().optional(),
  party_a: z.string().min(1, "甲方名称不能为空"),
  party_b: z.string().min(1, "乙方名称不能为空"),
  amount: z.string().min(1, "合同金额不能为空"),
  start_date: z.string().optional(),
  end_date: z.string().min(1, "请选择结束日期"),
  content: z.string().optional(),
  notes: z.string().optional(),
});

type ContractFormValues = z.infer<typeof contractSchema>;

interface ContractDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any, file?: File) => void;
  initialData?: any;
  isSubmitting?: boolean;
}

interface ContractFormOption {
  value: string;
  label: string;
}

interface ContractFormOptions {
  contract_type_options: ContractFormOption[];
  counterparty_type_options: ContractFormOption[];
  default_values: {
    contract_type: string;
    counterparty_type: string;
  };
}

export function ContractDialog({ open, onOpenChange, onSubmit, initialData, isSubmitting }: ContractDialogProps) {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { parseFile, isParsing, error: parseError, enabled: parserEnabled } = useDocumentParser('contract');
  const { toast } = useToast();
  const { data: formOptions } = useQuery({
    queryKey: ["contract-form-options"],
    queryFn: () => api.get<ContractFormOptions>("/api/v1/contracts/form_options"),
    enabled: open,
  });
  const contractTypeOptions = formOptions?.contract_type_options ?? [];
  const counterpartyTypeOptions = formOptions?.counterparty_type_options ?? [];

  const handleAutoParse = async () => {
    if (!selectedFile) return;
    const fields = await parseFile(selectedFile);
    if (fields) {
      if (fields.title || fields.contract_name) form.setValue('contract_name', fields.contract_name || fields.title);
      if (fields.contract_number) form.setValue('contract_number', fields.contract_number);
      if (fields.party_a) form.setValue('party_a', fields.party_a);
      if (fields.party_b) form.setValue('party_b', fields.party_b);
      if (fields.amount) form.setValue('amount', fields.amount);
      if (fields.start_date) form.setValue('start_date', fields.start_date);
      if (fields.end_date) form.setValue('end_date', fields.end_date);
      if (fields.content) form.setValue('content', fields.content);
      if (fields.notes) form.setValue('notes', fields.notes);
      if (fields.contract_type) form.setValue('contract_type', fields.contract_type);
      toast({ title: '✨ AI 识别完成', description: '已自动填入识别到的合同信息，请核对后提交' });
    }
  };

  const form = useForm<ContractFormValues>({
    resolver: zodResolver(contractSchema),
    defaultValues: {
      contract_name: "",
      contract_number: "",
      contract_type: "",
      counterparty_type: "",
      party_a: "",
      party_b: "",
      amount: "",
      start_date: "",
      end_date: "",
      content: "",
      notes: "",
    },
  });

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      contract_name: initialData.contract_name || initialData.title || "",
      contract_number: initialData.contract_number || "",
      contract_type: initialData.contract_type || defaults.contract_type,
      counterparty_type: initialData.counterparty_type || defaults.counterparty_type,
      party_a: initialData.party_a || "",
      party_b: initialData.party_b || "",
      amount: initialData.amount?.toString() || "",
      start_date: initialData.start_date || "",
      end_date: initialData.end_date || "",
      content: initialData.content || "",
      notes: initialData.notes || "",
    } : {
      contract_name: "",
      contract_number: "",
      contract_type: defaults.contract_type,
      counterparty_type: defaults.counterparty_type,
      party_a: "",
      party_b: "",
      amount: "",
      start_date: "",
      end_date: "",
      content: "",
      notes: "",
    });
    setSelectedFile(null);
  }, [open, initialData, formOptions?.default_values, form]);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      // 验证文件类型
      const allowedTypes = ['application/pdf', 'application/msword', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', 'image/jpeg', 'image/jpg', 'image/png'];
      if (!allowedTypes.includes(file.type)) {
        alert('仅支持 PDF、Word 和图片格式的文件');
        return;
      }

      // 验证文件大小 (10MB)
      if (file.size > 10 * 1024 * 1024) {
        alert('文件大小不能超过 10MB');
        return;
      }

      setSelectedFile(file);
    }
  };

  const handleSubmit = (values: ContractFormValues) => {
    onSubmit(values, selectedFile || undefined);

    if (!initialData) {
      form.reset();
      setSelectedFile(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? '编辑合同' : '新建合同'}</DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="contract_name"
                render={({ field }) => (
                  <FormItem className="col-span-2">
                    <FormLabel>合同标题 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入合同标题" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="contract_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>合同类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择合同类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {contractTypeOptions.map((option) => (
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
                name="counterparty_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>对方身份 (AI 防坑)</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value || ""}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择对方类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {counterpartyTypeOptions.map((option) => (
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
                name="amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>合同金额（元）*</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入合同金额" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="party_a"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>甲方 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入甲方名称" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="party_b"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>乙方 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入乙方名称" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="start_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>开始日期</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
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
                    <FormLabel>结束日期 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="content"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>合同内容</FormLabel>
                  <FormControl>
                    <Textarea {...field} placeholder="输入合同主要条款和内容" rows={4} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="notes"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>备注</FormLabel>
                  <FormControl>
                    <Textarea {...field} placeholder="输入备注信息" rows={2} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* 文件上传 */}
            <div className="space-y-2">
              <FormLabel>合同文件</FormLabel>
              <div className="flex items-center gap-4">
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".pdf,.doc,.docx,.jpg,.jpeg,.png"
                  onChange={handleFileChange}
                  className="hidden"
                />
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => fileInputRef.current?.click()}
                  className="gap-2"
                >
                  <Upload className="h-4 w-4" />
                  选择文件
                </Button>
                {selectedFile && (
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <FileText className="h-4 w-4" />
                    {selectedFile.name}
                  </div>
                )}
                {initialData?.file_name && !selectedFile && (
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <FileText className="h-4 w-4" />
                    {initialData.file_name}
                  </div>
                )}
              </div>
              <AutoParseButton
                file={selectedFile}
                isParsing={isParsing}
                enabled={parserEnabled}
                error={parseError}
                onParse={handleAutoParse}
                className="mt-2"
              />
              <p className="text-xs text-muted-foreground">
                支持 PDF、Word、JPG、PNG 格式，文件大小不超过 10MB
              </p>
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    处理中...
                  </>
                ) : (
                  initialData ? '更新' : '创建'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
