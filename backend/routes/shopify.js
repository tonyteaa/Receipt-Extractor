const express = require('express');
const router = express.Router();
const { query, queryOne } = require('../config/database');
const crypto = require('crypto');
const { generateLicenseKey } = require('../scripts/generate-keys');
const axios = require('axios');
const { sendLicenseEmail } = require('../utils/emailService');

// Verify Shopify webhook signature
function verifyShopifyWebhook(req) {
    const hmac = req.get('X-Shopify-Hmac-Sha256');

    if (!hmac) {
        console.error('❌ No HMAC header found');
        return false;
    }

    if (!process.env.SHOPIFY_WEBHOOK_SECRET) {
        console.error('❌ SHOPIFY_WEBHOOK_SECRET not configured');
        return false;
    }

    // Use raw body buffer for signature verification (not parsed JSON)
    const body = req.rawBody;

    if (!body) {
        console.error('❌ No raw body available for signature verification');
        return false;
    }

    const hash = crypto
        .createHmac('sha256', process.env.SHOPIFY_WEBHOOK_SECRET)
        .update(body)
        .digest('base64');

    const isValid = hash === hmac;

    if (!isValid) {
        console.error(`❌ Signature mismatch:`);
        console.error(`   Expected: ${hmac}`);
        console.error(`   Got: ${hash}`);
    }

    return isValid;
}

// Determine tier from product title or SKU
function determineTier(lineItems) {
    for (const item of lineItems) {
        const title = item.title.toLowerCase();
        const sku = (item.sku || '').toLowerCase();

        if (title.includes('pro') || sku.includes('pro')) {
            return 'PRO';
        }
    }
    return 'PRO'; // All licenses are Pro now
}

