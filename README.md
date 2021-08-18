semfilter
================
Simple command line tool to filter unstructured text data. Allows tokens to be cast to data types, filter data using an expression language, and sort the result.

## License
**semfilter** is licensed under the MIT license.

### Usage
Simple command line tool to filter unstructured text data. Allows tokens to be cast to data types, filter data using an expression language, and sort the result.

    USAGE:
        semfilter [FLAGS] [OPTIONS] expr [<inputstream]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
        -v               Sets the level of verbosity

    OPTIONS:
        -t, --token-sep <regex>   Specifies how tokens are separated in a row, e.g ",|\s|\t",  default is WHITESPACE        
        -s, --quote-string        Surround a string with single quotes, default a string is a single token

    ARGS:
        <expr>    The expression to use to filter the input rows, in douple quotes, e.g. "date(0) == 1900-01-01"
<br>

**Currently supported datatypes:**

| Data type | Operator                             | Default format          | Example                 |
|-----------| -------------------------------------|-------------------------|-------------------------|
| date      | ==, !=, <=, >=, <, >, in, !in        | yyyy-MM-dd              | 1970-01-01              |
| time      | ==, !=, <=, >=, <, >, in, !in        | HH:mm                   | 14:30                   |
| number    | ==, !=, <=, >=, <, >, in, !in        | decimal number          | 3.1415                  |
| integer   | ==, !=, <=, >=, <, >, in, !in        | non floating number     | 42                      |
| string    | ==, !=, <=, >=, <, >, in, !in, match | any character           | hello world             |
| email     | ==, !=, <=, >=, <, >, in, !in        | xxx@yyy.com             | test@gmail.com          |
| ivp4      | ==, !=, <=, >=, <, >, in, !in        | 127.0.0.1               | 127.0.0.1               |
| ivp6      | ==, !=, <=, >=, <, >, in, !in        | 1762:0:0:0:0:B03:1:AF18 | 1762:0:0:0:0:B03:1:AF18 |
| semver    | ==, !=, <=, >=, <, >, in, !in        | 1.0.0                   | 1.0.0                   |


<br>

**Supported operators (in precedense order, left to right evaluation)**

| Operator              | Remark             | Example                                                                 |
|-----------------------|--------------------|-------------------------------------------------------------------------|
| ()                    | Grouping           | (date(0) == 1900-01-01 \|\| date(1) > 2000-01-01) && string(*) != error |
| ==, !=, <=, >=, <, >  | logical            | date(0) > 200-01-01                                                     |
| match                 | regular expression | string(*) match \d{4}-\d{2}-\d{2}                                       |
| in                    | member             | ipv4(0) in [127.0.0.1, 10.0.0.1, 196.0.0.1]                             |
| !in                   | not member         | integer(*) !in [200, 401]                                               |
| &&                    | and                | "                                                                       |
| \|\|                  | or                 | "                                                                       |

<br>


### Expression

An simple expression is of the form:

`dataType(index [, format specifier]) OPERATOR VALUE`

where the index is the **0-indexed token** in the row currently being evaluated and the optional **format specifier** the format to look for. Currently only date and time formats are supported, a format specified for other datatypes is ignored. The index can also be a whildcard `*` which then means match any occurance of the token of that datatype in the row. 


<br>

### Examples
```
$ cat file.txt
the love for this file is amazing 1900-01-01
string ivp4 127.0.0.1 number 123.0 date 1970/07/31 email mikael@gmail.com
ipv6 1762:0:0:0:0:B03:1:AF18 with the ultimate answer 42
this line should be echoed out 1900-01-01 the love
```

<br>

This example uses a date formatter and logical operators to form an expression
```
$ semfilter "date(0, yyyy/mm/dd) == 1970/07/31 || date(0) >= 1900-01-01" < file.txt
the love for this file is amazing 1900-01-01
string ivp4 127.0.0.1 number 123.0 date 1970-07-31 email mikael@gmail.com
this line should be echoed out 1900-01-01
```
<br>

This example ues an integer wildcard
```
$ semfilter "integer(*) == 42" < file.txt
ipv6 1762:0:0:0:0:B03:1:AF18 with the ultimate answer 42
```
<br>

This example uses string wildcard and quote a string with single quotes
```
$ semfilter -s "string(*) == 'the love'" < file.txt
the love for this file is amazing 1900-01-01
this line should be echoed out 1900-01-01 the love
```

This example demonstrates in and match 
```
$ semfilter "string(*) in [the, amazing, number] && string(*) match \S+@\S+\.\S+" < file.txt
string ivp4 127.0.0.1 number 123.0 date 1970/07/31 email mikael@gmail.com
```
