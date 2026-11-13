import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, companies, setCurrentCompany } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  type LucideIcon,
  CheckCircle2,
  Circle,
  Building2,
  Users,
  FileText,
  DollarSign,
  FolderKanban,
  Zap,
  TrendingUp,
  ArrowRight,
  Home,
  FileSpreadsheet,
  Clock,
  Wallet,
  Package,
  Sparkles,
  Rocket,
} from "lucide-react";
import { cn } from "@/lib/utils";

type DisplayBadge = {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
};

interface TutorialStep {
  id: string;
  title: string;
  description: string;
  icon_key: string;
  route: string;
  completed: boolean;
  action_label: string;
  status_badge: DisplayBadge;
  tier: number;
  section_title: string;
}

interface ActivationSummary {
  completed_count: number;
  total_count: number;
  remaining_count: number;
  rate: number;
  rate_label: string;
  label: string;
  completed: boolean;
}

interface ActivationLoopStep {
  id: string;
  label: string;
  route: string;
  completed: boolean;
  evidence_label: string;
  status_badge: DisplayBadge;
}

interface ActivationLoopAction {
  route: string;
  label: string;
  description: string;
  action?: unknown | null;
}

interface ActivationLoop {
  id: string;
  title: string;
  completed: boolean;
  current_step_id: string;
  completed_count: number;
  total_count: number;
  remaining_count: number;
  rate: number;
  rate_label: string;
  label: string;
  status_badge: DisplayBadge;
  next_action: ActivationLoopAction | null;
  steps: ActivationLoopStep[];
}

type OperatingBackbone = ActivationLoop;

interface OnboardingCta {
  kind: string;
  title: string;
  description: string;
  action_label: string;
  input_placeholder: string;
}

interface OnboardingProgress {
  summary: {
    progress_rate: number;
    progress_rate_label: string;
    completed_label: string;
    remaining_count: number;
    all_completed: boolean;
    first_incomplete_index: number;
    demo_company: boolean;
    tutorial_mode: "data_completion" | "demo_walkthrough";
    tutorial_storage_key: string;
    activation: ActivationSummary;
    activation_loop: ActivationLoop;
    operating_backbone: OperatingBackbone;
    cta: OnboardingCta | null;
  };
  modules: unknown[];
  tutorial_steps: TutorialStep[];
  incomplete_tip: {
    title: string;
    description: string;
  };
  completed_tip: {
    title: string;
    description: string;
  };
  tutorial_completed_tip: {
    title: string;
    description: string;
  };
}

const emptyActivation: ActivationSummary = {
  completed_count: 0,
  total_count: 0,
  remaining_count: 0,
  rate: 0,
  rate_label: "0%",
  label: "",
  completed: false,
};

const emptyActivationLoop: ActivationLoop = {
  id: "first_paid_invoice",
  title: "",
  completed: false,
  current_step_id: "company",
  completed_count: 0,
  total_count: 0,
  remaining_count: 0,
  rate: 0,
  rate_label: "0%",
  label: "",
  status_badge: {
    label: "",
    variant: "secondary",
  },
  next_action: null,
  steps: [],
};

const emptyOperatingBackbone: OperatingBackbone = {
  id: "operating_backbone",
  title: "",
  completed: false,
  current_step_id: "contracts",
  completed_count: 0,
  total_count: 0,
  remaining_count: 0,
  rate: 0,
  rate_label: "0%",
  label: "",
  status_badge: {
    label: "",
    variant: "secondary",
  },
  next_action: null,
  steps: [],
};

const emptyProgress: OnboardingProgress = {
  summary: {
    progress_rate: 0,
    progress_rate_label: "0%",
    completed_label: "已完成 0/0 个模块",
    remaining_count: 0,
    all_completed: false,
    first_incomplete_index: 0,
    demo_company: false,
    tutorial_mode: "data_completion",
    tutorial_storage_key: "onboarding-tutorial-dismissed:global",
    activation: emptyActivation,
    activation_loop: emptyActivationLoop,
    operating_backbone: emptyOperatingBackbone,
    cta: null,
  },
  modules: [],
  tutorial_steps: [],
  incomplete_tip: {
    title: "",
    description: "",
  },
  completed_tip: {
    title: "",
    description: "",
  },
  tutorial_completed_tip: {
    title: "",
    description: "",
  },
};

const iconMap: Record<string, LucideIcon> = {
  Building2,
  Users,
  FileText,
  DollarSign,
  FolderKanban,
  Zap,
  TrendingUp,
  Home,
  FileSpreadsheet,
  Clock,
  Wallet,
  Package,
};

const DEFAULT_TUTORIAL_DISMISSAL_KEY = "onboarding-tutorial-dismissed:global";

