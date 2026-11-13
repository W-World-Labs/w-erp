import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { reportErrorFromException } from '@/lib/errorReporter';

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
    onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
    hasError: boolean;
    error: Error | null;
    errorInfo: ErrorInfo | null;
}

/**
 * 全局错误边界组件
 * 捕获子组件树中的 JavaScript 错误，显示备用 UI，防止整个应用白屏
 */
export class ErrorBoundary extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<State> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
        // 记录错误日志
        console.error('ErrorBoundary caught an error:', error);
        console.error('Error info:', errorInfo);

        this.setState({ errorInfo });

        // 自动上报到数据库
        reportErrorFromException(error, 'render_crash', {
            componentStack: errorInfo.componentStack || undefined,
        });

        // 调用外部错误处理回调
        this.props.onError?.(error, errorInfo);
    }

    handleReset = (): void => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    handleGoHome = (): void => {
        this.handleReset();
        window.location.href = '/';
    };

    handleRefresh = (): void => {
        window.location.reload();
    };

    render(): ReactNode {
        if (this.state.hasError) {
            // 如果提供了自定义 fallback，使用它
            if (this.props.fallback) {
                return this.props.fallback;
            }

            // 默认错误 UI — 始终显示错误详情便于调试
            return (
                <div className="min-h-screen flex items-center justify-center bg-background p-4">
                    <Card className="max-w-2xl w-full">
                        <CardHeader className="text-center">
                            <div className="mx-auto mb-4 h-16 w-16 rounded-full bg-destructive/10 flex items-center justify-center">
                                <AlertTriangle className="h-8 w-8 text-destructive" />
                            </div>
                            <CardTitle className="text-xl">页面出现错误</CardTitle>
                            <CardDescription>
                                {this.state.error?.message || '未知错误'}
                            </CardDescription>
                        </CardHeader>

                        <CardContent className="space-y-3">
                            {/* 错误消息 */}
                            {this.state.error && (
                                <div className="p-3 bg-destructive/5 border border-destructive/20 rounded-lg">
                                    <div className="flex items-center justify-between mb-1">
                                        <span className="text-xs font-semibold text-destructive uppercase tracking-wide">Error</span>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            className="h-6 text-xs"
                                            onClick={() => {
                                                const text = [
                                                    this.state.error?.toString(),
                                                    '\n--- Stack Trace ---',
                                                    this.state.error?.stack,
                                                    '\n--- Component Stack ---',
                                                    this.state.errorInfo?.componentStack,
                                                ].filter(Boolean).join('\n');
                                                navigator.clipboard.writeText(text);
                                            }}
                                        >
                                            复制错误信息
                                        </Button>
                                    </div>
                                    <p className="text-sm font-mono text-destructive break-all">
                                        {this.state.error.toString()}
                                    </p>
                                </div>
                            )}

                            {/* JS 堆栈追踪 */}
                            {this.state.error?.stack && (
                                <details className="group">
                                    <summary className="cursor-pointer text-xs font-semibold text-muted-foreground uppercase tracking-wide hover:text-foreground">
                                        ▸ Stack Trace
                                    </summary>
                                    <pre className="mt-2 p-3 bg-muted rounded-lg text-xs font-mono text-muted-foreground whitespace-pre-wrap overflow-auto max-h-48">
                                        {this.state.error.stack}
                                    </pre>
                                </details>
                            )}

                            {/* React 组件堆栈 */}
                            {this.state.errorInfo?.componentStack && (
                                <details className="group">
                                    <summary className="cursor-pointer text-xs font-semibold text-muted-foreground uppercase tracking-wide hover:text-foreground">
                                        ▸ Component Stack
                                    </summary>
                                    <pre className="mt-2 p-3 bg-muted rounded-lg text-xs font-mono text-muted-foreground whitespace-pre-wrap overflow-auto max-h-48">
                                        {this.state.errorInfo.componentStack}
                                    </pre>
                                </details>
                            )}
                        </CardContent>

                        <CardFooter className="flex justify-center gap-3">
                            <Button variant="outline" onClick={this.handleGoHome}>
                                <Home className="h-4 w-4 mr-2" />
                                返回首页
                            </Button>
                            <Button onClick={this.handleRefresh}>
                                <RefreshCw className="h-4 w-4 mr-2" />
                                刷新页面
                            </Button>
                        </CardFooter>
                    </Card>
                </div>
            );
        }

        return this.props.children;
    }
}

/**
 * 页面级别错误边界 — 复用全局 ErrorBoundary 的完整错误展示
 * 不再传入 fallback，这样它会自动使用增强版的错误 UI（含堆栈追踪）
 */
export function PageErrorBoundary({ children }: { children: ReactNode }) {
    return (
        <ErrorBoundary>
            {children}
        </ErrorBoundary>
    );
}
