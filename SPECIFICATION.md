# `if-changed` Specification

This document defines the syntax of `if-changed` in ABNF in accordance with [RFC5234](https://datatracker.ietf.org/doc/html/rfc5234):

```abnf
if-changed = "if-changed" ["(" <name> ")"]
name       = 1*name-char

then-change        = "then-change" "(" [LF] named-pattern-list [LF] ")"
named-pattern-list = named-pattern *(delimiter named-pattern)
named-pattern      = pattern [":" name]

pattern          = rooted-pattern / relative-pattern
rooted-pattern   = "/" relative-pattern
relative-pattern = 1*pattern-char *(continuation *pattern-char)

continuation = backslash LF
delimiter    = "," / LF

name-char     = %x00-%x28 / %x2A-%x10FFFF ; Any character except ")"
pattern-char  = %x00-%x09                 ; Skipping line feed
              / %x0B-%x2B                 ; Skipping ","
              / %x2D-%x5B                 ; Skipping "\"
              / %x5D-%x10FFFF
backslash     = %x5C                      ; "\"
```
