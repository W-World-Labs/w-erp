import { Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { useNavigate } from "react-router-dom";
import { API_ENDPOINTS } from "@/lib/api/contracts";

type BadgeData = {
  label?: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
};

type RemindersOverview = {
  stats: {
    pending: number;
  };
  pending_reminders: Array<{
    id: string;
    title: string;
    description?: string | null;
    type_label: string;
    remind_date_label: string;
    priority_badge?: BadgeData | null;
  }>;
};

const defaultOverview: RemindersOverview = {
  stats: {
    pending: 0,
  },
  pending_reminders: [],
};

export function NotificationCenter() {
  const navigate = useNavigate();

  const { data: overview = defaultOverview } = useQuery({
    queryKey: ["notifications-overview"],
    queryFn: async () => {
      return api.get<RemindersOverview>(API_ENDPOINTS.remindersOverview);
    },
  });

  const reminders = overview.pending_reminders;
  const unreadCount = overview.stats.pending;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-5 w-5" />
          {unreadCount > 0 && (
            <Badge
              variant="destructive"
              className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center p-0 text-xs"
            >
              {unreadCount > 99 ? '99+' : unreadCount}
            </Badge>
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-80 bg-background border shadow-lg z-50"
      >
        <DropdownMenuLabel className="flex items-center justify-between">
          <span>通知中心</span>
          {unreadCount > 0 && (
            <Badge variant="secondary" className="ml-2">
              {unreadCount} 条待处理
            </Badge>
          )}
        </DropdownMenuLabel>
        <DropdownMenuSeparator />

        <ScrollArea className="h-[400px]">
          {reminders.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
              <Bell className="h-12 w-12 mb-2 opacity-50" />
              <p className="text-sm">暂无待处理提醒</p>
            </div>
          ) : (
            <div className="space-y-1">
              {reminders.map((reminder) => (
                <DropdownMenuItem
                  key={reminder.id}
                  className="flex flex-col items-start gap-1 p-3 cursor-pointer hover:bg-accent"
                  onClick={() => navigate("/reminders")}
                >
                  <div className="flex items-center gap-2 w-full">
                    <Badge variant="outline" className="text-xs">
                      {reminder.type_label}
                    </Badge>
                    {reminder.priority_badge?.label && (
                      <Badge
                        variant={reminder.priority_badge.variant || "outline"}
                        className={`text-xs ${reminder.priority_badge.class_name || ""}`}
                      >
                        {reminder.priority_badge.label}
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm font-medium line-clamp-1">{reminder.title}</p>
                  {reminder.description && (
                    <p className="text-xs text-muted-foreground line-clamp-2">
                      {reminder.description}
                    </p>
                  )}
                  <p className="text-xs text-muted-foreground">
                    {reminder.remind_date_label}
                  </p>
                </DropdownMenuItem>
              ))}
            </div>
          )}
        </ScrollArea>

        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="text-center justify-center cursor-pointer"
          onClick={() => navigate("/reminders")}
        >
          查看全部提醒
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
