import { useState, useEffect, useLayoutEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api, files, type BackendAction } from "@/integrations/api/client";
import { toast } from 'sonner';
import { format } from 'date-fns';
import { CalendarIcon, Loader2, Upload, X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useDocumentParser } from '@/hooks/useDocumentParser';
import { AutoParseButton } from '@/components/common/AutoParseButton';

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';

/** Map legacy UI tabs/OCR values onto the canonical receivable|payable axis. */
function normalizeInvoiceTypeUi(raw?: string | null): string {
  const value = (raw || '').trim().toLowerCase();
  if (!value) return '';
  if (['receivable', 'output', 'income', '销项', '销项发票'].includes(value)) {
    return 'receivable';
  }
  if (['payable', 'input', 'expense', '进项', '进项发票'].includes(value)) {
    return 'payable';
  }
  return value;
}

function isReceivableType(raw?: string | null): boolean {
  return normalizeInvoiceTypeUi(raw) === 'receivable';
}

const invoiceSchema = z.object({
  // Empty on create → backend allocates INV-YYYY-NNNN.
  invoice_number: z.string().max(50, '发票号不能超过50字符').optional().or(z.literal('')),
  // Display name; for receivable, auto-filled from client dropdown when empty on backend.
  supplier: z.string().max(100, '名称不能超过100字符').optional().or(z.literal('')),
  client_id: z.string().optional().or(z.literal('')),
  amount: z.string().min(1, '金额不能为空'),
  tax_amount: z.string().optional(),
  invoice_date: z.date({ required_error: '请选择开票日期' }),
  due_date: z.date().optional(),
  description: z.string().max(500, '描述不能超过500字符').optional(),
  invoice_type: z.string().optional(),
}).superRefine((data, ctx) => {
  const receivable = isReceivableType(data.invoice_type);
  const hasClient = Boolean((data.client_id || '').trim());
  const hasSupplier = Boolean((data.supplier || '').trim());
  if (receivable && !hasClient && !hasSupplier) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: '请选择客户或填写客户名称',
      path: ['client_id'],
    });
  }
  if (!receivable && !hasSupplier) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: '供应商名称不能为空',
      path: ['supplier'],
    });
  }
});

type InvoiceFormData = z.infer<typeof invoiceSchema>;

interface Invoice {
  id: string;
  invoice_number: string;
  invoice_type: string | null;
  amount: number;
  tax_amount: number | null;
  invoice_date: string;
  due_date: string | null;
  status: string;
  supplier: string | null;
  client_id?: string | null;
  description: string | null;
  file_path: string | null;
  file_name?: string | null;
  file_label?: string | null;
  created_by: string | null;
  created_at: string;
  updated_at: string;
  update_action?: BackendAction | null;
}

interface InvoiceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  invoice?: Invoice | null;
  action?: BackendAction | null;
  onSuccess: () => void;
  /** Canonical receivable|payable, or legacy output|input tabs. */
  defaultInvoiceType?: string;
}

interface InvoiceFormOption {
  value: string;
  label: string;
  name?: string;
}

interface InvoiceFormOptions {
  invoice_type_options: InvoiceFormOption[];
  client_options?: InvoiceFormOption[];
  default_invoice_type: string;
}

