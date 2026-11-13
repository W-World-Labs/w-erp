import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useQuery } from '@tanstack/react-query';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from 'sonner';
import { Loader2 } from 'lucide-react';

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
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

const categorySchema = z.object({
  name: z.string().min(1, '分类名称不能为空').max(50, '名称不能超过50字符'),
  description: z.string().max(200, '描述不能超过200字符').optional(),
  category_type: z.string().min(1, '请选择类别类型'),
});

type CategoryFormData = z.infer<typeof categorySchema>;

interface Category {
  id: string;
  name: string;
  description: string | null;
  category_type: string;
  update_action?: BackendAction | null;
}

interface CategoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  category?: Category | null;
  action?: BackendAction | null;
  onSuccess: () => void;
}

interface CategoryFormOption {
  value: string;
  label: string;
}

interface CategoryFormOptions {
  category_type_options: CategoryFormOption[];
  default_values: {
    category_type: string;
  };
}

export function CategoryDialog({ open, onOpenChange, category, action: createAction, onSuccess }: CategoryDialogProps) {
  const [loading, setLoading] = useState(false);

  const form = useForm<CategoryFormData>({
    resolver: zodResolver(categorySchema),
    defaultValues: {
      name: '',
      description: '',
      category_type: '',
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ['asset-category-form-options'],
    queryFn: () => api.get<CategoryFormOptions>('/api/categories/form_options'),
    enabled: open,
  });
  const categoryTypeOptions = formOptions?.category_type_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (category) {
      form.reset({
        name: category.name,
        description: category.description || '',
        category_type: category.category_type || defaults.category_type,
      });
    } else {
      form.reset({
        name: '',
        description: '',
        category_type: defaults.category_type,
      });
    }
  }, [open, category, formOptions?.default_values, form]);

  const onSubmit = async (data: CategoryFormData) => {
    setLoading(true);

    try {
      const categoryData = {
        name: data.name,
        description: data.description || null,
        category_type: data.category_type,
      };

      const action = category?.update_action ?? createAction;
      if (!action) {
        toast.error('当前无法保存分类');
        return;
      }

      await api.runAction(action, categoryData);
      toast.success(action.success_message || '分类已保存');

      onSuccess();
      onOpenChange(false);
    } catch (error: any) {
      console.error('保存分类失败:', error);
      if (error.code === '23505') {
        toast.error('该分类名称已存在');
      } else {
        toast.error('保存失败，请重试');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{category ? '编辑分类' : '添加分类'}</DialogTitle>
          <DialogDescription>
            管理资产分类信息
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>分类名称 *</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：电子设备" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="category_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>类别类型 *</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="选择类别类型" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {categoryTypeOptions.map((option) => (
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
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>描述</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="分类描述..."
                      className="resize-none"
                      rows={3}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

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
                {category ? '更新' : '添加'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
