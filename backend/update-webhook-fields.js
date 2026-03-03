require('dotenv').config();
const axios = require('axios');

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;
const WEBHOOK_ID = 1841676386604; // From the check-webhook-config output

async function updateWebhook() {
    try {
        console.log('🔄 Updating webhook to include email field...\n');

        // Update the webhook to use 2024-01 API version and explicitly include email
        const response = await axios.put(
            `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks/${WEBHOOK_ID}.json`,
            {
                webhook: {
                    id: WEBHOOK_ID,
                    // Explicitly request these fields to ensure email is included
                    fields: [
                        'id',
                        'email',
                        'contact_email',
                        'order_number',
                        'name',
                        'total_price',
                        'currency',
                        'financial_status',
                        'customer',
                        'line_items'
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

        console.log('✅ Webhook updated successfully!\n');
        console.log('Updated configuration:');
        console.log(JSON.stringify(response.data.webhook, null, 2));

    } catch (error) {
        console.error('❌ Error updating webhook:', error.response?.data || error.message);
    }
}

updateWebhook();

