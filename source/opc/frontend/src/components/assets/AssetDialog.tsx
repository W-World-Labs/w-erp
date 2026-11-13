import { useState, useEffect, useRef } from 'react';
import { useForm } from 'react-hook-form';
import { useQuery } from '@tanstack/react-query';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from 'sonner';
import { format } from 'date-fns';
import { CalendarIcon, Loader2, Calculator, Info } from 'lucide-react';
import { cn } from '@/lib/utils';
import { AttachmentManager } from '@/components/common/AttachmentManager';
import { Separator } from '@/components/ui/separator';

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
  FormDescription,
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
import { Badge } from '@/components/ui/badge';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';

const assetSchema = z.object({
  name: z.string().min(1, '资产名称不能为空').max(100, '名称不能超过100字符'),
  asset_type: z.string().min(1, '请选择资产类型'),
  category_id: z.string().min(1, '请选择资产分类'),
  serial_number: z.string().max(50, '序列号不能超过50字符').optional(),
  purchase_date: z.date().optional(),
  purchase_price: z.string().regex(/^\d*\.?\d*$/, '请输入有效的价格').optional(),
  current_value: z.string().regex(/^\d*\.?\d*$/, '请输入有效的价格').optional(),
  location: z.string().max(100, '位置不能超过100字符').optional(),
  description: z.string().max(500, '描述不能超过500字符').optional(),
  // 虚拟资产 / 无形资产专用字段
  expiry_date: z.date().optional(),
  registration_number: z.string().max(100).optional(),
  holder_name: z.string().max(100).optional(),
  // 折旧字段（固定资产）
  useful_life: z.string().regex(/^\d*$/, '请输入整数').optional(),
  salvage_rate: z.string().regex(/^\d*\.?\d*$/, '请输入有效百分比').optional(),
  depreciation_method: z.string().optional(),
  // 数据资产元数据
  data_type: z.string().optional(),
  tech_stack: z.string().max(200).optional(),
  version: z.string().max(50).optional(),
  license_type: z.string().optional(),
  patent_number: z.string().max(100).optional(),
  patent_type: z.string().optional(),
  trademark_class: z.string().max(100).optional(),
  data_volume: z.string().max(100).optional(),
  data_format: z.string().max(100).optional(),
  access_url: z.string().max(500).optional(),
});

type AssetFormData = z.infer<typeof assetSchema>;

interface Category {
  id: string;
  name: string;
  category_type: string;
}

interface AssetFormOption {
  value: string;
  label: string;
  desc?: string;
}

interface AssetCategoryOption extends AssetFormOption {
  category_type: string;
  category_type_label?: string;
}

interface AssetFormOptions {
  asset_type_options: AssetFormOption[];
  category_options: AssetCategoryOption[];
  data_type_options: AssetFormOption[];
  intangible_type_options: AssetFormOption[];
  depreciation_method_options: AssetFormOption[];
  license_type_options: AssetFormOption[];
  patent_type_options: AssetFormOption[];
  default_values: {
    asset_type: string;
    salvage_rate: string;
    depreciation_method: string;
  };
}

interface Asset {
  id: string;
  name: string;
  category_id: string | null;
  serial_number: string | null;
  purchase_date: string | null;
  purchase_price: number | null;
  current_value: number | null;
  status: string;
  location: string | null;
  description: string | null;
  asset_type?: string;
  expiry_date?: string | null;
  registration_number?: string | null;
  holder_name?: string | null;
  useful_life?: number | null;
  salvage_rate?: number | null;
  depreciation_method?: string | null;
  monthly_depreciation?: number | null;
  accumulated_depreciation?: number | null;
  data_asset_meta?: any | null;
  update_action?: BackendAction | null;
}

interface AssetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset?: Asset | null;
  action?: BackendAction | null;
  onSuccess: () => void;
  defaultSerialNumber?: string;
}

