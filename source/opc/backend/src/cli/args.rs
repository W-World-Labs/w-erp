use clap::{Parser, Subcommand};

use super::helpers::parse_kv;

#[derive(Parser)]
#[command(
    name = "opc",
    version,
    about = "OPC — CLI for one-person companies",
    long_about = "OPC — Full-featured command line interface for one-person companies.\n\
         \n\
         Connect to a running opc-server and manage all business modules:\n\
         invoices, contracts, employees, assets, bank accounts, budgets, tax,\n\
         CRM, projects, HR, cloud resources, KPI, and 30+ more entities.\n\
         \n\
         QUICK START:\n\
           1. Start the backend:  cargo run --bin opc-server\n\
           2. Login:              opc auth login\n\
           3. Create a company:   opc company create --name \"My Corp\"\n\
           4. Add data:           opc create invoices -f amount=5000 -f invoice_number=INV-001\n\
           5. View dashboard:     opc dashboard"
)]
pub struct Cli {
    /// Backend server URL
    #[arg(long, env = "OPC_SERVER", default_value = "http://localhost:8987")]
    pub(crate) server: String,

    /// Session ID for authentication
    #[arg(long, env = "OPC_SESSION")]
    pub(crate) session: Option<String>,

    /// API Key for authentication (alternative to session)
    #[arg(long, env = "OPC_API_KEY")]
    pub(crate) api_key: Option<String>,

    /// Output format: table, json, csv
    #[arg(long, default_value = "table")]
    pub(crate) format: String,

    /// Quiet mode — only output IDs (for scripting)
    #[arg(short, long)]
    pub(crate) quiet: bool,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// System health check
    Status,
    /// Dashboard overview
    Dashboard,
    /// View operation logs
    Logs {
        #[arg(long, default_value = "20")]
        limit: i32,
    },

    // ── Backup ───────────────────────────────────
    /// Built-in data backup / restore
    Backup {
        #[command(subcommand)]
        action: BackupAction,
    },

    // ── Auth ──────────────────────────────────────
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    // ── Company Management ───────────────────────
    /// Company management
    Company {
        #[command(subcommand)]
        action: CompanyAction,
    },

    // ── Bank Module ──────────────────────────────
    /// Bank accounts & transactions
    Bank {
        #[command(subcommand)]
        action: BankAction,
    },

