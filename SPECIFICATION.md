# `if-changed` Specification

This document defines the syntax of `if-changed` in ABNF in accordance with [RFC5234](https://datatracker.ietf.org/doc/html/rfc5234):

```abnf
if-changed = "if-changed" ["(" <name> ")"]
name       = 1*name-char

then-change         = "then-change" "(" [LF] named-pathspec-list [LF] ")"
named-pathspec-list = named-pathspec *(delimiter named-pathspec)
named-pathspec      = pathspec [":" name]

pathspec          = rooted-pathspec / relative-pathspec
rooted-pathspec   = "/" relative-pathspec
relative-pathspec = 1*pathspec-char *(continuation *pathspec-char)

continuation = backslash LF
delimiter    = "," / LF

name-char     = %x00-%x28 / %x2A-%x10FFFF ; Any character except ")"
pathspec-char = %x00-%x09                 ; Skipping line feed
              / %x0B-%x2B                 ; Skipping ","
              / %x2D-%x5B                 ; Skipping "\"
              / %x5D-%x10FFFF
backslash     = %x5C                                              ; "\"
```