export function AssetDialog({ open, onOpenChange, asset, action: createAction, onSuccess, defaultSerialNumber }: AssetDialogProps) {
  const [loading, setLoading] = useState(false);
  const resetKeyRef = useRef('');

  const form = useForm<AssetFormData>({
    resolver: zodResolver(assetSchema),
    defaultValues: {
      name: '',
      asset_type: '',
      category_id: '',
      serial_number: '',
      purchase_price: '',
      current_value: '',
      location: '',
      description: '',
      expiry_date: undefined,
      registration_number: '',
      holder_name: '',
      useful_life: '',
      salvage_rate: '',
      depreciation_method: '',
      data_type: '',
      tech_stack: '',
      version: '',
      license_type: '',
      patent_number: '',
      patent_type: '',
      trademark_class: '',
      data_volume: '',
      data_format: '',
      access_url: '',
    },
  });

  const assetType = form.watch('asset_type');
  const dataType = form.watch('data_type');
  const { data: formOptions } = useQuery({
    queryKey: ['asset-form-options', assetType],
    queryFn: () => api.get<AssetFormOptions>(`/api/assets/form_options?asset_type=${encodeURIComponent(assetType || '')}`),
    enabled: open,
  });
  const categories: Category[] = (formOptions?.category_options ?? []).map((category) => ({
    id: category.value,
    name: category.label,
    category_type: category.category_type,
  }));
  const assetTypeOptions = formOptions?.asset_type_options ?? [];
  const dataTypeOptions = formOptions?.data_type_options ?? [];
  const intangibleTypeOptions = formOptions?.intangible_type_options ?? [];
  const depreciationMethodOptions = formOptions?.depreciation_method_options ?? [];
  const licenseTypeOptions = formOptions?.license_type_options ?? [];
  const patentTypeOptions = formOptions?.patent_type_options ?? [];

  // 当资产类型改变时，检查当前选择的类别是否仍然有效
  useEffect(() => {
    const currentCategoryId = form.getValues('category_id');
    if (currentCategoryId && categories.length > 0) {
      const currentCategoryAvailable = categories.some(cat => cat.id === currentCategoryId);
      if (!currentCategoryAvailable) {
        form.setValue('category_id', '');
      }
    }
  }, [assetType, categories]);

  useEffect(() => {
    if (!open) {
      resetKeyRef.current = '';
      return;
    }
    if (!formOptions?.default_values) {
      return;
    }
    const resetKey = `${asset?.id ?? 'new'}:${defaultSerialNumber ?? ''}`;
    if (resetKeyRef.current === resetKey) {
      return;
    }
    const defaults = formOptions.default_values;
    if (open) {
      if (asset) {
        form.reset({
          name: asset.name,
          asset_type: asset.asset_type || defaults.asset_type,
          category_id: asset.category_id || '',
          serial_number: asset.serial_number || '',
          purchase_date: asset.purchase_date ? new Date(asset.purchase_date) : undefined,
          purchase_price: asset.purchase_price?.toString() || '',
        current_value: asset.current_value?.toString() || '',
          location: asset.location || '',
          description: asset.description || '',
          expiry_date: asset.expiry_date ? new Date(asset.expiry_date) : undefined,
          registration_number: asset.registration_number || '',
          holder_name: asset.holder_name || '',
          useful_life: asset.useful_life?.toString() || '',
          salvage_rate: asset.salvage_rate != null ? (asset.salvage_rate * 100).toString() : defaults.salvage_rate,
          depreciation_method: asset.depreciation_method || defaults.depreciation_method,
          data_type: '',
          tech_stack: '',
          version: '',
          license_type: '',
          patent_number: '',
          patent_type: '',
          trademark_class: '',
          data_volume: '',
          data_format: '',
          access_url: '',
        });
        if ((asset.asset_type === 'data' || asset.asset_type === 'intangible') && asset.data_asset_meta) {
          const meta = asset.data_asset_meta;
          form.setValue('data_type', meta.data_type || '');
          form.setValue('tech_stack', meta.tech_stack || '');
          form.setValue('version', meta.version || '');
          form.setValue('license_type', meta.license_type || '');
          form.setValue('patent_number', meta.patent_number || '');
          form.setValue('patent_type', meta.patent_type || '');
          form.setValue('trademark_class', meta.trademark_class || '');
          form.setValue('data_volume', meta.data_volume || '');
          form.setValue('data_format', meta.data_format || '');
          form.setValue('access_url', meta.access_url || '');
        }
      } else {
        form.reset({
          name: '',
          asset_type: defaults.asset_type,
          category_id: '',
          serial_number: defaultSerialNumber || '',
          purchase_price: '',
          current_value: '',
          location: '',
          description: '',
          expiry_date: undefined,
          registration_number: '',
          holder_name: '',
          useful_life: '',
          salvage_rate: defaults.salvage_rate,
          depreciation_method: defaults.depreciation_method,
          data_type: '',
          tech_stack: '',
          version: '',
          license_type: '',
          patent_number: '',
          patent_type: '',
          trademark_class: '',
          data_volume: '',
          data_format: '',
          access_url: '',
        });
      }
      resetKeyRef.current = resetKey;
    }
  }, [open, asset, defaultSerialNumber, formOptions?.default_values, form]);

  // 是否显示折旧区域
  const showDepreciation = assetType === 'physical';
  // 是否显示虚拟资产字段
  const showVirtualFields = assetType === 'virtual' || assetType === 'intangible';
  // 是否显示数据资产/无形资产元数据
  const showDataMeta = assetType === 'data' || assetType === 'intangible';

  const onSubmit = async (data: AssetFormData) => {
    setLoading(true);

    try {
      const assetData: Record<string, any> = {
        name: data.name,
        asset_type: data.asset_type,
        category_id: data.category_id || null,
        serial_number: data.serial_number || null,
        purchase_date: data.purchase_date || null,
        purchase_price: data.purchase_price || null,
        current_value: data.current_value || null,
        location: data.location || null,
        description: data.description || null,
        expiry_date: data.expiry_date || null,
        registration_number: data.registration_number || null,
        holder_name: data.holder_name || null,
      };

      // 添加折旧字段
      if (showDepreciation && data.purchase_price && data.useful_life) {
        assetData.useful_life = data.useful_life;
        assetData.salvage_rate = data.salvage_rate || formOptions?.default_values.salvage_rate;
        assetData.depreciation_method = data.depreciation_method || formOptions?.default_values.depreciation_method;
      }

      if (showDataMeta && data.data_type) {
        assetData.data_type = data.data_type;
        assetData.tech_stack = data.tech_stack || null;
        assetData.version = data.version || null;
        assetData.license_type = data.license_type || null;
        assetData.patent_number = data.patent_number || null;
        assetData.patent_type = data.patent_type || null;
        assetData.trademark_class = data.trademark_class || null;
        assetData.data_volume = data.data_volume || null;
        assetData.data_format = data.data_format || null;
        assetData.access_url = data.access_url || null;
      }

      const action = asset?.update_action ?? createAction;
      if (!action) {
        toast.error('当前无法保存资产');
        return;
      }

      await api.runAction(action, assetData);
      toast.success(action.success_message || '资产已保存');

      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error('保存资产失败:', error);
      toast.error('保存失败，请重试');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{asset ? '编辑资产' : '添加资产'}</DialogTitle>
          <DialogDescription>
            填写资产信息，带 * 的为必填项
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>资产名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="例如：MacBook Pro" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="asset_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>资产类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {assetTypeOptions.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>
                            <div className="flex items-center gap-2">
                              <span>{opt.label}</span>
                              <span className="text-xs text-muted-foreground">({opt.desc})</span>
                            </div>
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
                name="category_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>资产分类 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择分类" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {categories.map((category) => (
                          <SelectItem key={category.id} value={category.id}>
                            {category.name}
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
                name="serial_number"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>序列号/资产编号</FormLabel>
                    <FormControl>
                      <Input placeholder="例如：AST-2024-001" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="purchase_date"
                render={({ field }) => (
                  <FormItem className="flex flex-col">
                    <FormLabel>购买日期</FormLabel>
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
                          autoFocus
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
                name="purchase_price"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>购买价格（元）</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="current_value"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>当前价值（元）</FormLabel>
                    <FormControl>
                      <Input type="number" step="0.01" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="location"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      {assetType === 'physical' ? '存放位置' :
                        assetType === 'data' ? '部署位置' : '持有地'}
                    </FormLabel>
                    <FormControl>
                      <Input
                        placeholder={
                          assetType === 'physical' ? '例如：办公室A栋3楼' :
                            assetType === 'data' ? '例如：阿里云/AWS/自建服务器' :
                              '例如：线上平台'
                        }
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            {/* ===== 折旧配置区（仅固定资产） ===== */}
            {showDepreciation && (
              <>
                <Separator />
                <div className="space-y-3">
                  <div className="flex items-center gap-2">
                    <Calculator className="h-4 w-4 text-primary" />
                    <h3 className="font-semibold text-sm">折旧配置</h3>
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="h-3.5 w-3.5 text-muted-foreground cursor-help" />
                        </TooltipTrigger>
                        <TooltipContent className="max-w-xs">
                          <p>设置折旧参数后，系统将自动计算月折旧额。固定资产净值 = 原值 - 累计折旧</p>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <FormField
                      control={form.control}
                      name="useful_life"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>使用年限（月）</FormLabel>
                          <FormControl>
                            <Input type="number" min="1" placeholder="60" {...field} />
                          </FormControl>
                          <FormDescription className="text-xs">
                            一般电子设备36月，办公设备60月
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control}
                      name="salvage_rate"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>残值率（%）</FormLabel>
                          <FormControl>
                            <Input type="number" step="0.1" min="0" max="100" placeholder="5" {...field} />
                          </FormControl>
                          <FormDescription className="text-xs">
                            通常为 3%~5%
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control}
                      name="depreciation_method"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>折旧方法</FormLabel>
                          <Select onValueChange={field.onChange} value={field.value}>
                            <FormControl>
                              <SelectTrigger>
                                <SelectValue placeholder="选择方法" />
                              </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                              {depreciationMethodOptions.map((opt) => (
                                <SelectItem key={opt.value} value={opt.value}>
                                  {opt.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  </div>
                </div>
              </>
            )}

            {/* ===== 虚拟/无形资产专用字段 ===== */}
            {showVirtualFields && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <FormField
                  control={form.control}
                  name="expiry_date"
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
                            autoFocus
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
                  name="registration_number"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>注册号/证书号</FormLabel>
                      <FormControl>
                        <Input placeholder="例如：专利号、域名注册号" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="holder_name"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>持有人姓名</FormLabel>
                      <FormControl>
                        <Input placeholder="例如：张三" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            )}

            {/* ===== 数据资产/无形资产 元数据 ===== */}
            {showDataMeta && (
              <>
                <Separator />
                <div className="space-y-3">
                  <h3 className="font-semibold text-sm flex items-center gap-2">
                    {assetType === 'data' ? '📊 数据资产详情' : '📜 无形资产详情'}
                  </h3>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <FormField
                      control={form.control}
                      name="data_type"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>子类型 *</FormLabel>
                          <Select onValueChange={field.onChange} value={field.value}>
                            <FormControl>
                              <SelectTrigger>
                                <SelectValue placeholder="选择子类型" />
                              </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                              {(assetType === 'data' ? dataTypeOptions : intangibleTypeOptions).map((opt) => (
                                <SelectItem key={opt.value} value={opt.value}>
                                  {opt.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    {/* 软件类字段 */}
                    {dataType === 'software' && (
                      <>
                        <FormField
                          control={form.control}
                          name="tech_stack"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>技术栈</FormLabel>
                              <FormControl>
                                <Input placeholder="例如：React, Rust, Python" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name="version"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>版本号</FormLabel>
                              <FormControl>
                                <Input placeholder="例如：v2.1.0" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name="license_type"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>许可类型</FormLabel>
                              <Select onValueChange={field.onChange} value={field.value}>
                                <FormControl>
                                  <SelectTrigger>
                                    <SelectValue placeholder="选择许可类型" />
                                  </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                  {licenseTypeOptions.map((opt) => (
                                    <SelectItem key={opt.value} value={opt.value}>
                                      {opt.label}
                                    </SelectItem>
                                  ))}
                                </SelectContent>
                              </Select>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </>
                    )}

                    {/* 专利类字段 */}
                    {dataType === 'patent' && (
                      <>
                        <FormField
                          control={form.control}
                          name="patent_number"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>专利号</FormLabel>
                              <FormControl>
                                <Input placeholder="例如：CN202412345678.9" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name="patent_type"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>专利类型</FormLabel>
                              <Select onValueChange={field.onChange} value={field.value}>
                                <FormControl>
                                  <SelectTrigger>
                                    <SelectValue placeholder="选择类型" />
                                  </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                  {patentTypeOptions.map((opt) => (
                                    <SelectItem key={opt.value} value={opt.value}>
                                      {opt.label}
                                    </SelectItem>
                                  ))}
                                </SelectContent>
                              </Select>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </>
                    )}

                    {/* 商标类字段 */}
                    {dataType === 'trademark' && (
                      <FormField
                        control={form.control}
                        name="trademark_class"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>商标分类</FormLabel>
                            <FormControl>
                              <Input placeholder="例如：第9类（科学仪器）" {...field} />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    )}

                    {/* 数据集类字段 */}
                    {dataType === 'dataset' && (
                      <>
                        <FormField
                          control={form.control}
                          name="data_volume"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>数据量</FormLabel>
                              <FormControl>
                                <Input placeholder="例如：500万条 / 2TB" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name="data_format"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>数据格式</FormLabel>
                              <FormControl>
                                <Input placeholder="例如：JSON, CSV, Parquet" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </>
                    )}

                    {/* 通用字段：访问地址 */}
                    {dataType && (
                      <FormField
                        control={form.control}
                        name="access_url"
                        render={({ field }) => (
                          <FormItem className="md:col-span-2">
                            <FormLabel>访问地址 / 仓库链接</FormLabel>
                            <FormControl>
                              <Input placeholder="例如：https://github.com/org/repo" {...field} />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    )}
                  </div>
                </div>
              </>
            )}

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>描述</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="资产详细描述、备注等..."
                      className="resize-none"
                      rows={3}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Separator />

            <div className="space-y-2">
              <h3 className="font-semibold text-sm">附件</h3>
              <AttachmentManager
                entityId={asset?.id}
                entityType="asset"
                disabled={loading}
              />
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
                {asset ? '更新' : '添加'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
