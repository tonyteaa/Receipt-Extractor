const crypto = require('crypto');
const axios = require('axios');

// Simulate a Shopify order webhook
const testOrder = {
    id: Date.now(), // Unique order ID
    email: "test-customer@example.com",
    customer: {
        first_name: "Test",
        last_name: "Customer",
        email: "test-customer@example.com"
    },
    line_items: [
        {
            title: "Receipt Extractor Pro License",
            sku: "RECEIPT-PRO",
            quantity: 1,
            price: "49.99"
        }
    ],
    total_price: "49.99",
    created_at: new Date().toISOString()
};

async function testWebhook() {
    const webhookSecret = process.env.SHOPIFY_WEBHOOK_SECRET || 'your-webhook-secret-here';
    const payload = JSON.stringify(testOrder);
    
    // Generate HMAC signature (like Shopify does)
    const hmac = crypto
        .createHmac('sha256', webhookSecret)
        .update(payload, 'utf8')
        .digest('base64');

    console.log('\n🧪 Testing Shopify Webhook...\n');
    console.log('Order Details:');
    console.log(`  Customer: ${testOrder.customer.first_name} ${testOrder.customer.last_name}`);
    console.log(`  Email: ${testOrder.email}`);
    console.log(`  Product: ${testOrder.line_items[0].title}`);
    console.log(`  Order ID: ${testOrder.id}`);
    console.log('\nSending webhook to backend...\n');

    try {
        const response = await axios.post('http://127.0.0.1:3000/api/shopify/order-created', testOrder, {
            headers: {
                'Content-Type': 'application/json',
                'X-Shopify-Hmac-Sha256': hmac,
                'X-Shopify-Topic': 'orders/create',
                'X-Shopify-Shop-Domain': 'test-shop.myshopify.com'
            }
        });

        console.log('✅ Webhook Response:');
        console.log(JSON.stringify(response.data, null, 2));
        
        if (response.data.license_key) {
            console.log('\n🎉 SUCCESS! License created:');
            console.log(`   License Key: ${response.data.license_key}`);
            console.log(`   Tier: ${response.data.tier}`);
            console.log(`   Customer: ${response.data.customer_email}`);
        }
    } catch (error) {
        console.error('❌ Webhook Failed:');
        if (error.response) {
            console.error(`   Status: ${error.response.status}`);
            console.error(`   Error: ${JSON.stringify(error.response.data, null, 2)}`);
        } else {
            console.error(`   Error: ${error.message}`);
        }
    }
}

// Load environment variables
require('dotenv').config();
testWebhook();

