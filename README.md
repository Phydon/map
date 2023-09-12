# map

**MA**nipulate **P**ipe

*change pipe input via regex or as literal string*

a better version of this program is ![sd](https://github.com/chmln/sd)

## Examples

todo: add easy examples


search all rust files in the current directory with ![sf](https://github.com/Phydon/sf)

![screenshot](https://github.com/Phydon/map/blob/master/assets/sf_all_rust_files.png)

extract the file names from the search results

![screenshot](https://github.com/Phydon/map/blob/master/assets/sf_map_extract_file_names_from_search_results.png)

## Usage

### Short Usage

```
Usage: map [OPTIONS] [OLD_PATTERN] [NEW_PATTERN] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [OLD_PATTERN] [NEW_PATTERN]  Replace a pattern with a new one

Options:
  -n, --num <NUMBER>  Replaces first N matches of a pattern with another string
  -s, --string        Treat the pattern as a literal string
  -h, --help          Print help (see more with '--help')
  -V, --version       Print version
```

### Long Usage

```
Usage: map [OPTIONS] [OLD_PATTERN] [NEW_PATTERN] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [OLD_PATTERN] [NEW_PATTERN]
          Replace a pattern with a new one

Options:
  -n, --num <NUMBER>
          Replaces first N matches of a pattern with another string

  -s, --string
          Treat the pattern as a literal string

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/map/releases)
