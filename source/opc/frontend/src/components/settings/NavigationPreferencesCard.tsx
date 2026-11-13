import { useEffect } from "react";
import { ChevronDown, ChevronUp, PanelLeft } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { useNavPreferences } from "@/hooks/useNavPreferences";
import { MENU_CATEGORIES } from "@/lib/nav-catalog";
import { NAV_ICONS } from "@/lib/nav-icons";
import {
  emptyNavPreferences,
  moveNavCategory,
  moveNavItem,
  sanitizeNavPreferences,
  toggleNavItemHidden,
  type NavPreferences,
} from "@/lib/nav-preferences";

export function NavigationPreferencesCard() {
  const { preferences, savePreferences, isSaving } = useNavPreferences();
  const sanitized = sanitizeNavPreferences(preferences);

  useEffect(() => {
    if (window.location.hash === "#navigation") {
      document.getElementById("navigation")?.scrollIntoView({ behavior: "smooth" });
    }
  }, []);

  const persist = async (next: NavPreferences) => {
    try {
      await savePreferences(next);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "导航配置保存失败");
    }
  };

  return (
    <Card id="navigation">
      <CardHeader>
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <PanelLeft className="h-5 w-5 text-primary" />
            <div>
              <CardTitle>导航栏</CardTitle>
              <CardDescription>
                按当前账号显示或隐藏侧栏入口，并调整分组与条目顺序。隐藏后仍可通过原地址打开页面。仪表盘和系统设置不可隐藏。
              </CardDescription>
            </div>
          </div>
          <Button
            variant="outline"
            size="sm"
            disabled={isSaving}
            onClick={() => void persist(emptyNavPreferences())}
          >
            恢复默认
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {sanitized.category_order.map((categoryId, categoryIndex) => {
          const category = MENU_CATEGORIES.find((entry) => entry.id === categoryId);
          if (!category) return null;
          const CategoryIcon = category.icon ? NAV_ICONS[category.icon] : undefined;
          const itemIds = sanitized.item_order[category.id] ?? [];
          return (
            <div key={category.id} className="rounded-lg border">
              <div className="flex items-center justify-between gap-2 border-b bg-muted/40 px-3 py-2">
                <div className="flex items-center gap-2 font-medium">
                  {CategoryIcon && <CategoryIcon className="h-4 w-4 text-muted-foreground" />}
                  <span>{category.label}</span>
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    disabled={isSaving || categoryIndex === 0}
                    onClick={() =>
                      void persist(moveNavCategory(sanitized, category.id, -1))
                    }
                    aria-label={`上移 ${category.label}`}
                  >
                    <ChevronUp className="h-4 w-4" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    disabled={isSaving || categoryIndex === sanitized.category_order.length - 1}
                    onClick={() =>
                      void persist(moveNavCategory(sanitized, category.id, 1))
                    }
                    aria-label={`下移 ${category.label}`}
                  >
                    <ChevronDown className="h-4 w-4" />
                  </Button>
                </div>
              </div>
              <div className="divide-y">
                {itemIds.map((itemId, itemIndex) => {
                  const item = category.items.find((entry) => entry.id === itemId);
                  if (!item) return null;
                  const ItemIcon = NAV_ICONS[item.icon];
                  const visible = !sanitized.hidden_ids.includes(item.id);
                  return (
                    <div
                      key={item.id}
                      className="flex items-center gap-3 px-3 py-2"
                    >
                      <ItemIcon className="h-4 w-4 shrink-0 text-muted-foreground" />
                      <div className="min-w-0 flex-1">
                        <div className="truncate text-sm font-medium">{item.label}</div>
                        <div className="truncate text-xs text-muted-foreground">
                          {item.path}
                        </div>
                      </div>
                      <div className="flex items-center gap-1">
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          disabled={isSaving || itemIndex === 0}
                          onClick={() =>
                            void persist(moveNavItem(sanitized, category.id, item.id, -1))
                          }
                          aria-label={`上移 ${item.label}`}
                        >
                          <ChevronUp className="h-4 w-4" />
                        </Button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          disabled={isSaving || itemIndex === itemIds.length - 1}
                          onClick={() =>
                            void persist(moveNavItem(sanitized, category.id, item.id, 1))
                          }
                          aria-label={`下移 ${item.label}`}
                        >
                          <ChevronDown className="h-4 w-4" />
                        </Button>
                      </div>
                      <Switch
                        checked={visible}
                        disabled={isSaving || item.locked}
                        onCheckedChange={(checked) =>
                          void persist(
                            toggleNavItemHidden(sanitized, item.id, !checked),
                          )
                        }
                        aria-label={`显示 ${item.label}`}
                      />
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
