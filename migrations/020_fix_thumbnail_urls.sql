-- Migration 020: Fix dead thumbnail URLs
-- Updates all thumbnail_url values that reference removed routes
-- (/hls/, /videos/, /images/, /documents/) to the unified /media/{slug}/thumbnail endpoint

UPDATE media_items
SET thumbnail_url = '/media/' || slug || '/thumbnail'
WHERE thumbnail_url LIKE '/hls/%'
   OR thumbnail_url LIKE '/videos/%'
   OR thumbnail_url LIKE '/images/%'
   OR thumbnail_url LIKE '/documents/%';
