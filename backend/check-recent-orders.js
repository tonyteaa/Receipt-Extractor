const { query } = require('./config/database');
require('dotenv').config();

async function checkRecentOrders() {
  try {
    console.log('📊 Checking recent orders and licenses...\n');
    
    // Check recent licenses
    const licenses = await query(`
      SELECT license_key, tier, customer_email, customer_name, order_id, assigned_at 
      FROM license_keys 
      ORDER BY assigned_at DESC 
      LIMIT 5
    `);
    
    console.log('🔑 Recent Licenses:');
    if (licenses.length === 0) {
      console.log('   No licenses found.');
    } else {
      licenses.forEach(license => {
        console.log(`   - ${license.license_key}`);
        console.log(`     Tier: ${license.tier}`);
        console.log(`     Email: ${license.customer_email}`);
        console.log(`     Name: ${license.customer_name}`);
        console.log(`     Order ID: ${license.order_id}`);
        console.log(`     Assigned: ${license.assigned_at}`);
        console.log('');
      });
    }
    
    // Check recent orders
    const orders = await query(`
      SELECT order_id, order_number, customer_email, customer_name, product_tier, amount_paid, currency, license_key, created_at 
      FROM shopify_orders 
      ORDER BY created_at DESC 
      LIMIT 5
    `);
    
    console.log('📦 Recent Shopify Orders:');
    if (orders.length === 0) {
      console.log('   No orders found.');
    } else {
      orders.forEach(order => {
        console.log(`   - Order ${order.order_number}`);
        console.log(`     Order ID: ${order.order_id}`);
        console.log(`     Email: ${order.customer_email}`);
        console.log(`     Name: ${order.customer_name}`);
        console.log(`     Tier: ${order.product_tier}`);
        console.log(`     Amount: ${order.currency} ${order.amount_paid}`);
        console.log(`     License: ${order.license_key}`);
        console.log(`     Created: ${order.created_at}`);
        console.log('');
      });
    }
    
    process.exit(0);
  } catch (error) {
    console.error('❌ Error:', error);
    process.exit(1);
  }
}

checkRecentOrders();

