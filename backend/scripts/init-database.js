const mysql = require('mysql2/promise');
const fs = require('fs');
const path = require('path');
require('dotenv').config();

async function initializeDatabase() {
    console.log('🚀 Initializing Receipt Extractor Database...\n');

    let connection;
    
    try {
        // Connect to MySQL server (without database)
        connection = await mysql.createConnection({
            host: process.env.DB_HOST || 'localhost',
            port: process.env.DB_PORT || 3306,
            user: process.env.DB_USER,
            password: process.env.DB_PASSWORD,
            multipleStatements: true
        });

        console.log('✅ Connected to MySQL server');

        // Read and execute schema
        const schemaPath = path.join(__dirname, '../database/schema.sql');
        const schema = fs.readFileSync(schemaPath, 'utf8');

        console.log('📝 Executing schema...');
        await connection.query(schema);

        console.log('✅ Database schema created successfully');

        // Create default admin user
        const adminApiKey = require('crypto').randomBytes(32).toString('hex');
        
        await connection.query(`
            USE ${process.env.DB_NAME};
            INSERT IGNORE INTO admin_users (username, api_key) 
            VALUES ('admin', ?);
        `, [adminApiKey]);

        console.log('\n✅ Database initialization complete!\n');
        console.log('📋 Admin API Key (save this!):', adminApiKey);
        console.log('\n💡 Next steps:');
        console.log('   1. Save the admin API key to your .env file');
        console.log('   2. Generate license keys: npm run generate-keys');
        console.log('   3. Start the server: npm start\n');

    } catch (error) {
        console.error('❌ Database initialization failed:', error.message);
        process.exit(1);
    } finally {
        if (connection) {
            await connection.end();
        }
    }
}

// Run if called directly
if (require.main === module) {
    initializeDatabase();
}

module.exports = { initializeDatabase };

