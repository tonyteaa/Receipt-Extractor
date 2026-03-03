use anyhow::Result;
use mysql::*;
use mysql::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    pub id: u32,
    pub license_key: String,
    pub tier: String,
    pub is_used: bool,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub created_at: String,
    pub assigned_at: Option<String>,
    pub revoked_at: Option<String>,
    pub activation_count: i32,
    pub max_activations: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceActivation {
    pub id: i32,
    pub license_key: String,
    pub device_name: String,
    pub device_fingerprint: String,
    pub ip_address: Option<String>,
    pub activated_at: String,
    pub last_seen: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            // Railway cloud database
            host: "mainline.proxy.rlwy.net".to_string(),
            port: 17497,
            user: "root".to_string(),
            password: "CQNCpjnIUjuiIVbuoDUXalJGtFZgLpOA".to_string(),
            database: "railway".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.user, config.password, config.host, config.port, config.database
        );
        
        let pool = Pool::new(url.as_str())?;
        
        Ok(Self { pool })
    }

    pub fn test_connection(&self) -> Result<()> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop("SELECT 1")?;
        Ok(())
    }

    pub fn insert_license(&self, license_key: &str, tier: &str) -> Result<()> {
        let mut conn = self.pool.get_conn()?;
        
        conn.exec_drop(
            "INSERT INTO license_keys (license_key, tier) VALUES (?, ?)",
            (license_key, tier),
        )?;
        
        Ok(())
    }

    pub fn get_all_licenses(&self) -> Result<Vec<LicenseKey>> {
        let mut conn = self.pool.get_conn()?;

        let licenses = conn.query_map(
            "SELECT
                lk.id,
                lk.license_key,
                lk.tier,
                lk.is_used,
                lk.customer_email,
                lk.customer_name,
                lk.created_at,
                lk.assigned_at,
                lk.revoked_at,
                COUNT(la.id) as activation_count,
                5 as max_activations,
                lk.notes
             FROM license_keys lk
             LEFT JOIN license_activations la ON lk.license_key = la.license_key AND la.is_active = TRUE
             GROUP BY lk.id
             ORDER BY lk.created_at DESC",
            |(id, license_key, tier, is_used, customer_email, customer_name, created_at, assigned_at, revoked_at, activation_count, max_activations, notes)| {
                LicenseKey {
                    id,
                    license_key,
                    tier,
                    is_used,
                    customer_email,
                    customer_name,
                    created_at,
                    assigned_at,
                    revoked_at,
                    activation_count,
                    max_activations,
                    notes,
                }
            },
        )?;

        Ok(licenses)
    }

    pub fn get_licenses_by_tier(&self, tier: &str) -> Result<Vec<LicenseKey>> {
        let mut conn = self.pool.get_conn()?;

        let licenses = conn.exec_map(
            "SELECT
                lk.id,
                lk.license_key,
                lk.tier,
                lk.is_used,
                lk.customer_email,
                lk.customer_name,
                lk.created_at,
                lk.assigned_at,
                lk.revoked_at,
                COUNT(la.id) as activation_count,
                5 as max_activations,
                lk.notes
             FROM license_keys lk
             LEFT JOIN license_activations la ON lk.license_key = la.license_key AND la.is_active = TRUE
             WHERE lk.tier = ?
             GROUP BY lk.id
             ORDER BY lk.created_at DESC",
            (tier,),
            |(id, license_key, tier, is_used, customer_email, customer_name, created_at, assigned_at, revoked_at, activation_count, max_activations, notes)| {
                LicenseKey {
                    id,
                    license_key,
                    tier,
                    is_used,
                    customer_email,
                    customer_name,
                    created_at,
                    assigned_at,
                    revoked_at,
                    activation_count,
                    max_activations,
                    notes,
                }
            },
        )?;

        Ok(licenses)
    }

    pub fn assign_license(&self, license_key: &str, customer_email: &str, customer_name: &str) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE license_keys
             SET is_used = TRUE, customer_email = ?, customer_name = ?, assigned_at = NOW()
             WHERE license_key = ?",
            (customer_email, customer_name, license_key),
        )?;

        Ok(())
    }

    pub fn revoke_license(&self, license_key: &str) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        // Delete activations
        conn.exec_drop(
            "DELETE FROM license_activations WHERE license_key = ?",
            (license_key,),
        )?;

        // Reset license
        conn.exec_drop(
            "UPDATE license_keys
             SET is_used = FALSE, customer_email = NULL, customer_name = NULL, assigned_at = NULL
             WHERE license_key = ?",
            (license_key,),
        )?;

        Ok(())
    }

    // Deactivate a single device by activation ID
    pub fn deactivate_device(&self, activation_id: i32) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "DELETE FROM license_activations WHERE id = ?",
            (activation_id,),
        )?;

        Ok(())
    }

    // Get notes for a license
    pub fn get_license_notes(&self, license_key: &str) -> Result<Option<String>> {
        let mut conn = self.pool.get_conn()?;

        let result: Option<Option<String>> = conn.exec_first(
            "SELECT notes FROM license_keys WHERE license_key = ?",
            (license_key,),
        )?;

        Ok(result.flatten())
    }

    // Update notes for a license
    pub fn update_license_notes(&self, license_key: &str, notes: &str) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE license_keys SET notes = ? WHERE license_key = ?",
            (notes, license_key),
        )?;

        Ok(())
    }

    // Search for users by email (partial match)
    pub fn search_users(&self, query: &str) -> Result<Vec<(String, Option<String>, i32)>> {
        let mut conn = self.pool.get_conn()?;

        let results: Vec<(String, Option<String>, i32)> = conn.exec_map(
            "SELECT customer_email, customer_name, COUNT(*) as license_count
             FROM license_keys
             WHERE customer_email IS NOT NULL
             AND (customer_email LIKE ? OR customer_name LIKE ?)
             GROUP BY customer_email, customer_name
             ORDER BY customer_email",
            (format!("%{}%", query), format!("%{}%", query)),
            |(email, name, count)| (email, name, count),
        )?;

        Ok(results)
    }

    // Get all licenses for a specific customer email
    pub fn get_user_licenses(&self, email: &str) -> Result<Vec<LicenseKey>> {
        let mut conn = self.pool.get_conn()?;

        let licenses: Vec<LicenseKey> = conn.exec_map(
            "SELECT
                lk.id,
                lk.license_key,
                lk.tier,
                lk.is_used,
                lk.customer_email,
                lk.customer_name,
                lk.created_at,
                lk.assigned_at,
                lk.revoked_at,
                COUNT(la.id) as activation_count,
                5 as max_activations,
                lk.notes
             FROM license_keys lk
             LEFT JOIN license_activations la ON lk.license_key = la.license_key AND la.is_active = TRUE
             WHERE lk.customer_email = ?
             GROUP BY lk.id
             ORDER BY lk.created_at DESC",
            (email,),
            |(id, license_key, tier, is_used, customer_email, customer_name, created_at, assigned_at, revoked_at, activation_count, max_activations, notes)| {
                LicenseKey {
                    id,
                    license_key,
                    tier,
                    is_used,
                    customer_email,
                    customer_name,
                    created_at,
                    assigned_at,
                    revoked_at,
                    activation_count,
                    max_activations,
                    notes,
                }
            },
        )?;

        Ok(licenses)
    }

    // Get all device activations for a specific customer email
    pub fn get_user_activations(&self, email: &str) -> Result<Vec<DeviceActivation>> {
        let mut conn = self.pool.get_conn()?;

        let rows: Vec<mysql::Row> = conn.exec(
            "SELECT la.id, la.license_key, la.device_name, la.device_fingerprint, la.ip_address,
                    CAST(la.activated_at AS CHAR) as activated_at,
                    CAST(la.last_seen AS CHAR) as last_seen,
                    la.is_active
             FROM license_activations la
             JOIN license_keys lk ON la.license_key = lk.license_key
             WHERE lk.customer_email = ?
             ORDER BY la.last_seen DESC",
            (email,),
        )?;

        let mut activations = Vec::new();
        for mut row in rows {
            activations.push(DeviceActivation {
                id: row.take("id").unwrap(),
                license_key: row.take("license_key").unwrap(),
                device_name: row.take("device_name").unwrap(),
                device_fingerprint: row.take("device_fingerprint").unwrap(),
                ip_address: row.take("ip_address"),
                activated_at: row.take("activated_at").unwrap(),
                last_seen: row.take("last_seen").unwrap(),
                is_active: row.take("is_active").unwrap(),
            });
        }

        Ok(activations)
    }

    // Get device activations for a specific license key
    pub fn get_license_activations(&self, license_key: &str) -> Result<Vec<DeviceActivation>> {
        let mut conn = self.pool.get_conn()?;

        let rows: Vec<mysql::Row> = conn.exec(
            "SELECT id, license_key, device_name, device_fingerprint, ip_address,
                    CAST(activated_at AS CHAR) as activated_at,
                    CAST(last_seen AS CHAR) as last_seen,
                    is_active
             FROM license_activations
             WHERE license_key = ?
             ORDER BY last_seen DESC",
            (license_key,),
        )?;

        let mut activations = Vec::new();
        for mut row in rows {
            activations.push(DeviceActivation {
                id: row.take("id").unwrap(),
                license_key: row.take("license_key").unwrap(),
                device_name: row.take("device_name").unwrap(),
                device_fingerprint: row.take("device_fingerprint").unwrap(),
                ip_address: row.take("ip_address"),
                activated_at: row.take("activated_at").unwrap(),
                last_seen: row.take("last_seen").unwrap(),
                is_active: row.take("is_active").unwrap(),
            });
        }

        Ok(activations)
    }
}

