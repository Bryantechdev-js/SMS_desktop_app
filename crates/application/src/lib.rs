//! Application/use-case boundary.
//!
//! Keep business decisions out of HTTP handlers. These small validators are
//! intentionally pure so they can be reused by commands and covered by tests.
use anyhow::{bail, Result};
use rust_decimal::Decimal;

pub fn validate_positive_quantity(quantity: Decimal) -> Result<()> {
    if quantity <= Decimal::ZERO {
        bail!("Quantity must be greater than zero");
    }
    Ok(())
}

pub fn validate_non_negative_money(amount: Decimal) -> Result<()> {
    if amount < Decimal::ZERO {
        bail!("Money amount cannot be negative");
    }
    Ok(())
}

pub fn validate_required_text(value: &str, field: &str, max_chars: usize) -> Result<()> {
    let normalized = value.trim();
    if normalized.is_empty() {
        bail!("{field} is required");
    }
    if normalized.chars().count() > max_chars {
        bail!("{field} exceeds the maximum length of {max_chars} characters");
    }
    Ok(())
}

pub fn validate_sku(sku: &str) -> Result<()> {
    let normalized = sku.trim();
    if normalized.is_empty() {
        bail!("SKU is required");
    }
    if normalized.len() > 64 {
        bail!("SKU cannot exceed 64 characters");
    }
    if !normalized.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        bail!("SKU must contain only alphanumeric characters, hyphens and underscores");
    }
    Ok(())
}

pub fn validate_product_name(name: &str) -> Result<()> {
    validate_required_text(name, "Product name", 160)
}

pub fn validate_barcode(barcode: &str) -> Result<()> {
    if barcode.trim().is_empty() {
        return Ok(());
    }
    if barcode.len() > 128 {
        bail!("Barcode cannot exceed 128 characters");
    }
    Ok(())
}

pub fn validate_storage_location(location: &str) -> Result<()> {
    if location.trim().is_empty() {
        return Ok(());
    }
    if location.len() > 256 {
        bail!("Storage location cannot exceed 256 characters");
    }
    Ok(())
}

pub fn validate_tax_rate(rate: Decimal) -> Result<()> {
    if rate < Decimal::ZERO || rate > Decimal::from(100) {
        bail!("Tax rate must be between 0 and 100");
    }
    Ok(())
}

pub fn validate_stock_threshold(threshold: Option<Decimal>) -> Result<()> {
    if let Some(t) = threshold {
        if t <= Decimal::ZERO {
            bail!("Stock threshold must be greater than zero");
        }
    }
    Ok(())
}

pub fn validate_stock_adjustment_quantity(quantity: Decimal) -> Result<()> {
    // Adjustment can be positive or negative, but not zero
    if quantity == Decimal::ZERO {
        bail!("Adjustment quantity cannot be zero");
    }
    Ok(())
}

