import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { 
  CheckCircle2, 
  Circle, 
  Clock, 
  ArrowRight,
  Sparkles,
  Target,
  TrendingUp
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useNavigate } from "react-router-dom";

interface TodoItem {
  id: string;
  title: string;
  category: string;
  priority: 'critical' | 'high' | 'medium' | 'low';
  completed: boolean;
  path: string;
}

interface TodayFocusProps {
  todos: TodoItem[];
  aiSuggestions: string[];
  completionRate: number;
}

export function TodayFocus({ todos, aiSuggestions, completionRate }: TodayFocusProps) {
  const navigate = useNavigate();

  return (
    <Card className="animate-fade-in" style={{ animationDelay: '0.4s' }}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Target className="h-5 w-5 text-primary" />
            今日焦点
          </CardTitle>
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">完成度</span>
            <div className="flex items-center gap-2">
              <Progress value={completionRate} className="w-20 h-2" />
              <span className="text-sm font-medium">{completionRate}%</span>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* 待办事项列表 */}
        <div className="space-y-2">
          {todos.slice(0, 4).map((todo, index) => (
            <div
              key={todo.id}
              className={cn(
                "flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 cursor-pointer",
                "hover:bg-muted/50 hover:border-primary/30",
                todo.completed && "opacity-60 bg-muted/30"
              )}
              style={{ animationDelay: `${index * 0.05}s` }}
              onClick={() => navigate(todo.path)}
            >
              <div className={cn(
                "h-5 w-5 rounded-full border-2 flex items-center justify-center transition-colors",
                todo.completed 
                  ? "border-accent bg-accent text-accent-foreground" 
                  : "border-muted-foreground/30"
              )}>
                {todo.completed && <CheckCircle2 className="h-3 w-3" />}
              </div>
              <div className="flex-1">
                <span className={cn(
                  "font-medium",
                  todo.completed && "line-through text-muted-foreground"
                )}>
                  {todo.title}
                </span>
              </div>
              <Badge variant="secondary" className="text-xs">
                {todo.category}
              </Badge>
            </div>
          ))}
        </div>

        {/* AI 建议 */}
        {aiSuggestions.length > 0 && (
          <div className="mt-4 p-4 rounded-lg bg-gradient-to-r from-primary/5 to-primary/10 border border-primary/20">
            <div className="flex items-center gap-2 mb-3">
              <Sparkles className="h-4 w-4 text-primary" />
              <span className="text-sm font-medium">AI 智能建议</span>
            </div>
            <ul className="space-y-2">
              {aiSuggestions.map((suggestion, index) => (
                <li 
                  key={index} 
                  className="flex items-start gap-2 text-sm text-muted-foreground"
                >
                  <TrendingUp className="h-4 w-4 mt-0.5 text-primary flex-shrink-0" />
                  {suggestion}
                </li>
              ))}
            </ul>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
