import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Plus,
  Cloud,
  Server,
  Globe,
  Package,
  DollarSign,
  TrendingUp,
} from "lucide-react";
import CloudProviderDialog from "@/components/cloud-cost/CloudProviderDialog";
import CloudResourceDialog from "@/components/cloud-cost/CloudResourceDialog";
import CloudBillDialog from "@/components/cloud-cost/CloudBillDialog";
import DomainSubscriptionDialog from "@/components/cloud-cost/DomainSubscriptionDialog";
import SoftwareSubscriptionDialog from "@/components/cloud-cost/SoftwareSubscriptionDialog";
import CloudProviderList from "@/components/cloud-cost/CloudProviderList";
import CloudResourceList from "@/components/cloud-cost/CloudResourceList";
import CloudBillList from "@/components/cloud-cost/CloudBillList";
import DomainSubscriptionList from "@/components/cloud-cost/DomainSubscriptionList";
import SoftwareSubscriptionList from "@/components/cloud-cost/SoftwareSubscriptionList";
import CostOverviewStats from "@/components/cloud-cost/CostOverviewStats";

const CloudCostManagement = () => {
  const [activeTab, setActiveTab] = useState("overview");
  const [providerDialogOpen, setProviderDialogOpen] = useState(false);
  const [resourceDialogOpen, setResourceDialogOpen] = useState(false);
  const [billDialogOpen, setBillDialogOpen] = useState(false);
  const [domainDialogOpen, setDomainDialogOpen] = useState(false);
  const [softwareDialogOpen, setSoftwareDialogOpen] = useState(false);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const handleRefresh = () => {
    setRefreshTrigger((prev) => prev + 1);
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">云成本管理</h1>
          <p className="text-muted-foreground mt-1">
            统一管理云服务、域名、软件订阅等成本
          </p>
        </div>
      </div>

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-6"
      >
        <TabsList className="grid w-full grid-cols-6">
          <TabsTrigger value="overview">
            <TrendingUp className="h-4 w-4 mr-2" />
            总览
          </TabsTrigger>
          <TabsTrigger value="providers">
            <Cloud className="h-4 w-4 mr-2" />
            云服务商
          </TabsTrigger>
          <TabsTrigger value="resources">
            <Server className="h-4 w-4 mr-2" />
            云资源
          </TabsTrigger>
          <TabsTrigger value="bills">
            <DollarSign className="h-4 w-4 mr-2" />
            账单
          </TabsTrigger>
          <TabsTrigger value="domains">
            <Globe className="h-4 w-4 mr-2" />
            域名
          </TabsTrigger>
          <TabsTrigger value="software">
            <Package className="h-4 w-4 mr-2" />
            软件订阅
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6">
          <CostOverviewStats key={refreshTrigger} />
        </TabsContent>

        <TabsContent value="providers" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-semibold">云服务商账户</h2>
              <p className="text-muted-foreground">
                管理阿里云、AWS、腾讯云等账户
              </p>
            </div>
            <Button onClick={() => setProviderDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              添加云账户
            </Button>
          </div>
          <CloudProviderList
            key={refreshTrigger}
            onEdit={() => handleRefresh()}
          />
        </TabsContent>

        <TabsContent value="resources" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-semibold">云资源实例</h2>
              <p className="text-muted-foreground">
                服务器、存储、数据库等资源
              </p>
            </div>
            <Button onClick={() => setResourceDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              添加资源
            </Button>
          </div>
          <CloudResourceList
            key={refreshTrigger}
            onEdit={() => handleRefresh()}
          />
        </TabsContent>

        <TabsContent value="bills" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-semibold">云账单</h2>
              <p className="text-muted-foreground">账单明细和支付管理</p>
            </div>
            <Button onClick={() => setBillDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              录入账单
            </Button>
          </div>
          <CloudBillList key={refreshTrigger} onEdit={() => handleRefresh()} />
        </TabsContent>

        <TabsContent value="domains" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-semibold">域名订阅</h2>
              <p className="text-muted-foreground">域名注册和续费管理</p>
            </div>
            <Button onClick={() => setDomainDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              添加域名
            </Button>
          </div>
          <DomainSubscriptionList
            key={refreshTrigger}
            onEdit={() => handleRefresh()}
          />
        </TabsContent>

        <TabsContent value="software" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-semibold">软件订阅</h2>
              <p className="text-muted-foreground">SaaS工具和软件许可</p>
            </div>
            <Button onClick={() => setSoftwareDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              添加订阅
            </Button>
          </div>
          <SoftwareSubscriptionList
            key={refreshTrigger}
            onEdit={() => handleRefresh()}
          />
        </TabsContent>
      </Tabs>

      <CloudProviderDialog
        open={providerDialogOpen}
        onOpenChange={setProviderDialogOpen}
        onSuccess={handleRefresh}
      />
      <CloudResourceDialog
        open={resourceDialogOpen}
        onOpenChange={setResourceDialogOpen}
        onSuccess={handleRefresh}
      />
      <CloudBillDialog
        open={billDialogOpen}
        onOpenChange={setBillDialogOpen}
        onSuccess={handleRefresh}
      />
      <DomainSubscriptionDialog
        open={domainDialogOpen}
        onOpenChange={setDomainDialogOpen}
        onSuccess={handleRefresh}
      />
      <SoftwareSubscriptionDialog
        open={softwareDialogOpen}
        onOpenChange={setSoftwareDialogOpen}
        onSuccess={handleRefresh}
      />
    </div>
  );
};

export default CloudCostManagement;
