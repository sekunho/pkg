/// Generates the value rows of an INSERT query. Helper for the lazy (like me).
/// The minimum is 1. If `rows = 0`, it'll still generate 1 value row.
///
/// ## Example
///
///  ```sql
///   INSERT INTO foo(fizz, bar, baz)
///     VALUES {values}
///   RETURNING fizz;
///  ```
///
///  With `let values = generate_value_rows(2, 3)`, we generate 2 rows and 3 columns.
///
///  ```sql
///  (?, ?, ?), (?, ?, ?)
///  ```
///
///  Which can be later stitched with `format!(include_str!("insert.sql", values = values))`,
///  and get this:
///
///  ```sql
///   INSERT INTO foo(fizz, bar, baz)
///     VALUES (?, ?, ?), (?, ?, ?)
///   RETURNING fizz;
///  ```
#[must_use]
pub fn generate_value_rows(rows: usize, columns: usize) -> String {
    let columns = vec!["?"; columns].join(", ");
    vec![format!("({columns})"); rows.max(1)].join(", ")
}

/// Generates the value rows of an INSERT query. Helper for the lazy (like me).
/// The minimum is 1. If `rows = 0`, it'll still generate 1 value row.
///
/// ## Example
///
///  ```sql
///   INSERT INTO foo(fizz, bar, baz)
///     VALUES {values}
///   RETURNING fizz;
///  ```
///
///  With `let values = generate_custom_value_rows(2, &[":fizz", ":bar", ":baz")`,
///  we generate 2 rows and 3 columns.
///
///  ```sql
///  (:fizz_0, :bar_0, :baz_0), (:fizz_1, :bar_1, :baz_1)
///  ```
///
///  Which can be later stitched with `format!(include_str!("insert.sql", values = values))`,
///  and get this:
///
///  ```sql
///   INSERT INTO foo(fizz, bar, baz)
///     VALUES (:fizz_0, :bar_0, :baz_0), (:fizz_1, :bar_1, :baz_1)
///   RETURNING fizz;
///  ```
#[must_use]
pub fn generate_custom_value_rows(rows: usize, columns: &[&str]) -> String {
    (0..rows.max(1))
        .map(|idx| {
            let columns = columns
                .iter()
                .map(move |col| format!("{col}_{idx}"))
                .collect::<Vec<_>>()
                .join(", ");

            format!("({columns})")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::query::{generate_custom_value_rows, generate_value_rows};

    #[test]
    fn assert_multiple_positional_values() {
        let values = generate_value_rows(3, 3);
        assert_eq!(values, "(?, ?, ?), (?, ?, ?), (?, ?, ?)");
    }

    #[test]
    fn assert_generates_at_least_1_positional_value() {
        let values = generate_value_rows(0, 3);
        assert_eq!(values, "(?, ?, ?)");
    }

    #[test]
    fn assert_multiple_custom_values() {
        let values = generate_custom_value_rows(3, &[":foo", ":bar", ":baz"]);
        assert_eq!(
            values,
            "(:foo_0, :bar_0, :baz_0), (:foo_1, :bar_1, :baz_1), (:foo_2, :bar_2, :baz_2)"
        );
    }

    #[test]
    fn assert_generates_at_least_1_custom_value() {
        let values = generate_custom_value_rows(0, &[":foo", ":bar", ":baz"]);
        assert_eq!(values, "(:foo_0, :bar_0, :baz_0)");
    }
}
