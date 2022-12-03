# `vale-ls`

The Vale Language Server (`vale-ls`) is an implementation of the 
[Language Server Protocol][1] (LSP) for [Vale][2].

It provides high-level interface for managing Vale and its assets
(binary, `StylesPath`, etc.) with the goal of making it easy to add IDE-like 
features to any text editor that supports the Language Server Protocol.

## Installation

## Intellisense

<table>
    <tr>
        <th>Hover</th>
        <th>Autocomplete</th>
    </tr>
    <tr>
        <td width="50%">
            <a href="https://user-images.githubusercontent.com/8785025/229923172-36fde7c3-da7b-45ec-a6f4-0465be99ca14.png">
                <img src="https://user-images.githubusercontent.com/8785025/229923172-36fde7c3-da7b-45ec-a6f4-0465be99ca14.png" width="100%">
            </a>
        </td>
        <td width="50%">
            <a href="https://user-images.githubusercontent.com/8785025/229919933-f500ef73-1a13-4dbd-8d01-95be12f46e6f.png">
                <img src="https://user-images.githubusercontent.com/8785025/229919933-f500ef73-1a13-4dbd-8d01-95be12f46e6f.png" width="100%">
            </a>
        </td>
    </tr>
    <tr>
        <td width="50%">
          See in-editor documentation for any symbol.
        </td>
        <td width="50%">Autocomplete all <code>StylesPath</code> assets: Styles, Packages, Vocabularies, etc.</td>
    </tr>
</table>

## Asset Management

### Binary

- [x] Install Vale
- [x] Update Vale

### Vocabularies

- [ ] Create
- [ ] Update

### Rules

- [ ] Compile

[1]: https://microsoft.github.io/language-server-protocol/
[2]: https://github.com/errata-ai/vale
