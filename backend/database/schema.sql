-- Receipt Extractor License Management Database Schema

-- Create database
CREATE DATABASE IF NOT EXISTS receipt_extractor;
USE receipt_extractor;

-- License Keys Table
CREATE TABLE IF NOT EXISTS license_keys (
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
    INDEX idx_license_key (license_key),
    INDEX idx_tier_used (tier, is_used),
    INDEX idx_email (customer_email),
    INDEX idx_revoked (revoked_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- License Activations Table (track device activations)
CREATE TABLE IF NOT EXISTS license_activations (
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- API Usage Logs (optional - track API calls)
CREATE TABLE IF NOT EXISTS api_logs (
    id INT AUTO_INCREMENT PRIMARY KEY,
    license_key VARCHAR(50),
    endpoint VARCHAR(100) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    request_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    response_status INT,
    INDEX idx_license_key (license_key),
    INDEX idx_timestamp (request_timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Shopify Orders Table (track purchases)
CREATE TABLE IF NOT EXISTS shopify_orders (
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
    FOREIGN KEY (license_key) REFERENCES license_keys(license_key),
    INDEX idx_order_id (order_id),
    INDEX idx_email (customer_email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Admin Users Table (for admin panel access)
CREATE TABLE IF NOT EXISTS admin_users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    api_key VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP NULL,
    is_active BOOLEAN DEFAULT TRUE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Statistics View (for dashboard)
CREATE OR REPLACE VIEW license_statistics AS
SELECT
    tier,
    COUNT(*) as total_keys,
    SUM(CASE WHEN is_used = TRUE AND revoked_at IS NULL THEN 1 ELSE 0 END) as used_keys,
    SUM(CASE WHEN is_used = FALSE AND revoked_at IS NULL THEN 1 ELSE 0 END) as available_keys,
    SUM(CASE WHEN revoked_at IS NOT NULL THEN 1 ELSE 0 END) as revoked_keys
FROM license_keys
GROUP BY tier;

-- Activation Statistics View
CREATE OR REPLACE VIEW activation_statistics AS
SELECT
    lk.tier,
    COUNT(DISTINCT la.license_key) as activated_licenses,
    SUM(COALESCE(la.activations_per_license, 0)) as total_activations,
    AVG(COALESCE(la.activations_per_license, 0)) as avg_activations_per_license
FROM license_keys lk
LEFT JOIN (
    SELECT license_key, COUNT(*) as activations_per_license
    FROM license_activations
    WHERE is_active = TRUE
    GROUP BY license_key
) la ON lk.license_key = la.license_key
GROUP BY lk.tier;

