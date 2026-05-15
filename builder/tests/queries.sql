-- Buscar por lectura (punto de entrada habitual)
SELECT e.id, e.jlpt
FROM entries e
JOIN readings r ON r.entry_id = e.id
WHERE r.text = '食べる';

-- ── Entry ──────────────────────────────────────────────────────────────────
SELECT id, jlpt
FROM entries
WHERE id = 1000530;

-- ── Kanji readings ─────────────────────────────────────────────────────────
SELECT kr.kanji, kr.priority, kr.info
FROM kanji_readings kr
WHERE kr.entry_id = 1000530;

-- ── Readings (con restricciones si las hay) ────────────────────────────────
SELECT r.text, r.no_kanji, r.priority, r.info,
       GROUP_CONCAT(kr.kanji, ' | ') AS restricted_to
FROM readings r
LEFT JOIN reading_restrictions rr ON rr.reading_id = r.id
LEFT JOIN kanji_readings kr       ON kr.id = rr.kanji_reading_id
WHERE r.entry_id = 1000530
GROUP BY r.id;

-- ── Senses y glosses ───────────────────────────────────────────────────────
SELECT s.id   AS sense_id,
       s.pos,
       s.misc,
       s.dialects,
       s.info,
       g.lang,
       g.text AS gloss,
       g.type AS gloss_type
FROM senses s
JOIN glosses g ON g.sense_id = s.id
WHERE s.entry_id = 1000530
ORDER BY s.id, g.lang;

-- ── Kanji relacionados (con sus datos completos) ───────────────────────────
SELECT k.literal,
       k.grade,
       k.stroke_count,
       k.freq,
       k.jlpt,
       k.on_readings,
       k.kun_readings,
       GROUP_CONCAT(km.text, ' / ') FILTER (WHERE km.lang = 'en') AS meanings_en
FROM entry_kanji ek
JOIN kanji         k  ON k.literal  = ek.literal
LEFT JOIN kanji_meanings km ON km.literal = k.literal
WHERE ek.entry_id = 1000530
GROUP BY k.literal;

-- ── Todo junto en una sola query ───────────────────────────────────────────
SELECT
    e.id                                                      AS entry_id,
    e.jlpt                                                    AS entry_jlpt,
    GROUP_CONCAT(DISTINCT kr.kanji)                           AS kanji_forms,
    GROUP_CONCAT(DISTINCT r.text)                             AS readings,
    g.lang                                                    AS gloss_lang,
    GROUP_CONCAT(g.text, ' / ')                               AS glosses,
    s.pos,
    GROUP_CONCAT(DISTINCT k.literal)                          AS kanji_chars
FROM entries e
LEFT JOIN kanji_readings kr ON kr.entry_id = e.id
LEFT JOIN readings r        ON r.entry_id  = e.id
LEFT JOIN senses s          ON s.entry_id  = e.id
LEFT JOIN glosses g         ON g.sense_id  = s.id
LEFT JOIN entry_kanji ek    ON ek.entry_id = e.id
LEFT JOIN kanji k           ON k.literal   = ek.literal
WHERE e.id = 1000530
GROUP BY s.id, g.lang
ORDER BY s.id, g.lang;