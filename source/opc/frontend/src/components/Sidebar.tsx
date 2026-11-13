import { NavLink, useLocation } from "react-router-dom";
import { cn } from "@/lib/utils";
import {
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { CompanySwitcher } from "@/components/CompanySwitcher";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useState, useEffect } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api/contracts";
import { useNavPreferences } from "@/hooks/useNavPreferences";
import { categoryIdForPath } from "@/lib/nav-catalog";
import { NAV_ICONS } from "@/lib/nav-icons";

type SidebarRemindersOverview = {
  stats: {
    pending: number;
  };
};

const Sidebar = () => {
  const location = useLocation();
  const { categories } = useNavPreferences();
  const [openCategories, setOpenCategories] = useState<Record<string, boolean>>({
    overview: true,
    finance: false,
    business: false,
    team: false,
    system: false,
  });

  useEffect(() => {
    const categoryId = categoryIdForPath(location.pathname);
    if (!categoryId) return;
    setOpenCategories((prev) => ({ ...prev, [categoryId]: true }));
  }, [location.pathname]);

  const { data: unreadCount = 0, refetch } = useQuery({
    queryKey: ["sidebar-unread-reminders"],
    queryFn: async () => {
      const overview = await api.get<SidebarRemindersOverview>(
        API_ENDPOINTS.remindersOverview,
      );
      return overview.stats.pending;
    },
  });

  useEffect(() => {
    const channel = null;

    return () => {
      void channel;
    };
  }, [refetch]);

  const toggleCategory = (id: string) => {
    setOpenCategories((prev) => ({
      ...prev,
      [id]: !prev[id],
    }));
  };

  return (
    <div className="flex h-screen w-64 flex-col bg-sidebar text-sidebar-foreground">
      <CompanySwitcher />

      <nav className="flex-1 overflow-y-auto space-y-2 p-4">
        {categories.map((category) => {
          const CategoryIcon = category.icon ? NAV_ICONS[category.icon] : undefined;
          return (
            <Collapsible
              key={category.id}
              open={openCategories[category.id]}
              onOpenChange={() => toggleCategory(category.id)}
            >
              <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm font-semibold hover:bg-sidebar-accent transition-all group">
                <div className="flex items-center gap-2">
                  {CategoryIcon && (
                    <CategoryIcon className="h-4 w-4 text-sidebar-foreground/70" />
                  )}
                  <span>{category.label}</span>
                </div>
                {openCategories[category.id] ? (
                  <ChevronDown className="h-4 w-4 text-sidebar-foreground/70" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-sidebar-foreground/70" />
                )}
              </CollapsibleTrigger>
              <CollapsibleContent className="space-y-1 mt-1">
                {category.items.map((item) => {
                  const ItemIcon = NAV_ICONS[item.icon];
                  return (
                    <NavLink
                      key={item.path}
                      to={item.path}
                      end={item.path === "/"}
                      className={({ isActive }) =>
                        cn(
                          "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-all hover:bg-sidebar-accent ml-2",
                          isActive && "bg-sidebar-accent text-sidebar-primary",
                        )
                      }
                    >
                      <ItemIcon className="h-4 w-4" />
                      <span className="flex-1">{item.label}</span>
                      {item.showBadge && unreadCount > 0 && (
                        <Badge
                          variant="destructive"
                          className="h-5 px-1.5 text-xs"
                        >
                          {unreadCount > 99 ? "99+" : unreadCount}
                        </Badge>
                      )}
                    </NavLink>
                  );
                })}
              </CollapsibleContent>
            </Collapsible>
          );
        })}
      </nav>

      <div className="border-t border-sidebar-border p-4 flex-shrink-0 space-y-2">
        <NavLink
          to="/settings#navigation"
          className="block rounded-lg px-3 py-2 text-xs font-medium text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-foreground"
        >
          自定义导航
        </NavLink>
        <div className="flex items-center justify-between rounded-lg bg-sidebar-accent p-3">
          <div>
            <p className="text-xs font-medium">版本信息</p>
            <p className="text-xs text-sidebar-foreground/60 mt-1">v1.0.0</p>
          </div>
          <ThemeToggle />
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
