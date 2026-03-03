const express = require('express');
const router = express.Router();
const { query, queryOne } = require('../config/database');
const crypto = require('crypto');

// Validate license key format
function isValidLicenseFormat(key) {
    if (!key || typeof key !== 'string') return false;

    const trimmed = key.trim().toUpperCase();

    // Check format: PRO-XXXX-XXXX-XXXX (18 chars)
    if (trimmed.startsWith('PRO-')) {
        return trimmed.length === 18 && /^PRO-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}$/.test(trimmed);
    }

    return false;
}

// Extract tier from license key (all licenses are PRO now)
function getTierFromKey(key) {
    const trimmed = key.trim().toUpperCase();
    if (trimmed.startsWith('PRO-')) return 'PRO';
    return null;
}

// POST /api/license/validate
// Validate a license key and optionally activate it on a device
router.post('/validate', async (req, res) => {
    try {
        const { license_key, device_fingerprint, device_name, ip_address } = req.body;

        console.log('📝 License validation request:', {
            license_key,
            device_fingerprint: device_fingerprint ? device_fingerprint.substring(0, 10) + '...' : 'none',
            device_name
        });

        // Validate input
        if (!license_key) {
            console.log('❌ No license key provided');
            return res.status(400).json({
                valid: false,
                error: 'License key is required'
            });
        }

        // Check format
        if (!isValidLicenseFormat(license_key)) {
            console.log('❌ Invalid license key format:', license_key);
            return res.status(400).json({
                valid: false,
                error: 'Invalid license key format'
            });
        }

        const normalizedKey = license_key.trim().toUpperCase();
        console.log('🔍 Looking up license:', normalizedKey);

        // Check if license exists in database
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [normalizedKey]
        );

        if (!license) {
            console.log('❌ License not found in database:', normalizedKey);
            // Log failed attempt
            await query(
                'INSERT INTO api_logs (license_key, endpoint, ip_address, response_status) VALUES (?, ?, ?, ?)',
                [normalizedKey, '/api/license/validate', ip_address || req.ip, 404]
            );

            return res.status(404).json({
                valid: false,
                error: 'License key not found'
            });
        }

        console.log('✅ License found:', {
            tier: license.tier,
            is_used: license.is_used,
            customer_email: license.customer_email,
            revoked_at: license.revoked_at
        });

        // Check if license is revoked
        if (license.revoked_at) {
            console.log('❌ License has been revoked:', license.revoked_at);
            return res.status(403).json({
                valid: false,
                error: 'This license has been revoked. Please contact support.',
                revoked_at: license.revoked_at
            });
        }

        // Check if license has been assigned to a customer (purchased)
        const licenseEmail = (license.customer_email || '').trim().toLowerCase();
        if (!licenseEmail) {
            console.log('❌ License has no associated email (not purchased through Shopify)');
            return res.status(400).json({
                valid: false,
                error: 'This license key has not been activated. Please purchase through cbtooling.com.'
            });
        }

        console.log('✅ License is valid and assigned to:', licenseEmail);

        // Check if license is already used by someone else
        if (license.is_used && license.customer_email) {
            // If device fingerprint provided, check if it's an existing activation
            if (device_fingerprint) {
                const existingActivation = await queryOne(
                    'SELECT * FROM license_activations WHERE license_key = ? AND device_fingerprint = ?',
                    [normalizedKey, device_fingerprint]
                );

                if (existingActivation) {
                    // Update last seen
                    await query(
                        'UPDATE license_activations SET last_seen = NOW() WHERE id = ?',
                        [existingActivation.id]
                    );

                    console.log('✅ Existing activation updated');

                    // Return success for existing activation
                    return res.json({
                        valid: true,
                        tier: license.tier.toLowerCase(),
                        api_keys: license.tier === 'PRO' ? {
                            groq: process.env.GROQ_API_KEY,
                            openai: process.env.OPENAI_API_KEY
                        } : undefined
                    });
                }
            }
        }
        
        // If device fingerprint provided, try to activate
        if (device_fingerprint) {
            // Check activation limit (3 devices per license)
            const maxActivations = parseInt(process.env.MAX_ACTIVATIONS_PRO || 3);

            const currentActivations = await query(
                'SELECT COUNT(*) as count FROM license_activations WHERE license_key = ? AND is_active = TRUE',
                [normalizedKey]
            );

            if (currentActivations[0].count >= maxActivations) {
                // Get list of active devices
                const activeDevices = await query(
                    'SELECT id, device_name, device_fingerprint, activated_at, last_seen FROM license_activations WHERE license_key = ? AND is_active = TRUE ORDER BY last_seen DESC',
                    [normalizedKey]
                );

                return res.status(403).json({
                    valid: true,
                    tier: license.tier.toLowerCase(),
                    error: 'Maximum device activations reached',
                    max_activations: maxActivations,
                    current_activations: currentActivations[0].count,
                    active_devices: activeDevices.map(d => ({
                        id: d.id,
                        name: d.device_name,
                        fingerprint: d.device_fingerprint.substring(0, 10) + '...',
                        activated_at: d.activated_at,
                        last_seen: d.last_seen
                    }))
                });
            }
            
            // Create new activation
            await query(
                `INSERT INTO license_activations 
                (license_key, device_fingerprint, device_name, ip_address) 
                VALUES (?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE 
                last_seen = NOW(), is_active = TRUE`,
                [normalizedKey, device_fingerprint, device_name, ip_address || req.ip]
            );
            
            console.log(`✅ License activated: ${normalizedKey} on device ${device_fingerprint}`);
        }
        
        // Log successful validation
        await query(
            'INSERT INTO api_logs (license_key, endpoint, ip_address, response_status) VALUES (?, ?, ?, ?)',
            [normalizedKey, '/api/license/validate', ip_address || req.ip, 200]
        );
        
        // Return success with tier and API keys (if Pro)
        const response = {
            valid: true,
            tier: license.tier.toLowerCase()
        };
        
        if (license.tier === 'PRO') {
            response.api_keys = {
                groq: process.env.GROQ_API_KEY,
                openai: process.env.OPENAI_API_KEY
            };
        }
        
        res.json(response);
        
    } catch (error) {
        console.error('License validation error:', error);
        res.status(500).json({
            valid: false,
            error: 'Internal server error'
        });
    }
});

