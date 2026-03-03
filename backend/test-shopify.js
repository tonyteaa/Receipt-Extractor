const axios = require('axios');

const SHOPIFY_STORE_URL = 'cb-tools.myshopify.com';
const SHOPIFY_ACCESS_TOKEN = 'your_shopify_access_token_here';

async function testShopifyConnection() {
  try {
    const response = await axios.get(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/shop.json`,
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN
        }
      }
    );

    console.log('✅ Shopify API Connection Successful!');
    console.log('Store Name:', response.data.shop.name);
    console.log('Store Email:', response.data.shop.email);
    console.log('Store Domain:', response.data.shop.domain);
    console.log('\n🎉 Your Shopify integration is ready!');
  } catch (error) {
    console.error('❌ Shopify API Connection Failed!');
    if (error.response) {
      console.error('Status:', error.response.status);
      console.error('Error:', error.response.data);
    } else {
      console.error('Error:', error.message);
    }
  }
}

testShopifyConnection();

