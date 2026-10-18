/// Generates the value rows of an INSERT query. Helper for the lazy (like me).
/// The minimum is 1. If `additional_rows = 0`, it'll still generate 1 value row.
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
pub fn generate_value_rows(rows: usize, columns: usize) -> String {
    let columns = vec!["?"; columns].join(", ");
    vec![format!("({columns})"); rows + 1].join(", ")
}

/// Generates the value rows of an INSERT query. Helper for the lazy (like me).
/// The minimum is 1. If `additional_rows = 0`, it'll still generate 1 value row.
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
///  (:fizz, :bar, :baz), (:fizz, :bar, :baz)
///  ```
///
///  Which can be later stitched with `format!(include_str!("insert.sql", values = values))`,
///  and get this:
///
///  ```sql
///   INSERT INTO foo(fizz, bar, baz)
///     VALUES (:fizz, :bar, :baz), (:fizz, :bar, :baz)
///   RETURNING fizz;
///  ```

pub fn generate_custom_value_rows(additional_rows: usize, columns: &[&str]) -> String {
    vec![format!("({})", columns.join(", ")); additional_rows + 1].join(", ")
}

#[cfg(test)]
mod tests {
    use crate::query::{generate_custom_value_rows, generate_value_rows};

    #[test]
    fn assert_multiple_positional_values() {
        let values = generate_value_rows(3, 3);
        assert!(
            values == "(?, ?, ?), (?, ?, ?), (?, ?, ?), (?, ?, ?)",
            "{}",
            format!("actual: {values}")
        );
    }

    #[test]
    fn assert_generates_at_least_1_positional_value() {
        let values = generate_value_rows(0, 3);
        assert!(values == "(?, ?, ?)", "{}", format!("actual: {values}"));
    }

    #[test]
    fn assert_multiple_custom_values() {
        let values = generate_custom_value_rows(3, &[":foo", ":bar", ":baz"]);
        assert!(
            values
                == "(:foo, :bar, :baz), (:foo, :bar, :baz), (:foo, :bar, :baz), (:foo, :bar, :baz)",
            "{}",
            format!("actual: {values}")
        );
    }

    #[test]
    fn assert_generates_at_least_1_custom_value() {
        let values = generate_custom_value_rows(0, &[":foo", ":bar", ":baz"]);
        assert!(
            values == "(:foo, :bar, :baz)",
            "{}",
            format!("actual: {values}")
        );
    }
}
