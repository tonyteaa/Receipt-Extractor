const { query } = require('../config/database');
require('dotenv').config();

function generateSegment(length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

function generateLicenseKey(tier) {
    const prefix = tier.toUpperCase();
    const segments = [
        generateSegment(4),
        generateSegment(4),
        generateSegment(4)
    ];
    return `${prefix}-${segments.join('-')}`;
}

async function generateAndInsertKeys(tier, count) {
    console.log(`\n🔑 Generating ${count} ${tier.toUpperCase()} license keys...\n`);
    
    const keys = [];
    const insertedKeys = [];
    
    // Generate unique keys
    for (let i = 0; i < count; i++) {
        let key;
        do {
            key = generateLicenseKey(tier);
        } while (keys.includes(key));
        keys.push(key);
    }
    
    // Insert into database
    for (let i = 0; i < keys.length; i++) {
        try {
            await query(
                'INSERT INTO license_keys (license_key, tier) VALUES (?, ?)',
                [keys[i], tier.toUpperCase()]
            );
            insertedKeys.push(keys[i]);
            console.log(`${i + 1}. ${keys[i]} ✅`);
        } catch (error) {
            if (error.code === 'ER_DUP_ENTRY') {
                console.log(`${i + 1}. ${keys[i]} ⚠️  (duplicate, skipped)`);
            } else {
                console.error(`${i + 1}. ${keys[i]} ❌ Error:`, error.message);
            }
        }
    }
    
    console.log(`\n✅ Successfully inserted ${insertedKeys.length}/${count} keys\n`);
    return insertedKeys;
}

async function main() {
    const args = process.argv.slice(2);

    if (args.length < 1) {
        console.log('\n📋 License Key Generator');
        console.log('════════════════════════════════════════');
        console.log('\nUsage:');
        console.log('  npm run generate-keys <count>');
        console.log('\nArguments:');
        console.log('  <count>  - Number of keys to generate');
        console.log('\nExamples:');
        console.log('  npm run generate-keys 100');
        console.log('  npm run generate-keys 50');
        console.log('\n════════════════════════════════════════\n');
        process.exit(0);
    }

    // All licenses are Pro tier now
    const tier = 'pro';
    const count = parseInt(args[0]);

    if (isNaN(count) || count < 1 || count > 1000) {
        console.error('❌ Error: Count must be between 1 and 1000');
        process.exit(1);
    }

    try {
        await generateAndInsertKeys(tier, count);
        
        // Show statistics
        const stats = await query(`
            SELECT 
                tier,
                COUNT(*) as total,
                SUM(CASE WHEN is_used = 0 THEN 1 ELSE 0 END) as available
            FROM license_keys
            GROUP BY tier
        `);
        
        console.log('📊 Current Database Statistics:');
        console.log('─────────────────────────────────────');
        stats.forEach(stat => {
            console.log(`${stat.tier}: ${stat.available} available / ${stat.total} total`);
        });
        console.log('─────────────────────────────────────\n');
        
        process.exit(0);
    } catch (error) {
        console.error('❌ Error:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = { generateLicenseKey, generateAndInsertKeys };

