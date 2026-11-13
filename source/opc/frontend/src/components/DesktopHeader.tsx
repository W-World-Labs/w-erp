import { NotificationCenter } from "@/components/NotificationCenter";
import { UserMenu } from "@/components/UserMenu";
import { ProgressTracker } from "@/components/tutorial/ProgressTracker";

export function DesktopHeader() {
  return (
    <header className="hidden lg:block sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container flex h-14 items-center justify-between px-6">
        <div className="flex-1">
          <ProgressTracker />
        </div>
        
        <div className="flex items-center gap-2">
          <NotificationCenter />
          <UserMenu />
        </div>
      </div>
    </header>
  );
}
