const mysql = require('mysql2/promise');
require('dotenv').config();

async function migrate() {
    console.log('🔄 Adding revoked_at column to license_keys table...\n');
    
    const connection = await mysql.createConnection({
        host: process.env.DB_HOST,
        user: process.env.DB_USER,
        password: process.env.DB_PASSWORD,
        database: process.env.DB_NAME,
        multipleStatements: true
    });

    try {
        // Check if column already exists
        const [columns] = await connection.execute(
            `SHOW COLUMNS FROM license_keys LIKE 'revoked_at'`
        );

        if (columns.length > 0) {
            console.log('✅ Column revoked_at already exists. Skipping migration.');
            await connection.end();
            return;
        }

        // Add the revoked_at column
        console.log('📝 Adding revoked_at column...');
        await connection.execute(
            `ALTER TABLE license_keys 
             ADD COLUMN revoked_at TIMESTAMP NULL DEFAULT NULL AFTER assigned_at`
        );
        console.log('✅ Column added successfully');

        // Add index
        console.log('📝 Adding index on revoked_at...');
        await connection.execute(
            `ALTER TABLE license_keys 
             ADD INDEX idx_revoked (revoked_at)`
        );
        console.log('✅ Index added successfully');

        // Update the statistics view
        console.log('📝 Updating license_statistics view...');
        await connection.execute(`DROP VIEW IF EXISTS license_statistics`);
        await connection.execute(`
            CREATE VIEW license_statistics AS
            SELECT 
                tier,
                COUNT(*) as total_keys,
                SUM(CASE WHEN is_used = TRUE AND revoked_at IS NULL THEN 1 ELSE 0 END) as used_keys,
                SUM(CASE WHEN is_used = FALSE AND revoked_at IS NULL THEN 1 ELSE 0 END) as available_keys,
                SUM(CASE WHEN revoked_at IS NOT NULL THEN 1 ELSE 0 END) as revoked_keys
            FROM license_keys
            GROUP BY tier
        `);
        console.log('✅ View updated successfully');

        console.log('\n🎉 Migration completed successfully!\n');

    } catch (error) {
        console.error('❌ Migration failed:', error.message);
        throw error;
    } finally {
        await connection.end();
    }
}

migrate().catch(console.error);

