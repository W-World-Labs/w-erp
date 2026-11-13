import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { AlertCircle, ArrowRight } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useSmartWorkbench } from "@/hooks/useSmartWorkbench";

export function AssetExpiryReminders() {
  const navigate = useNavigate();
  const { assetExpiryReminders } = useSmartWorkbench();

  if (assetExpiryReminders.length === 0) {
    return null;
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertCircle className="h-5 w-5 text-destructive" />
          虚拟资产到期提醒
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {assetExpiryReminders.map((asset) => (
            <div
              key={asset.id}
              className="p-4 rounded-lg border border-border"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium">{asset.title}</span>
                    <Badge variant="outline" className={asset.priorityBadge.class_name}>
                      {asset.priorityBadge.label}
                    </Badge>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    <div>{asset.description}</div>
                    <div className="mt-1">{asset.daysLeftLabel || asset.dueDate}</div>
                  </div>
                </div>
                <Button size="sm" variant="outline" onClick={() => navigate(asset.path)}>
                  {asset.action}
                  <ArrowRight className="h-4 w-4 ml-1" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
