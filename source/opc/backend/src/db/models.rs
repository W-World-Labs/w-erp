use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub github_login: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Asset {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price: Option<f64>,
    pub current_value: Option<f64>,
    pub expiry_date: Option<String>,
    pub category_id: Option<String>,
    pub image_url: Option<String>,
    pub useful_life: Option<i64>,
    pub salvage_rate: Option<f64>,
    pub depreciation_method: Option<String>,
    pub monthly_depreciation: Option<f64>,
    pub accumulated_depreciation: Option<f64>,
    pub invoice_id: Option<String>,
    pub contract_id: Option<String>,
    pub registration_number: Option<String>,
    pub holder_name: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: String,
    pub company_id: String,
    pub invoice_number: String,
    pub invoice_type: Option<String>,
    pub amount: f64,
    pub tax_amount: Option<f64>,
    pub status: Option<String>,
    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub supplier: Option<String>,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub contract_id: Option<String>,
    pub currency: Option<String>,
    pub client_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Employee {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub full_name: Option<String>,
    pub employee_number: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub hire_date: Option<String>,
    pub status: Option<String>,
    pub employment_type: Option<String>,
    pub base_salary: Option<f64>,
    pub social_security_base: Option<f64>,
    pub housing_fund_base: Option<f64>,
    pub notes: Option<String>,
    pub avatar_url: Option<String>,
    pub contract_start_date: Option<String>,
    pub contract_end_date: Option<String>,
    pub bank_account: Option<String>,
    pub bank_name: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub id_number: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Budget {
    pub id: String,
    pub company_id: String,
    pub category: String,
    pub budget_type: String,
    pub period: String,
    pub budget_amount: Option<f64>,
    pub actual_amount: Option<f64>,
    pub status: Option<String>,
    pub project_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BankAccount {
    pub id: String,
    pub company_id: String,
    pub bank_name: String,
    pub account_name: String,
    pub account_number: String,
    pub account_type: Option<String>,
    pub balance: Option<f64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contract {
    pub id: String,
    pub company_id: String,
    pub contract_name: String,
    pub contract_number: Option<String>,
    pub contract_type: Option<String>,
    pub counterparty: Option<String>,
    pub amount: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
