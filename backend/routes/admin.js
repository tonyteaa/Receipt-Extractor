const express = require('express');
const router = express.Router();
const { query, queryOne } = require('../config/database');

// Middleware to verify admin API key
function requireAdmin(req, res, next) {
    const apiKey = req.get('X-Admin-API-Key');
    
    if (!apiKey || apiKey !== process.env.ADMIN_API_KEY) {
        return res.status(401).json({ error: 'Unauthorized' });
    }
    
    next();
}

// Apply admin middleware to all routes
router.use(requireAdmin);

// GET /api/admin/stats
// Get overall statistics
router.get('/stats', async (req, res) => {
    try {
        const licenseStats = await query('SELECT * FROM license_statistics');
        const activationStats = await query('SELECT * FROM activation_statistics');
        
        const recentOrders = await query(`
            SELECT * FROM shopify_orders 
            ORDER BY created_at DESC 
            LIMIT 10
        `);
        
        const recentActivations = await query(`
            SELECT la.*, lk.tier, lk.customer_email
            FROM license_activations la
            JOIN license_keys lk ON la.license_key = lk.license_key
            ORDER BY la.activated_at DESC
            LIMIT 10
        `);
        
        res.json({
            success: true,
            license_stats: licenseStats,
            activation_stats: activationStats,
            recent_orders: recentOrders,
            recent_activations: recentActivations
        });
    } catch (error) {
        console.error('Stats error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// GET /api/admin/licenses
// Get all licenses with filters
router.get('/licenses', async (req, res) => {
    try {
        const { tier, is_used, customer_email } = req.query;
        const limitNum = parseInt(req.query.limit) || 100;
        const offsetNum = parseInt(req.query.offset) || 0;

        let sql = 'SELECT * FROM license_keys WHERE 1=1';
        const params = [];

        if (tier) {
            sql += ' AND tier = ?';
            params.push(tier.toUpperCase());
        }

        if (is_used !== undefined) {
            sql += ' AND is_used = ?';
            params.push(is_used === 'true' ? 1 : 0);
        }

        if (customer_email) {
            sql += ' AND customer_email LIKE ?';
            params.push(`%${customer_email}%`);
        }

        // Use string interpolation for LIMIT/OFFSET since they're validated integers
        sql += ` ORDER BY created_at DESC LIMIT ${limitNum} OFFSET ${offsetNum}`;

        const licenses = await query(sql, params);

        res.json({ success: true, licenses, count: licenses.length });
    } catch (error) {
        console.error('Get licenses error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// GET /api/admin/license/:key
// Get detailed info about a specific license
router.get('/license/:key', async (req, res) => {
    try {
        const { key } = req.params;
        
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [key.toUpperCase()]
        );
        
        if (!license) {
            return res.status(404).json({ error: 'License not found' });
        }
        
        const activations = await query(
            'SELECT * FROM license_activations WHERE license_key = ?',
            [key.toUpperCase()]
        );
        
        const order = await queryOne(
            'SELECT * FROM shopify_orders WHERE license_key = ?',
            [key.toUpperCase()]
        );
        
        res.json({
            success: true,
            license,
            activations,
            order
        });
    } catch (error) {
        console.error('Get license error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// POST /api/admin/license/revoke
// Revoke a license key (marks as revoked, doesn't delete)
router.post('/license/revoke', async (req, res) => {
    try {
        const { license_key, reason } = req.body;

        if (!license_key) {
            return res.status(400).json({ error: 'License key required' });
        }

        // Check if already revoked
        const existing = await queryOne(
            'SELECT revoked_at FROM license_keys WHERE license_key = ?',
            [license_key.toUpperCase()]
        );

        if (!existing) {
            return res.status(404).json({ error: 'License not found' });
        }

        if (existing.revoked_at) {
            return res.status(400).json({
                error: 'License already revoked',
                revoked_at: existing.revoked_at
            });
        }

        // Deactivate all activations
        await query(
            'UPDATE license_activations SET is_active = FALSE WHERE license_key = ?',
            [license_key.toUpperCase()]
        );

        // Mark license as revoked with timestamp
        await query(
            'UPDATE license_keys SET revoked_at = NOW(), notes = CONCAT(COALESCE(notes, ""), ?, ?) WHERE license_key = ?',
            ['\n[REVOKED] ', reason || 'No reason provided', license_key.toUpperCase()]
        );

        console.log(`🚫 License revoked: ${license_key} - Reason: ${reason || 'No reason provided'}`);

        res.json({
            success: true,
            message: 'License revoked successfully',
            revoked_at: new Date().toISOString()
        });
    } catch (error) {
        console.error('Revoke license error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// DELETE /api/admin/activation/:id
// Remove a specific device activation
router.delete('/activation/:id', async (req, res) => {
    try {
        const { id } = req.params;

        await query(
            'UPDATE license_activations SET is_active = FALSE WHERE id = ?',
            [id]
        );

        res.json({ success: true, message: 'Activation removed' });
    } catch (error) {
        console.error('Remove activation error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// GET /api/admin/search/customer?email=user@example.com
// Search licenses by customer email
router.get('/search/customer', async (req, res) => {
    try {
        const { email } = req.query;

        if (!email) {
            return res.status(400).json({
                success: false,
                error: 'Email parameter is required'
            });
        }

        const licenses = await query(
            `SELECT
                lk.*,
                COUNT(DISTINCT la.id) as active_devices,
                GROUP_CONCAT(DISTINCT la.device_name SEPARATOR ', ') as devices
            FROM license_keys lk
            LEFT JOIN license_activations la ON lk.license_key = la.license_key AND la.is_active = TRUE
            WHERE lk.customer_email LIKE ?
            GROUP BY lk.license_key
            ORDER BY lk.created_at DESC`,
            [`%${email}%`]
        );

        res.json({
            success: true,
            count: licenses.length,
            licenses
        });
    } catch (error) {
        console.error('Error searching by customer:', error);
        res.status(500).json({
            success: false,
            error: 'Failed to search licenses'
        });
    }
});

// GET /api/admin/license/:key/activations
// Get all device activations for a specific license
router.get('/license/:key/activations', async (req, res) => {
    try {
        const { key } = req.params;

        const activations = await query(
            `SELECT
                id,
                device_fingerprint,
                device_name,
                activated_at,
                last_seen,
                ip_address,
                is_active
            FROM license_activations
            WHERE license_key = ?
            ORDER BY activated_at DESC`,
            [key.toUpperCase()]
        );

        res.json({
            success: true,
            license_key: key.toUpperCase(),
            count: activations.length,
            activations
        });
    } catch (error) {
        console.error('Error fetching activations:', error);
        res.status(500).json({
            success: false,
            error: 'Failed to fetch activations'
        });
    }
});

// GET /api/admin/search/device?fingerprint=abc123...
// Search activations by device fingerprint
router.get('/search/device', async (req, res) => {
    try {
        const { fingerprint } = req.query;

        if (!fingerprint) {
            return res.status(400).json({
                success: false,
                error: 'Fingerprint parameter is required'
            });
        }

        const activations = await query(
            `SELECT
                la.*,
                lk.tier,
                lk.customer_email,
                lk.customer_name
            FROM license_activations la
            JOIN license_keys lk ON la.license_key = lk.license_key
            WHERE la.device_fingerprint LIKE ?
            ORDER BY la.last_seen DESC`,
            [`%${fingerprint}%`]
        );

        res.json({
            success: true,
            count: activations.length,
            activations
        });
    } catch (error) {
        console.error('Error searching by device:', error);
        res.status(500).json({
            success: false,
            error: 'Failed to search devices'
        });
    }
});

// GET /api/admin/search/all?q=searchterm
// Universal search across licenses, customers, and devices
router.get('/search/all', async (req, res) => {
    try {
        const { q } = req.query;

        if (!q) {
            return res.status(400).json({
                success: false,
                error: 'Search query parameter (q) is required'
            });
        }

        const searchTerm = `%${q}%`;

        // Search licenses
        const licenses = await query(
            `SELECT
                lk.*,
                COUNT(DISTINCT la.id) as active_devices
            FROM license_keys lk
            LEFT JOIN license_activations la ON lk.license_key = la.license_key AND la.is_active = TRUE
            WHERE lk.license_key LIKE ?
                OR lk.customer_email LIKE ?
                OR lk.customer_name LIKE ?
            GROUP BY lk.license_key
            LIMIT 50`,
            [searchTerm, searchTerm, searchTerm]
        );

        // Search devices
        const devices = await query(
            `SELECT
                la.*,
                lk.tier,
                lk.customer_email
            FROM license_activations la
            JOIN license_keys lk ON la.license_key = lk.license_key
            WHERE la.device_name LIKE ?
                OR la.device_fingerprint LIKE ?
            LIMIT 50`,
            [searchTerm, searchTerm]
        );

        res.json({
            success: true,
            query: q,
            results: {
                licenses: {
                    count: licenses.length,
                    data: licenses
                },
                devices: {
                    count: devices.length,
                    data: devices
                }
            }
        });
    } catch (error) {
        console.error('Error in universal search:', error);
        res.status(500).json({
            success: false,
            error: 'Failed to perform search'
        });
    }
});

// POST /api/admin/resend-email
// Resend license email to customer
router.post('/resend-email', async (req, res) => {
    try {
        const { license_key, email, name, tier } = req.body;

        if (!license_key || !email) {
            return res.status(400).json({
                success: false,
                error: 'License key and email are required'
            });
        }

        // Import email service
        const { sendLicenseEmail } = require('../utils/emailService');

        // Send the email
        const result = await sendLicenseEmail(
            email,
            name || 'Customer',
            license_key,
            tier || 'PRO',
            'Resent'
        );

        if (result.success) {
            console.log(`📧 License email resent to ${email} for ${license_key}`);
            res.json({
                success: true,
                message: `Email sent to ${email}`,
                emailId: result.emailId
            });
        } else {
            console.error(`❌ Failed to resend email: ${result.error}`);
            res.status(500).json({
                success: false,
                error: result.error
            });
        }
    } catch (error) {
        console.error('Error resending email:', error);
        res.status(500).json({
            success: false,
            error: 'Failed to resend email'
        });
    }
});

module.exports = router;

