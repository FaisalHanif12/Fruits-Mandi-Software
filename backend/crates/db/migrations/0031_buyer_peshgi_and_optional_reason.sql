-- Migration 31 (plan 99, PRD FR-15.11 v1.60, Architecture 15.7): cash lent to a buyer.
--
-- 1. `vouchers.entered_as` remembers how a voucher was typed when that differs from what it posts
--    as. The only value is 'advance': a پیشگی to a buyer, saved as cash lent (a payment to him). The
--    screens, روکڑ بک and his ledger then name it پیشگی. Its type, number series, posting and
--    matching are those of the payment: no journal line, amount, id or chain hash changes. NULL for
--    every old voucher (which tab an old one was typed on was never stored).
--
-- 2. The reason of cash lent to a buyer is optional, so the two triggers of migration 1 that
--    required one are dropped. The reason of ذاتی خرچہ and ادھار رقم (the table CHECK) is not
--    changed.
ALTER TABLE vouchers ADD COLUMN entered_as TEXT
    CHECK (entered_as IS NULL OR (entered_as = 'advance' AND type = 'payment'));

DROP TRIGGER vouchers_payment_to_buyer_needs_reason;
DROP TRIGGER vouchers_payment_to_buyer_needs_reason_update;
