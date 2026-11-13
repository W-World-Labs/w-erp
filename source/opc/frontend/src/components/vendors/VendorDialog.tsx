import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useQuery } from '@tanstack/react-query';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useToast } from '@/hooks/use-toast';
import { useDocumentParser } from '@/hooks/useDocumentParser';
import { AutoParseButton } from '@/components/common/AutoParseButton';
import { api } from '@/integrations/api/client';

const vendorSchema = z.object({
    name: z.string().min(1, '友商名称不能为空').max(200, '名称不能超过200字符'),
    short_name: z.string().max(50, '简称不能超过50字符').optional().or(z.literal('')),
    unified_code: z.string().max(50, '信用代码不能超过50字符').optional().or(z.literal('')),
    legal_person: z.string().max(50).optional().or(z.literal('')),
    registered_capital: z.string().max(50).optional().or(z.literal('')),
    establishment_date: z.string().optional().or(z.literal('')),
    business_scope: z.string().max(1000).optional().or(z.literal('')),
    address: z.string().max(500).optional().or(z.literal('')),
    contact_person: z.string().max(50).optional().or(z.literal('')),
    contact_phone: z.string().max(30).optional().or(z.literal('')),
    contact_email: z.string().email('邮箱格式不正确').optional().or(z.literal('')),
    bank_name: z.string().max(100).optional().or(z.literal('')),
    bank_account: z.string().max(50).optional().or(z.literal('')),
    vendor_type: z.string().min(1, '请选择友商类型'),
    status: z.string().min(1, '请选择状态'),
    notes: z.string().max(1000).optional().or(z.literal('')),
});

type VendorFormValues = z.infer<typeof vendorSchema>;

interface VendorFormOption {
    value: string;
    label: string;
}

interface VendorFormOptions {
    vendor_type_options: VendorFormOption[];
    status_options: VendorFormOption[];
    default_values: {
        vendor_type: string;
        status: string;
    };
}

interface VendorDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    vendor?: any;
    onSubmit: (values: VendorFormValues) => void;
}

