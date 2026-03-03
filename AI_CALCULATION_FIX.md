# AI Calculation Fix - Stop AI from Making Up Numbers

## The Problem

The AI was **calculating** numbers instead of **reading** them from the receipt!

**Receipt shows:**
```
Item Total    1,170.81  ← Before tax
G.S.T           58.54   ← Tax 1
P.S.T           81.96   ← Tax 2
Sub Total    1,311.31   ← Final total (printed on receipt)
```

**AI was extracting:**
- amount befor taxe: 1,311.31 ❌ (used "Sub Total" instead of "Item Total")
- Tax Amount: 140.50 ✅ (correct - GST + PST)
- Total Amount: 1,451.81 ❌ **CALCULATED** 1311.31 + 140.50 = 1451.81 (NOT on receipt!)

**Should extract:**
- amount befor taxe: 1,170.81 ✅ (Item Total - what's printed)
- Tax Amount: 140.50 ✅ (58.54 + 81.96 - sum of taxes)
- Total Amount: 1,311.31 ✅ (Sub Total - what's printed)

---

## The Fix - Three Layers

### 1. Aggressive "NO MATH!" Warning at Top of Prompt

```
🚨🚨🚨 CRITICAL RULES - READ FIRST 🚨🚨🚨

❌ NEVER CALCULATE! ❌ NEVER DO MATH! ❌ NEVER ADD OR SUBTRACT! ❌

✅ ONLY extract numbers that are LITERALLY PRINTED on the receipt!
✅ For Tax Amount ONLY: Find ALL tax lines (GST, PST, etc.) and ADD them together.
✅ For Subtotal and Total: READ the EXACT numbers from the receipt. DO NOT calculate!

If you calculate Subtotal + Tax to get Total, you are WRONG!
If you calculate Total - Tax to get Subtotal, you are WRONG!
Just READ what's printed!
```

### 2. Field-Specific "DO NOT CALCULATE!" Instructions

**Subtotal Field:**
```
🚨 DO NOT CALCULATE! READ the amount BEFORE taxes from the receipt.

FORMAT 2 (Taxes appear BEFORE subtotal line):
  Item Total: $1,170.81  ← Amount BEFORE tax (READ THIS!)
  GST: $58.54
  PST: $81.96
  Sub Total: $1,311.31  ← This already includes taxes! (DON'T use this)
→ Extract '1170.81' (the Item Total, NOT the Sub Total line)

✅ CORRECT: Read the number that appears BEFORE GST/PST lines
❌ WRONG: Use 'Sub Total' when it appears AFTER tax lines
❌ WRONG: Calculate Total - Tax (NO MATH! Just READ!)
```

**Total Amount Field:**
```
🚨 DO NOT CALCULATE! READ the exact number from the receipt.

⚠️ CRITICAL - DO NOT ADD SUBTOTAL + TAX! Just READ what's printed!

✅ CORRECT: Extract '1311.31' (the Sub Total line - what's PRINTED)
❌ WRONG: Calculate 1170.81 + 140.50 = 1311.31 (NO MATH!)
❌ WRONG: Calculate 1311.31 + 140.50 = 1451.81 (NEVER ADD! Just READ!)
```

### 3. Post-Processing Validation (New!)

Added `validate_extraction()` function that checks if the AI calculated:

```rust
fn validate_extraction(&self, result: &HashMap<String, String>) {
    // Check if total = subtotal + tax (indicates calculation)
    if let (Some(sub), Some(tax_amt), Some(tot)) = (subtotal, tax, total) {
        let calculated_total = sub + tax_amt;
        let diff = (calculated_total - tot).abs();
        
        // If total exactly equals subtotal + tax, AI likely calculated it
        if diff < 0.02 {  // Allow 2 cent rounding
            ⚠️ WARNING: Total = Subtotal + Tax - AI may have CALCULATED!
        }
    }
}
```

This will log a warning if the AI is calculating instead of reading.

---

## What Changed

### Files Modified:
1. **src/ai_extractor.rs** (Lines 732-744, 552-581, 603-629, 752-766, 872-895)
   - Added aggressive "NO MATH!" warning at top
   - Updated Subtotal field instructions
   - Updated Total Amount field instructions
   - Updated examples with wrong calculation examples
   - Added validation function

### Key Improvements:
- ✅ Triple warning symbols (🚨🚨🚨) at the top
- ✅ "NEVER CALCULATE!" in huge letters
- ✅ Explicit examples of wrong calculations (1451.81)
- ✅ Post-processing validation to detect calculation
- ✅ Warnings logged to console and file

---

## Testing

```bash
cargo build --release
./run.sh
```

Process the failing receipt: `/home/tony/pCloudDrive/MyHome/work done/IMG20251225142131.jpg`

**Expected output:**
- amount befor taxe: `1170.81` (Item Total)
- Tax Amount: `140.50` (GST 58.54 + PST 81.96)
- Total Amount: `1311.31` (Sub Total)

**If AI still calculates:**
- Check `receipt_extractor_debug.log` for validation warnings
- The warning will show: "Total (1451.81) = Subtotal (1311.31) + Tax (140.50)"

---

## Window Controls Issue

Also added `.with_close_button(true)` to `src/main.rs` line 18.

If window controls still don't appear, this is likely a window manager issue (GNOME, KDE, etc.), not the app.

