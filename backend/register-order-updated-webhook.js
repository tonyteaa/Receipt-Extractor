require('dotenv').config();
const axios = require('axios');

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;
const NGROK_URL = 'https://rosily-pithecoid-joye.ngrok-free.dev'; // Your ngrok URL

async function registerOrderUpdatedWebhook() {
    try {
        console.log('🔄 Registering orders/updated webhook...\n');

        const response = await axios.post(
            `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks.json`,
            {
                webhook: {
                    topic: 'orders/updated',
                    address: `${NGROK_URL}/api/shopify/order-updated`,
                    format: 'json'
                }
            },
            {
                headers: {
                    'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
                    'Content-Type': 'application/json'
                }
            }
        );

        console.log('✅ Webhook registered successfully!\n');
        console.log('Webhook Details:');
        console.log(`  ID: ${response.data.webhook.id}`);
        console.log(`  Topic: ${response.data.webhook.topic}`);
        console.log(`  Address: ${response.data.webhook.address}`);
        console.log(`  Created: ${response.data.webhook.created_at}`);

    } catch (error) {
        if (error.response?.data?.errors) {
            console.error('❌ Error:', JSON.stringify(error.response.data.errors, null, 2));
        } else {
            console.error('❌ Error:', error.response?.data || error.message);
        }
    }
}

registerOrderUpdatedWebhook();

