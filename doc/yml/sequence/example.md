```yaml
extends: sequence
# `%[4]s` is like `%s`, but specifically refers to the 4th token in our
# sequence.
message: "The infinitive '%[4]s' after 'be' requires 'to'. Did you mean '%[2]s %[3]s *to* %[4]s'?"
tokens:
  - tag: MD
  - pattern: be
  - tag: JJ
  # The `|` notation means that we'll accept `VB` or `VBN` in position 4.
  - tag: VB|VBN
```