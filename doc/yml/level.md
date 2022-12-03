```yaml
level: suggestion
```

`level` assigns the rule's severity. The available options are `suggestion`, 
`warning`, and `error`. 

## Changing the severity level

A rule's `level` can be changed in your `.vale.ini` file:

```bash
SomeStyle.SomeRule = warning
```

## Filtering output

You can also filter the output of Vale by severity level:

```console
$ vale --filter='.Level in ["warning", "error"]' somefile.md
```
