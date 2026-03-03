const axios = require('axios');
require('dotenv').config();

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;
const NGROK_URL = 'https://rosily-pithecoid-joye.ngrok-free.dev';

async function listWebhooks() {
  try {
    const response = await axios.get(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks.json`,
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN
        }
      }
    );

    console.log('\n📋 Existing Webhooks:');
    if (response.data.webhooks.length === 0) {
      console.log('   No webhooks registered yet.');
    } else {
      response.data.webhooks.forEach(webhook => {
        console.log(`   - ID: ${webhook.id}`);
        console.log(`     Topic: ${webhook.topic}`);
        console.log(`     Address: ${webhook.address}`);
        console.log('');
      });
    }
    
    return response.data.webhooks;
  } catch (error) {
    console.error('❌ Failed to list webhooks!');
    if (error.response) {
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
    return [];
  }
}

async function deleteWebhook(webhookId) {
  try {
    await axios.delete(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks/${webhookId}.json`,
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN
        }
      }
    );
    console.log(`✅ Deleted webhook ${webhookId}`);
  } catch (error) {
    console.error(`❌ Failed to delete webhook ${webhookId}`);
  }
}

async function registerWebhook() {
  try {
    const webhookData = {
      webhook: {
        topic: "orders/create",
        address: `${NGROK_URL}/api/shopify/order-created`,
        format: "json"
      }
    };

    const response = await axios.post(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/webhooks.json`,
      webhookData,
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
          'Content-Type': 'application/json'
        }
      }
    );

    console.log('\n✅ Webhook registered successfully!');
    console.log('Webhook ID:', response.data.webhook.id);
    console.log('Topic:', response.data.webhook.topic);
    console.log('Address:', response.data.webhook.address);
    console.log('\n🎉 Ready to receive orders!');
    console.log('\n📦 Product URL: https://cb-tools.myshopify.com/admin/products/10162740166956');
    console.log('💰 Make a test purchase to see the magic happen!');
  } catch (error) {
    console.error('\n❌ Failed to register webhook!');
    if (error.response) {
      console.error('Status:', error.response.status);
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
  }
}

async function main() {
  console.log('🔗 Shopify Webhook Registration');
  console.log('📍 Ngrok URL:', NGROK_URL);
  
  const existingWebhooks = await listWebhooks();
  
  // Delete old webhooks for orders/create
  for (const webhook of existingWebhooks) {
    if (webhook.topic === 'orders/create') {
      console.log(`\n🗑️  Deleting old webhook: ${webhook.address}`);
      await deleteWebhook(webhook.id);
    }
  }
  
  await registerWebhook();
}

main();

