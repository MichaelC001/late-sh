-- The tailor's rack is gated by level now (pieces and tints unlock every
-- three levels, `runner/state.rs`), and the looks rolled before the gate
-- drew from the whole table. Every look is re-rolled into the level-1
-- rack: one of the three street pieces per slot, static or amber, the
-- mark kept. The tailor's code expects a worn piece to be on the runner's
-- rack, so no look may sit above it. `random()` is volatile, so each row
-- and each slot rolls its own dice. The change trigger fires once per
-- row and the directory re-reads, which is the point.
UPDATE deadchannel_runners
SET look = jsonb_build_object(
        'hood', jsonb_build_object(
            'piece', (ARRAY['hood.plain', 'hood.flat', 'hood.cap'])[1 + floor(random() * 3)::int],
            'tint', (ARRAY['static', 'amber'])[1 + floor(random() * 2)::int]
        ),
        'eyes', jsonb_build_object(
            'piece', (ARRAY['eyes.dot', 'eyes.round', 'eyes.square'])[1 + floor(random() * 3)::int],
            'tint', (ARRAY['static', 'amber'])[1 + floor(random() * 2)::int]
        ),
        'coat', jsonb_build_object(
            'piece', (ARRAY['coat.plain', 'coat.thin', 'coat.narrow'])[1 + floor(random() * 3)::int],
            'tint', (ARRAY['static', 'amber'])[1 + floor(random() * 2)::int]
        ),
        'mark', look -> 'mark'
    ),
    updated = current_timestamp;
