import { useQuery } from '@tanstack/react-query';
import { api } from "@/integrations/api/client";
import { useAuth } from '@/contexts/AuthContext';

export const AI_INSIGHTS_ENDPOINT = '/api/v1/ai/insights';

type BackendInsightType = 'urgent' | 'finance' | 'completion' | 'status' | string;

interface BackendInsight {
  id: string;
  type: BackendInsightType;
  priority: 'critical' | 'high' | 'medium' | 'low';
  priority_badge?: {
    label?: string;
    class_name?: string;
    color_class?: string;
  };
  title: string;
  description: string;
  icon?: string;
  display_type?: 'warning' | 'suggestion' | 'saving' | 'data' | 'completion';
  display_icon?: string;
  icon_class_name?: string;
  icon_bg_class_name?: string;
  card_border_class_name?: string;
  card_ring_class_name?: string;
  action_class_name?: string;
  category: string;
  action: string;
  action_label?: string;
  confidence: number;
  data?: Record<string, unknown>;
}

interface BackendCompletenessItem {
  key: string;
  label: string;
  icon: string;
  path: string;
  score: number;
  max_score: number;
  percentage: number;
  percentage_label?: string;
  missing_items: string[];
  description: string;
}

interface BackendCompleteness {
  overall_score?: number;
  overall_score_label?: string;
  overall_gradient_class?: string;
  unlock_title?: string;
  unlock_description?: string;
  completed_count?: number;
  incomplete_count?: number;
  total_count?: number;
  completed_label?: string;
  incomplete_label?: string;
  is_complete?: boolean;
  status_title?: string;
  status_description?: string;
  completion_badge_label?: string;
  remaining_overflow_count?: number;
  remaining_overflow_label?: string | null;
  items?: BackendCompletenessItem[];
  completed_items?: BackendCompletenessItem[];
  incomplete_items?: BackendCompletenessItem[];
  top_incomplete_items?: BackendCompletenessItem[];
}

interface BackendAIInsightsData {
  insights?: BackendInsight[];
  critical_insights?: BackendInsight[];
  other_insights?: BackendInsight[];
  completeness?: BackendCompleteness;
  summary?: {
    total?: number;
    critical?: number;
    high?: number;
    medium?: number;
    low?: number;
  };
}

export interface Insight {
  id: string;
  type: 'warning' | 'suggestion' | 'saving' | 'data' | 'completion';
  priority: 'critical' | 'high' | 'medium' | 'low';
  priorityBadge: {
    label: string;
    className: string;
    colorClass: string;
  };
  title: string;
  description: string;
  icon: string;
  displayIcon: string;
  iconClassName: string;
  iconBgClassName: string;
  cardBorderClassName: string;
  cardRingClassName: string;
  actionClassName: string;
  action?: {
    label: string;
    path: string;
  };
  metadata?: Record<string, unknown>;
}

export interface CompletenessItem {
  key: string;
  label: string;
  icon: string;
  path: string;
  score: number;
  maxScore: number;
  percentage: number;
  percentageLabel: string;
  missingItems: string[];
  description: string;
}

export interface AIInsightsData {
  insights: Insight[];
  criticalInsights: Insight[];
  otherInsights: Insight[];
  completeness: {
    overall: number;
    overallLabel: string;
    overallGradientClass: string;
    unlockTitle: string;
    unlockDescription: string;
    completedCount: number;
    incompleteCount: number;
    totalCount: number;
    completedLabel: string;
    incompleteLabel: string;
    isComplete: boolean;
    statusTitle: string;
    statusDescription: string;
    completionBadgeLabel: string;
    remainingOverflowCount: number;
    remainingOverflowLabel?: string | null;
    items: CompletenessItem[];
    completedItems: CompletenessItem[];
    incompleteItems: CompletenessItem[];
    topIncompleteItems: CompletenessItem[];
  };
  summary: {
    criticalCount: number;
    highCount: number;
    mediumCount: number;
    totalInsights: number;
  };
}

