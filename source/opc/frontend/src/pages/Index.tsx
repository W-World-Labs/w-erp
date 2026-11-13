import Sidebar from "@/components/Sidebar";
import { Outlet } from "react-router-dom";
import { MobileHeader } from "@/components/MobileHeader";
import { DesktopHeader } from "@/components/DesktopHeader";
import { AICopilot } from "@/components/ai/AICopilot";

const Index = () => {
  return (
    <div className="flex h-screen bg-background">
      <div className="hidden lg:block">
        <Sidebar />
      </div>
      <div className="flex-1 flex flex-col overflow-hidden">
        <MobileHeader />
        <DesktopHeader />
        <main className="flex-1 overflow-y-auto">
          <div className="container py-6">
            <Outlet />
          </div>
        </main>
      </div>
      {/* AI 副驾驶 */}
      <AICopilot />
    </div>
  );
};

export default Index;