function BackendBadge({ badge }: { badge?: DisplayBadge | null }) {
  if (!badge) return null;
  return (
    <Badge variant={badge.variant || "outline"} className={badge.class_name || ""}>
      {badge.label}
    </Badge>
  );
}

function StepIcon({
  iconKey,
  className,
}: {
  iconKey: string;
  className: string;
}) {
  const Icon = iconMap[iconKey] || Building2;
  return <Icon className={className} />;
}

export function OnboardingTutorial() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [dismissed, setDismissed] = useState(false);
  const [companyName, setCompanyName] = useState("");

  const { data } = useQuery({
    queryKey: ["onboarding-progress"],
    queryFn: () => api.get<OnboardingProgress>(API_ENDPOINTS.onboardingProgress),
  });

  const progress = data || emptyProgress;
  const activeStep =
    progress.tutorial_steps[currentStep] || progress.tutorial_steps[0];
  const activation = progress.summary.activation || emptyActivation;
  const activationLoop =
    progress.summary.activation_loop || emptyActivationLoop;
  const operatingBackbone =
    progress.summary.operating_backbone || emptyOperatingBackbone;
  const activationGateCompleted =
    activationLoop.total_count > 0 ? activationLoop.completed : activation.completed;
  const operatingBackboneCompleted =
    operatingBackbone.total_count > 0 ? operatingBackbone.completed : true;
  const onboardingGateCompleted =
    activationGateCompleted && operatingBackboneCompleted;
  const activationGateRemaining =
    activationLoop.total_count > 0
      ? activationLoop.remaining_count
      : activation.remaining_count;
  const onboardingGateRemaining = activationGateCompleted
    ? operatingBackbone.remaining_count
    : activationGateRemaining;
  const cta = progress.summary.cta;
  const tutorialStorageKey =
    progress.summary.tutorial_storage_key || DEFAULT_TUTORIAL_DISMISSAL_KEY;

  // Create the user's own company straight from the demo CTA, then drop them on
  // the company profile (step 1 of the value path).
  const createCompany = useMutation({
    mutationFn: async (name: string) => companies.create({ name }),
    onSuccess: async (result) => {
      try {
        await companies.switch(result.id);
      } catch (_err) {
        // best-effort switch; the new company is still created
      }
      setCurrentCompany(result.id);
      setCompanyName("");
      setOpen(false);
      toast.success(`公司 "${result.name}" 创建成功，开始完善信息`);
      await queryClient.invalidateQueries();
      navigate("/company-info");
    },
    onError: () => {
      toast.error("创建公司失败，请重试");
    },
  });

  useEffect(() => {
    if (!data) return;

    const storageKey =
      data.summary.tutorial_storage_key || DEFAULT_TUTORIAL_DISMISSAL_KEY;
    const tutorialDismissed = localStorage.getItem(storageKey);
    if (tutorialDismissed === "true") {
      setDismissed(true);
      setOpen(false);
      return;
    }

    setDismissed(false);
    const loopCompleted =
      data.summary.activation_loop?.total_count > 0
        ? data.summary.activation_loop.completed
        : data.summary.activation?.completed;
    const backboneCompleted =
      data.summary.operating_backbone?.total_count > 0
        ? data.summary.operating_backbone.completed
        : true;
    if (loopCompleted && backboneCompleted && !data.summary.demo_company) {
      setOpen(false);
      return;
    }

    setOpen(true);
    setCurrentStep(
      data.summary.demo_company ? 0 : data.summary.first_incomplete_index || 0,
    );
  }, [data]);

  const handleDismiss = () => {
    localStorage.setItem(tutorialStorageKey, "true");
    setDismissed(true);
    setOpen(false);
  };

  const handleReopen = () => {
    localStorage.removeItem(tutorialStorageKey);
    setDismissed(false);
    setOpen(true);
  };

  const handleStepAction = () => {
    if (!activeStep) return;
    navigate(activeStep.route);
    setOpen(false);
  };

  const handleActivationAction = () => {
    if (!activationLoop.next_action?.route) return;
    navigate(activationLoop.next_action.route);
    setOpen(false);
  };

  const handleOperatingBackboneAction = () => {
    if (!operatingBackbone.next_action?.route) return;
    navigate(operatingBackbone.next_action.route);
    setOpen(false);
  };

  const handleNext = () => {
    if (currentStep < progress.tutorial_steps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleCreateCompany = () => {
    if (companyName.trim()) {
      createCompany.mutate(companyName.trim());
    }
  };

  if (!data) {
    return null;
  }

  if (onboardingGateCompleted && !progress.summary.demo_company && !open) {
    return null;
  }

  if (dismissed && (!onboardingGateCompleted || progress.summary.demo_company)) {
    return (
      <Button
        onClick={handleReopen}
        className="fixed bottom-6 right-6 z-50 rounded-full shadow-lg"
        size="lg"
      >
        <Zap className="h-5 w-5 mr-2" />
        {progress.summary.demo_company ? "查看 Demo 教程" : "继续新手引导"}
        {!progress.summary.demo_company && onboardingGateRemaining > 0 && (
          <Badge variant="destructive" className="ml-2">
            {onboardingGateRemaining}
          </Badge>
        )}
      </Button>
    );
  }

  if (!activeStep) {
    return null;
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Rocket className="h-6 w-6 text-primary" />
            <DialogTitle className="text-2xl">新手引导 · 激活与运营骨架</DialogTitle>
          </div>
          <DialogDescription>
            先跑通获客到收款。做完一步，早晨工作台会指出下一步；证照和保单补上后，停业风险也会出现在同一页。
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Demo CTA — foreground "create my company" while exploring sample data */}
          {cta && (
            <Card className="border-2 border-amber-400/60 bg-amber-50 dark:bg-amber-950/20">
              <CardContent className="pt-6 space-y-3">
                <div className="flex items-start gap-3">
                  <Sparkles className="h-6 w-6 text-amber-500 flex-shrink-0" />
                  <div className="space-y-1">
                    <h3 className="font-semibold text-amber-900 dark:text-amber-200">
                      {cta.title}
                    </h3>
                    <p className="text-sm text-amber-800/80 dark:text-amber-200/70">
                      {cta.description}
                    </p>
                  </div>
                </div>
                <div className="flex flex-col sm:flex-row gap-2">
                  <Input
                    value={companyName}
                    onChange={(e) => setCompanyName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleCreateCompany()}
                    placeholder={cta.input_placeholder}
                  />
                  <Button
                    onClick={handleCreateCompany}
                    disabled={!companyName.trim() || createCompany.isPending}
                    className="whitespace-nowrap"
                  >
                    {createCompany.isPending ? "创建中..." : cta.action_label}
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">{activation.label}</span>
              <span className="font-medium">{activation.rate_label}</span>
            </div>
            <Progress value={activation.rate} className="h-2" />
          </div>

          {activationLoop.total_count > 0 && (
            <div className="space-y-3 rounded-lg border bg-muted/30 p-4">
              <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <Wallet className="h-5 w-5 text-primary" />
                    <h3 className="font-semibold">{activationLoop.title}</h3>
                    <BackendBadge badge={activationLoop.status_badge} />
                  </div>
                  {activationLoop.next_action?.description && (
                    <p className="mt-1 text-sm text-muted-foreground">
                      {activationLoop.next_action.description}
                    </p>
                  )}
                </div>
                <div className="text-sm font-medium">
                  {activationLoop.rate_label}
                </div>
              </div>
              <Progress value={activationLoop.rate} className="h-2" />
              <div className="grid gap-2">
                {activationLoop.steps.map((step) => (
                  <div
                    key={step.id}
                    className={cn(
                      "flex items-center gap-3 rounded-md border px-3 py-2 text-sm",
                      activationLoop.current_step_id === step.id
                        ? "border-primary/60 bg-primary/5"
                        : "border-border bg-background",
                    )}
                  >
                    {step.completed ? (
                      <CheckCircle2 className="h-4 w-4 flex-shrink-0 text-green-600" />
                    ) : (
                      <Circle className="h-4 w-4 flex-shrink-0 text-muted-foreground" />
                    )}
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{step.label}</span>
                        <BackendBadge badge={step.status_badge} />
                      </div>
                      <div className="truncate text-xs text-muted-foreground">
                        {step.evidence_label}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
              {activationLoop.next_action?.route && (
                <Button
                  onClick={handleActivationAction}
                  className="w-full sm:w-auto"
                >
                  {activationLoop.next_action.label}
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              )}
            </div>
          )}

          {activationLoop.completed && operatingBackbone.total_count > 0 && (
            <div className="space-y-3 rounded-lg border bg-muted/30 p-4">
              <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <FolderKanban className="h-5 w-5 text-primary" />
                    <h3 className="font-semibold">{operatingBackbone.title}</h3>
                    <BackendBadge badge={operatingBackbone.status_badge} />
                  </div>
                  {operatingBackbone.next_action?.description && (
                    <p className="mt-1 text-sm text-muted-foreground">
                      {operatingBackbone.next_action.description}
                    </p>
                  )}
                </div>
                <div className="text-sm font-medium">
                  {operatingBackbone.rate_label}
                </div>
              </div>
              <Progress value={operatingBackbone.rate} className="h-2" />
              <div className="grid gap-2">
                {operatingBackbone.steps.map((step) => (
                  <div
                    key={step.id}
                    className={cn(
                      "flex items-center gap-3 rounded-md border px-3 py-2 text-sm",
                      operatingBackbone.current_step_id === step.id
                        ? "border-primary/60 bg-primary/5"
                        : "border-border bg-background",
                    )}
                  >
                    {step.completed ? (
                      <CheckCircle2 className="h-4 w-4 flex-shrink-0 text-green-600" />
                    ) : (
                      <Circle className="h-4 w-4 flex-shrink-0 text-muted-foreground" />
                    )}
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{step.label}</span>
                        <BackendBadge badge={step.status_badge} />
                      </div>
                      <div className="truncate text-xs text-muted-foreground">
                        {step.evidence_label}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
              {operatingBackbone.next_action?.route && (
                <Button
                  onClick={handleOperatingBackboneAction}
                  className="w-full sm:w-auto"
                >
                  {operatingBackbone.next_action.label}
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              )}
            </div>
          )}

          <Card className="border-2 border-primary">
            <CardContent className="pt-6">
              <div className="flex items-start gap-4">
                <div
                  className={cn(
                    "rounded-full p-3",
                    activeStep.completed
                      ? "bg-green-100 text-green-600"
                      : "bg-primary/10 text-primary",
                  )}
                >
                  <StepIcon iconKey={activeStep.icon_key} className="h-5 w-5" />
                </div>
                <div className="flex-1 space-y-3">
                  <div className="flex items-center gap-2">
                    <h3 className="text-lg font-semibold">
                      {currentStep + 1}. {activeStep.title}
                    </h3>
                    {activeStep.completed && (
                      <BackendBadge badge={activeStep.status_badge} />
                    )}
                  </div>
                  <p className="text-muted-foreground">
                    {activeStep.description}
                  </p>
                  <Button
                    onClick={handleStepAction}
                    className="w-full sm:w-auto"
                  >
                    {activeStep.action_label}
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="space-y-2">
            <h4 className="text-sm font-medium text-muted-foreground mb-3">
              所有步骤
            </h4>
            <div className="grid gap-2">
              {progress.tutorial_steps.map((step, index) => (
                <div key={step.id}>
                  {step.section_title && (
                    <div className="text-xs font-semibold text-muted-foreground/80 uppercase tracking-wide mt-3 mb-1">
                      {step.section_title}
                    </div>
                  )}
                  <button
                    aria-label={step.action_label}
                    onClick={() => setCurrentStep(index)}
                    className={cn(
                      "w-full flex items-center gap-3 p-3 rounded-lg border-2 transition-all text-left",
                      currentStep === index
                        ? "border-primary bg-primary/5"
                        : "border-transparent hover:border-border hover:bg-accent",
                    )}
                  >
                    {step.completed ? (
                      <CheckCircle2 className="h-5 w-5 text-green-600 flex-shrink-0" />
                    ) : (
                      <Circle className="h-5 w-5 text-muted-foreground flex-shrink-0" />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium truncate">{step.title}</span>
                        <BackendBadge badge={step.status_badge} />
                      </div>
                      <div className="text-xs text-muted-foreground truncate">
                        {step.description}
                      </div>
                    </div>
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="flex items-center justify-between pt-4 border-t">
            <Button
              variant="outline"
              onClick={handlePrevious}
              disabled={currentStep === 0}
            >
              上一步
            </Button>
            <div className="flex items-center gap-3">
              <Button variant="ghost" onClick={handleDismiss}>
                稍后再看
              </Button>
              <div className="text-sm text-muted-foreground">
                {currentStep + 1} / {progress.tutorial_steps.length}
              </div>
            </div>
            <Button
              onClick={handleNext}
              disabled={currentStep === progress.tutorial_steps.length - 1}
            >
              下一步
            </Button>
          </div>

          {activationGateCompleted && (
            <Card className="bg-green-50 border-green-200">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3">
                  <CheckCircle2 className="h-6 w-6 text-green-600" />
                  <div>
                    <h4 className="font-semibold text-green-900">
                      {progress.tutorial_completed_tip.title}
                    </h4>
                    <p className="text-sm text-green-700">
                      {progress.tutorial_completed_tip.description}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>

        {!progress.summary.all_completed && progress.incomplete_tip.title && (
          <div className="sr-only">{progress.incomplete_tip.title}</div>
        )}
        {progress.completed_tip.title && (
          <div className="sr-only">{progress.completed_tip.title}</div>
        )}
      </DialogContent>
    </Dialog>
  );
}
