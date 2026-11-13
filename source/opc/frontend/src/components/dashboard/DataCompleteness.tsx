import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { ChevronRight, Sparkles, CheckCircle2 } from 'lucide-react';
import { AIInsightsData } from '@/hooks/useAIInsights';
import { cn } from '@/lib/utils';

interface DataCompletenessProps {
  completeness: AIInsightsData['completeness'];
  className?: string;
}

export function DataCompleteness({ completeness, className }: DataCompletenessProps) {
  const navigate = useNavigate();

  return (
    <Card className={cn('overflow-hidden', className)}>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-lg">
          <Sparkles className="h-5 w-5 text-primary" />
          企业数据完整度
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-6">
          <div className="relative h-24 w-24 flex-shrink-0">
            <svg className="h-24 w-24 -rotate-90 transform">
              <circle
                cx="48"
                cy="48"
                r="40"
                stroke="currentColor"
                strokeWidth="8"
                fill="none"
                className="text-muted/20"
              />
              <circle
                cx="48"
                cy="48"
                r="40"
                stroke="url(#progressGradient)"
                strokeWidth="8"
                fill="none"
                strokeLinecap="round"
                strokeDasharray={`${completeness.overall * 2.51} 251`}
                className="transition-all duration-1000 ease-out"
              />
              <defs>
                <linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" className="stop-color text-primary" stopColor="currentColor" />
                  <stop offset="100%" className="stop-color text-accent" stopColor="currentColor" />
                </linearGradient>
              </defs>
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className={cn('text-2xl font-bold bg-gradient-to-r bg-clip-text text-transparent', completeness.overallGradientClass)}>
                {completeness.overallLabel}
              </span>
              <span className="text-xs text-muted-foreground">完成</span>
            </div>
          </div>

          <div className="flex-1 space-y-1">
            <p className="text-sm text-muted-foreground">
              {completeness.statusDescription}
            </p>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              {completeness.completedItems.length > 0 && (
                <CheckCircle2 className="h-3 w-3 text-green-500" />
              )}
              <span>{completeness.completedLabel}</span>
              {completeness.incompleteItems.length > 0 && (
                <>
                  <span>·</span>
                  <span>{completeness.incompleteLabel}</span>
                </>
              )}
            </div>
          </div>
        </div>

        {completeness.incompleteItems.length > 0 && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground">待完善模块</p>
            <div className="space-y-2">
              {completeness.topIncompleteItems.map((item) => (
                <div
                  key={item.key}
                  className="group flex items-center gap-3 rounded-lg border border-border/50 bg-card/50 p-3 transition-all hover:border-primary/30 hover:bg-accent/50"
                >
                  <span className="text-xl">{item.icon}</span>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-sm font-medium truncate">{item.label}</span>
                      <span className="text-xs text-muted-foreground">{item.percentageLabel}</span>
                    </div>
                    <Progress value={item.percentage} className="h-1.5" />
                    {item.missingItems.length > 0 && (
                      <p className="text-xs text-muted-foreground mt-1 truncate">
                        {item.missingItems[0]}
                      </p>
                    )}
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="opacity-0 group-hover:opacity-100 transition-opacity"
                    onClick={() => navigate(item.path)}
                  >
                    去完善
                    <ChevronRight className="h-4 w-4 ml-1" />
                  </Button>
                </div>
              ))}
            </div>
          </div>
        )}

        {completeness.incompleteItems.length > 0 && (
          <div className="rounded-lg bg-gradient-to-r from-primary/10 to-primary/5 p-3 text-sm">
            <p className="text-primary font-medium">{completeness.unlockTitle}</p>
            <p className="text-muted-foreground text-xs mt-1">
              {completeness.unlockDescription}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
