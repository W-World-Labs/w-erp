import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
  CheckCircle2,
  AlertCircle,
  ArrowRight,
  Sparkles,
} from "lucide-react";
import { useAIInsights } from "@/hooks/useAIInsights";

export function CompletionOverview() {
  const navigate = useNavigate();
  const { data } = useAIInsights();
  const completeness = data?.completeness;

  if (!completeness) {
    return null;
  }

  const modules = completeness.items;

  if (completeness.isComplete) {
    return (
      <Card className="bg-gradient-to-br from-green-50 to-emerald-50 border-green-200">
        <CardHeader>
          <div className="flex items-center gap-2">
            <div className="p-2 bg-green-100 rounded-lg">
              <CheckCircle2 className="h-6 w-6 text-green-600" />
            </div>
            <div>
              <CardTitle className="text-green-900">
                {completeness.statusTitle}
              </CardTitle>
              <CardDescription className="text-green-700">
                {completeness.statusDescription}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-3xl font-bold text-green-900">
                  {completeness.completedCount}
                </div>
                <div className="text-sm text-green-700">已完善模块</div>
              </div>
              <Badge className="bg-green-600 text-white px-4 py-2 text-base">
                <Sparkles className="h-4 w-4 mr-1" />
                {completeness.completionBadgeLabel}
              </Badge>
            </div>
            <div className="grid grid-cols-3 gap-2 text-center">
              {modules.map((module) => (
                <div key={module.key} className="p-2 bg-white rounded-lg">
                  <div className="text-lg mb-1">{module.icon}</div>
                  <div className="text-xs text-green-900 font-medium">{module.label}</div>
                  <div className="text-xs text-green-700">
                    {module.percentageLabel}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="p-2 bg-blue-100 rounded-lg">
            <AlertCircle className="h-6 w-6 text-blue-600" />
          </div>
          <div className="flex-1">
            <CardTitle className="text-blue-900">
              {completeness.statusTitle}
            </CardTitle>
            <CardDescription className="text-blue-700">
              {completeness.statusDescription}
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-blue-900 font-medium">
                {completeness.completedLabel}
              </span>
              <Badge variant="secondary" className="bg-blue-100 text-blue-900">
                {completeness.overallLabel}
              </Badge>
            </div>
            <Progress value={completeness.overall} className="h-2" />
          </div>

          {completeness.incompleteItems.length > 0 && (
            <div className="space-y-2">
              <div className="text-sm font-medium text-blue-900">
                {completeness.incompleteLabel}
              </div>
              <div className="grid gap-2">
                {completeness.topIncompleteItems.map((module) => (
                  <Button
                    key={module.key}
                    variant="outline"
                    className="justify-between h-auto py-2 bg-white hover:bg-blue-50 border-blue-200"
                    onClick={() => navigate(module.path)}
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-base">{module.icon}</span>
                      <span className="text-blue-900">{module.label}</span>
                      <Badge variant="outline" className="text-xs">
                        {module.percentageLabel}
                      </Badge>
                    </div>
                    <ArrowRight className="h-4 w-4 text-blue-600" />
                  </Button>
                ))}
              </div>
              {completeness.remainingOverflowLabel && (
                <div className="text-xs text-center text-blue-600">
                  {completeness.remainingOverflowLabel}
                </div>
              )}
            </div>
          )}

          {completeness.completedItems.length > 0 && (
            <div className="pt-2 border-t border-blue-200">
              <div className="text-xs text-blue-700 mb-2">已完成模块：</div>
              <div className="flex flex-wrap gap-2">
                {completeness.completedItems.map((module) => (
                  <Badge
                    key={module.key}
                    className="bg-green-100 text-green-700 hover:bg-green-200"
                  >
                    <CheckCircle2 className="h-3 w-3 mr-1" />
                    {module.label}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
