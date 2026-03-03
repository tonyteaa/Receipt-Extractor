require('dotenv').config();
const { sendLicenseEmail } = require('./utils/emailService');

async function testEmail() {
    console.log('🧪 Testing Resend email...\n');
    
    const result = await sendLicenseEmail(
        'a_truchon@outlook.com',       // Test with your Outlook email
        'Antoine Truchon',             // Customer name
        'BUDGET-TEST-1234-5678',       // Test license key
        'BUDGET',                      // Tier
        '#TEST-001'                    // Order number
    );
    
    if (result.success) {
        console.log('\n✅ Test email sent successfully!');
        console.log('Check your inbox at a_truchon@outlook.com');
    } else {
        console.log('\n❌ Failed to send test email');
        console.log('Error:', result.error);
    }
}

testEmail();

