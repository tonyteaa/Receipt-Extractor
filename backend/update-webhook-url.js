require('dotenv').config();
const axios = require('axios');

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;
const PRODUCTION_URL = 'https://clever-vision-production.up.railway.app';

async function updateWebhooks() {
    try {
        console.log('🔄 Updating webhook URLs to production...\n');

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

        for (const webhook of webhooks) {
            // Check if this is pointing to ngrok or other dev URL
            if (!webhook.address.includes('clever-vision-production.up.railway.app')) {
                const newAddress = webhook.address.replace(
                    /https?:\/\/[^\/]+/,
                    PRODUCTION_URL
                );

                console.log(`Updating webhook #${webhook.id} (${webhook.topic}):`);
                console.log(`  Old: ${webhook.address}`);
                console.log(`  New: ${newAddress}`);

                // Update the webhook
                await axios.put(
                    `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks/${webhook.id}.json`,
                    {
                        webhook: {
                            id: webhook.id,
                            address: newAddress
                        }
                    },
                    {
                        headers: {
                            'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
                            'Content-Type': 'application/json'
                        }
                    }
                );

                console.log(`  ✅ Updated successfully!\n`);
            } else {
                console.log(`Webhook #${webhook.id} (${webhook.topic}) already pointing to production.`);
            }
        }

        console.log('\n🎉 All webhooks updated to production URL!');
        console.log('\nNext step: Make a test purchase on Shopify to verify.');

    } catch (error) {
        console.error('❌ Error updating webhooks:', error.response?.data || error.message);
    }
}

updateWebhooks();

