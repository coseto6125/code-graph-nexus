;; Capture top-level SFC block elements so the parser can locate
;; their start positions and extract embedded script content.

;; <script> / <script context="module"> block — raw_text holds the JS/TS source.
(script_element
  (start_tag) @script.tag
  (raw_text)? @script.body
) @script

;; <style> block — span-only treatment; contents not parsed.
(style_element) @style
