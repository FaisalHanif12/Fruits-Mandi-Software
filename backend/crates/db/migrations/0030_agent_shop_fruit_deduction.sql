-- Migration 30 (plan 97 item 1, PRD FR-16.7 and FR-16.11 v1.59, Architecture 15.4 and 15.6): fruit
-- taken from another commission agent's shop.
--
-- 1. `accounts.is_agent_shop` marks a seller account as another commission agent's shop (آڑھتی
--    دکان). The آڑھتی box of the پھل screen lists only these accounts, and a new shop is added on the
--    spot as a seller account with this mark (PRD FR-7.69). A marked account is not offered as the
--    seller of a lot in بکری; it stays a seller in every other list (seller payables, the ادائیگی
--    picker). 0 for every old account: no list, balance, amount, id or chain hash changes.
--
-- 2. `fruit_takes.deduction_paisa` is the کل کٹوتی typed on a fruit entry from another shop: the other
--    shop's commission, freight and other charges in one amount. `value_paisa` stays the amount that
--    is POSTED: the agent is credited it and the partner (or the guests' partners, C27) are charged
--    it. With a deduction it is the net, `qty x rate - deduction_paisa`; so the registers and the
--    partner statement, which add up `value_paisa`, still add up to the journal. The gross value is
--    `value_paisa + deduction_paisa`. Whole rupees and never negative, as every money column. Every
--    old entry gets 0: a deduction of 0 posts exactly as before.
ALTER TABLE accounts ADD COLUMN is_agent_shop INTEGER NOT NULL DEFAULT 0
    CHECK (is_agent_shop IN (0, 1));

ALTER TABLE fruit_takes ADD COLUMN deduction_paisa INTEGER NOT NULL DEFAULT 0
    CHECK (deduction_paisa >= 0) CHECK (deduction_paisa % 100 = 0)
    CHECK (deduction_paisa = 0 OR source = 'other_shop');