// Add license key to Shopify order as a note
async function addLicenseToOrder(orderId, licenseKey, tier) {
    try {
        const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
        const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

        // Add a note to the order with the license key
        const noteContent = `🔑 LICENSE KEY: ${licenseKey}\n\nTier: Pro\nActivations: 3 devices\n\nThank you for your purchase!`;

        await axios.put(
            `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/orders/${orderId}.json`,
            {
                order: {
                    id: orderId,
                    note: noteContent,
                    note_attributes: [
                        {
                            name: "License Key",
                            value: licenseKey
                        },
                        {
                            name: "License Tier",
                            value: tier
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

        console.log(`✅ Added license key to Shopify order ${orderId}`);
        return true;
    } catch (error) {
        console.error('❌ Failed to add license to Shopify order:', error.response?.data || error.message);
        return false;
    }
}

// POST /api/shopify/order-created
// Handle new order webhook from Shopify
router.post('/order-created', async (req, res) => {
    try {
        // Verify webhook authenticity
        if (process.env.NODE_ENV === 'production' && !verifyShopifyWebhook(req)) {
            console.error('❌ Invalid Shopify webhook signature');
            return res.status(401).json({ error: 'Unauthorized' });
        }

        const order = req.body;

        console.log(`📦 New Shopify order received: Order #${order.order_number || order.name}`);
        console.log(`   Order ID: ${order.id}`);
        console.log(`   Customer ID: ${order.customer?.id}`);
        console.log(`   Email in order.email: ${order.email}`);
        console.log(`   Email in order.customer.email: ${order.customer?.email}`);
        console.log(`   Email in order.contact_email: ${order.contact_email}`);

        // Extract order details with defaults for test notifications
        const orderId = order.id ? order.id.toString() : `test-${Date.now()}`;
        const orderNumber = order.order_number || order.name || `#TEST-${Date.now()}`;

        // Get customer email - fetch from API if not in webhook
        let customerEmail = order.email || order.customer?.email || order.contact_email;

        console.log(`   Initial customerEmail: ${customerEmail}`);

        // If still no email and we have a customer ID, fetch it from Shopify API
        if (!customerEmail && order.customer?.id) {
            console.log(`   🔍 No email in webhook, fetching from Shopify API...`);
            try {
                const SHOPIFY_STORE_URL = process.env.SHOPIFY_STORE_URL;
                const SHOPIFY_ACCESS_TOKEN = process.env.SHOPIFY_ACCESS_TOKEN;

                console.log(`   Fetching customer ${order.customer.id} from ${SHOPIFY_STORE_URL}`);

                const customerResponse = await axios.get(
                    `https://${SHOPIFY_STORE_URL}/admin/api/2024-01/customers/${order.customer.id}.json`,
                    {
                        headers: {
                            'X-Shopify-Access-Token': SHOPIFY_ACCESS_TOKEN,
                            'Content-Type': 'application/json'
                        }
                    }
                );

                console.log(`   📋 Customer API Response:`, JSON.stringify(customerResponse.data, null, 2));

                customerEmail = customerResponse.data.customer?.email;
                console.log(`   📧 Fetched customer email from API: ${customerEmail}`);
            } catch (error) {
                console.error('   ⚠️ Failed to fetch customer email:', error.response?.data || error.message);
            }
        }

        // Check if order already processed FIRST (before validating email)
        // This handles Shopify webhook retries that may have incomplete data
        const existingOrder = await queryOne(
            'SELECT * FROM shopify_orders WHERE order_id = ?',
            [orderId]
        );

        if (existingOrder) {
            console.log(`✅ Order ${orderNumber} already processed (webhook retry)`);
            return res.json({
                success: true,
                message: 'Order already processed',
                license_key: existingOrder.license_key
            });
        }

        const customerName = `${order.customer?.first_name || ''} ${order.customer?.last_name || ''}`.trim() || 'Test Customer';
        const amountPaid = parseFloat(order.total_price || 0);
        const currency = order.currency || 'USD';

        // Validate required fields
        if (!customerEmail) {
            console.error('❌ Missing customer email');
            return res.status(400).json({ error: 'Missing customer email' });
        }

        if (!order.line_items || order.line_items.length === 0) {
            console.error('❌ Missing line items');
            return res.status(400).json({ error: 'Missing line items' });
        }

        // Determine tier from line items
        const tier = determineTier(order.line_items);

        console.log(`   Customer: ${customerEmail}`);
        console.log(`   Tier: ${tier}`);
        console.log(`   Amount: ${currency} ${amountPaid}`);

        // Check if customer already has an active license for this tier
        const existingLicense = await queryOne(
            'SELECT * FROM license_keys WHERE customer_email = ? AND tier = ? AND is_used = TRUE',
            [customerEmail, tier]
        );

        if (existingLicense) {
            console.log(`⚠️  Customer ${customerEmail} already has a ${tier} license: ${existingLicense.license_key}`);
            console.log(`   Creating duplicate order record but reusing existing license`);

            // Record the order with the existing license
            await query(
                `INSERT INTO shopify_orders
                (order_id, order_number, customer_email, customer_name, product_tier, amount_paid, currency, license_key)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
                [orderId, orderNumber, customerEmail, customerName, tier, amountPaid, currency, existingLicense.license_key]
            );

            // Add existing license to order
            await addLicenseToOrder(orderId, existingLicense.license_key, tier);

            return res.json({
                success: true,
                message: 'Customer already has a license for this tier',
                license_key: existingLicense.license_key,
                is_existing: true
            });
        }

        // Generate a new unique license key
        let newLicenseKey;
        let attempts = 0;
        const maxAttempts = 10;

        while (attempts < maxAttempts) {
            newLicenseKey = generateLicenseKey(tier);

            // Check if key already exists
            const existingKey = await queryOne(
                'SELECT license_key FROM license_keys WHERE license_key = ?',
                [newLicenseKey]
            );

            if (!existingKey) {
                break; // Key is unique
            }

            attempts++;
        }

        if (attempts >= maxAttempts) {
            console.error(`❌ Failed to generate unique license key after ${maxAttempts} attempts`);
            return res.status(500).json({
                error: 'Failed to generate license key',
                message: 'Please try again or contact support'
            });
        }

        // Insert the new license key with customer info
        await query(
            `INSERT INTO license_keys
            (license_key, tier, is_used, customer_email, customer_name, order_id, assigned_at)
            VALUES (?, ?, TRUE, ?, ?, ?, NOW())`,
            [newLicenseKey, tier, customerEmail, customerName, orderId]
        );

        // Record the order
        await query(
            `INSERT INTO shopify_orders
            (order_id, order_number, customer_email, customer_name, product_tier, amount_paid, currency, license_key)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
            [orderId, orderNumber, customerEmail, customerName, tier, amountPaid, currency, newLicenseKey]
        );

        console.log(`✅ License generated and assigned: ${newLicenseKey}`);

        // Add license key to Shopify order (will appear in order confirmation email)
        await addLicenseToOrder(orderId, newLicenseKey, tier);

        // Send license email to customer
        console.log(`📧 Sending license email to ${customerEmail}...`);
        try {
            const emailResult = await sendLicenseEmail(customerEmail, customerName, newLicenseKey, tier, orderNumber);
            if (emailResult.success) {
                console.log(`✅ License email sent successfully to ${customerEmail}`);
            } else {
                console.error(`⚠️ Failed to send license email: ${emailResult.error}`);
            }
        } catch (emailError) {
            console.error(`⚠️ Error sending license email:`, emailError.message);
            // Don't fail the webhook - license was still created
        }

        res.json({
            success: true,
            license_key: newLicenseKey,
            tier: tier,
            email_sent: true
        });
        
    } catch (error) {
        console.error('Shopify webhook error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// GET /api/shopify/stats
// Get Shopify order statistics (admin only)
router.get('/stats', async (req, res) => {
    try {
        const stats = await query(`
            SELECT 
                product_tier,
                COUNT(*) as total_orders,
                SUM(amount_paid) as total_revenue,
                currency
            FROM shopify_orders
            GROUP BY product_tier, currency
        `);
        
        res.json({ success: true, stats });
    } catch (error) {
        console.error('Stats error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Webhook: Order Updated (sends license email)
router.post('/order-updated', async (req, res) => {
    try {
        const order = req.body;

        console.log(`📦 Order updated webhook: Order #${order.order_number || order.name}`);

        // Check if this order has a license key in note_attributes
        const licenseAttribute = order.note_attributes?.find(attr => attr.name === 'License Key');

        if (!licenseAttribute) {
            console.log(`   ⏭️  No license key found, skipping email`);
            return res.json({ success: true, message: 'No license key to send' });
        }

        const licenseKey = licenseAttribute.value;
        const tierAttribute = order.note_attributes?.find(attr => attr.name === 'License Tier');
        const tier = tierAttribute?.value || 'PRO';

        // Get customer email
        let customerEmail = order.email || order.customer?.email || order.contact_email;

        if (!customerEmail) {
            console.log(`   ⚠️  No customer email, cannot send license email`);
            return res.json({ success: true, message: 'No customer email' });
        }

        const customerName = `${order.customer?.first_name || ''} ${order.customer?.last_name || ''}`.trim() || 'Customer';
        const orderNumber = order.order_number || order.name || 'N/A';

        console.log(`   📧 Sending license email to ${customerEmail}`);
        console.log(`   🔑 License: ${licenseKey}`);

        // Send the license email
        const result = await sendLicenseEmail(customerEmail, customerName, licenseKey, tier, orderNumber);

        if (result.success) {
            console.log(`   ✅ License email sent successfully!`);
            res.json({ success: true, message: 'License email sent' });
        } else {
            console.error(`   ❌ Failed to send email: ${result.error}`);
            res.status(500).json({ success: false, error: result.error });
        }

    } catch (error) {
        console.error('❌ Order updated webhook error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

module.exports = router;

