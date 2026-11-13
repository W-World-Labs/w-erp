import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { FileCheck, Users, Banknote, Package, AlertCircle, ArrowRight } from "lucide-react";
import { useNavigate } from "react-router-dom";

interface StartupStageProps {
  stats: {
    totalAssets: number;
    totalAssetsLabel: string;
    employeeCount: number;
    employeeCountLabel: string;
    pendingInvoices: number;
    totalSalary: number;
    totalSalaryLabel: string;
  };
}

export const StartupStage = ({ stats }: StartupStageProps) => {
  const navigate = useNavigate();

  return (
    <div className="space-y-6">
      {/* 核心指标 - 创业期关注基础建设 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/licenses')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">证照管理</CardTitle>
            <FileCheck className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">证照台账</div>
            <p className="text-xs text-muted-foreground">
              查看已登记证照与有效期
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/employees')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">团队成员</CardTitle>
            <Users className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.employeeCountLabel}</div>
            <p className="text-xs text-muted-foreground">
              月薪支出: {stats.totalSalaryLabel}
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/assets')}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">企业资产</CardTitle>
            <Package className="h-4 w-4 text-warning" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalAssetsLabel}</div>
            <p className="text-xs text-muted-foreground">
              开始登记企业资产
            </p>
          </CardContent>
        </Card>
      </div>

      {/* 创业指引 */}
      <Card className="border-primary/20 bg-gradient-to-r from-primary/5 to-accent/5">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5 text-primary" />
            创业准备与经营入口
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-background/60 rounded-lg">
              <div>
                <p className="font-medium">登记营业执照</p>
                <p className="text-sm text-muted-foreground">记录证照与到期时间，具体办理要求需核实</p>
              </div>
              <Button size="sm" onClick={() => navigate('/licenses')}>
                登记证照
              </Button>
            </div>
            <div className="flex items-center justify-between p-3 bg-background/60 rounded-lg">
              <div>
                <p className="font-medium">按需管理团队</p>
                <p className="text-sm text-muted-foreground">录入员工信息</p>
              </div>
              <Button size="sm" variant="outline" onClick={() => navigate('/employees')}>
                添加员工
              </Button>
            </div>
            <div className="flex items-center justify-between p-3 bg-background/60 rounded-lg">
              <div>
                <p className="font-medium">建立收支台账</p>
                <p className="text-sm text-muted-foreground">先记录发票与回款，掌握应收应付</p>
              </div>
              <Button size="sm" variant="outline" onClick={() => navigate('/invoices')}>
                记录发票
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 快捷操作 */}
      <Card>
        <CardHeader>
          <CardTitle>快捷操作</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-4 gap-4">
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/licenses')}>
              <FileCheck className="h-6 w-6 mb-2 text-primary" />
              <span>证照登记</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/employees')}>
              <Users className="h-6 w-6 mb-2 text-accent" />
              <span>员工管理</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/salary')}>
              <Banknote className="h-6 w-6 mb-2 text-primary" />
              <span>工资录入</span>
            </Button>
            <Button variant="outline" className="h-auto flex-col py-4" onClick={() => navigate('/assets')}>
              <Package className="h-6 w-6 mb-2 text-accent" />
              <span>资产登记</span>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
