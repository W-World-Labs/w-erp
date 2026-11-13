import { api, auth } from "@/integrations/api/client";

type OperationType = 'CREATE' | 'UPDATE' | 'DELETE' | 'VIEW';

interface LogOptions {
  operationType: OperationType;
  tableName: string;
  recordId?: string;
  operationDetail?: Record<string, any>;
}

export const logOperation = async (options: LogOptions) => {
  try {
    if (!auth.isAuthenticated()) return;
    const user = await auth.getCurrentUser();
    if (!user) return;

    await api.post('/api/logs', {
      user_id: user.id,
      operation_type: options.operationType,
      table_name: options.tableName,
      record_id: options.recordId,
      operation_detail: options.operationDetail,
      ip_address: null,
      user_agent: navigator.userAgent,
    });
  } catch (error) {
    console.error('记录操作日志失败:', error);
    // 静默失败，不影响主流程
  }
};

// 只记录关键操作
export const logSensitiveOperation = async (
  type: OperationType,
  table: string,
  recordId?: string,
  detail?: Record<string, any>
) => {
  // 只记录以下表的操作
  const sensitiveTables = [
    'employees', 'salaries', 'contracts',
    'invoices', 'payments', 'company_info',
    'user_roles', 'tax_declarations', 'budgets'
  ];

  if (sensitiveTables.includes(table)) {
    await logOperation({
      operationType: type,
      tableName: table,
      recordId,
      operationDetail: detail,
    });
  }
};
