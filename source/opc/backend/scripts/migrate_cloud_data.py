import os
import sqlite3
import uuid
from datetime import datetime

DB_PATH = os.environ["OPC_MIGRATION_DATABASE"]

def migrate():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    # Get virtual assets
    cursor.execute("SELECT * FROM assets WHERE asset_type = 'virtual'")
    assets = cursor.fetchall()
    
    print(f"Found {len(assets)} virtual assets to migrate.")

    for asset in assets:
        name = asset['name']
        price = asset['purchase_price'] or 0
        company_id = asset['company_id']
        asset_id = asset['id']
        created_at = asset['created_at']
        
        # 1. Domains
        if name.startswith('域名'):
            domain_name = name.replace('域名', '').strip()
            print(f"Migrating Domain: {domain_name}")
            cursor.execute("""
                INSERT OR IGNORE INTO domain_subscriptions 
                (id, company_id, domain_name, registrar, registration_date, expiry_date, annual_fee, status, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?)
            """, (asset_id, company_id, domain_name, 'Unknown', created_at, created_at, price, created_at))
            
        # 2. Cloud Resources (ECS, Server)
        elif 'ECS' in name or '服务器' in name:
            print(f"Migrating Cloud Resource: {name}")
            # Assume annual if price > 1000, else monthly
            monthly_cost = price / 12 if price > 1000 else price
            resource_type = 'compute'
            
            cursor.execute("""
                INSERT OR IGNORE INTO cloud_resources
                (id, company_id, resource_name, resource_type, monthly_cost, start_date, status, created_at)
                VALUES (?, ?, ?, ?, ?, ?, 'running', ?)
            """, (asset_id, company_id, name, resource_type, monthly_cost, created_at, created_at))
            
        # 3. Everything else -> Software Subscription
        else:
            print(f"Migrating Software: {name}")
            vendor = name.split(' ')[0] # Simple guess
            billing_cycle = 'annually'
            
            cursor.execute("""
                INSERT OR IGNORE INTO software_subscriptions
                (id, company_id, software_name, vendor, subscription_type, total_price, billing_cycle, start_date, next_billing_date, status, created_at)
                VALUES (?, ?, ?, ?, 'annual', ?, ?, ?, ?, 'active', ?)
            """, (asset_id, company_id, name, vendor, price, billing_cycle, created_at, created_at, created_at))

    conn.commit()
    conn.close()
    print("Migration complete.")

if __name__ == '__main__':
    migrate()