export function InvoiceDialog({ open, onOpenChange, invoice, action: createAction, onSuccess, defaultInvoiceType }: InvoiceDialogProps) {
  const [loading, setLoading] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [uploadProgress, setUploadProgress] = useState(0);
  const { parseFile, isParsing, error: parseError, enabled: parserEnabled } = useDocumentParser('invoice');
  const { data: formOptions } = useQuery({
    queryKey: ['invoice-form-options'],
    queryFn: () => api.get<InvoiceFormOptions>('/api/invoices/form_options'),
  });
  const invoiceTypeOptions = formOptions?.invoice_type_options ?? [];
  const clientOptions = formOptions?.client_options ?? [];
  const defaultType = normalizeInvoiceTypeUi(
    defaultInvoiceType || formOptions?.default_invoice_type || '',
  );

  const parseGeneration = useRef(0);
  // A result belongs only to the file and dialog instance that requested it.
  useLayoutEffect(() => {
    parseGeneration.current += 1;
    return () => { parseGeneration.current += 1; };
  }, [open, invoice, defaultType, file]);

  const handleAutoParse = async () => {
    if (!file || !open) return;
    const generation = ++parseGeneration.current;
    const fields = await parseFile(file);
    if (generation !== parseGeneration.current) return;
    if (fields) {
      if (fields.invoice_number) form.setValue('invoice_number', fields.invoice_number);
      // OCR amount is net; ERP amount is the total receivable/payable.
      const gross = fields.total_amount;
      if (gross != null && gross !== '') {
        form.setValue('amount', String(gross));
      } else if (fields.amount != null && fields.amount !== '') {
        const net = Number(fields.amount);
        const tax = Number(fields.tax_amount);
        const hasTax = fields.tax_amount != null && fields.tax_amount !== '';
        form.setValue('amount', hasTax && Number.isFinite(net) && Number.isFinite(tax)
          ? (Math.round((net + tax) * 100) / 100).toString()
          : String(fields.amount));
      }
      if (fields.tax_amount != null && fields.tax_amount !== '') {
        form.setValue('tax_amount', String(fields.tax_amount));
      }
      const invoiceType = normalizeInvoiceTypeUi(fields.invoice_type)
        || form.getValues('invoice_type');
      if (invoiceType) form.setValue('invoice_type', invoiceType);
      const counterparty = isReceivableType(invoiceType) ? fields.buyer : fields.supplier;
      if (counterparty) form.setValue('supplier', counterparty);
      if (fields.invoice_date) {
        const d = new Date(fields.invoice_date + 'T00:00:00');
        if (!isNaN(d.getTime())) form.setValue('invoice_date', d);
      }
      if (fields.description) form.setValue('description', fields.description);
      const src = fields.parse_source as string | undefined;
      const warn = Array.isArray(fields.warnings) ? (fields.warnings as string[]) : [];
      if (warn.length) {
        toast.success(
          `识别完成（${src || 'vision'}），请核对：${warn.slice(0, 2).join('；')}`,
        );
      } else if (src && src !== 'vision') {
        toast.success(`✨ 结构化直读完成（${src}），请核对已填入的信息`);
      } else {
        toast.success('✨ AI 识别完成，请核对已填入的信息');
      }
    }
  };

  const form = useForm<InvoiceFormData>({
    resolver: zodResolver(invoiceSchema),
    defaultValues: {
      invoice_number: '',
      supplier: '',
      client_id: '',
      amount: '',
      tax_amount: '',
      description: '',
      invoice_type: defaultType,
    },
  });

  const watchedType = form.watch('invoice_type');
  const receivable = isReceivableType(watchedType);

  useEffect(() => {
    if (open) {
      setFile(null);
      setUploadProgress(0);
      if (invoice) {
        form.reset({
          invoice_number: invoice.invoice_number,
          supplier: invoice.supplier || '',
          client_id: invoice.client_id || '',
          amount: invoice.amount.toString(),
          tax_amount: invoice.tax_amount?.toString() || '',
          invoice_date: new Date(invoice.invoice_date),
          due_date: invoice.due_date ? new Date(invoice.due_date) : undefined,
          description: invoice.description || '',
          invoice_type: normalizeInvoiceTypeUi(invoice.invoice_type) || defaultType,
        });
      } else {
        form.reset({
          invoice_number: '',
          supplier: '',
          client_id: '',
          amount: '',
          tax_amount: '',
          description: '',
          invoice_type: defaultType,
        });
      }
    }
  }, [open, invoice, defaultType]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      // 验证文件大小（最大 10MB）
      if (selectedFile.size > 10 * 1024 * 1024) {
        toast.error('文件大小不能超过 10MB');
        return;
      }
      // 验证文件类型（含数电票 XML/OFD 结构化直读 + 图片/PDF OCR）
      const name = selectedFile.name.toLowerCase();
      const validTypes = [
        'application/pdf',
        'image/jpeg',
        'image/png',
        'image/jpg',
        'image/webp',
        'application/xml',
        'text/xml',
        'application/ofd',
        'application/octet-stream',
      ];
      const validExt = ['.pdf', '.jpg', '.jpeg', '.png', '.webp', '.xml', '.ofd'].some((ext) =>
        name.endsWith(ext),
      );
      if (!validTypes.includes(selectedFile.type) && !validExt) {
        toast.error('只支持 PDF、XML、OFD、JPG、PNG 格式的文件');
        return;
      }
      setFile(selectedFile);
    }
  };

  const uploadFile = async (): Promise<string | null> => {
    if (!file) return null;

    try {
      const uploadResult = await files.upload(file);
      return uploadResult.id;
    } catch (error) {
      console.error('文件上传失败:', error);
      throw error;
    }
  };

  const onSubmit = async (data: InvoiceFormData) => {
    parseGeneration.current += 1;
    setLoading(true);

    try {
      let filePath = invoice?.file_path || null;
      if (file) {
        filePath = await uploadFile();
      }

      const invoiceType =
        normalizeInvoiceTypeUi(data.invoice_type) ||
        normalizeInvoiceTypeUi(formOptions?.default_invoice_type) ||
        'receivable';
      const clientId = (data.client_id || '').trim();
      const supplier = (data.supplier || '').trim();
      const invoiceData: Record<string, any> = {
        invoice_number: data.invoice_number,
        supplier: supplier || null,
        amount: data.amount,
        tax_amount: data.tax_amount || null,
        invoice_date: data.invoice_date,
        due_date: data.due_date || null,
        description: data.description || null,
        invoice_type: invoiceType,
        file_path: filePath,
      };
      if (clientId) {
        invoiceData.client_id = clientId;
      } else if (invoice) {
        // Clear client link on edit when user removes selection (empty string, not JSON null).
        invoiceData.client_id = '';
      }

      const action = invoice?.update_action ?? createAction;
      if (!action) {
        toast.error('当前无法保存发票');
        return;
      }

      await api.runAction(action, invoiceData);
      toast.success(action.success_message || '发票已保存');

      onSuccess();
      onOpenChange(false);
    } catch (error: any) {
      console.error('保存发票失败:', error);
      if (error.code === '23505') {
        toast.error('该发票号已存在');
      } else {
        toast.error('保存失败，请重试');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{invoice ? '编辑发票' : '添加发票'}</DialogTitle>
          <DialogDescription>
            填写发票信息，带 * 的为必填项
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="invoice_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>发票类型</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择发票类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {invoiceTypeOptions.map((option) => (
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
                name="invoice_number"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>发票号</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="留空自动生成（如 INV-2026-0001）"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {receivable && (
                <FormField
                  control={form.control}
                  name="client_id"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>客户 *</FormLabel>
                      <Select
                        onValueChange={(value) => {
                          const next = value === '__none__' ? '' : value;
                          field.onChange(next);
                          const option = clientOptions.find((c) => c.value === next);
                          if (option) {
                            form.setValue('supplier', option.name || option.label || '');
                          }
                        }}
                        value={field.value || '__none__'}
                      >
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="选择客户" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="__none__">不关联客户（仅填名称）</SelectItem>
                          {clientOptions.map((option) => (
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
              )}

              <FormField
                control={form.control}
                name="supplier"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      {receivable ? '客户名称' : '供应商名称'}
                      {receivable ? '' : ' *'}
                    </FormLabel>
                    <FormControl>
                      <Input
                        placeholder={
                          receivable
                            ? '选择客户后自动带出，也可手填'
                            : '例如：XX供应商有限公司'
                        }
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>价税合计（元）*</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="tax_amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>税额（元）</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />



              <FormField
                control={form.control}
                name="invoice_date"
                render={({ field }) => (
                  <FormItem className="flex flex-col">
                    <FormLabel>开票日期 *</FormLabel>
                    <Popover>
                      <PopoverTrigger asChild>
                        <FormControl>
                          <Button
                            variant="outline"
                            className={cn(
                              'w-full pl-3 text-left font-normal',
                              !field.value && 'text-muted-foreground'
                            )}
                          >
                            {field.value ? (
                              format(field.value, 'yyyy-MM-dd')
                            ) : (
                              <span>选择日期</span>
                            )}
                            <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                          </Button>
                        </FormControl>
                      </PopoverTrigger>
                      <PopoverContent className="w-auto p-0" align="start">
                        <Calendar
                          mode="single"
                          selected={field.value}
                          onSelect={field.onChange}
                          disabled={(date) => date > new Date()}
                          initialFocus
                          className="pointer-events-auto"
                        />
                      </PopoverContent>
                    </Popover>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="due_date"
                render={({ field }) => (
                  <FormItem className="flex flex-col">
                    <FormLabel>到期日期</FormLabel>
                    <Popover>
                      <PopoverTrigger asChild>
                        <FormControl>
                          <Button
                            variant="outline"
                            className={cn(
                              'w-full pl-3 text-left font-normal',
                              !field.value && 'text-muted-foreground'
                            )}
                          >
                            {field.value ? (
                              format(field.value, 'yyyy-MM-dd')
                            ) : (
                              <span>选择日期</span>
                            )}
                            <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                          </Button>
                        </FormControl>
                      </PopoverTrigger>
                      <PopoverContent className="w-auto p-0" align="start">
                        <Calendar
                          mode="single"
                          selected={field.value}
                          onSelect={field.onChange}
                          initialFocus
                          className="pointer-events-auto"
                        />
                      </PopoverContent>
                    </Popover>
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
                  <FormLabel>描述</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="发票备注信息..."
                      className="resize-none"
                      rows={3}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* 文件上传 */}
            <div className="space-y-2">
              <label className="text-sm font-medium">发票附件</label>
              <div className="flex items-center gap-2">
                <Input
                  type="file"
                  accept=".pdf,.jpg,.jpeg,.png,.webp,.xml,.ofd,image/*,application/pdf,application/xml,text/xml"
                  onChange={handleFileChange}
                  className="hidden"
                  id="invoice-file"
                />
                <label
                  htmlFor="invoice-file"
                  className="flex items-center gap-2 px-4 py-2 border rounded-md cursor-pointer hover:bg-accent"
                >
                  <Upload className="h-4 w-4" />
                  选择文件
                </label>
                {file && (
                  <div className="flex items-center gap-2 flex-1 px-3 py-2 border rounded-md">
                    <span className="text-sm truncate">{file.name}</span>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => setFile(null)}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                )}
                {invoice?.file_path && !file && (
                  <span className="text-sm text-muted-foreground">
                    当前文件：{invoice.file_label || invoice.file_name || '附件'}
                  </span>
                )}
              </div>
              <AutoParseButton
                file={file}
                isParsing={isParsing}
                error={parseError}
                onParse={handleAutoParse}
                enabled={parserEnabled}
                className="mt-2"
              />
              <p className="text-xs text-muted-foreground">
                支持 PDF、JPG、PNG 格式，最大 10MB
              </p>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={loading}
              >
                取消
              </Button>
              <Button type="submit" disabled={loading}>
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {invoice ? '更新' : '添加'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
