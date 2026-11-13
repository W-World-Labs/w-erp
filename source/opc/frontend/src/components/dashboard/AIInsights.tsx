import { useNavigate } from 'react-router-dom';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { 
  AlertTriangle, 
  Lightbulb, 
  TrendingUp, 
  PiggyBank,
  CheckCircle,
  ChevronRight,
  Brain,
  Sparkles,
  X
} from 'lucide-react';
import { Insight, useAIInsights } from '@/hooks/useAIInsights';
import { DataCompleteness } from './DataCompleteness';
import { cn } from '@/lib/utils';
import { useState } from 'react';

const insightIconMap = {
  AlertTriangle,
  Lightbulb,
  TrendingUp,
  PiggyBank,
  CheckCircle,
};

function InsightCard({ insight, onDismiss }: { insight: Insight; onDismiss?: () => void }) {
  const navigate = useNavigate();
  const Icon = insightIconMap[insight.displayIcon as keyof typeof insightIconMap] || Lightbulb;

  return (
    <Card 
      className={cn(
        'relative overflow-hidden transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5',
        'border-l-4',
        insight.cardBorderClassName,
        insight.cardRingClassName
      )}
    >
      {/* 背景装饰 */}
      <div className={cn('absolute inset-0 opacity-50', insight.iconBgClassName)} />
      
      <CardContent className="relative p-4">
        <div className="flex items-start gap-3">
          {/* 图标 */}
          <div className={cn('rounded-lg p-2', insight.iconBgClassName)}>
            <Icon className={cn('h-5 w-5', insight.iconClassName)} />
          </div>
          
          {/* 内容 */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-lg">{insight.icon}</span>
              <h4 className="font-medium text-sm truncate">{insight.title}</h4>
              <Badge className={cn('text-xs px-1.5 py-0', insight.priorityBadge.className)}>
                {insight.priorityBadge.label}
              </Badge>
            </div>
            <p className="text-sm text-muted-foreground line-clamp-2">
              {insight.description}
            </p>
            
            {insight.action && (
              <Button
                variant="link"
                size="sm"
                className={cn('p-0 h-auto mt-2', insight.actionClassName)}
                onClick={() => navigate(insight.action!.path)}
              >
                {insight.action.label}
                <ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            )}
          </div>

          {/* 关闭按钮 */}
          {onDismiss && (
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 opacity-50 hover:opacity-100"
              onClick={onDismiss}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function InsightsSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <Skeleton className="h-6 w-6 rounded-full" />
        <Skeleton className="h-5 w-32" />
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        {[1, 2, 3, 4].map((i) => (
          <Skeleton key={i} className="h-24 rounded-lg" />
        ))}
      </div>
    </div>
  );
}

export function AIInsights() {
  const { data, isLoading, error } = useAIInsights();
  const [dismissedIds, setDismissedIds] = useState<Set<string>>(new Set());

  const handleDismiss = (id: string) => {
    setDismissedIds(prev => new Set([...prev, id]));
  };

  if (isLoading) {
    return <InsightsSkeleton />;
  }

  if (error || !data) {
    return null; // 静默失败，不影响其他功能
  }

  const { criticalInsights, otherInsights, completeness, summary } = data;
  const isDismissed = (id: string) => dismissedIds.has(id);

  return (
    <div className="space-y-6 animate-fade-in">
      {/* AI 洞察头部 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="rounded-lg bg-gradient-to-r from-primary/20 to-primary/10 p-2">
            <Brain className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h2 className="text-lg font-semibold flex items-center gap-2">
              AI 智能洞察
              <Sparkles className="h-4 w-4 text-primary animate-pulse" />
            </h2>
            <p className="text-xs text-muted-foreground">
              基于您的数据，AI 主动发现了 {summary.totalInsights} 条洞察
            </p>
          </div>
        </div>
        
        {/* 摘要标签 */}
        <div className="hidden sm:flex items-center gap-2">
          {summary.criticalCount > 0 && (
            <Badge variant="destructive" className="animate-pulse">
              {summary.criticalCount} 紧急
            </Badge>
          )}
          {summary.highCount > 0 && (
            <Badge variant="secondary" className="bg-orange-500/20 text-orange-600">
              {summary.highCount} 重要
            </Badge>
          )}
        </div>
      </div>

      {/* 双栏布局 */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* 左侧：数据完整度 */}
        <div className="lg:col-span-1">
          <DataCompleteness completeness={completeness} />
        </div>

        {/* 右侧：洞察卡片 */}
        <div className="lg:col-span-2 space-y-4">
          {/* 紧急洞察 */}
          {criticalInsights.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-red-500 flex items-center gap-2">
                <AlertTriangle className="h-4 w-4" />
                需要立即处理
              </h3>
              <div className="grid gap-3 sm:grid-cols-2">
                {criticalInsights.map((insight) => (
                  isDismissed(insight.id) ? null :
                  <InsightCard 
                    key={insight.id} 
                    insight={insight} 
                    onDismiss={() => handleDismiss(insight.id)}
                  />
                ))}
              </div>
            </div>
          )}

          {/* 其他洞察 */}
          {otherInsights.length > 0 && (
            <div className="space-y-3">
              {criticalInsights.length > 0 && (
                <h3 className="text-sm font-medium text-muted-foreground">
                  其他建议
                </h3>
              )}
              <div className="grid gap-3 sm:grid-cols-2">
                {otherInsights.map((insight) => (
                  isDismissed(insight.id) ? null :
                  <InsightCard 
                    key={insight.id} 
                    insight={insight}
                    onDismiss={() => handleDismiss(insight.id)}
                  />
                ))}
              </div>
            </div>
          )}

          {/* 无洞察时的提示 */}
          {summary.totalInsights === 0 && (
            <Card className="border-dashed">
              <CardContent className="flex flex-col items-center justify-center py-8 text-center">
                <div className="rounded-full bg-green-500/10 p-3 mb-3">
                  <CheckCircle className="h-8 w-8 text-green-500" />
                </div>
                <h3 className="font-medium">一切运转正常！</h3>
                <p className="text-sm text-muted-foreground mt-1">
                  当前没有需要处理的事项，继续保持 👏
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
