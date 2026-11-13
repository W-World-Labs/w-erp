import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { User, Mail, Upload, Loader2 } from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, files, type BackendAction } from "@/integrations/api/client";
import { useState, useRef } from "react";
import { toast } from "@/components/ui/use-toast";
import { z } from "zod";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";

const profileSchema = z.object({
  full_name: z.string().min(1, "姓名不能为空").max(50, "姓名不能超过50个字符"),
  email: z.string().email("请输入有效的邮箱地址"),
});

type ProfileFormValues = z.infer<typeof profileSchema>;

interface ProfileRecord {
  id: string;
  full_name: string;
  display_name?: string | null;
  email: string;
  avatar_url?: string | null;
  created_at_label?: string;
  updated_at_label?: string;
  update_action?: BackendAction | null;
}

const Profile = () => {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // 获取用户资料
  const { data: profile, isLoading } = useQuery({
    queryKey: ['profile', user?.id],
    queryFn: async () => {
      if (!user?.id) throw new Error('未登录');

      return api.get<ProfileRecord>('/api/profiles/current');
    },
    enabled: !!user?.id,
  });

  const form = useForm<ProfileFormValues>({
    resolver: zodResolver(profileSchema),
    values: {
      full_name: profile?.full_name || '',
      email: profile?.email || '',
    },
  });

  // 更新资料
  const updateProfileMutation = useMutation({
    mutationFn: async (values: ProfileFormValues) => {
      if (!profile?.id) throw new Error('未找到个人资料');
      if (!profile.update_action) throw new Error('后端未返回资料保存动作');

      await api.runAction(profile.update_action, {
          display_name: values.full_name,
          email: values.email,
        });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['profile'] });
      toast({
        title: "保存成功",
        description: "个人资料已更新",
      });
    },
    onError: (error) => {
      toast({
        title: "保存失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 上传头像
  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file || !user?.id) return;

    // 验证文件类型
    const allowedTypes = ['image/jpeg', 'image/jpg', 'image/png', 'image/webp'];
    if (!allowedTypes.includes(file.type)) {
      toast({
        title: "文件类型错误",
        description: "仅支持 JPG、PNG 和 WEBP 格式的图片",
        variant: "destructive",
      });
      return;
    }

    // 验证文件大小 (5MB)
    if (file.size > 5 * 1024 * 1024) {
      toast({
        title: "文件过大",
        description: "头像文件大小不能超过 5MB",
        variant: "destructive",
      });
      return;
    }

    setIsUploading(true);

    try {
      if (!profile?.id) throw new Error('未找到个人资料');
      const uploadResult = await files.upload(file);

      // 更新数据库
      if (!profile.update_action) throw new Error('后端未返回头像保存动作');
      await api.runAction(profile.update_action, { avatar_url: uploadResult.url });
      queryClient.invalidateQueries({ queryKey: ['profile'] });
      toast({
        title: "上传成功",
        description: "头像已更新",
      });
    } catch (error: any) {
      toast({
        title: "上传失败",
        description: error.message,
        variant: "destructive",
      });
    } finally {
      setIsUploading(false);
    }
  };

  const onSubmit = (values: ProfileFormValues) => {
    updateProfileMutation.mutate(values);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <h1 className="text-3xl font-bold text-foreground">个人资料</h1>
        <p className="text-muted-foreground mt-2">管理您的个人信息和头像</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>基本信息</CardTitle>
          <CardDescription>更新您的个人资料信息</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* 头像部分 */}
          <div className="flex items-center gap-6">
            <Avatar className="h-24 w-24">
              <AvatarImage src={profile?.avatar_url || undefined} alt={profile?.full_name || '用户'} />
              <AvatarFallback className="text-2xl">
                {profile?.full_name?.charAt(0)?.toUpperCase() || <User className="h-12 w-12" />}
              </AvatarFallback>
            </Avatar>
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                支持 JPG、PNG、WEBP 格式，文件大小不超过 5MB
              </p>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/jpeg,image/jpg,image/png,image/webp"
                onChange={handleAvatarUpload}
                className="hidden"
              />
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => fileInputRef.current?.click()}
                disabled={isUploading}
              >
                {isUploading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    上传中...
                  </>
                ) : (
                  <>
                    <Upload className="mr-2 h-4 w-4" />
                    上传头像
                  </>
                )}
              </Button>
            </div>
          </div>

          {/* 表单部分 */}
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormField
                control={form.control}
                name="full_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>姓名</FormLabel>
                    <FormControl>
                      <div className="relative">
                        <User className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                        <Input {...field} className="pl-10" placeholder="请输入您的姓名" />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>邮箱</FormLabel>
                    <FormControl>
                      <div className="relative">
                        <Mail className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                        <Input {...field} className="pl-10" type="email" placeholder="请输入您的邮箱" />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <div className="pt-4">
                <Button 
                  type="submit" 
                  disabled={updateProfileMutation.isPending}
                >
                  {updateProfileMutation.isPending ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      保存中...
                    </>
                  ) : (
                    '保存更改'
                  )}
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>账户信息</CardTitle>
          <CardDescription>查看您的账户详情</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4">
            <div className="space-y-2">
              <Label className="text-muted-foreground">用户 ID</Label>
              <p className="font-mono text-sm bg-muted p-2 rounded">{user?.id}</p>
            </div>
            <div className="space-y-2">
              <Label className="text-muted-foreground">创建时间</Label>
              <p className="text-sm">
                {profile?.created_at_label || '-'}
              </p>
            </div>
            <div className="space-y-2">
              <Label className="text-muted-foreground">最后更新</Label>
              <p className="text-sm">
                {profile?.updated_at_label || '-'}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Profile;
