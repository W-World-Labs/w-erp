import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Loader2, LogIn } from 'lucide-react';

export default function Auth() {
  const { user, loading, signIn } = useAuth();
  const navigate = useNavigate();

  // 如果已登录，重定向到首页
  useEffect(() => {
    if (user) {
      navigate('/');
    }
  }, [user, navigate]);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-secondary/20 to-background">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-secondary/20 to-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold text-center">OPC</CardTitle>
          <CardDescription className="text-center">
            企业资源管理系统
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            onClick={signIn}
            className="w-full h-12 text-lg"
            size="lg"
          >
            <LogIn className="mr-2 h-5 w-5" />
            使用 SSO 登录
          </Button>

          <p className="text-xs text-center text-muted-foreground">
            支持 GitHub、ORCID 等账号登录
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
