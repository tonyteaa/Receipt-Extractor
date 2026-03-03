require('dotenv').config();
const axios = require('axios');

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

async function checkWebhookConfig() {
    try {
        console.log('🔍 Checking webhook configuration...\n');

        // Get all webhooks
        const response = await axios.get(
            `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks.json`,
            {
                headers: {
                    'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
                    'Content-Type': 'application/json'
                }
            }
        );

        const webhooks = response.data.webhooks;

        console.log(`Found ${webhooks.length} webhook(s):\n`);

        webhooks.forEach((webhook, index) => {
            console.log(`Webhook #${index + 1}:`);
            console.log(`  ID: ${webhook.id}`);
            console.log(`  Topic: ${webhook.topic}`);
            console.log(`  Address: ${webhook.address}`);
            console.log(`  Format: ${webhook.format}`);
            console.log(`  API Version: ${webhook.api_version}`);
            console.log(`  Fields: ${webhook.fields || 'ALL (default)'}`);
            console.log(`  Metafield Namespaces: ${webhook.metafield_namespaces || 'NONE'}`);
            console.log(`  Private Metafield Namespaces: ${webhook.private_metafield_namespaces || 'NONE'}`);
            console.log('');
        });

        // Check if we need to update the webhook to include specific fields
        const orderWebhook = webhooks.find(w => w.topic === 'orders/create');
        
        if (orderWebhook) {
            console.log('📋 Current orders/create webhook configuration:');
            console.log(JSON.stringify(orderWebhook, null, 2));
            
            if (!orderWebhook.fields || orderWebhook.fields.length === 0) {
                console.log('\n⚠️  No specific fields configured - webhook sends ALL fields by default');
                console.log('   This should include customer email, but it might be missing due to:');
                console.log('   1. Customer privacy settings');
                console.log('   2. API version differences');
                console.log('   3. Shopify test mode behavior');
            }
        }

    } catch (error) {
        console.error('❌ Error checking webhook config:', error.response?.data || error.message);
    }
}

checkWebhookConfig();

