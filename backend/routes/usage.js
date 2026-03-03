const express = require('express');
const router = express.Router();
const { query, queryOne } = require('../config/database');

// GET /api/usage - Get current usage for a license
router.get('/', async (req, res) => {
    try {
        const { license_key, customer_email } = req.query;

        if (!license_key || !customer_email) {
            return res.status(400).json({
                success: false,
                error: 'License key and email are required'
            });
        }

        const normalizedKey = license_key.trim().toUpperCase();
        const normalizedEmail = customer_email.trim().toLowerCase();

        // Verify license and email
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [normalizedKey]
        );

        if (!license) {
            return res.status(404).json({ success: false, error: 'License not found' });
        }

        const licenseEmail = (license.customer_email || '').trim().toLowerCase();
        if (normalizedEmail !== licenseEmail) {
            return res.status(403).json({ success: false, error: 'Email does not match' });
        }

        // Get plan details
        const plan = await queryOne(
            'SELECT * FROM subscription_plans WHERE plan_code = ?',
            [license.plan_code || 'STARTER']
        );

        // Get current month usage
        const now = new Date();
        const usage = await queryOne(
            `SELECT receipts_processed FROM usage_tracking 
             WHERE license_key = ? AND period_year = ? AND period_month = ?`,
            [normalizedKey, now.getFullYear(), now.getMonth() + 1]
        );

        const receiptsUsed = usage ? usage.receipts_processed : 0;
        const monthlyLimit = plan ? plan.receipts_per_month : 100;
        const isUnlimited = monthlyLimit === -1;

        // Calculate days until reset
        const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
        const daysUntilReset = Math.ceil((nextMonth - now) / (1000 * 60 * 60 * 24));

        res.json({
            success: true,
            plan_code: license.plan_code || 'STARTER',
            plan_name: plan ? plan.plan_name : 'Starter',
            billing_cycle: license.billing_cycle || 'YEARLY',
            receipts_used: receiptsUsed,
            receipts_limit: isUnlimited ? -1 : monthlyLimit,
            receipts_remaining: isUnlimited ? -1 : Math.max(0, monthlyLimit - receiptsUsed),
            is_unlimited: isUnlimited,
            limit_reached: !isUnlimited && receiptsUsed >= monthlyLimit,
            days_until_reset: daysUntilReset,
            subscription_end: license.subscription_end
        });

    } catch (error) {
        console.error('❌ Error getting usage:', error);
        res.status(500).json({ success: false, error: 'Internal server error' });
    }
});

// POST /api/usage/increment - Increment usage count
router.post('/increment', async (req, res) => {
    try {
        const { license_key, customer_email, count = 1, device_fingerprint } = req.body;

        if (!license_key || !customer_email) {
            return res.status(400).json({
                success: false,
                error: 'License key and email are required'
            });
        }

        const normalizedKey = license_key.trim().toUpperCase();
        const normalizedEmail = customer_email.trim().toLowerCase();

        // Verify license and email
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [normalizedKey]
        );

        if (!license) {
            return res.status(404).json({ success: false, error: 'License not found' });
        }

        const licenseEmail = (license.customer_email || '').trim().toLowerCase();
        if (normalizedEmail !== licenseEmail) {
            return res.status(403).json({ success: false, error: 'Email does not match' });
        }

        // Get plan details
        const plan = await queryOne(
            'SELECT * FROM subscription_plans WHERE plan_code = ?',
            [license.plan_code || 'STARTER']
        );

        const monthlyLimit = plan ? plan.receipts_per_month : 100;
        const isUnlimited = monthlyLimit === -1;

        // Get current usage
        const now = new Date();
        const currentUsage = await queryOne(
            `SELECT receipts_processed FROM usage_tracking 
             WHERE license_key = ? AND period_year = ? AND period_month = ?`,
            [normalizedKey, now.getFullYear(), now.getMonth() + 1]
        );

        const currentCount = currentUsage ? currentUsage.receipts_processed : 0;

        // Check if limit would be exceeded (skip for unlimited)
        if (!isUnlimited && currentCount + count > monthlyLimit) {
            const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
            const daysUntilReset = Math.ceil((nextMonth - now) / (1000 * 60 * 60 * 24));

            return res.status(403).json({
                success: false,
                error: 'Monthly limit reached',
                limit_reached: true,
                receipts_used: currentCount,
                receipts_limit: monthlyLimit,
                days_until_reset: daysUntilReset,
                plan_code: license.plan_code || 'STARTER'
            });
        }

        // Insert or update usage tracking
        await query(
            `INSERT INTO usage_tracking (license_key, period_year, period_month, receipts_processed)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE receipts_processed = receipts_processed + ?`,
            [normalizedKey, now.getFullYear(), now.getMonth() + 1, count, count]
        );

        // Log the usage
        await query(
            `INSERT INTO usage_log (license_key, device_fingerprint, receipts_count)
             VALUES (?, ?, ?)`,
            [normalizedKey, device_fingerprint || null, count]
        );

        const newCount = currentCount + count;

        console.log(`📊 Usage incremented: ${normalizedKey} - ${newCount}/${isUnlimited ? '∞' : monthlyLimit}`);

        res.json({
            success: true,
            receipts_used: newCount,
            receipts_limit: isUnlimited ? -1 : monthlyLimit,
            receipts_remaining: isUnlimited ? -1 : Math.max(0, monthlyLimit - newCount)
        });

    } catch (error) {
        console.error('❌ Error incrementing usage:', error);
        res.status(500).json({ success: false, error: 'Internal server error' });
    }
});

