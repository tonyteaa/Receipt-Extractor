require('dotenv').config();
const axios = require('axios');

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;
const ORDER_ID = '7948082118956'; // Order #1012

async function checkOrderAttributes() {
    try {
        console.log(`🔍 Checking order ${ORDER_ID} attributes...\n`);

        const response = await axios.get(
            `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/orders/${ORDER_ID}.json`,
            {
                headers: {
                    'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
                    'Content-Type': 'application/json'
                }
            }
        );

        const order = response.data.order;

        console.log(`Order #${order.order_number}:`);
        console.log(`  Order ID: ${order.id}`);
        console.log(`  Customer Email: ${order.email || 'NOT FOUND'}`);
        console.log(`  Contact Email: ${order.contact_email || 'NOT FOUND'}`);
        console.log('');

        console.log('📝 Note:');
        console.log(`  ${order.note || 'EMPTY'}`);
        console.log('');

        console.log('🏷️  Note Attributes:');
        if (order.note_attributes && order.note_attributes.length > 0) {
            order.note_attributes.forEach(attr => {
                console.log(`  - ${attr.name}: ${attr.value}`);
            });
        } else {
            console.log('  NONE - This is the problem! ❌');
        }
        console.log('');

        console.log('📋 Full Order Data (note_attributes section):');
        console.log(JSON.stringify({
            note: order.note,
            note_attributes: order.note_attributes
        }, null, 2));

    } catch (error) {
        console.error('❌ Error:', error.response?.data || error.message);
    }
}

checkOrderAttributes();

