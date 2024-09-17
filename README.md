# jstern

Jstern is a command-line tool for filtering and formatting JSON logs from Kubernetes pods using the `stern` CLI. This tool allows you to extract specific keys, apply filters, and pretty-print JSON logs with enhanced readability using colored output.

## Prerequisites

- **Stern CLI**: `stern` must be installed as a prerequisite for `jstern`. Stern is used to fetch Kubernetes logs.

> You can install `stern` by following the instructions on its [official GitHub repository](https://github.com/stern/stern).

## Installation

Download the latest release from the [releases page](https://github.com/uherman/jstern/releases) and extract the binary to a directory in your system path.

### Build from Source

1. Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.
2. Clone this repository and build the project:

```
git clone https://github.com/uherman/jstern
cd jstern
cargo build --release
```

3. Ensure `jstern` is executable and available in your system path.

For MacOS and Linux, you can copy the binary to a directory in your system path:

```
cp target/release/jstern /usr/local/bin
```

## Usage

```
jstern <pod_query> [options]
```

### Options

- `-n, --namespace <namespace>`
  Specify the Kubernetes namespace. If not provided, the current namespace will be used.

- `-s, --selector <selector>`
  Use a selector to extract specific values from the output JSON. The selector can point to nested fields (e.g., `metadata.name`).

- `-k, --keys <key1 key2 ...>`
  Extract specific keys from the output JSON logs. Multiple keys can be provided. Supports dot notation for nested fields (e.g., `metadata.name`).

- `-f, --filter <key> <value>`
  Apply a filter to the logs, only outputting logs where the given JSON key matches the specified value. Multiple filters can be applied.

- `--separator`
  Print a separator between each log entry for improved readability.

- `--padding`
  Add extra blank lines between each log entry to visually separate the logs.

- `-h, --help`
  Display help message.

## Examples

1. **Basic Usage**:
   Fetch logs from a pod with a given name in the default namespace:

```
jstern my-pod
```

2. **Specify Namespace**:
   Fetch logs from a pod in a specific namespace:

```
jstern my-pod -n my-namespace
```

3. **Select Specific Fields**:
   Extract specific keys from the JSON logs:

```
jstern my-pod -k metadata.name status.phase
```

4. **Apply Filters**:
   Filter logs where the `status.phase` is `Running`:

```
jstern my-pod -f status.phase Running
```

5. **Use a Selector**:
   Use a selector to extract the value of a specific field:

```
jstern my-pod -s metadata.name
```

6. **Pretty Output with Separator and Padding**:
   Print logs with a separator between each entry and padding around entries:

```
jstern my-pod --separator --padding
```

## How It Works

- `jstern` uses `stern` to fetch raw logs from Kubernetes.
- The tool then parses the logs as JSON and allows you to apply selectors, extract keys, or filter entries based on specified conditions.
- It formats the output using colored JSON for better readability and can optionally add separators or padding between log entries.

## Contributing

Feel free to open issues or submit pull requests if you'd like to contribute to the project.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