pub fn validate_physical_count_quantity(quantity: Decimal) -> Result<()> {
    if quantity < Decimal::ZERO {
        bail!("Physical count quantity cannot be negative");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn positive_quantity_accepts_positive_decimal() {
        assert!(validate_positive_quantity(dec!(0.125)).is_ok());
    }

    #[test]
    fn positive_quantity_rejects_zero_and_negative() {
        assert!(validate_positive_quantity(Decimal::ZERO).is_err());
        assert!(validate_positive_quantity(dec!(-1)).is_err());
    }

    #[test]
    fn money_accepts_zero_and_positive_amounts() {
        assert!(validate_non_negative_money(Decimal::ZERO).is_ok());
        assert!(validate_non_negative_money(dec!(12500.50)).is_ok());
    }

    #[test]
    fn money_rejects_negative_amounts() {
        assert!(validate_non_negative_money(dec!(-0.01)).is_err());
    }

    #[test]
    fn required_text_trims_and_checks_length() {
        assert!(validate_required_text("  SKU-01  ", "SKU", 64).is_ok());
        assert!(validate_required_text("  ", "SKU", 64).is_err());
        assert!(validate_required_text("12345", "SKU", 4).is_err());
    }

    #[test]
    fn sku_validation() {
        assert!(validate_sku("BAK-001").is_ok());
        assert!(validate_sku("CROISSANT_BUTTER_LARGE").is_ok());
        assert!(validate_sku("  ").is_err());
        assert!(validate_sku("SKU WITH SPACES").is_err());
        assert!(validate_sku("SKU@INVALID").is_err());
    }

    #[test]
    fn product_name_validation() {
        assert!(validate_product_name("Butter Croissant").is_ok());
        assert!(validate_product_name("  ").is_err());
    }

    #[test]
    fn barcode_validation() {
        assert!(validate_barcode("").is_ok()); // optional
        assert!(validate_barcode("  ").is_ok()); // empty when trimmed
        assert!(validate_barcode("123456789012345").is_ok());
    }

    #[test]
    fn tax_rate_validation() {
        assert!(validate_tax_rate(dec!(0)).is_ok());
        assert!(validate_tax_rate(dec!(19.5)).is_ok());
        assert!(validate_tax_rate(dec!(100)).is_ok());
        assert!(validate_tax_rate(dec!(-1)).is_err());
        assert!(validate_tax_rate(dec!(101)).is_err());
    }

    #[test]
    fn stock_adjustment_validation() {
        assert!(validate_stock_adjustment_quantity(dec!(10)).is_ok());
        assert!(validate_stock_adjustment_quantity(dec!(-5)).is_ok());
        assert!(validate_stock_adjustment_quantity(Decimal::ZERO).is_err());
    }

    #[test]
    fn physical_count_validation() {
        assert!(validate_physical_count_quantity(Decimal::ZERO).is_ok());
        assert!(validate_physical_count_quantity(dec!(100)).is_ok());
        assert!(validate_physical_count_quantity(dec!(-1)).is_err());
    }
}


// POS & Checkout Validators (NEW for Phase 2)

pub fn validate_checkout_line_item_quantity(quantity: Decimal) -> Result<()> {
    if quantity <= Decimal::ZERO {
        bail!("Line item quantity must be greater than zero");
    }
    Ok(())
}

pub fn validate_checkout_line_item_unit_price(price: Decimal) -> Result<()> {
    if price < Decimal::ZERO {
        bail!("Unit price cannot be negative");
    }
    Ok(())
}

pub fn validate_line_discount(discount: Decimal, line_total: Decimal) -> Result<()> {
    if discount < Decimal::ZERO {
        bail!("Line discount cannot be negative");
    }
    if discount > line_total {
        bail!("Line discount cannot exceed line total");
    }
    Ok(())
}

pub fn validate_checkout_discount(discount: Decimal, subtotal: Decimal) -> Result<()> {
    if discount < Decimal::ZERO {
        bail!("Discount cannot be negative");
    }
    if discount > subtotal {
        bail!("Discount cannot exceed subtotal");
    }
    Ok(())
}

pub fn validate_checkout_tax(tax: Decimal) -> Result<()> {
    if tax < Decimal::ZERO {
        bail!("Tax amount cannot be negative");
    }
    Ok(())
}

pub fn validate_payment_amount(amount: Decimal) -> Result<()> {
    if amount <= Decimal::ZERO {
        bail!("Payment amount must be greater than zero");
    }
    Ok(())
}

pub fn validate_payment_method(method: &str) -> Result<()> {
    let valid_methods = vec!["cash", "mobile_money", "card", "bank_transfer", "other"];
    if !valid_methods.contains(&method.to_lowercase().as_str()) {
        bail!(
            "Invalid payment method. Must be one of: {}",
            valid_methods.join(", ")
        );
    }
    Ok(())
}

pub fn validate_checkout_total(
    subtotal: Decimal,
    discount: Decimal,
    tax: Decimal,
    total: Decimal,
) -> Result<()> {
    // Verify: total = subtotal - discount + tax
    let expected_total = subtotal - discount + tax;
    if (expected_total - total).abs() > Decimal::new(1, 2) {
        // Allow 0.01 rounding difference
        bail!(
            "Total amount mismatch. Expected {}, got {}",
            expected_total,
            total
        );
    }
    Ok(())
}

pub fn validate_refund_amount(refund: Decimal, sale_total: Decimal) -> Result<()> {
    if refund <= Decimal::ZERO {
        bail!("Refund amount must be greater than zero");
    }
    if refund > sale_total {
        bail!("Refund amount cannot exceed sale total");
    }
    Ok(())
}

pub fn validate_payment_reference(reference: &str) -> Result<()> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return Ok(()); // Optional
    }
    if trimmed.len() > 256 {
        bail!("Payment reference cannot exceed 256 characters");
    }
    Ok(())
}

