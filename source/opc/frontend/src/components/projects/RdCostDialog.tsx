import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from 'sonner';
import { format } from 'date-fns';
import { CalendarIcon, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

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

const rdCostSchema = z.object({
    cost_type: z.string().min(1, '请选择费用类型'),
    amount: z.string().min(1, '请输入金额').regex(/^\d+\.?\d*$/, '请输入有效金额'),
    cost_date: z.date().optional(),
    description: z.string().max(500).optional(),
    employee_id: z.string().optional(),
    asset_id: z.string().optional(),
    invoice_id: z.string().optional(),
});

type RdCostFormData = z.infer<typeof rdCostSchema>;

interface RdCostOption {
    value: string;
    label: string;
    description?: string;
}

interface RdCostDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    projectId: string;
    cost?: any | null;
    action?: BackendAction | null;
    onSuccess: () => void;
    rdCostTypeOptions: RdCostOption[];
    employeeOptions: RdCostOption[];
    assetOptions: RdCostOption[];
}

export function RdCostDialog({
    open,
    onOpenChange,
    projectId,
    cost,
    action: createAction,
    onSuccess,
    rdCostTypeOptions,
    employeeOptions,
    assetOptions,
}: RdCostDialogProps) {
    const [loading, setLoading] = useState(false);

    const form = useForm<RdCostFormData>({
        resolver: zodResolver(rdCostSchema),
        defaultValues: {
            cost_type: '',
            amount: '',
            cost_date: undefined,
            description: '',
            employee_id: '',
            asset_id: '',
            invoice_id: '',
        },
    });

    const costType = form.watch('cost_type');

    useEffect(() => {
        if (open) {
            if (cost) {
                form.reset({
                    cost_type: cost.cost_type || '',
                    amount: cost.amount?.toString() || '',
                    cost_date: cost.cost_date ? new Date(cost.cost_date) : undefined,
                    description: cost.description || '',
                    employee_id: cost.employee_id || '',
                    asset_id: cost.asset_id || '',
                    invoice_id: cost.invoice_id || '',
                });
            } else {
                form.reset({
                    cost_type: '',
                    amount: '',
                    cost_date: undefined,
                    description: '',
                    employee_id: '',
                    asset_id: '',
                    invoice_id: '',
                });
            }
        }
    }, [open, cost]);

    const onSubmit = async (data: RdCostFormData) => {
        setLoading(true);
        try {
            const costData = {
                project_id: projectId,
                cost_type: data.cost_type,
                amount: data.amount,
                cost_date: data.cost_date || undefined,
                description: data.description || null,
                employee_id: data.employee_id || null,
                asset_id: data.asset_id || null,
                invoice_id: data.invoice_id || null,
            };

            const action = cost?.update_action ?? createAction;
            if (!action) {
                toast.error('当前无法保存研发费用');
                return;
            }

            await api.runAction(action, costData);
            toast.success(action.success_message || '研发费用已保存');

            onSuccess();
            onOpenChange(false);
        } catch (error) {
            console.error('保存研发费用失败:', error);
            toast.error('保存失败，请重试');
        } finally {
            setLoading(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>{cost ? '编辑研发费用' : '添加研发费用'}</DialogTitle>
                    <DialogDescription>录入项目研发费用明细</DialogDescription>
                </DialogHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                        <FormField
                            control={form.control}
                            name="cost_type"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>费用类型 *</FormLabel>
                                    <Select onValueChange={field.onChange} value={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择费用类型" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                            {rdCostTypeOptions.map((opt) => (
                                                <SelectItem key={opt.value} value={opt.value}>
                                                    <div className="flex flex-col">
                                                        <span>{opt.label}</span>
                                                        <span className="text-xs text-muted-foreground">{opt.description}</span>
                                                    </div>
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="grid grid-cols-2 gap-4">
                            <FormField
                                control={form.control}
                                name="amount"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>金额（元） *</FormLabel>
                                        <FormControl>
                                            <Input type="number" step="0.01" placeholder="0.00" {...field} />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />

                            <FormField
                                control={form.control}
                                name="cost_date"
                                render={({ field }) => (
                                    <FormItem className="flex flex-col">
                                        <FormLabel>日期</FormLabel>
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
                                                        {field.value ? format(field.value, 'yyyy-MM-dd') : <span>选择日期</span>}
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

                        {/* 人员费用 → 选择员工 */}
                        {costType === 'personnel' && employeeOptions.length > 0 && (
                            <FormField
                                control={form.control}
                                name="employee_id"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>关联员工</FormLabel>
                                        <Select onValueChange={field.onChange} value={field.value}>
                                            <FormControl>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="选择员工（可选）" />
                                                </SelectTrigger>
                                            </FormControl>
                                            <SelectContent>
                                                {employeeOptions.map((option) => (
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

                        {/* 设备/折旧费用 → 选择资产 */}
                        {(costType === 'equipment' || costType === 'depreciation') && assetOptions.length > 0 && (
                            <FormField
                                control={form.control}
                                name="asset_id"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>关联设备</FormLabel>
                                        <Select onValueChange={field.onChange} value={field.value}>
                                            <FormControl>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="选择设备（可选）" />
                                                </SelectTrigger>
                                            </FormControl>
                                            <SelectContent>
                                                {assetOptions.map((option) => (
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
                            name="description"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>说明</FormLabel>
                                    <FormControl>
                                        <Textarea
                                            placeholder="费用说明、备注..."
                                            className="resize-none"
                                            rows={2}
                                            {...field}
                                        />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => onOpenChange(false)} disabled={loading}>
                                取消
                            </Button>
                            <Button type="submit" disabled={loading}>
                                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                {cost ? '更新' : '添加'}
                            </Button>
                        </DialogFooter>
                    </form>
                </Form>
            </DialogContent>
        </Dialog>
    );
}
