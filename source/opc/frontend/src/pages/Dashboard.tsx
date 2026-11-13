import { Building2, Upload } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useState, useEffect } from "react";
import { StageSelector, EnterpriseStage } from "@/components/dashboard/StageSelector";
import { StartupStage } from "@/components/dashboard/StartupStage";
import { GrowthStage } from "@/components/dashboard/GrowthStage";
import { MatureStage } from "@/components/dashboard/MatureStage";
import { ImportDataDialog } from "@/components/dashboard/ImportDataDialog";
import { OnboardingTutorial } from "@/components/tutorial/OnboardingTutorial";
import { CompletionOverview } from "@/components/dashboard/CompletionOverview";
import { ApplicationReminders } from "@/components/dashboard/ApplicationReminders";
import { AssetExpiryReminders } from "@/components/dashboard/AssetExpiryReminders";
import { SmartWorkbench } from "@/components/dashboard/SmartWorkbench";
import { TodayFocus } from "@/components/dashboard/TodayFocus";
import { useSmartWorkbench } from "@/hooks/useSmartWorkbench";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Sparkles, LayoutDashboard } from "lucide-react";
import { QueryStateGate } from "@/components/common/QueryStateGate";

const Dashboard = () => {
  // 企业阶段状态管理
  const [selectedStage, setSelectedStage] = useState<EnterpriseStage>(() => {
    const saved = localStorage.getItem('enterprise-stage');
    return (saved as EnterpriseStage) || 'startup';
  });
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [viewMode, setViewMode] = useState<'smart' | 'classic'>(() => {
    const saved = localStorage.getItem('dashboard-view-mode');
    return (saved as 'smart' | 'classic') || 'smart';
  });

  useEffect(() => {
    localStorage.setItem('enterprise-stage', selectedStage);
  }, [selectedStage]);

  useEffect(() => {
    localStorage.setItem('dashboard-view-mode', viewMode);
  }, [viewMode]);

  // 智能工作台数据
  const {
    urgentItems,
    attentionItems,
    survivalItems,
    survivalTitle,
    survivalSubtitle,
    survivalCountLabel,
    guidance,
    operatingControl,
    stats,
    greeting,
    generatedDateLabel,
    todayTasks,
    headerBadges,
    todos,
    aiSuggestions,
    completionRate,
    stageStats,
    monthlyExpenseTrend,
    overviewCards,
    quickActions,
    isLoading,
    isError,
    refetch,
  } = useSmartWorkbench();

  const loadingFallback = (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">企业辅助管理系统</h1>
        <p className="text-muted-foreground mt-2">从创业到运营，全方位管理助手</p>
      </div>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[1, 2, 3, 4, 5, 6].map((i) => (
          <Card key={i}>
            <CardHeader className="pb-2">
              <Skeleton className="h-4 w-24" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-8 w-32 mb-2" />
              <Skeleton className="h-3 w-20" />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );


  // 根据阶段渲染不同内容
  const renderStageContent = () => {
    switch (selectedStage) {
      case 'startup':
        return (
          <StartupStage stats={stageStats} />
        );
      case 'growth':
        return (
          <GrowthStage
            stats={{
              ongoingProjects: stageStats.ongoingProjects,
              activeContracts: stageStats.activeContracts,
              pendingInvoices: stageStats.pendingInvoices,
              totalProjects: stageStats.totalProjects,
              totalContracts: stageStats.totalContracts,
            }}
          />
        );
      case 'mature':
        return (
          <MatureStage
            stats={stageStats}
            monthlyExpenseTrend={monthlyExpenseTrend}
          />
        );
      default:
        return null;
    }
  };

  return (
    <QueryStateGate
      isLoading={isLoading}
      isError={isError}
      refetch={refetch}
      loadingFallback={loadingFallback}
    >
      <div className="space-y-6">
        {/* 视图切换 */}
        <div className="flex items-center justify-between">
          <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as 'smart' | 'classic')}>
            <TabsList className="grid w-[280px] grid-cols-2">
              <TabsTrigger value="smart" className="flex items-center gap-2">
                <Sparkles className="h-4 w-4" />
                智能工作台
              </TabsTrigger>
              <TabsTrigger value="classic" className="flex items-center gap-2">
                <LayoutDashboard className="h-4 w-4" />
                经典视图
              </TabsTrigger>
            </TabsList>
          </Tabs>
          <Button
            variant="outline"
            onClick={() => setImportDialogOpen(true)}
            className="flex items-center gap-2"
          >
            <Upload className="h-4 w-4" />
            批量导入
          </Button>
        </div>

        {viewMode === 'smart' ? (
          <>
            {/* 智能工作台视图 */}
            <SmartWorkbench
              urgentItems={attentionItems}
              totalUrgentItems={urgentItems.length}
              survivalItems={survivalItems}
              survivalTitle={survivalTitle}
              survivalSubtitle={survivalSubtitle}
              survivalCountLabel={survivalCountLabel}
              guidance={guidance}
              stats={stats}
              operatingControl={operatingControl}
              greeting={greeting}
              generatedDateLabel={generatedDateLabel}
              todayTasks={todayTasks}
              headerBadges={headerBadges}
              overviewCards={overviewCards}
              quickActions={quickActions}
            />

            {/* 今日焦点 */}
            <TodayFocus
              todos={todos}
              aiSuggestions={aiSuggestions}
              completionRate={completionRate}
            />
          </>
        ) : (
          <>
            {/* 经典视图 */}
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-3xl font-bold text-foreground flex items-center gap-2">
                  <Building2 className="h-8 w-8 text-primary" />
                  企业辅助管理系统
                </h1>
                <p className="text-muted-foreground mt-2">从创业到运营，全方位管理助手 · 让企业管理更简单</p>
              </div>
            </div>

            {/* 阶段选择器 */}
            <StageSelector
              selectedStage={selectedStage}
              onStageChange={setSelectedStage}
            />

            {/* 申报进度提醒 */}
            <ApplicationReminders />

            {/* 虚拟资产到期提醒 */}
            <AssetExpiryReminders />

            {/* 完成度概览 */}
            <CompletionOverview />

            {/* 根据阶段显示不同内容 */}
            {renderStageContent()}
          </>
        )}

        {/* 导入数据对话框 */}
        <ImportDataDialog open={importDialogOpen} onOpenChange={setImportDialogOpen} />

        {/* 新手引导教程 */}
        <OnboardingTutorial />
      </div>
    </QueryStateGate>
  );
};

export default Dashboard;