export function normalizeAIInsightsData(data?: BackendAIInsightsData | null): AIInsightsData {
  const completeness = data?.completeness;
  const normalizeInsight = (insight: BackendInsight): Insight => ({
    id: insight.id,
    type: insight.display_type || 'suggestion',
    priority: insight.priority,
    priorityBadge: {
      label: insight.priority_badge?.label || '',
      className: insight.priority_badge?.class_name || '',
      colorClass: insight.priority_badge?.color_class || '',
    },
    title: insight.title,
    description: insight.description,
    icon: insight.icon || '',
    displayIcon: insight.display_icon || 'Lightbulb',
    iconClassName: insight.icon_class_name || '',
    iconBgClassName: insight.icon_bg_class_name || '',
    cardBorderClassName: insight.card_border_class_name || '',
    cardRingClassName: insight.card_ring_class_name || '',
    actionClassName: insight.action_class_name || '',
    action: insight.action ? { label: insight.action_label || '', path: insight.action } : undefined,
    metadata: {
      ...(insight.data || {}),
      category: insight.category,
      confidence: insight.confidence,
    },
  });

  return {
    insights: (data?.insights || []).map(normalizeInsight),
    criticalInsights: (data?.critical_insights || []).map(normalizeInsight),
    otherInsights: (data?.other_insights || []).map(normalizeInsight),
    completeness: {
      overall: completeness?.overall_score || 0,
      overallLabel: completeness?.overall_score_label || '',
      overallGradientClass: completeness?.overall_gradient_class || 'from-orange-500 to-red-500',
      unlockTitle: completeness?.unlock_title || '',
      unlockDescription: completeness?.unlock_description || '',
      completedCount: completeness?.completed_count || 0,
      incompleteCount: completeness?.incomplete_count || 0,
      totalCount: completeness?.total_count || 0,
      completedLabel: completeness?.completed_label || '',
      incompleteLabel: completeness?.incomplete_label || '',
      isComplete: completeness?.is_complete || false,
      statusTitle: completeness?.status_title || '',
      statusDescription: completeness?.status_description || '',
      completionBadgeLabel: completeness?.completion_badge_label || '',
      remainingOverflowCount: completeness?.remaining_overflow_count || 0,
      remainingOverflowLabel: completeness?.remaining_overflow_label || null,
      items: (completeness?.items || []).map(normalizeCompletenessItem),
      completedItems: (completeness?.completed_items || []).map(normalizeCompletenessItem),
      incompleteItems: (completeness?.incomplete_items || []).map(normalizeCompletenessItem),
      topIncompleteItems: (completeness?.top_incomplete_items || []).map(normalizeCompletenessItem),
    },
    summary: {
      criticalCount: data?.summary?.critical || 0,
      highCount: data?.summary?.high || 0,
      mediumCount: data?.summary?.medium || 0,
      totalInsights: data?.summary?.total || 0,
    },
  };
}

function normalizeCompletenessItem(item: BackendCompletenessItem): CompletenessItem {
  return {
    key: item.key,
    label: item.label,
    icon: moduleIcon(item.icon),
    path: item.path,
    score: item.score,
    maxScore: item.max_score,
    percentage: item.percentage,
    percentageLabel: item.percentage_label || '',
    missingItems: item.missing_items,
    description: item.description,
  };
}

export function useAIInsights() {
  const { user } = useAuth();

  return useQuery({
    queryKey: ['ai-insights', user?.id],
    queryFn: async (): Promise<AIInsightsData> => {
      const result = await api.get<BackendAIInsightsData>(AI_INSIGHTS_ENDPOINT);
      return normalizeAIInsightsData(result);
    },
    enabled: !!user,
    staleTime: 5 * 60 * 1000,
    refetchOnWindowFocus: false,
  });
}

function moduleIcon(icon: string): string {
  const icons: Record<string, string> = {
    Building2: '🏢',
    Users: '👥',
    ReceiptText: '📄',
    FileText: '📝',
    Archive: '🗄️',
    Briefcase: '💼',
    Landmark: '🏦',
    WalletCards: '💳',
    BadgePercent: '％',
    ClipboardCheck: '✅',
  };

  return icons[icon] || icon;
}
