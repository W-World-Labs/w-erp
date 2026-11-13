import { Sparkles, Loader2, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface AutoParseButtonProps {
    file: File | null;
    isParsing: boolean;
    error: string | null;
    onParse: () => void;
    className?: string;
    enabled?: boolean;
}

export function AutoParseButton({ file, isParsing, error, onParse, className = '', enabled = true }: AutoParseButtonProps) {
    if (!enabled || !file) return null;

    return (
        <div className={`flex items-center gap-2 ${className}`}>
            <TooltipProvider>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={onParse}
                            disabled={isParsing || !file}
                            className="gap-2 border-dashed border-primary/50 text-primary hover:bg-primary/5"
                        >
                            {isParsing ? (
                                <>
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                    AI 识别中...
                                </>
                            ) : (
                                <>
                                    <Sparkles className="h-4 w-4" />
                                    AI 智能识别
                                </>
                            )}
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                        <p>使用 AI 自动识别文档内容并填入表单</p>
                    </TooltipContent>
                </Tooltip>
            </TooltipProvider>

            {error && (
                <span className="text-xs text-destructive flex items-center gap-1">
                    <AlertCircle className="h-3 w-3" />
                    {error}
                </span>
            )}
        </div>
    );
}
