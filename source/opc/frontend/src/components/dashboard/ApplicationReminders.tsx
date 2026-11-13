import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Bell, Clock, ChevronRight } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useSmartWorkbench } from "@/hooks/useSmartWorkbench";

export const ApplicationReminders = () => {
  const navigate = useNavigate();
  const { applicationReminders } = useSmartWorkbench();

  if (applicationReminders.length === 0) {
    return null;
  }

  return (
    <Card className="border-orange-200 bg-orange-50/50 dark:border-orange-900 dark:bg-orange-950/20">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-orange-700 dark:text-orange-400">
          <Bell className="h-5 w-5" />
          申报进度提醒
        </CardTitle>
        <CardDescription>
          后端已识别以下临近截止的申报事项
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {applicationReminders.map((item) => (
          <Card key={item.id} className="border-l-4">
            <CardContent className="p-4">
              <div className="space-y-3">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h4 className="font-medium">{item.title}</h4>
                      <Badge variant="secondary" className={item.priorityBadge.class_name}>
                        {item.priorityBadge.label}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <Clock className="h-4 w-4" />
                      <span>{item.daysLeftLabel || item.dueDate}</span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-2">
                      {item.description}
                    </p>
                  </div>
                </div>

                <div className="flex justify-end">
                  <Button 
                    size="sm" 
                    variant="default"
                    onClick={() => navigate(item.path)}
                    className="gap-1"
                  >
                    {item.action}
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}

        <div className="flex items-center justify-between pt-2 border-t">
          <div className="text-sm text-muted-foreground">
            共 {applicationReminders.length} 个项目需要关注
          </div>
          <Button 
            variant="ghost" 
            size="sm"
            onClick={() => navigate('/applications')}
          >
            查看全部
            <ChevronRight className="h-4 w-4 ml-1" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};
