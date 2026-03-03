const mysql = require('mysql2/promise');

async function setupDatabase() {
    const connection = await mysql.createConnection({
        host: 'mainline.proxy.rlwy.net',
        port: 17497,
        user: 'root',
        password: 'CQNCpjnIUjuiIVbuoDUXalJGtFZgLpOA',
        database: 'railway',
        multipleStatements: true
    });
    
    console.log('Connected to Railway MySQL!');
    
    // Create tables one by one to avoid issues
    const tables = [
        `CREATE TABLE IF NOT EXISTS license_keys (
            id INT AUTO_INCREMENT PRIMARY KEY,
            license_key VARCHAR(50) UNIQUE NOT NULL,
            tier ENUM('BUDGET', 'PRO') NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            is_used BOOLEAN DEFAULT FALSE,
            customer_email VARCHAR(255),
            customer_name VARCHAR(255),
            order_id VARCHAR(100),
            assigned_at TIMESTAMP NULL,
            revoked_at TIMESTAMP NULL DEFAULT NULL,
            notes TEXT,
            plan_code VARCHAR(50) DEFAULT 'STARTER',
            billing_cycle ENUM('MONTHLY', 'YEARLY') DEFAULT 'YEARLY',
            subscription_start TIMESTAMP NULL,
            subscription_end TIMESTAMP NULL,
            shopify_subscription_id VARCHAR(100) NULL,
            INDEX idx_license_key (license_key),
            INDEX idx_tier_used (tier, is_used),
            INDEX idx_email (customer_email),
            INDEX idx_revoked (revoked_at),
            INDEX idx_plan_code (plan_code)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS license_activations (
            id INT AUTO_INCREMENT PRIMARY KEY,
            license_key VARCHAR(50) NOT NULL,
            device_fingerprint VARCHAR(255) NOT NULL,
            device_name VARCHAR(255),
            activated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            ip_address VARCHAR(45),
            is_active BOOLEAN DEFAULT TRUE,
            FOREIGN KEY (license_key) REFERENCES license_keys(license_key) ON DELETE CASCADE,
            UNIQUE KEY unique_license_device (license_key, device_fingerprint),
            INDEX idx_license_key (license_key),
            INDEX idx_device (device_fingerprint)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS api_logs (
            id INT AUTO_INCREMENT PRIMARY KEY,
            license_key VARCHAR(50),
            endpoint VARCHAR(100) NOT NULL,
            ip_address VARCHAR(45),
            user_agent TEXT,
            request_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            response_status INT,
            INDEX idx_license_key (license_key),
            INDEX idx_timestamp (request_timestamp)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS subscription_plans (
            id INT AUTO_INCREMENT PRIMARY KEY,
            plan_code VARCHAR(50) UNIQUE NOT NULL,
            plan_name VARCHAR(100) NOT NULL,
            receipts_per_month INT NOT NULL,
            price_monthly DECIMAL(10, 2) NOT NULL,
            price_yearly DECIMAL(10, 2) NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_plan_code (plan_code)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS shopify_orders (
            id INT AUTO_INCREMENT PRIMARY KEY,
            order_id VARCHAR(100) UNIQUE NOT NULL,
            order_number VARCHAR(50),
            customer_email VARCHAR(255) NOT NULL,
            customer_name VARCHAR(255),
            product_tier ENUM('BUDGET', 'PRO') NOT NULL,
            amount_paid DECIMAL(10, 2),
            currency VARCHAR(3) DEFAULT 'USD',
            license_key VARCHAR(50),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            webhook_received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            plan_code VARCHAR(50) DEFAULT 'STARTER',
            billing_cycle ENUM('MONTHLY', 'YEARLY') DEFAULT 'YEARLY',
            is_subscription BOOLEAN DEFAULT FALSE,
            INDEX idx_order_id (order_id),
            INDEX idx_email (customer_email)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS admin_users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(50) UNIQUE NOT NULL,
            api_key VARCHAR(255) UNIQUE NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            last_login TIMESTAMP NULL,
            is_active BOOLEAN DEFAULT TRUE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS usage_tracking (
            id INT AUTO_INCREMENT PRIMARY KEY,
            license_key VARCHAR(50) NOT NULL,
            period_year INT NOT NULL,
            period_month INT NOT NULL,
            receipts_processed INT DEFAULT 0,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            FOREIGN KEY (license_key) REFERENCES license_keys(license_key) ON DELETE CASCADE,
            UNIQUE KEY unique_license_period (license_key, period_year, period_month),
            INDEX idx_license_key (license_key),
            INDEX idx_period (period_year, period_month)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`,

        `CREATE TABLE IF NOT EXISTS usage_log (
            id INT AUTO_INCREMENT PRIMARY KEY,
            license_key VARCHAR(50) NOT NULL,
            device_fingerprint VARCHAR(255),
            receipts_count INT DEFAULT 1,
            processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (license_key) REFERENCES license_keys(license_key) ON DELETE CASCADE,
            INDEX idx_license_key (license_key),
            INDEX idx_processed_at (processed_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`
    ];

    for (const sql of tables) {
        await connection.query(sql);
        console.log('Created table successfully');
    }
    
    // Insert subscription plans
    const plans = `
        INSERT INTO subscription_plans (plan_code, plan_name, receipts_per_month, price_monthly, price_yearly) VALUES
            ('STARTER', 'Starter', 100, 2.99, 19.00),
            ('PERSONAL', 'Personal', 500, 5.99, 49.00),
            ('FAMILY', 'Family', 1500, 9.99, 99.00),
            ('POWER', 'Power User', 5000, 19.99, 199.00),
            ('UNLIMITED', 'Unlimited', -1, 34.99, 349.00)
        ON DUPLICATE KEY UPDATE
            plan_name = VALUES(plan_name),
            receipts_per_month = VALUES(receipts_per_month),
            price_monthly = VALUES(price_monthly),
            price_yearly = VALUES(price_yearly)
    `;
    
    await connection.query(plans);
    console.log('Subscription plans inserted!');
    
    // Show tables
    const [tablesList] = await connection.query('SHOW TABLES');
    console.log('Tables in database:', tablesList);
    
    await connection.end();
    console.log('Database setup complete!');
}

setupDatabase().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});