    // ── AI Module ────────────────────────────────
    /// AI-powered analysis
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    // ── Reports ──────────────────────────────────
    /// Financial reports
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },

    // ── PDF Generation ───────────────────────────
    /// Generate PDF documents
    Pdf {
        #[command(subcommand)]
        action: PdfAction,
    },

    // ── Policies ─────────────────────────────────
    /// Search global startup policies
    Policies {
        #[command(subcommand)]
        action: PolicyAction,
    },

    /// Global OPC-friendly registration locations
    Locations {
        #[command(subcommand)]
        action: Option<OpcAction>,
    },

    // ── File Management ──────────────────────────
    /// File upload/download
    Files {
        #[command(subcommand)]
        action: FileAction,
    },

    // ── API Key Management ──────────────────────
    /// Manage API keys
    ApiKeys {
        #[command(subcommand)]
        action: ApiKeyAction,
    },

    // ── Tax Tools ────────────────────────────────
    /// Tax calculation & compliance tools
    Tax {
        #[command(subcommand)]
        action: TaxToolAction,
    },

    // ── Currency ─────────────────────────────────
    /// Multi-currency tools
    Currency {
        #[command(subcommand)]
        action: CurrencyAction,
    },

    // ── AR (Accounts Receivable) ─────────────────
    /// Accounts receivable analysis
    Ar {
        #[command(subcommand)]
        action: ArAction,
    },

    // ── Webhooks ─────────────────────────────────
    /// Webhook management
    Webhooks {
        #[command(subcommand)]
        action: WebhookAction,
    },

    /// Convert time entries to invoice
    TimeToInvoice {
        /// Time entry IDs (comma-separated)
        #[arg(long)]
        entries: String,
        /// Hourly rate
        #[arg(long)]
        rate: Option<f64>,
    },

    /// Convert quote to invoice
    QuoteToInvoice {
        /// Quote ID to convert
        quote_id: String,
    },

    /// List records for any entity
    List {
        /// Entity name (e.g., invoices, contracts, employees, payments, assets)
        entity: String,
        /// Max records to return
        #[arg(long, default_value = "20")]
        limit: i32,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<i32>,
    },
    /// Get a single record by ID
    Get {
        /// Entity name (e.g., contracts, invoices)
        entity: String,
        /// Record UUID
        id: String,
    },
    /// Create a new record
    Create {
        /// Entity name (e.g., contracts, invoices, employees)
        entity: String,
        /// JSON body: '{"field": "value"}'
        #[arg(long)]
        json: Option<String>,
        /// Key=value pairs (repeatable): -f name=张三 -f amount=5000
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    /// Update an existing record
    Update {
        /// Entity name
        entity: String,
        /// Record UUID to update
        id: String,
        /// JSON body with fields to update
        #[arg(long)]
        json: Option<String>,
        /// Key=value pairs to update (repeatable)
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    /// Delete a record by ID
    Delete {
        /// Entity name
        entity: String,
        /// Record UUID to delete
        id: String,
    },
    /// Export entity data as CSV
    Export {
        /// Entity/table name to export
        entity: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum AuthAction {
    /// Login with GitHub OAuth (opens browser)
    Login,
    /// Show current session status
    Status,
}

#[derive(Subcommand)]
pub(crate) enum CompanyAction {
    /// List all companies
    List,
    /// Create a new company
    Create {
        #[arg(long)]
        json: Option<String>,
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    /// Get company details
    Get { id: String },
    /// Update company
    Update {
        id: String,
        #[arg(long)]
        json: Option<String>,
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    /// Delete (soft-delete) company
    Delete { id: String },
    /// Switch active company
    Switch { id: String },
    /// Manage company members
    Members {
        #[command(subcommand)]
        action: MemberAction,
    },
}

#[derive(Subcommand)]
pub(crate) enum BackupAction {
    /// Create a backup of SQLite + uploads (POST /settings/backups)
    Create,
    /// List available backups (GET /settings/backups)
    List,
}

#[derive(Subcommand)]
pub(crate) enum MemberAction {
    List {
        /// Company UUID
        company_id: String,
    },
    Add {
        /// Company UUID
        company_id: String,
        #[arg(long)]
        email: String,
        #[arg(long, default_value = "member")]
        role: String,
    },
    Remove {
        /// Company UUID
        company_id: String,
        #[arg(long)]
        member_id: String,
    },
    UpdateRole {
        /// Company UUID
        company_id: String,
        #[arg(long)]
        member_id: String,
        #[arg(long)]
        role: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum BankAction {
    /// List bank accounts
    Accounts,
    /// Create bank account
    CreateAccount {
        #[arg(long)]
        json: Option<String>,
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    /// Create a transaction
    Transact {
        #[arg(long)]
        account: String,
        /// deposit | withdrawal
        #[arg(long, name = "type")]
        txn_type: String,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        date: String,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum AiAction {
    /// Chat with AI assistant
    Chat {
        #[arg(long)]
        message: String,
    },
    /// AI data analysis
    Analyze {
        #[arg(long)]
        data_type: String,
        #[arg(long)]
        data: String,
        #[arg(long)]
        question: Option<String>,
    },
    /// AI compliance audit
    Audit {
        #[arg(long)]
        audit_type: String,
        #[arg(long)]
        application_id: Option<String>,
        #[arg(long)]
        material_id: Option<String>,
        #[arg(long)]
        contract_id: Option<String>,
        #[arg(long)]
        employee_id: Option<String>,
        #[arg(long)]
        budget_id: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
    },
    /// Parse a document with AI
    ParseDoc {
        #[arg(long)]
        file: String,
        #[arg(long)]
        document_type: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// AI contract risk review
    ContractReview {
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        contract_type: Option<String>,
        #[arg(long)]
        counterparty_type: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ReportAction {
    /// Financial health overview
    FinancialHealth,
    /// Cash flow forecast
    Cashflow,
    /// Profit analysis
    Profit,
    /// Deal conversion funnel
    Deals,
}

#[derive(Subcommand)]
pub(crate) enum PdfAction {
    /// Generate invoice PDF
    Invoice {
        id: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate quote PDF
    Quote {
        id: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum PolicyAction {
    /// List all policies
    List,
    /// Smart match policies for your company
    Match,
    /// Get policy details
    Get { id: String },
    /// Manage policy applications
    Applications {
        #[command(subcommand)]
        action: CrudAction,
    },
}

#[derive(Subcommand)]
pub(crate) enum FileAction {
    /// Upload a file
    Upload {
        /// Path to the file to upload
        file: String,
        /// Entity type to attach to (e.g., invoices, contracts)
        #[arg(long)]
        entity_type: Option<String>,
        /// Entity ID to attach to
        #[arg(long)]
        entity_id: Option<String>,
    },
    /// List files for an entity
    List {
        #[arg(long)]
        entity_type: String,
        #[arg(long)]
        entity_id: String,
    },
    /// Download a file
    Download {
        /// File ID
        id: String,
        /// Output path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Delete a file
    Delete {
        /// File ID
        id: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum OpcAction {
    /// Calculate OPC tax for all hubs
    Calculate {
        /// Estimated annual revenue
        #[arg(long)]
        revenue: f64,
        /// Current base cost-of-living index
        #[arg(long, default_value = "50")]
        col_index: f64,
    },
}

#[derive(Subcommand)]
pub(crate) enum ApiKeyAction {
    /// List API keys
    List,
    /// Create a new API key
    Create {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        expires_in_days: Option<i32>,
    },
    /// Revoke an API key
    Revoke { id: String },
    /// Delete an API key
    Delete { id: String },
}

#[derive(Subcommand)]
pub(crate) enum TaxToolAction {
    /// Calculate tax for given amount
    Calculate {
        #[arg(long)]
        amount: Option<f64>,
    },
    /// Run compliance check
    ComplianceCheck,
    /// Show invoice requirements by region
    InvoiceRequirements,
    /// Input credit summary
    InputCredit,
}

#[derive(Subcommand)]
pub(crate) enum CurrencyAction {
    /// Convert between currencies
    Convert {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: f64,
    },
    /// Show multi-currency balances
    Balances,
}

#[derive(Subcommand)]
pub(crate) enum ArAction {
    /// Aging analysis
    Aging,
    /// AR summary
    Summary,
}

#[derive(Subcommand)]
pub(crate) enum WebhookAction {
    /// List webhooks
    List,
    /// Create a webhook
    Create {
        #[arg(long)]
        url: String,
        #[arg(long)]
        events: String,
    },
    /// Delete a webhook
    Delete { id: String },
    /// Test a webhook
    Test { id: String },
}

#[derive(Subcommand)]
pub(crate) enum CrudAction {
    List {
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    Get {
        id: String,
    },
    Create {
        #[arg(long)]
        json: Option<String>,
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    Update {
        id: String,
        #[arg(long)]
        json: Option<String>,
        #[arg(short = 'f', long = "field", value_parser = parse_kv)]
        fields: Vec<(String, String)>,
    },
    Delete {
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn cli_parses_root_status_command() {
        let cli = crate::cli::args::Cli::try_parse_from(["opc", "--format", "json", "status"])
            .expect("parse cli");

        assert_eq!(cli.format, "json");
        assert_eq!(cli.server, "http://localhost:8987");
        assert!(matches!(cli.command, crate::cli::args::Commands::Status));
    }
}
