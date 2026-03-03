const axios = require('axios');
require('dotenv').config();

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

// Use the most recent order ID from your test
const TEST_ORDER_ID = '7947737137452'; // Order 1007
const TEST_LICENSE_KEY = 'BUDGET-TEST-1234-5678';
const TEST_TIER = 'BUDGET';

async function testAddLicenseToOrder() {
  try {
    console.log('🧪 Testing: Add license to Shopify order');
    console.log(`Order ID: ${TEST_ORDER_ID}`);
    console.log(`License: ${TEST_LICENSE_KEY}\n`);
    
    const noteContent = `🔑 LICENSE KEY: ${TEST_LICENSE_KEY}\n\nTier: ${TEST_TIER}\nActivations: ${TEST_TIER === 'PRO' ? '5' : '3'} devices\n\nThank you for your purchase!`;
    
    const response = await axios.put(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/orders/${TEST_ORDER_ID}.json`,
      {
        order: {
          id: parseInt(TEST_ORDER_ID),
          note: noteContent,
          note_attributes: [
            {
              name: "License Key",
              value: TEST_LICENSE_KEY
            },
            {
              name: "License Tier",
              value: TEST_TIER
            }
          ]
        }
      },
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
          'Content-Type': 'application/json'
        }
      }
    );
    
    console.log('✅ Successfully added license to order!');
    console.log('Order Note:', response.data.order.note);
    console.log('\n📧 Now check the order in Shopify admin:');
    console.log(`https://${SHOPIFY_STORE_URL}/admin/orders/${TEST_ORDER_ID}`);
    
  } catch (error) {
    console.error('❌ Failed to add license to order!');
    if (error.response) {
      console.error('Status:', error.response.status);
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
  }
}

testAddLicenseToOrder();

