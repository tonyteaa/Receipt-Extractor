const { Resend } = require('resend');

// Initialize Resend
const resend = new Resend(process.env.RESEND_API_KEY);

// Send license key email
async function sendLicenseEmail(customerEmail, customerName, licenseKey, tier, orderNumber) {
    const maxActivations = 3; // All licenses are Pro with 3 activations

    const emailHtml = `
<!DOCTYPE html>
<html>
<head>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
            border-radius: 10px 10px 0 0;
        }
        .content {
            background: #f9f9f9;
            padding: 30px;
            border-radius: 0 0 10px 10px;
        }
        .license-box {
            background: white;
            border: 3px solid #4CAF50;
            border-radius: 10px;
            padding: 25px;
            margin: 25px 0;
            text-align: center;
        }
        .license-key {
            font-size: 32px;
            font-weight: bold;
            color: #333;
            font-family: 'Courier New', monospace;
            letter-spacing: 3px;
            margin: 15px 0;
            word-break: break-all;
        }
        .info-box {
            background: #e3f2fd;
            border-left: 4px solid #2196F3;
            padding: 15px;
            margin: 20px 0;
        }
        .footer {
            text-align: center;
            color: #666;
            font-size: 12px;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
        }
        .button {
            display: inline-block;
            background: #4CAF50;
            color: white;
            padding: 12px 30px;
            text-decoration: none;
            border-radius: 5px;
            margin: 10px 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎉 Thank You for Your Purchase!</h1>
        <p>Your Receipt Extractor License is Ready</p>
    </div>
    
    <div class="content">
        <p>Hi ${customerName},</p>
        
        <p>Thank you for purchasing Receipt Extractor! Here's your license key:</p>
        
        <div class="license-box">
            <p style="margin: 0; color: #4CAF50; font-weight: bold;">🔑 YOUR LICENSE KEY</p>
            <div class="license-key">${licenseKey}</div>
            <p style="margin: 10px 0 0 0; color: #666; font-size: 14px;">
                ${tier} Tier - ${maxActivations} Device Activations
            </p>
        </div>
        
        <div class="info-box">
            <h3 style="margin-top: 0;">📋 Important Information:</h3>
            <ul style="margin: 10px 0;">
                <li><strong>Order Number:</strong> ${orderNumber}</li>
                <li><strong>License Tier:</strong> ${tier}</li>
                <li><strong>Max Activations:</strong> ${maxActivations} devices</li>
            </ul>
        </div>
        
        <h3>🚀 How to Activate:</h3>
        <ol>
            <li>Download and install Receipt Extractor</li>
            <li>Launch the application</li>
            <li>Enter your license key when prompted</li>
            <li>Start extracting receipts!</li>
        </ol>
        
        <div style="text-align: center; margin: 30px 0;">
            <p style="color: #666; font-size: 14px;">Need help? Contact us anytime!</p>
        </div>
        
        <p><strong>Important:</strong> Please save this email for your records. You'll need this license key to activate your software.</p>
    </div>
    
    <div class="footer">
        <p>This email was sent to ${customerEmail}</p>
        <p>© 2025 CB Tools. All rights reserved.</p>
    </div>
</body>
</html>
        `;

    try {
        const { data, error } = await resend.emails.send({
            from: `${process.env.FROM_NAME} <${process.env.FROM_EMAIL}>`,
            to: [customerEmail],
            subject: `🔑 Your Receipt Extractor License Key - Order ${orderNumber}`,
            html: emailHtml
        });

        if (error) {
            console.error(`❌ Failed to send email to ${customerEmail}:`, error);
            return { success: false, error: error.message };
        }

        console.log(`✅ License email sent to ${customerEmail}`);
        console.log(`   Email ID: ${data.id}`);
        return { success: true, emailId: data.id };
    } catch (error) {
        console.error(`❌ Failed to send email to ${customerEmail}:`, error.message);
        return { success: false, error: error.message };
    }
}

module.exports = {
    sendLicenseEmail
};

