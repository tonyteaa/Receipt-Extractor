# Multiple Tax Summing - Enhanced Instructions

## Real-World Problem Found

**Receipt:** IMG20251225142131.jpg (Nelson Building Centre)

**What the receipt shows:**
```
Item Total    1,170.81
G.S.T           58.54  ← First tax
P.S.T           81.96  ← Second tax
Sub Total    1,311.31  (= 1,170.81 + 58.54 + 81.96)
```

**What the AI extracted (WRONG):**
- Subtotal: $1,311.31 ❌ (should be $1,170.81)
- Tax Amount: $82.50 ❌ (should be $140.50 = GST $58.54 + PST $81.96)
- Total: $1,393.81 ❌ (should be $1,311.31)

**The AI made TWO mistakes:**
1. **Did NOT sum both taxes** (only extracted one tax, not GST + PST)
2. **Misidentified the subtotal** (used the final "Sub Total" line instead of "Item Total")

## Solution - Maximum Emphasis + Real Example

We've added **FIVE layers of reinforcement** with the EXACT real-world example:

### Layer 1: Pre-Prompt Warning (Line 723)
**BEFORE** the AI even sees the field list:
```
⚠️ BEFORE YOU START: If extracting 'Tax Amount', scan the ENTIRE receipt
for ALL tax lines (GST, PST, TPS, TVQ, etc.) and ADD them together! ⚠️
```

### Layer 2: Tax Field Instructions (Lines 577-588)
Includes the EXACT real-world example that failed:
```
⚠️⚠️⚠️ CRITICAL - SCAN ENTIRE RECEIPT FOR ALL TAX LINES ⚠️⚠️⚠️

STEP 1: Look through the ENTIRE receipt and find EVERY line with these labels:
        GST, G.S.T, PST, P.S.T, HST, QST, VAT, TPS, TVQ, TVH, Tax 1, Tax 2
STEP 2: Write down each tax amount you find:
        Example: Found 'GST 58.54' and 'PST 81.96'
STEP 3: ADD all tax amounts together:
        Example: 58.54 + 81.96 = 140.50
STEP 4: Return ONLY the sum as a number:
        Example: Return '140.50' (NOT '58.54' or '81.96')

REAL RECEIPT EXAMPLE (COMMON FORMAT):
  Item Total    1,170.81
  G.S.T           58.54  ← FIRST TAX
  P.S.T           81.96  ← SECOND TAX
  Sub Total    1,311.31
✅ CORRECT: Return '140.50' (58.54 + 81.96)
❌ WRONG: Return '58.54' (missing PST)
❌ WRONG: Return '81.96' (missing GST)
```

### Layer 3: Subtotal Field Instructions (Lines 552-576)
Handles both receipt formats:
```
⚠️ IMPORTANT: Receipts have different formats:

FORMAT 1 (Most common):
  Subtotal: $100.00  ← Amount BEFORE tax
  GST: $5.00
  PST: $8.00
  Total: $113.00
→ Extract '100.00' as Subtotal

FORMAT 2 (Some receipts):
  Item Total: $1,170.81  ← Amount BEFORE tax
  GST: $58.54
  PST: $81.96
  Sub Total: $1,311.31  ← This already includes taxes!
→ Extract '1170.81' as Subtotal (the Item Total, NOT the Sub Total line)
```

### Layer 4: Total Amount Field Instructions (Lines 603-624)
Explains when "Sub Total" is actually the final total:
```
IMPORTANT: Some receipts show 'Sub Total' as the final line (which already includes taxes).
Example:
  Item Total: $1,170.81
  GST: $58.54
  PST: $81.96
  Sub Total: $1,311.31  ← This is the FINAL total (Item Total + GST + PST)
→ Extract '1311.31' as Total Amount
```

### Layer 5: Concrete Examples with Real Receipt (Lines 729-756)
Shows the EXACT failing case:
```
EXAMPLE 2 - Taxes Before Subtotal (VERY COMMON):
Receipt shows:
- Item Total    1,170.81
- G.S.T           58.54
- P.S.T           81.96
- Sub Total    1,311.31

✅ CORRECT: {"amount befor taxe": "1170.81", "Tax Amount": "140.50", "Total Amount": "1311.31"}
❌ WRONG: {"Tax Amount": "58.54"} - Missing PST!
❌ WRONG: {"Tax Amount": "81.96"} - Missing GST!
Note: Tax Amount = 58.54 + 81.96 = 140.50 (SUM both taxes!)
```

## What Changed - Summary

### 1. Tax Amount Field (Lines 577-588)
- ✅ Added REAL failing example (GST $58.54 + PST $81.96)
- ✅ Shows correct answer: $140.50 (sum of both)
- ✅ Shows wrong answers: $58.54 or $81.96 (only one tax)
- ✅ 4-step process to find and sum ALL taxes
- ✅ Lists all tax label variations (G.S.T, P.S.T, etc.)

### 2. Subtotal Field (Lines 552-576)
- ✅ Explains FORMAT 1: Subtotal before taxes
- ✅ Explains FORMAT 2: "Item Total" before taxes, "Sub Total" after taxes
- ✅ Shows when to use "Item Total" vs "Sub Total"

### 3. Total Amount Field (Lines 603-624)
- ✅ Explains when "Sub Total" is the final amount
- ✅ Shows example where Sub Total = Item Total + GST + PST

### 4. Examples Section (Lines 729-756)
- ✅ Added EXAMPLE 2 with the exact failing receipt format
- ✅ Shows Item Total, G.S.T, P.S.T, Sub Total structure
- ✅ Shows correct extraction: Tax = 140.50 (not 58.54 or 81.96)

## Testing

Rebuild and test:
```bash
cargo build --release
./run.sh
```

Process the failing receipt again: `/home/tony/pCloudDrive/MyHome/work done/IMG20251225142131.jpg`

**Expected correct output:**
- amount befor taxe: `1170.81` (Item Total)
- Tax Amount: `140.50` (GST 58.54 + PST 81.96)
- Total Amount: `1311.31` (Sub Total line)

**Previous wrong output:**
- amount befor taxe: `1311.31` ❌
- Tax Amount: `82.50` ❌
- Total Amount: `1393.81` ❌

## Key Improvements

1. **Real-world example** - Uses the EXACT receipt that failed
2. **Format awareness** - Handles "taxes before subtotal" format
3. **Explicit labels** - Searches for G.S.T and P.S.T (with periods)
4. **Step-by-step** - Forces AI to find ALL taxes, write them down, then sum
5. **Wrong answer examples** - Shows what NOT to return

## Files Modified
- `src/ai_extractor.rs` - Lines 552-576, 577-588, 603-624, 729-756
- `MULTIPLE_TAX_FIX.md` - This documentation

