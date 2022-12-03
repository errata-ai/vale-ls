```bash
# Default: None
BlockIgnores = (?s) *({< file [^>]* >}.*?{</ ?file >})
```

`BlockIgnores` allow you to exclude block-level sections of text that don't 
have an associated HTML tag that could be used with `SkippedScopes`.