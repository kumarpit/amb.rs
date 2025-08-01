/// The `amb!` macro enables backtracking search over a space of choices by providing an
/// imeplementation of the `amb` operator.
pub use amb::amb;

/// Prunes the current execution path if the predicate is false.
///
/// This macro can only be used inside an `amb!` block.
#[macro_export]
macro_rules! require {
    ($pred:expr) => {
        if !$pred {
            return None;
        }
    };
}

#[cfg(test)]
mod tests {
    use amb::amb;

    ///////////////////////////
    // Simple Examples
    //////////////////////////

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

    ///////////////////////////
    // Map Coloring Example
    //////////////////////////

    use std::collections::HashMap;

    #[test]
    fn map_coloring() {
        #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
        enum Color {
            Red,
            Green,
            Yellow,
            Blue,
        }

        #[derive(Eq, PartialEq, Debug, Hash, Clone)]
        enum Node {
            A,
            B,
            C,
            D,
            E,
            F,
        }

        let colors = vec![Color::Red, Color::Yellow, Color::Green, Color::Blue];
        let num_colors = colors.len();

        let adjacency_list = HashMap::from([
            (Node::A, vec![Node::B, Node::C, Node::D, Node::F]),
            (Node::B, vec![Node::A, Node::C, Node::D]),
            (Node::C, vec![Node::A, Node::B, Node::D, Node::E]),
            (Node::D, vec![Node::A, Node::B, Node::C, Node::E, Node::F]),
            (Node::E, vec![Node::C, Node::D, Node::F]),
            (Node::F, vec![Node::A, Node::D, Node::E]),
        ]);

        // Need to define these closures outside nested iterators to avoid moving adjacency_list, colors into FnMuts

        // Note: Assumes index is in bounds!!!
        let get_color = |a: usize| colors[a];

        // Given color assignments for each node, checks whether they satisy the constraint that no
        // two adjacent nodes have the same color
        let is_valid_assignment = |assignment: &HashMap<Node, Color>| {
            adjacency_list
                .iter()
                .flat_map(|(node, neighbours)| {
                    neighbours.iter().map(move |neighbour| (node, neighbour))
                })
                .all(|(node, neighbour)| assignment[node] != assignment[neighbour])
        };

        let mut solution = amb!({
            let a = choice!(0..num_colors);
            let b = choice!(0..num_colors);
            let c = choice!(0..num_colors);
            let d = choice!(0..num_colors);
            let e = choice!(0..num_colors);
            let f = choice!(0..num_colors);

            let assignment = HashMap::from([
                (Node::A, get_color(a)),
                (Node::B, get_color(b)),
                (Node::C, get_color(c)),
                (Node::D, get_color(d)),
                (Node::E, get_color(e)),
                (Node::F, get_color(f)),
            ]);

            require!(is_valid_assignment(&assignment));

            assignment
        });

        println!("Solution: {:?}", solution.next());
    }

    ///////////////////////////
    // 8 Queens
    //////////////////////////

    #[test]
    fn eight_queens() {
        let mut solution = amb!({
            let col1 = choice!(1..=8);
            let col2 = choice!(1..=8);
            let col3 = choice!(1..=8);
            let col4 = choice!(1..=8);
            let col5 = choice!(1..=8);
            let col6 = choice!(1..=8);
            let col7 = choice!(1..=8);
            let col8 = choice!(1..=8);

            let row_assignments: Vec<usize> = vec![col1, col2, col3, col4, col5, col6, col7, col8];

            require!((1..=7).into_iter().all(|upto_column| {
                (0..upto_column).into_iter().all(|curr_column| {
                    let row1 = row_assignments[curr_column];
                    let row2 = row_assignments[upto_column];
                    // Ensure queens are in different rows and not on the same diagonal
                    row1 != row2 && upto_column.abs_diff(curr_column) != row1.abs_diff(row2)
                })
            }));

            row_assignments
        });

        render_board(solution.next().as_deref());
    }

    // FOR TESTING
    // Given row assignments for a queen in each row, render a chess board of the form:
    //  a b c d e f g h
    // 8 . . Q . . . . .
    // 7 . . . . . Q . .
    // 6 . . . Q . . . .
    // 5 . Q . . . . . .
    // 4 . . . . . . . Q
    // 3 . . . . Q . . .
    // 2 . . . . . . Q .
    // 1 Q . . . . . . .
    fn render_board(solution_opt: Option<&[usize]>) {
        if solution_opt.is_none() {
            return;
        }

        let solution = solution_opt.unwrap();

        if solution.len() != 8 {
            println!("Error: Solution must have exactly 8 values.");
            return;
        }

        let queen_positions: std::collections::HashSet<(usize, usize)> = solution
            .iter()
            .enumerate()
            .map(|(col, &row)| (row - 1, col))
            .collect();

        println!("  a b c d e f g h");

        for row_idx in (0..8).rev() {
            print!("{} ", row_idx + 1);

            for col_idx in 0..8 {
                let cell = (row_idx, col_idx);
                if queen_positions.contains(&cell) {
                    print!("Q ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
    }
}
