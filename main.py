import sys
from src.run_json_query_expr import (
    OutputNode,
    run_json_query_expr,
    to_printable_str,
)


def main():
    query = "in"
    if len(sys.argv) >= 2:
        query = sys.argv[1].strip()
    result = run_json_query_expr(None, query)
    if isinstance(result, OutputNode):
        assert isinstance(result, str)
        sys.stdout.write(result.value)
        return
    print(to_printable_str(result))


if __name__ == "__main__":
    main()
