const mysql = require('mysql2/promise');

// Local database (your existing data)
const localConfig = {
    host: 'localhost',
    port: 3306,
    user: 'Antoine',
    password: '3e3.]Koh=+meXwXo',
    database: 'receipt_extractor'
};

// Railway database (new cloud database)
const railwayConfig = {
    host: 'mainline.proxy.rlwy.net',
    port: 17497,
    user: 'root',
    password: 'CQNCpjnIUjuiIVbuoDUXalJGtFZgLpOA',
    database: 'railway'
};

async function migrate() {
    console.log('🔄 Starting migration from local to Railway...\n');
    
    let localConn, railwayConn;
    
    try {
        // Connect to both databases
        console.log('📡 Connecting to local database...');
        localConn = await mysql.createConnection(localConfig);
        console.log('✅ Connected to local database');
        
        console.log('📡 Connecting to Railway database...');
        railwayConn = await mysql.createConnection(railwayConfig);
        console.log('✅ Connected to Railway database\n');
        
        // Migrate license_keys
        console.log('📋 Migrating license_keys...');
        const [licenses] = await localConn.query('SELECT * FROM license_keys');
        console.log(`   Found ${licenses.length} licenses`);
        
        for (const license of licenses) {
            try {
                await railwayConn.query(`
                    INSERT INTO license_keys 
                    (license_key, tier, created_at, is_used, customer_email, customer_name, 
                     order_id, assigned_at, revoked_at, notes, plan_code, billing_cycle,
                     subscription_start, subscription_end, shopify_subscription_id)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                    tier = VALUES(tier),
                    is_used = VALUES(is_used),
                    customer_email = VALUES(customer_email),
                    customer_name = VALUES(customer_name)
                `, [
                    license.license_key,
                    license.tier,
                    license.created_at,
                    license.is_used,
                    license.customer_email,
                    license.customer_name,
                    license.order_id,
                    license.assigned_at,
                    license.revoked_at,
                    license.notes,
                    license.plan_code || 'STARTER',
                    license.billing_cycle || 'YEARLY',
                    license.subscription_start,
                    license.subscription_end,
                    license.shopify_subscription_id
                ]);
            } catch (err) {
                console.log(`   ⚠️ Skipping ${license.license_key}: ${err.message}`);
            }
        }
        console.log('✅ License keys migrated\n');
        
        // Migrate license_activations
        console.log('📋 Migrating license_activations...');
        const [activations] = await localConn.query('SELECT * FROM license_activations');
        console.log(`   Found ${activations.length} activations`);
        
        for (const activation of activations) {
            try {
                await railwayConn.query(`
                    INSERT INTO license_activations 
                    (license_key, device_fingerprint, device_name, activated_at, 
                     last_seen, ip_address, is_active)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                    last_seen = VALUES(last_seen),
                    is_active = VALUES(is_active)
                `, [
                    activation.license_key,
                    activation.device_fingerprint,
                    activation.device_name,
                    activation.activated_at,
                    activation.last_seen,
                    activation.ip_address,
                    activation.is_active
                ]);
            } catch (err) {
                console.log(`   ⚠️ Skipping activation: ${err.message}`);
            }
        }
        console.log('✅ License activations migrated\n');
        
        // Migrate admin_users
        console.log('📋 Migrating admin_users...');
        const [admins] = await localConn.query('SELECT * FROM admin_users');
        console.log(`   Found ${admins.length} admin users`);
        
        for (const admin of admins) {
            try {
                await railwayConn.query(`
                    INSERT INTO admin_users 
                    (username, api_key, created_at, last_login, is_active)
                    VALUES (?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                    api_key = VALUES(api_key)
                `, [
                    admin.username,
                    admin.api_key,
                    admin.created_at,
                    admin.last_login,
                    admin.is_active
                ]);
            } catch (err) {
                console.log(`   ⚠️ Skipping admin: ${err.message}`);
            }
        }
        console.log('✅ Admin users migrated\n');
        
        // Verify migration
        const [railwayLicenses] = await railwayConn.query('SELECT COUNT(*) as count FROM license_keys');
        const [railwayActivations] = await railwayConn.query('SELECT COUNT(*) as count FROM license_activations');
        const [railwayAdmins] = await railwayConn.query('SELECT COUNT(*) as count FROM admin_users');
        
        console.log('📊 Railway database now contains:');
        console.log(`   - ${railwayLicenses[0].count} license keys`);
        console.log(`   - ${railwayActivations[0].count} activations`);
        console.log(`   - ${railwayAdmins[0].count} admin users`);
        
        console.log('\n✅ Migration complete!');
        
    } catch (error) {
        console.error('❌ Migration error:', error.message);
    } finally {
        if (localConn) await localConn.end();
        if (railwayConn) await railwayConn.end();
    }
}

migrate();