export function VendorDialog({ open, onOpenChange, vendor, onSubmit }: VendorDialogProps) {
    const { toast } = useToast();
    const [licenseFile, setLicenseFile] = useState<File | null>(null);
    const { parseFile, isParsing, error: parseError, enabled: parserEnabled } = useDocumentParser('vendor');

    const form = useForm<VendorFormValues>({
        resolver: zodResolver(vendorSchema),
        defaultValues: {
            name: '',
            short_name: '',
            unified_code: '',
            legal_person: '',
            registered_capital: '',
            establishment_date: '',
            business_scope: '',
            address: '',
            contact_person: '',
            contact_phone: '',
            contact_email: '',
            bank_name: '',
            bank_account: '',
            vendor_type: '',
            status: '',
            notes: '',
        },
    });

    const { data: formOptions } = useQuery({
        queryKey: ['vendor-form-options'],
        queryFn: () => api.get<VendorFormOptions>('/api/v1/vendors/form_options'),
        enabled: open,
    });
    const vendorTypeOptions = formOptions?.vendor_type_options ?? [];
    const statusOptions = formOptions?.status_options ?? [];

    useEffect(() => {
        if (!open || !formOptions?.default_values) {
            return;
        }
        const defaults = formOptions.default_values;
        if (vendor) {
            form.reset({
                name: vendor.name || '',
                short_name: vendor.short_name || '',
                unified_code: vendor.unified_code || '',
                legal_person: vendor.legal_person || '',
                registered_capital: vendor.registered_capital || '',
                establishment_date: vendor.establishment_date || '',
                business_scope: vendor.business_scope || '',
                address: vendor.address || '',
                contact_person: vendor.contact_person || '',
                contact_phone: vendor.contact_phone || '',
                contact_email: vendor.contact_email || '',
                bank_name: vendor.bank_name || '',
                bank_account: vendor.bank_account || '',
                vendor_type: vendor.vendor_type || defaults.vendor_type,
                status: vendor.status || defaults.status,
                notes: vendor.notes || '',
            });
        } else {
            form.reset({
                name: '',
                short_name: '',
                unified_code: '',
                legal_person: '',
                registered_capital: '',
                establishment_date: '',
                business_scope: '',
                address: '',
                contact_person: '',
                contact_phone: '',
                contact_email: '',
                bank_name: '',
                bank_account: '',
                vendor_type: defaults.vendor_type,
                status: defaults.status,
                notes: '',
            });
        }
        setLicenseFile(null);
    }, [vendor, open, formOptions?.default_values]);

    const handleLicenseFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file) {
            if (file.size > 10 * 1024 * 1024) {
                toast({ title: '文件过大', description: '文件大小不能超过10MB', variant: 'destructive' });
                return;
            }
            setLicenseFile(file);
        }
    };

    // All fillable form fields - AI returns whatever it can find
    const fillableFields: (keyof VendorFormValues)[] = [
        'name', 'short_name', 'unified_code', 'legal_person',
        'registered_capital', 'establishment_date', 'business_scope',
        'address', 'contact_person', 'contact_phone', 'contact_email',
        'bank_name', 'bank_account',
    ];

    const handleAutoParse = async () => {
    if (!licenseFile) return;
    const fields = await parseFile(licenseFile);
    if (fields) {
            let filled = 0;
            for (const [key, value] of Object.entries(fields)) {
                if (value && fillableFields.includes(key as keyof VendorFormValues)) {
                    form.setValue(key as keyof VendorFormValues, value as string);
                    filled++;
                }
            }

            toast({
                title: 'AI 识别完成',
                description: filled > 0
                    ? `已自动填入 ${filled} 个字段，请核对后提交`
                    : '未能从文档中提取到信息，请尝试其他文件',
            });
        }
    };

    const handleFormSubmit = (values: VendorFormValues) => {
        onSubmit(values);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>{vendor ? '编辑友商' : '新增友商'}</DialogTitle>
                    <DialogDescription>
                        {vendor ? '修改友商资料信息' : '添加新的供应商或客户资料，支持上传企业文件AI自动识别'}
                    </DialogDescription>
                </DialogHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(handleFormSubmit)} className="space-y-4">
                        <Tabs defaultValue="basic" className="w-full">
                            <TabsList className="grid w-full grid-cols-3">
                                <TabsTrigger value="basic">基本信息</TabsTrigger>
                                <TabsTrigger value="contact">联系信息</TabsTrigger>
                                <TabsTrigger value="license">智能识别</TabsTrigger>
                            </TabsList>

                            <TabsContent value="basic" className="space-y-4 mt-4">
                                <div className="grid grid-cols-2 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="name"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>友商名称 *</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：杭州科技有限公司" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="short_name"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>简称</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：杭州科技" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="vendor_type"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>友商类型 *</FormLabel>
                                                <Select onValueChange={field.onChange} value={field.value}>
                                                    <FormControl>
                                                        <SelectTrigger>
                                                            <SelectValue placeholder="选择类型" />
                                                        </SelectTrigger>
                                                    </FormControl>
                                                    <SelectContent>
                                                        {vendorTypeOptions.map((option) => (
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
                                        name="status"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>状态 *</FormLabel>
                                                <Select onValueChange={field.onChange} value={field.value}>
                                                    <FormControl>
                                                        <SelectTrigger>
                                                            <SelectValue placeholder="选择状态" />
                                                        </SelectTrigger>
                                                    </FormControl>
                                                    <SelectContent>
                                                        {statusOptions.map((option) => (
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
                                        name="unified_code"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>统一社会信用代码</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：91330100MA2XXXXXX" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="legal_person"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>法定代表人</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：张三" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="registered_capital"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>注册资本</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：500万元" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="establishment_date"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>成立日期</FormLabel>
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
                                    name="address"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>注册地址</FormLabel>
                                            <FormControl>
                                                <Input placeholder="例如：浙江省杭州市..." {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />

                                <FormField
                                    control={form.control}
                                    name="business_scope"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>经营范围</FormLabel>
                                            <FormControl>
                                                <Textarea placeholder="经营范围描述..." rows={3} {...field} />
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
                                                <Textarea placeholder="备注信息..." rows={2} {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                            </TabsContent>

                            <TabsContent value="contact" className="space-y-4 mt-4">
                                <div className="grid grid-cols-2 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="contact_person"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>联系人</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：李经理" {...field} />
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
                                                    <Input placeholder="例如：00000138000" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                <FormField
                                    control={form.control}
                                    name="contact_email"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>联系邮箱</FormLabel>
                                            <FormControl>
                                                <Input type="email" placeholder="例如：contact@example.com" {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />

                                <div className="grid grid-cols-2 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="bank_name"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>开户银行</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="例如：中国银行杭州分行" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="bank_account"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>银行账号</FormLabel>
                                                <FormControl>
                                                    <Input placeholder="银行账号" {...field} />
                                                </FormControl>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>
                            </TabsContent>

                            <TabsContent value="license" className="space-y-4 mt-4">
                                <div className="border-2 border-dashed border-muted-foreground/25 rounded-lg p-6 text-center space-y-4">
                                    <div className="text-muted-foreground">
                                        <p className="font-medium">上传企业文件</p>
                                        <p className="text-sm mt-1">营业执照、开户许可证、合同、税务登记证等，支持 JPG、PNG、PDF 格式</p>
                                    </div>

                                    <Input
                                        type="file"
                                        accept="image/*,.pdf"
                                        onChange={handleLicenseFileChange}
                                        className="max-w-xs mx-auto"
                                    />

                                    {licenseFile && (
                                        <div className="space-y-2">
                                            <p className="text-sm text-muted-foreground">
                                                已选择: {licenseFile.name}
                                            </p>
                                            <AutoParseButton
                                                file={licenseFile}
                                                isParsing={isParsing}
                                                enabled={parserEnabled}
                                                error={parseError}
                                                onParse={handleAutoParse}
                                                className="justify-center"
                                            />
                                        </div>
                                    )}
                                </div>

                                <div className="bg-muted/50 rounded-lg p-4">
                                    <p className="text-sm font-medium mb-2">AI 智能识别说明</p>
                                    <ul className="text-xs text-muted-foreground space-y-1">
                                        <li>• 上传任意企业文件（营业执照、开户许可证、合同、税务登记证等）</li>
                                        <li>• 点击「✨ AI 智能识别」，AI 将自动从文件中提取所有能识别的字段</li>
                                        <li>• 可多次上传不同文件，逐步补全信息（如先传营业执照，再传开户许可证）</li>
                                        <li>• 识别结果将自动填入对应字段，请核对后提交</li>
                                        <li>• 需要本地运行 Ollama 并加载视觉识别模型</li>
                                    </ul>
                                </div>
                            </TabsContent>
                        </Tabs>

                        <div className="flex justify-end gap-2 pt-4 border-t">
                            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                                取消
                            </Button>
                            <Button type="submit">
                                {vendor ? '保存修改' : '创建友商'}
                            </Button>
                        </div>
                    </form>
                </Form>
            </DialogContent>
        </Dialog>
    );
}
