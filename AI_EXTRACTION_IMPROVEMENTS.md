# AI Extraction Improvements

## Issue 1: Multiple Taxes Not Being Summed

### Problem
When receipts show multiple taxes (e.g., GST + PST, or Tax 1 + Tax 2), the AI was only extracting one tax or getting confused about which to use.

**Example:**
- Receipt shows: GST: $5.00, PST: $8.00
- AI might extract: `"5.00"` or `"8.00"` instead of `"13.00"`

### Solution
Added specific instructions to SUM all taxes together:
- **Before**: No specific guidance on multiple taxes
- **After**: "If there are MULTIPLE taxes, you MUST ADD them together"
- Added concrete example showing GST + PST = Total Tax

---

## Issue 2: Subtotal Confusion

### Problem
The AI was struggling to understand what "Subtotal" means, sometimes returning "item total" or confusing it with the total amount.

### Solution
Added clear definition and specific instructions for Subtotal:
- Subtotal = amount BEFORE taxes
- Look for labels: "Subtotal", "Sub-total", "Item Total", "Before Tax"
- Made it clear: Subtotal is usually SMALLER than Total

---

## Issue 3: Currency Symbols in Amount Fields

### Problem
Amount fields were being extracted WITH currency symbols (e.g., `$45.99`, `CAD $113.50`) instead of just numeric values.

### Solution
Updated all amount-related prompts to extract ONLY numeric values:
- **Before**: `"$45.99"`, `"CAD $113.50"`, `"-$260.00"`
- **After**: `"45.99"`, `"113.50"`, `"-260.00"`

---

## Issue 4: Incorrect Total Amount Extraction

### Problem
The AI was sometimes extracting the **subtotal** (before tax) instead of the **total** (after tax) when processing receipts.

For example, on a receipt showing:
- Subtotal: $100.00
- Tax: $13.00
- **Total: $113.00**

The AI might incorrectly extract `100.00` instead of `113.00`.

### Root Cause
The AI prompt wasn't explicit enough about choosing the final total that includes tax when multiple amount fields are present on a receipt.

---

## Solutions Implemented
### 1. Field-Specific Instructions (Lines 547-585)

Now provides **different instructions** based on the field type:

#### For Subtotal Fields:
```
Extract the SUBTOTAL amount (price BEFORE taxes are added).
Look for labels like 'Subtotal', 'Sub-total', 'Item Total', 'Merchandise Total',
'Items', or 'Before Tax'.
This is the sum of all items BEFORE any taxes.
If you cannot find a subtotal, calculate it by subtracting all taxes from the total.
```

#### For Tax Amount Fields:
```
Extract the TOTAL TAX amount.
CRITICAL: If there are MULTIPLE taxes (e.g., GST $5.00 + PST $8.00),
you MUST ADD them together.
For example: if you see 'GST: $5.00' and 'PST: $8.00', return '13.00' (the sum).
Look for labels like 'Tax', 'GST', 'PST', 'HST', 'VAT', 'Sales Tax', 'QST',
'TVQ', 'TPS', 'TVH'.
If there's only one tax, extract that amount. If multiple taxes, SUM them all.
```

#### For Total Amount Fields:
```
Extract ONLY the numeric value WITHOUT any currency symbols.
CRITICAL: If the receipt shows both 'Subtotal' and 'Total', you MUST use the
'Total' (which includes tax), NOT the subtotal.
The total amount should be the LARGEST number on the receipt - the final amount
the customer actually paid.
```

### 2. General Amount Emphasis (Lines 650-671)

Added comprehensive reminders:
```
CRITICAL FOR SUBTOTAL: The Subtotal is the amount BEFORE taxes.
Look for 'Subtotal', 'Sub-total', 'Item Total', or 'Before Tax'.
This is usually SMALLER than the Total.

CRITICAL FOR TAX AMOUNT: If there are MULTIPLE taxes listed (e.g., GST + PST),
you MUST ADD them together.
Example: 'GST: $5.00' + 'PST: $8.00' = extract '13.00' as the Tax Amount.
Common tax labels: GST, PST, HST, QST, VAT, Sales Tax, TPS, TVQ, TVH.
SUM all taxes into one number.

CRITICAL FOR TOTAL AMOUNT: When extracting 'Total Amount', you MUST use the
FINAL TOTAL (after tax), NOT the subtotal.
```

### 3. Concrete Example (Lines 673-688)

