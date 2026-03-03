-- Migration: Add Usage Tracking and Subscription Plans
-- Run this after the initial schema.sql

USE receipt_extractor;

-- Subscription Plans Table
CREATE TABLE IF NOT EXISTS subscription_plans (
    id INT AUTO_INCREMENT PRIMARY KEY,
    plan_code VARCHAR(50) UNIQUE NOT NULL,
    plan_name VARCHAR(100) NOT NULL,
    receipts_per_month INT NOT NULL,  -- -1 for unlimited
    price_monthly DECIMAL(10, 2) NOT NULL,
    price_yearly DECIMAL(10, 2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_plan_code (plan_code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Insert default subscription plans
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
    price_yearly = VALUES(price_yearly);

-- Add plan and billing columns to license_keys table
ALTER TABLE license_keys 
    ADD COLUMN IF NOT EXISTS plan_code VARCHAR(50) DEFAULT 'STARTER',
    ADD COLUMN IF NOT EXISTS billing_cycle ENUM('MONTHLY', 'YEARLY') DEFAULT 'YEARLY',
    ADD COLUMN IF NOT EXISTS subscription_start TIMESTAMP NULL,
    ADD COLUMN IF NOT EXISTS subscription_end TIMESTAMP NULL,
    ADD COLUMN IF NOT EXISTS shopify_subscription_id VARCHAR(100) NULL;

-- Add index for plan lookups
ALTER TABLE license_keys ADD INDEX IF NOT EXISTS idx_plan_code (plan_code);

-- Usage Tracking Table (monthly usage per license)
CREATE TABLE IF NOT EXISTS usage_tracking (
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Usage Log Table (detailed log of each processing)
CREATE TABLE IF NOT EXISTS usage_log (
    id INT AUTO_INCREMENT PRIMARY KEY,
    license_key VARCHAR(50) NOT NULL,
    device_fingerprint VARCHAR(255),
    receipts_count INT DEFAULT 1,
    processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (license_key) REFERENCES license_keys(license_key) ON DELETE CASCADE,
    INDEX idx_license_key (license_key),
    INDEX idx_processed_at (processed_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- View: Current month usage per license
CREATE OR REPLACE VIEW current_month_usage AS
SELECT 
    lk.license_key,
    lk.plan_code,
    lk.billing_cycle,
    lk.subscription_start,
    lk.subscription_end,
    sp.plan_name,
    sp.receipts_per_month AS monthly_limit,
    COALESCE(ut.receipts_processed, 0) AS receipts_used,
    CASE 
        WHEN sp.receipts_per_month = -1 THEN 999999
        ELSE sp.receipts_per_month - COALESCE(ut.receipts_processed, 0)
    END AS receipts_remaining,
    CASE
        WHEN sp.receipts_per_month = -1 THEN FALSE
        WHEN COALESCE(ut.receipts_processed, 0) >= sp.receipts_per_month THEN TRUE
        ELSE FALSE
    END AS limit_reached
FROM license_keys lk
LEFT JOIN subscription_plans sp ON lk.plan_code = sp.plan_code
LEFT JOIN usage_tracking ut ON lk.license_key = ut.license_key 
    AND ut.period_year = YEAR(CURRENT_DATE())
    AND ut.period_month = MONTH(CURRENT_DATE());

-- Update shopify_orders table to support subscriptions
ALTER TABLE shopify_orders
    ADD COLUMN IF NOT EXISTS plan_code VARCHAR(50) DEFAULT 'STARTER',
    ADD COLUMN IF NOT EXISTS billing_cycle ENUM('MONTHLY', 'YEARLY') DEFAULT 'YEARLY',
    ADD COLUMN IF NOT EXISTS is_subscription BOOLEAN DEFAULT FALSE;

