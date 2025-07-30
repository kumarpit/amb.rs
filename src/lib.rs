#[cfg(test)]
mod tests {
    use amb::amb;

    #[test]
    fn it_works() {
        assert_eq!(
            amb!({
                let x = choice!(1..=5);
                let y = choice!(1..=5);
                require!(x + y == 5);
                (x, y)
            })
            .collect::<Vec<_>>(),
            vec![(1, 4), (2, 3), (3, 2), (4, 1)]
        );
    }

    #[test]
    fn with_explicit_return() {
        assert_eq!(
            amb!({
                let x = choice!(1..=5);
                let y = choice!(1..=5);
                require!(x + y == 5);
                return (x, y);
            })
            .collect::<Vec<_>>(),
            vec![(1, 4), (2, 3), (3, 2), (4, 1)]
        );
    }

    #[test]
    fn nested_amb() {
        assert_eq!(
            amb!({
                let x = choice!(1..=5);
                let y = choice!(1..=5);
                let z = choice!(1..=5);
                require!(x + y + z == 15);
                (x, y, z)
            })
            .collect::<Vec<_>>(),
            vec![(5, 5, 5)]
        );
    }
}
