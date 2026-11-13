import { useState, useEffect, createContext, useContext, ReactNode } from 'react';
import { AlertCircle, Wifi, WifiOff } from 'lucide-react';
import { cn } from '@/lib/utils';

interface NetworkStatusContextType {
    isOnline: boolean;
    wasOffline: boolean;
}

const NetworkStatusContext = createContext<NetworkStatusContextType>({
    isOnline: true,
    wasOffline: false,
});

export function useNetworkStatus() {
    return useContext(NetworkStatusContext);
}

interface NetworkStatusProviderProps {
    children: ReactNode;
}

/**
 * 网络状态提供者
 * 监听浏览器的 online/offline 事件，提供全局网络状态
 */
export function NetworkStatusProvider({ children }: NetworkStatusProviderProps) {
    const [isOnline, setIsOnline] = useState(
        typeof navigator !== 'undefined' ? navigator.onLine : true
    );
    const [wasOffline, setWasOffline] = useState(false);

    useEffect(() => {
        const handleOnline = () => {
            setIsOnline(true);
            // 标记曾经离线过，用于显示恢复提示
            if (!isOnline) {
                setWasOffline(true);
                // 3秒后清除恢复提示
                setTimeout(() => setWasOffline(false), 3000);
            }
        };

        const handleOffline = () => {
            setIsOnline(false);
        };

        window.addEventListener('online', handleOnline);
        window.addEventListener('offline', handleOffline);

        return () => {
            window.removeEventListener('online', handleOnline);
            window.removeEventListener('offline', handleOffline);
        };
    }, [isOnline]);

    return (
        <NetworkStatusContext.Provider value={{ isOnline, wasOffline }}>
            {children}
        </NetworkStatusContext.Provider>
    );
}

/**
 * 网络状态指示条
 * 在离线时显示提示，恢复在线时显示恢复提示
 */
export function NetworkStatusBar() {
    const { isOnline, wasOffline } = useNetworkStatus();
    const [visible, setVisible] = useState(false);

    useEffect(() => {
        if (!isOnline) {
            setVisible(true);
        } else if (wasOffline) {
            setVisible(true);
            const timer = setTimeout(() => setVisible(false), 3000);
            return () => clearTimeout(timer);
        } else {
            setVisible(false);
        }
    }, [isOnline, wasOffline]);

    if (!visible) return null;

    return (
        <div
            className={cn(
                'fixed top-0 left-0 right-0 z-50 py-2 px-4 text-center text-sm font-medium transition-all duration-300',
                !isOnline
                    ? 'bg-destructive text-destructive-foreground'
                    : 'bg-green-500 text-white'
            )}
        >
            <div className="flex items-center justify-center gap-2">
                {!isOnline ? (
                    <>
                        <WifiOff className="h-4 w-4" />
                        <span>网络已断开，部分功能可能不可用</span>
                    </>
                ) : (
                    <>
                        <Wifi className="h-4 w-4" />
                        <span>网络已恢复</span>
                    </>
                )}
            </div>
        </div>
    );
}

/**
 * 简化的离线状态组件 - 用于内联显示
 */
export function OfflineIndicator() {
    const { isOnline } = useNetworkStatus();

    if (isOnline) return null;

    return (
        <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted px-2 py-1 rounded">
            <AlertCircle className="h-3 w-3" />
            <span>离线模式</span>
        </div>
    );
}
