import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Rocket, TrendingUp, Award } from "lucide-react";

export type EnterpriseStage = 'startup' | 'growth' | 'mature';

interface StageSelectorProps {
  selectedStage: EnterpriseStage;
  onStageChange: (stage: EnterpriseStage) => void;
}

const stages = [
  {
    id: 'startup' as EnterpriseStage,
    icon: Rocket,
    title: '创业期',
    description: '证照办理、团队组建、基础管理',
    color: 'text-primary',
    bgColor: 'bg-primary/10',
    borderColor: 'border-primary/20',
  },
  {
    id: 'growth' as EnterpriseStage,
    icon: TrendingUp,
    title: '成长期',
    description: '项目扩展、财务精细化、流程规范',
    color: 'text-accent',
    bgColor: 'bg-accent/10',
    borderColor: 'border-accent/20',
  },
  {
    id: 'mature' as EnterpriseStage,
    icon: Award,
    title: '成熟期',
    description: '数据分析、战略规划、优化管理',
    color: 'text-warning',
    bgColor: 'bg-warning/10',
    borderColor: 'border-warning/20',
  },
];

export const StageSelector = ({ selectedStage, onStageChange }: StageSelectorProps) => {
  return (
    <Card className="border-muted">
      <CardHeader>
        <CardTitle>选择企业阶段</CardTitle>
        <CardDescription>根据您的企业发展阶段，定制专属仪表盘</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid md:grid-cols-3 gap-4">
          {stages.map((stage) => {
            const Icon = stage.icon;
            const isSelected = selectedStage === stage.id;
            
            return (
              <button
                key={stage.id}
                onClick={() => onStageChange(stage.id)}
                className={`
                  relative p-4 rounded-lg border-2 text-left transition-all
                  ${isSelected 
                    ? `${stage.borderColor} ${stage.bgColor} shadow-md` 
                    : 'border-border hover:border-muted-foreground/30 bg-card'
                  }
                `}
              >
                <div className="flex items-start gap-3">
                  <div className={`rounded-full p-2 ${isSelected ? 'bg-background/80' : stage.bgColor}`}>
                    <Icon className={`h-5 w-5 ${stage.color}`} />
                  </div>
                  <div className="flex-1">
                    <p className={`font-semibold mb-1 ${isSelected ? stage.color : 'text-foreground'}`}>
                      {stage.title}
                    </p>
                    <p className="text-xs text-muted-foreground leading-relaxed">
                      {stage.description}
                    </p>
                  </div>
                </div>
                {isSelected && (
                  <div className={`absolute top-2 right-2 h-2 w-2 rounded-full ${stage.color.replace('text-', 'bg-')}`} />
                )}
              </button>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
};