// POST /api/license/deactivate-device
// Deactivate a device to free up a slot
router.post('/deactivate-device', async (req, res) => {
    try {
        const { license_key, device_fingerprint, customer_email } = req.body;

        console.log('📝 Device deactivation request:', {
            license_key,
            customer_email,
            device_fingerprint: device_fingerprint ? device_fingerprint.substring(0, 10) + '...' : 'none'
        });

        // Validate input
        if (!license_key || !device_fingerprint || !customer_email) {
            return res.status(400).json({
                success: false,
                error: 'License key, device fingerprint, and email are required'
            });
        }

        const normalizedKey = license_key.trim().toUpperCase();
        const normalizedEmail = customer_email.trim().toLowerCase();

        // Verify license exists and email matches
        const license = await queryOne(
            'SELECT * FROM license_keys WHERE license_key = ?',
            [normalizedKey]
        );

        if (!license) {
            return res.status(404).json({
                success: false,
                error: 'License key not found'
            });
        }

        const licenseEmail = (license.customer_email || '').trim().toLowerCase();
        if (normalizedEmail !== licenseEmail) {
            return res.status(403).json({
                success: false,
                error: 'Email does not match the license owner'
            });
        }

        // Deactivate the device
        const result = await query(
            'UPDATE license_activations SET is_active = FALSE WHERE license_key = ? AND device_fingerprint = ?',
            [normalizedKey, device_fingerprint]
        );

        if (result.affectedRows === 0) {
            return res.status(404).json({
                success: false,
                error: 'Device activation not found'
            });
        }

        console.log(`✅ Device deactivated: ${device_fingerprint.substring(0, 10)}... for license ${normalizedKey}`);

        res.json({
            success: true,
            message: 'Device deactivated successfully'
        });

    } catch (error) {
        console.error('❌ Error deactivating device:', error);
        res.status(500).json({
            success: false,
            error: 'Internal server error'
        });
    }
});

module.exports = router;