Added a real-world example showing multiple taxes:
```
Example receipt:
- Subtotal: $100.00
- GST (5%): $5.00
- PST (8%): $8.00
- Total: $113.00

Correct extraction:
{
  "Subtotal": "100.00",
  "Tax Amount": "13.00",  ← GST + PST summed
  "Total Amount": "113.00"
}
```

---

## Testing
After rebuilding with these changes:
```bash
cargo build --release
./run.sh
```

The AI should now:
1. ✅ Extract amounts as **numeric values only** (no currency symbols)
2. ✅ Correctly identify and extract **Subtotal** (before tax)
3. ✅ **SUM multiple taxes** into one Tax Amount field
4. ✅ Consistently extract the correct **Total Amount** (after tax)

---

## Summary of Changes

### 1. Subtotal Handling
- **Clear definition**: Amount BEFORE taxes
- **Label recognition**: "Subtotal", "Sub-total", "Item Total", "Before Tax"
- **Fallback**: Calculate by subtracting taxes from total if not found

### 2. Multiple Tax Handling
- **Automatic summing**: GST + PST + any other taxes = one Tax Amount
- **Tax label recognition**: GST, PST, HST, QST, VAT, Sales Tax, TPS, TVQ, TVH
- **Concrete example**: Shows $5 + $8 = $13

### 3. Amount Format
- **All amount fields** now return numeric values only
- **Examples**: `45.99`, `113.50`, `-260.00` (for refunds)
- **No longer includes**: `$`, `CAD`, `USD`, `€`, `£`, or any currency symbols

### 4. Total Amount Accuracy
- The prompt now uses multiple reinforcement strategies:
  1. Field-specific instructions for each amount type
  2. Explicit "AFTER TAX" instruction for Total
  3. Explicit "BEFORE TAX" instruction for Subtotal
  4. Explicit "SUM ALL TAXES" instruction for Tax Amount
  5. Concrete example showing all three fields
  6. Heuristic: "LARGEST number on the receipt" for Total
  7. Multiple label variations for each field type

### Supported Receipt Formats
- This should work for various receipt formats:
  - Standard retail receipts (with single or multiple taxes)
  - Restaurant bills
  - Online order confirmations
  - Service invoices
  - Refund documents (with negative amounts)
  - Interac e-Transfer confirmations
  - Canadian receipts (GST + PST, HST, QST)
  - International receipts (VAT, Sales Tax)

---

## Related Files
- `src/ai_extractor.rs` - Lines 547-585 (field-specific), 650-671 (emphasis), 673-688 (example)

---

## Example Outputs

### Example 1: Simple Receipt (Before vs After)

**Before Changes:**
```json
{
  "Date": "15/01/2024",
  "Vendor/Store Name": "Amazon.com",
  "Total Amount": "$45.99",
  "Tax Amount": "$5.99"
}
```

**After Changes:**
```json
{
  "Date": "15/01/2024",
  "Vendor/Store Name": "Amazon.com",
  "Total Amount": "45.99",
  "Tax Amount": "5.99"
}
```

### Example 2: Multiple Taxes (New Capability)

**Receipt shows:**
```
Subtotal:     $100.00
GST (5%):       $5.00
PST (8%):       $8.00
Total:        $113.00
```

**Before Changes:**
```json
{
  "Subtotal": "item total",  ← Confused
  "Tax Amount": "5.00",      ← Only extracted GST
  "Total Amount": "100.00"   ← Wrong! Used subtotal
}
```
**After Changes:**
```json
{
  "Subtotal": "100.00",      ← Correct! Before tax
  "Tax Amount": "13.00",     ← Correct! GST + PST summed
  "Total Amount": "113.00"   ← Correct! Final total
}
```

### Example 3: Restaurant Bill with HST

**Receipt shows:**
```
Food & Drinks:  $85.50
HST (13%):      $11.12
Total:          $96.62
```

**Extraction:**
```json
{
  "Subtotal": "85.50",
  "Tax Amount": "11.12",
  "Total Amount": "96.62"
}
```

---

## Key Improvements

1. **Subtotal Recognition**: Now understands "Item Total", "Merchandise Total", "Before Tax"
2. **Multi-Tax Summing**: Automatically adds GST + PST + any other taxes
3. **Clear Distinctions**: Knows Subtotal < Total, and Tax = Total - Subtotal
4. **Numeric Only**: All amounts are clean numbers without symbols
5. **Concrete Examples**: AI has clear examples to follow

---

## Common Tax Combinations Supported

- **Canada**: GST + PST, HST, QST + GST
- **US**: Sales Tax, State Tax + Local Tax
- **Europe**: VAT
- **Quebec**: TPS + TVQ
- **Any combination**: The AI will sum all taxes it finds

