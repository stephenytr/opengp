#!/usr/bin/env python3
"""SQLite -> PostgreSQL data migration tool for OpenGP.

Features:
- Migrates all SQLite user tables (no table skipping)
- Converts SQLite UUID BLOBs to PostgreSQL UUID text values
- Normalizes timestamp/date/time values for PostgreSQL
- Handles malformed/invalid data with explicit errors
- Prevents accidental re-run unless --truncate-first is provided
- Verifies row counts table-by-table after migration
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import os
import sqlite3
import subprocess
import sys
import tempfile
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class PgColumn:
    name: str
    data_type: str
    udt_name: str
    is_nullable: bool


def qident(name: str) -> str:
    return '"' + name.replace('"', '""') + '"'


def run_psql(db_url: str, sql: str, *, capture_output: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["psql", db_url, "-X", "-v", "ON_ERROR_STOP=1", "-c", sql],
        text=True,
        capture_output=capture_output,
        check=False,
    )


def psql_query_tsv(db_url: str, sql: str) -> list[list[str]]:
    proc = subprocess.run(
        ["psql", db_url, "-X", "-v", "ON_ERROR_STOP=1", "-At", "-F", "\t", "-c", sql],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"psql query failed: {proc.stderr.strip()}\nSQL: {sql}")
    out = proc.stdout.strip()
    if not out:
        return []
    return [line.split("\t") for line in out.splitlines()]


def sqlite_tables(conn: sqlite3.Connection) -> list[str]:
    rows = conn.execute(
        """
        SELECT name
        FROM sqlite_master
        WHERE type = 'table'
          AND name NOT LIKE 'sqlite_%'
        ORDER BY name
        """
    ).fetchall()
    return [r[0] for r in rows]


def sqlite_fk_dependencies(conn: sqlite3.Connection, tables: list[str]) -> dict[str, set[str]]:
    table_set = set(tables)
    deps: dict[str, set[str]] = {t: set() for t in tables}
    for table in tables:
        fk_rows = conn.execute(f"PRAGMA foreign_key_list({qident(table)})").fetchall()
        for row in fk_rows:
            ref_table = row[2]
            if ref_table in table_set and ref_table != table:
                deps[table].add(ref_table)
    return deps


def topo_sort_tables(deps: dict[str, set[str]]) -> list[str]:
    remaining = {k: set(v) for k, v in deps.items()}
    ordered: list[str] = []

    while remaining:
        ready = sorted([t for t, d in remaining.items() if not d])
        if not ready:
            ordered.extend(sorted(remaining.keys()))
            break
        for table in ready:
            ordered.append(table)
            del remaining[table]
        for d in remaining.values():
            for table in ready:
                d.discard(table)

    return ordered


def postgres_table_exists(pg_url: str, schema: str, table: str) -> bool:
    sql = (
        "SELECT COUNT(*) FROM information_schema.tables "
        f"WHERE table_schema = '{schema}' AND table_name = '{table}'"
    )
    rows = psql_query_tsv(pg_url, sql)
    return int(rows[0][0]) > 0


def postgres_columns(pg_url: str, schema: str, table: str) -> list[PgColumn]:
    sql = (
        "SELECT column_name, data_type, udt_name, is_nullable "
        "FROM information_schema.columns "
        f"WHERE table_schema = '{schema}' AND table_name = '{table}' "
        "ORDER BY ordinal_position"
    )
    rows = psql_query_tsv(pg_url, sql)
    cols: list[PgColumn] = []
    for name, data_type, udt_name, nullable in rows:
        cols.append(PgColumn(name, data_type, udt_name, nullable == "YES"))
    return cols


def sqlite_columns(conn: sqlite3.Connection, table: str) -> list[tuple[str, str, int, Any, int]]:
    rows = conn.execute(f"PRAGMA table_info({qident(table)})").fetchall()
    return [(r[1], r[2] or "", r[3], r[4], r[5]) for r in rows]


def map_sqlite_type_to_pg(col_name: str, col_type: str, is_pk: bool) -> str:
    t = (col_type or "").upper()
    n = col_name.lower()
    if is_pk and "INT" in t:
        return "BIGSERIAL"
    if "BLOB" in t:
        if n == "id" or n.endswith("_id"):
            return "UUID"
        return "BYTEA"
    if "INT" in t:
        return "BIGINT"
    if "REAL" in t or "FLOA" in t or "DOUB" in t:
        return "DOUBLE PRECISION"
    if "BOOL" in t:
        return "BOOLEAN"
    if "DATE" in t and "TIME" not in t:
        return "DATE"
    if "TIME" in t:
        if n.endswith("_time") and n not in {"created_at", "updated_at", "expires_at", "signed_at", "consultation_date", "measured_at", "changed_at", "last_login", "start_time", "end_time"}:
            return "TIME"
        return "TIMESTAMPTZ"
    return "TEXT"


def create_missing_postgres_table(pg_url: str, schema: str, table: str, cols: list[tuple[str, str, int, Any, int]]) -> None:
    if not cols:
        raise RuntimeError(f"Table {table} has no columns in SQLite metadata")

    col_defs: list[str] = []
    for col_name, col_type, not_null, default, pk in cols:
        is_pk = pk == 1
        pg_type = map_sqlite_type_to_pg(col_name, col_type, is_pk)
        parts = [f"{qident(col_name)} {pg_type}"]
        if is_pk:
            parts.append("PRIMARY KEY")
        if not_null and not is_pk:
            parts.append("NOT NULL")
        if default is not None:
            default_str = str(default)
            if default_str.upper() == "CURRENT_TIMESTAMP":
                parts.append("DEFAULT CURRENT_TIMESTAMP")
        col_defs.append(" ".join(parts))

    create_sql = f"CREATE TABLE IF NOT EXISTS {qident(schema)}.{qident(table)} ({', '.join(col_defs)})"
    proc = run_psql(pg_url, create_sql)
    if proc.returncode != 0:
        raise RuntimeError(f"Failed to create missing table {table}: {proc.stderr.strip()}")


def sqlite_count(conn: sqlite3.Connection, table: str) -> int:
    return int(conn.execute(f"SELECT COUNT(*) FROM {qident(table)}").fetchone()[0])


def postgres_count(pg_url: str, schema: str, table: str) -> int:
    rows = psql_query_tsv(pg_url, f"SELECT COUNT(*) FROM {qident(schema)}.{qident(table)}")
    return int(rows[0][0])


def normalize_uuid(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        if len(value) == 16:
            return str(uuid.UUID(bytes=value))
        raise ValueError(f"Invalid UUID BLOB length {len(value)} (expected 16)")
    if isinstance(value, str):
        value = value.strip()
        if value == "":
            return None
        return str(uuid.UUID(value))
    raise ValueError(f"Unsupported UUID value type: {type(value).__name__}")


def normalize_timestamp(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        d = dt.datetime.fromtimestamp(float(value), tz=dt.timezone.utc)
        return d.isoformat()
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if not isinstance(value, str):
        raise ValueError(f"Unsupported timestamp value type: {type(value).__name__}")

    s = value.strip()
    if s == "":
        return None

    try:
        if s.endswith("Z"):
            s = s[:-1] + "+00:00"
        d = dt.datetime.fromisoformat(s)
    except ValueError:
        d = None

    if d is None:
        for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M:%S.%f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M:%S.%f"):
            try:
                d = dt.datetime.strptime(s, fmt)
                break
            except ValueError:
                continue

    if d is None:
        raise ValueError(f"Invalid timestamp format: {value!r}")

    if d.tzinfo is None:
        d = d.replace(tzinfo=dt.timezone.utc)
    return d.isoformat()


def normalize_date(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if isinstance(value, str):
        s = value.strip()
        if s == "":
            return None
        try:
            return dt.date.fromisoformat(s).isoformat()
        except ValueError:
            raise ValueError(f"Invalid date format: {value!r}")
    raise ValueError(f"Unsupported date value type: {type(value).__name__}")


def normalize_time(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if isinstance(value, str):
        s = value.strip()
        if s == "":
            return None
        for fmt in ("%H:%M", "%H:%M:%S", "%H:%M:%S.%f"):
            try:
                return dt.datetime.strptime(s, fmt).time().isoformat()
            except ValueError:
                continue
        raise ValueError(f"Invalid time format: {value!r}")
    raise ValueError(f"Unsupported time value type: {type(value).__name__}")


def normalize_bool(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return "true" if int(value) != 0 else "false"
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if isinstance(value, str):
        s = value.strip().lower()
        if s == "":
            return None
        if s in {"1", "t", "true", "y", "yes"}:
            return "true"
        if s in {"0", "f", "false", "n", "no"}:
            return "false"
    raise ValueError(f"Invalid boolean value: {value!r}")


def normalize_int(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return "1" if value else "0"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        if value.is_integer():
            return str(int(value))
        raise ValueError(f"Expected integer-compatible number, got {value!r}")
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if isinstance(value, str):
        s = value.strip()
        if s == "":
            return None
        return str(int(s))
    raise ValueError(f"Invalid integer value type: {type(value).__name__}")


def normalize_float(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        return str(float(value))
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="strict")
    if isinstance(value, str):
        s = value.strip()
        if s == "":
            return None
        return str(float(s))
    raise ValueError(f"Invalid float value type: {type(value).__name__}")


def normalize_bytea(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        return "\\x" + value.hex()
    if isinstance(value, str):
        if value.strip() == "":
            return None
        return "\\x" + value.encode("utf-8").hex()
    raise ValueError(f"Invalid BYTEA value type: {type(value).__name__}")


def normalize_text(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        try:
            return value.decode("utf-8")
        except UnicodeDecodeError:
            return "\\x" + value.hex()
    return str(value)


def convert_value(value: Any, pg_col: PgColumn) -> str | None:
    t = pg_col.udt_name.lower()
    if t == "uuid":
        return normalize_uuid(value)
    if t in {"timestamptz", "timestamp"}:
        return normalize_timestamp(value)
    if t == "date":
        return normalize_date(value)
    if t == "time":
        return normalize_time(value)
    if t == "bool":
        return normalize_bool(value)
    if t == "bytea":
        return normalize_bytea(value)
    if t in {"int2", "int4", "int8"}:
        return normalize_int(value)
    if t in {"float4", "float8", "numeric"}:
        return normalize_float(value)
    return normalize_text(value)


def write_csv(rows: list[tuple[Any, ...]], pg_cols: list[PgColumn], out_file: Path, table: str) -> None:
    with out_file.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.writer(fh)
        for row_idx, row in enumerate(rows, start=1):
            converted: list[str] = []
            for i, val in enumerate(row):
                col = pg_cols[i]
                try:
                    cval = convert_value(val, col)
                except Exception as exc:
                    raise ValueError(
                        f"Table={table} row={row_idx} column={col.name}: {exc}"
                    ) from exc
                converted.append("\\N" if cval is None else cval)
            writer.writerow(converted)


def copy_csv_into_table(pg_url: str, schema: str, table: str, columns: list[str], csv_path: Path) -> None:
    cols_sql = ", ".join(qident(c) for c in columns)
    copy_cmd = (
        f"\\copy {qident(schema)}.{qident(table)} ({cols_sql}) "
        f"FROM '{str(csv_path)}' WITH (FORMAT csv, NULL '\\N', HEADER false)"
    )
    proc = run_psql(pg_url, copy_cmd)
    if proc.returncode != 0:
        raise RuntimeError(
            f"Failed loading table {table} from {csv_path}:\n{proc.stderr.strip()}"
        )


def source_only_columns_with_data(conn: sqlite3.Connection, table: str, cols: list[str]) -> list[str]:
    risky: list[str] = []
    for col in cols:
        q = f"SELECT COUNT(*) FROM {qident(table)} WHERE {qident(col)} IS NOT NULL"
        count = int(conn.execute(q).fetchone()[0])
        if count > 0:
            risky.append(col)
    return risky


def ensure_target_is_empty_or_truncated(pg_url: str, schema: str, tables: list[str], truncate_first: bool) -> None:
    non_empty: list[str] = []
    for table in tables:
        if not postgres_table_exists(pg_url, schema, table):
            continue
        if postgres_count(pg_url, schema, table) > 0:
            non_empty.append(table)

    if not non_empty:
        return

    if not truncate_first:
        raise RuntimeError(
            "Target PostgreSQL tables already contain data. "
            "Refusing to run a second migration. Re-run with --truncate-first if intentional. "
            f"Non-empty tables: {', '.join(non_empty)}"
        )

    truncate_sql = "TRUNCATE TABLE " + ", ".join(
        f"{qident(schema)}.{qident(t)}" for t in sorted(non_empty)
    ) + " RESTART IDENTITY CASCADE"

    proc = run_psql(pg_url, truncate_sql)
    if proc.returncode != 0:
        raise RuntimeError(f"Failed to truncate target tables: {proc.stderr.strip()}")


def migrate(args: argparse.Namespace) -> int:
    sqlite_path = Path(args.sqlite_path)
    if not sqlite_path.exists():
        raise RuntimeError(f"SQLite database does not exist: {sqlite_path}")

    pg_url = args.pg_url or os.environ.get("DATABASE_URL")
    if not pg_url:
        raise RuntimeError("PostgreSQL URL is required via --pg-url or DATABASE_URL")

    conn = sqlite3.connect(str(sqlite_path))
    conn.row_factory = sqlite3.Row
    try:
        tables = sqlite_tables(conn)
        if not tables:
            raise RuntimeError("No tables found in SQLite database")

        print("SQLite tables discovered:")
        for t in tables:
            print(f"  - {t}")

        deps = sqlite_fk_dependencies(conn, tables)
        ordered_tables = topo_sort_tables(deps)
        print("\nMigration order:")
        for t in ordered_tables:
            print(f"  - {t}")

        ensure_target_is_empty_or_truncated(pg_url, args.pg_schema, ordered_tables, args.truncate_first)

        with tempfile.TemporaryDirectory(prefix="opengp_sqlite_to_pg_") as tmp_dir:
            tmp_root = Path(tmp_dir)

            for table in ordered_tables:
                src_cols_meta = sqlite_columns(conn, table)
                if not src_cols_meta:
                    raise RuntimeError(f"No source columns discovered for table {table}")

                if not postgres_table_exists(pg_url, args.pg_schema, table):
                    print(f"[INFO] Target table missing, creating: {table}")
                    create_missing_postgres_table(pg_url, args.pg_schema, table, src_cols_meta)

                pg_cols = postgres_columns(pg_url, args.pg_schema, table)
                if not pg_cols:
                    raise RuntimeError(f"Target table has no columns: {table}")

                src_cols = [c[0] for c in src_cols_meta]
                pg_col_map = {c.name: c for c in pg_cols}
                common_cols = [c for c in src_cols if c in pg_col_map]

                if not common_cols:
                    raise RuntimeError(f"No overlapping columns between source and target for table {table}")

                source_only = [c for c in src_cols if c not in pg_col_map]
                if source_only:
                    risky = source_only_columns_with_data(conn, table, source_only)
                    if risky:
                        raise RuntimeError(
                            f"Refusing data loss for table {table}. Source-only columns with data: {', '.join(risky)}"
                        )

                select_sql = (
                    "SELECT " + ", ".join(qident(c) for c in common_cols) + f" FROM {qident(table)}"
                )
                rows = conn.execute(select_sql).fetchall()

                csv_path = tmp_root / f"{table}.csv"
                write_csv(
                    rows=[tuple(r[c] for c in common_cols) for r in rows],
                    pg_cols=[pg_col_map[c] for c in common_cols],
                    out_file=csv_path,
                    table=table,
                )

                copy_csv_into_table(pg_url, args.pg_schema, table, common_cols, csv_path)
                print(f"[OK] Migrated {table}: {len(rows)} row(s)")

        print("\nVerifying row counts...")
        mismatches: list[tuple[str, int, int]] = []
        for table in ordered_tables:
            s_count = sqlite_count(conn, table)
            p_count = postgres_count(pg_url, args.pg_schema, table)
            if s_count != p_count:
                mismatches.append((table, s_count, p_count))
            print(f"  {table}: sqlite={s_count}, postgres={p_count}")

        if mismatches:
            lines = [f"{t}: sqlite={s}, postgres={p}" for t, s, p in mismatches]
            raise RuntimeError("Row count verification failed:\n" + "\n".join(lines))

        print("\nMigration completed successfully. All row counts match.")
        return 0
    finally:
        conn.close()


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Migrate OpenGP data from SQLite to PostgreSQL")
    parser.add_argument("--sqlite-path", default="opengp.db", help="Path to SQLite database (default: opengp.db)")
    parser.add_argument("--pg-url", help="PostgreSQL connection URL. If omitted, DATABASE_URL is used.")
    parser.add_argument("--pg-schema", default="public", help="Target PostgreSQL schema (default: public)")
    parser.add_argument(
        "--truncate-first",
        action="store_true",
        help="Truncate non-empty target tables before migration. Required to intentionally rerun migration.",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        return migrate(args)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
