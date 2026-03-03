const axios = require('axios');
const readline = require('readline');
require('dotenv').config();

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

async function registerWebhook(ngrokUrl) {
  try {
    const webhookData = {
      webhook: {
        topic: "orders/create",
        address: `${ngrokUrl}/api/shopify/order-created`,
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

    console.log('✅ Webhook registered successfully!');
    console.log('Webhook ID:', response.data.webhook.id);
    console.log('Topic:', response.data.webhook.topic);
    console.log('Address:', response.data.webhook.address);
    console.log('\n🎉 Ready to receive orders!');
  } catch (error) {
    console.error('❌ Failed to register webhook!');
    if (error.response) {
      console.error('Status:', error.response.status);
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
  }
}

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
  } catch (error) {
    console.error('❌ Failed to list webhooks!');
    if (error.response) {
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
  }
}

async function main() {
  console.log('🔗 Shopify Webhook Registration\n');
  
  await listWebhooks();
  
  rl.question('\nEnter your ngrok URL (e.g., https://abc123.ngrok.io): ', (ngrokUrl) => {
    ngrokUrl = ngrokUrl.trim();
    
    if (!ngrokUrl.startsWith('http')) {
      console.error('❌ Invalid URL. Must start with http:// or https://');
      rl.close();
      return;
    }
    
    registerWebhook(ngrokUrl);
    rl.close();
  });
}

main();

