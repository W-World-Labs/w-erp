import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Rocket, FileText, TrendingUp, Clock, CheckCircle2, ArrowRight, DollarSign } from "lucide-react";
import { useNavigate } from "react-router-dom";

interface GrowthStageProps {
  stats: {
    ongoingProjects: number;
    activeContracts: number;
    pendingInvoices: number;
    totalProjects: number;
    totalContracts: number;
  };
}

export const GrowthStage = ({ stats }: GrowthStageProps) => {
  const navigate = useNavigate();

  return (
    <div className="space-y-6">
      {/* 核心指标 - 成长期关注项目和合同 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/projects')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">研发项目</CardTitle>
            <Rocket className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">{stats.ongoingProjects}</div>
            <p className="text-xs text-muted-foreground">
              项目总数: {stats.totalProjects}
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/contracts')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">合同管理</CardTitle>
            <FileText className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-accent">{stats.activeContracts}</div>
            <p className="text-xs text-muted-foreground">
              合同总数: {stats.totalContracts}
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/invoices')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">待处理发票</CardTitle>
            <DollarSign className="h-4 w-4 text-warning" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-warning">{stats.pendingInvoices}</div>
            <p className="text-xs text-muted-foreground">
              需要及时处理
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/budget')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">预算管理</CardTitle>
            <TrendingUp className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">精细化</div>
            <p className="text-xs text-muted-foreground">
              财务管控
            </p>
          </CardContent>
        </Card>
      </div>

      {/* 项目和合同进度 */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-primary" />
              项目进度追踪
            </CardTitle>
            <CardDescription>重点关注进行中的项目</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">进行中项目</span>
                <span className="text-2xl font-bold text-primary">{stats.ongoingProjects}</span>
              </div>
              <Button 
                variant="outline" 
                className="w-full"
                onClick={() => navigate('/projects')}
              >
                查看项目详情 <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <CheckCircle2 className="h-5 w-5 text-accent" />
              合同执行状态
            </CardTitle>
            <CardDescription>监控合同履行情况</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">执行中合同</span>
                <span className="text-2xl font-bold text-accent">{stats.activeContracts}</span>
              </div>
              <Button 
                variant="outline" 
                className="w-full"
                onClick={() => navigate('/contracts')}
              >
                管理合同 <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 快捷操作 */}
      <Card>
        <CardHeader>
          <CardTitle>快捷操作</CardTitle>
          <CardDescription>成长期核心功能</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-4 gap-4">
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/projects')}>
              <Rocket className="h-6 w-6 mb-2 text-primary" />
              <span>创建项目</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/contracts')}>
              <FileText className="h-6 w-6 mb-2 text-accent" />
              <span>签订合同</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/invoices')}>
              <DollarSign className="h-6 w-6 mb-2 text-warning" />
              <span>发票管理</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/budget')}>
              <TrendingUp className="h-6 w-6 mb-2 text-primary" />
              <span>预算控制</span>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
