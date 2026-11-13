use sqlx::SqlitePool;

pub async fn seed_global_opc_hubs(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM global_opc_hubs")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    let hubs = vec![
        (
            "hub_001",
            "AE",
            "Dubai",
            "USD",
            0.09,
            0.0,
            0.0,
            0,
            1,
            1,
            1,
            90.0,
            300.0,
            2500.0,
            8000.0,
            2000.0,
            "No personal income tax. Corporate tax is 9% but many Free Zones offer 0% on qualifying offshore income. Expensive initial setup but extremely tax-efficient. Great for Crypto & E-commerce.",
        ),
        (
            "hub_002",
            "SG",
            "Singapore",
            "USD",
            0.17,
            0.0,
            0.0,
            0,
            0,
            0,
            0,
            110.0,
            500.0,
            3500.0,
            2500.0,
            1500.0,
            "Top-tier financial hub. 17% corporate tax but effective rate is much lower for startups (e.g., first 100k SGD is 75% exempt). High living costs. No capital gains or dividend tax.",
        ),
        (
            "hub_003",
            "US",
            "Wyoming",
            "USD",
            0.21,
            0.2,
            0.2,
            1,
            0,
            0,
            0,
            80.0,
            200.0,
            1500.0,
            200.0,
            100.0,
            "Ultimate pass-through entity for non-US residents. Zero US corporate or personal tax if you have no US physical presence/nexus. Dirt cheap to setup and maintain. True 'nomad' paper company.",
        ),
        (
            "hub_004",
            "HK",
            "Hong Kong",
            "USD",
            0.165,
            0.0,
            0.0,
            0,
            0,
            1,
            0,
            105.0,
            450.0,
            2800.0,
            1200.0,
            800.0,
            "Territorial tax system. Profits sourced outside HK are 100% tax-free. No capital gains or dividend taxes. First 2M HKD of local profit is taxed at 8.25%. Difficult banking setup.",
        ),
        (
            "hub_005",
            "EE",
            "Tallinn",
            "EUR",
            0.22,
            0.22,
            0.22,
            0,
            1,
            0,
            1,
            60.0,
            250.0,
            1000.0,
            300.0,
            600.0,
            "e-Residency pioneer. 0% tax on retained/reinvested profits! Only pay 20-22% tax when distributing dividends. Entirely digital admin. Excellent digital nomad visa program.",
        ),
    ];

    for hub in hubs {
        sqlx::query(
            r#"
            INSERT INTO global_opc_hubs (
                id, country_code, city, currency, 
                corporate_tax_rate, dividend_tax_rate, capital_gains_tax, 
                pass_through_allowed, zero_tax_on_reinvested, foreign_income_exempt, digital_nomad_visa,
                col_index, avg_coworking_desk, avg_apartment_rent_1bed, setup_cost, annual_compliance_cost,
                hub_notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(hub.0)
        .bind(hub.1)
        .bind(hub.2)
        .bind(hub.3)
        .bind(hub.4)
        .bind(hub.5)
        .bind(hub.6)
        .bind(hub.7)
        .bind(hub.8)
        .bind(hub.9)
        .bind(hub.10)
        .bind(hub.11)
        .bind(hub.12)
        .bind(hub.13)
        .bind(hub.14)
        .bind(hub.15)
        .bind(hub.16)
        .execute(pool)
        .await?;
    }

    Ok(())
}