// GET /api/usage/plans - Get available subscription plans
router.get('/plans', async (req, res) => {
    try {
        const plans = await query(
            'SELECT plan_code, plan_name, receipts_per_month, price_monthly, price_yearly FROM subscription_plans WHERE is_active = TRUE ORDER BY receipts_per_month ASC'
        );

        res.json({
            success: true,
            plans: plans.map(p => ({
                code: p.plan_code,
                name: p.plan_name,
                receipts_per_month: p.receipts_per_month,
                is_unlimited: p.receipts_per_month === -1,
                price_monthly: parseFloat(p.price_monthly),
                price_yearly: parseFloat(p.price_yearly)
            }))
        });

    } catch (error) {
        console.error('❌ Error getting plans:', error);
        res.status(500).json({ success: false, error: 'Internal server error' });
    }
});

// POST /api/usage/check - Check if processing is allowed (without incrementing)
router.post('/check', async (req, res) => {
    try {
        const { license_key, customer_email, count = 1 } = req.body;

        if (!license_key || !customer_email) {
            return res.status(400).json({
                allowed: false,
                error: 'License key and email are required'
            });
        }

        const normalizedKey = license_key.trim().toUpperCase();
        const normalizedEmail = customer_email.trim().toLowerCase();

        // Verify license and email
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [normalizedKey]
        );

        if (!license) {
            return res.status(404).json({ allowed: false, error: 'License not found' });
        }

        const licenseEmail = (license.customer_email || '').trim().toLowerCase();
        if (normalizedEmail !== licenseEmail) {
            return res.status(403).json({ allowed: false, error: 'Email does not match' });
        }

        // Get plan details
        const plan = await queryOne(
            'SELECT * FROM subscription_plans WHERE plan_code = ?',
            [license.plan_code || 'STARTER']
        );

        const monthlyLimit = plan ? plan.receipts_per_month : 100;
        const isUnlimited = monthlyLimit === -1;

        // Get current usage
        const now = new Date();
        const currentUsage = await queryOne(
            `SELECT receipts_processed FROM usage_tracking
             WHERE license_key = ? AND period_year = ? AND period_month = ?`,
            [normalizedKey, now.getFullYear(), now.getMonth() + 1]
        );

        const currentCount = currentUsage ? currentUsage.receipts_processed : 0;
        const wouldExceed = !isUnlimited && (currentCount + count > monthlyLimit);

        const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
        const daysUntilReset = Math.ceil((nextMonth - now) / (1000 * 60 * 60 * 24));

        res.json({
            allowed: !wouldExceed,
            receipts_used: currentCount,
            receipts_limit: isUnlimited ? -1 : monthlyLimit,
            receipts_remaining: isUnlimited ? -1 : Math.max(0, monthlyLimit - currentCount),
            is_unlimited: isUnlimited,
            days_until_reset: daysUntilReset,
            plan_code: license.plan_code || 'STARTER',
            plan_name: plan ? plan.plan_name : 'Starter'
        });

    } catch (error) {
        console.error('❌ Error checking usage:', error);
        res.status(500).json({ allowed: false, error: 'Internal server error' });
    }
});

module.exports = router;