#[cfg(test)]
mod pos_tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn line_quantity_must_be_positive() {
        assert!(validate_checkout_line_item_quantity(dec!(1)).is_ok());
        assert!(validate_checkout_line_item_quantity(dec!(0.5)).is_ok());
        assert!(validate_checkout_line_item_quantity(Decimal::ZERO).is_err());
        assert!(validate_checkout_line_item_quantity(dec!(-1)).is_err());
    }

    #[test]
    fn line_price_cannot_be_negative() {
        assert!(validate_checkout_line_item_unit_price(dec!(2500)).is_ok());
        assert!(validate_checkout_line_item_unit_price(Decimal::ZERO).is_ok());
        assert!(validate_checkout_line_item_unit_price(dec!(-100)).is_err());
    }

    #[test]
    fn line_discount_validation() {
        assert!(validate_line_discount(dec!(100), dec!(2500)).is_ok());
        assert!(validate_line_discount(Decimal::ZERO, dec!(2500)).is_ok());
        assert!(validate_line_discount(dec!(-10), dec!(2500)).is_err());
        assert!(validate_line_discount(dec!(3000), dec!(2500)).is_err());
    }

    #[test]
    fn checkout_discount_cannot_exceed_subtotal() {
        assert!(validate_checkout_discount(dec!(1000), dec!(5000)).is_ok());
        assert!(validate_checkout_discount(dec!(5000), dec!(5000)).is_ok());
        assert!(validate_checkout_discount(dec!(5001), dec!(5000)).is_err());
        assert!(validate_checkout_discount(dec!(-100), dec!(5000)).is_err());
    }

    #[test]
    fn tax_must_be_non_negative() {
        assert!(validate_checkout_tax(Decimal::ZERO).is_ok());
        assert!(validate_checkout_tax(dec!(250)).is_ok());
        assert!(validate_checkout_tax(dec!(-50)).is_err());
    }

    #[test]
    fn payment_amount_must_be_positive() {
        assert!(validate_payment_amount(dec!(5000)).is_ok());
        assert!(validate_payment_amount(Decimal::ZERO).is_err());
        assert!(validate_payment_amount(dec!(-100)).is_err());
    }

    #[test]
    fn payment_method_must_be_valid() {
        assert!(validate_payment_method("cash").is_ok());
        assert!(validate_payment_method("mobile_money").is_ok());
        assert!(validate_payment_method("card").is_ok());
        assert!(validate_payment_method("bank_transfer").is_ok());
        assert!(validate_payment_method("other").is_ok());
        assert!(validate_payment_method("invalid").is_err());
        assert!(validate_payment_method("CASH").is_ok()); // Case-insensitive
    }

    #[test]
    fn total_must_equal_subtotal_minus_discount_plus_tax() {
        let subtotal = dec!(5000);
        let discount = dec!(500);
        let tax = dec!(250);
        let total = subtotal - discount + tax;

        assert!(validate_checkout_total(subtotal, discount, tax, total).is_ok());
        assert!(validate_checkout_total(subtotal, discount, tax, total - dec!(1)).is_err());
        assert!(validate_checkout_total(subtotal, discount, tax, total + dec!(1)).is_err());
    }

    #[test]
    fn refund_validation() {
        let sale_total = dec!(5000);
        assert!(validate_refund_amount(dec!(2500), sale_total).is_ok());
        assert!(validate_refund_amount(dec!(5000), sale_total).is_ok());
        assert!(validate_refund_amount(dec!(5001), sale_total).is_err());
        assert!(validate_refund_amount(Decimal::ZERO, sale_total).is_err());
        assert!(validate_refund_amount(dec!(-100), sale_total).is_err());
    }

    #[test]
    fn payment_reference_validation() {
        assert!(validate_payment_reference("").is_ok()); // Empty is OK
        assert!(validate_payment_reference("TXN-123456").is_ok());
        assert!(validate_payment_reference("  ").is_ok()); // Whitespace OK
        let long_ref = "x".repeat(257);
        assert!(validate_payment_reference(&long_ref).is_err());
    }
}
