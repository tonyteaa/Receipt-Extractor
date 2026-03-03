require('dotenv').config();
const mysql = require('mysql2/promise');

async function checkRecentLicenses() {
    const connection = await mysql.createConnection({
        host: process.env.DB_HOST || 'localhost',
        user: process.env.DB_USER || 'Antoine',
        password: process.env.DB_PASSWORD,
        database: process.env.DB_NAME || 'receipt_extractor'
    });

    console.log('\n📋 Most Recent Licenses (Last 10):\n');
    console.log('='.repeat(120));

    const [licenses] = await connection.execute(
        `SELECT 
            license_key,
            tier,
            customer_email,
            customer_name,
            order_id,
            assigned_at,
            TIMESTAMPDIFF(MINUTE, assigned_at, NOW()) as minutes_ago
        FROM license_keys 
        ORDER BY assigned_at DESC 
        LIMIT 10`
    );

    if (licenses.length === 0) {
        console.log('No licenses found in database.');
    } else {
        licenses.forEach((license, index) => {
            const timeAgo = license.minutes_ago < 60 
                ? `${license.minutes_ago} minutes ago`
                : `${Math.floor(license.minutes_ago / 60)} hours ago`;
            
            console.log(`${index + 1}. ${license.license_key} (${license.tier})`);
            console.log(`   Customer: ${license.customer_name || 'N/A'} <${license.customer_email || 'N/A'}>`);
            console.log(`   Order ID: ${license.order_id || 'N/A'}`);
            console.log(`   Created: ${license.assigned_at} (${timeAgo})`);
            console.log('-'.repeat(120));
        });
    }

    await connection.end();
}

checkRecentLicenses().catch(console.error);

