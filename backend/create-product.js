const axios = require('axios');
require('dotenv').config();

const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

async function createProduct() {
  try {
    // Create the product
    const productData = {
      product: {
        title: "Receipt Extractor - Budget License",
        body_html: "<p>Extract data from receipts with AI-powered OCR technology.</p><ul><li>Up to 3 device activations</li><li>Unlimited receipt processing</li><li>AI-powered extraction</li><li>Lifetime license</li></ul>",
        vendor: "CB Tools",
        product_type: "Software License",
        tags: ["software", "license", "receipt-extractor", "budget"],
        variants: [
          {
            price: "9.99",
            sku: "RECEIPT-BUDGET",
            inventory_management: null,
            inventory_policy: "continue"
          }
        ],
        metafields: [
          {
            namespace: "license",
            key: "tier",
            value: "budget",
            type: "single_line_text_field"
          },
          {
            namespace: "license",
            key: "max_activations",
            value: "3",
            type: "number_integer"
          }
        ]
      }
    };

    const response = await axios.post(
      `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/products.json`,
      productData,
      {
        headers: {
          'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
          'Content-Type': 'application/json'
        }
      }
    );

    console.log('✅ Product Created Successfully!');
    console.log('Product ID:', response.data.product.id);
    console.log('Product Title:', response.data.product.title);
    console.log('Product Price:', response.data.product.variants[0].price);
    console.log('Product URL:', `https://${SHOPIFY_STORE_URL}/admin/products/${response.data.product.id}`);
    console.log('\n🎉 You can now make a test purchase!');
    
    return response.data.product;
  } catch (error) {
    console.error('❌ Failed to create product!');
    if (error.response) {
      console.error('Status:', error.response.status);
      console.error('Error:', JSON.stringify(error.response.data, null, 2));
    } else {
      console.error('Error:', error.message);
    }
  }
}

createProduct();

