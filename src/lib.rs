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
    use enum_iterator::Sequence;

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

        #[derive(Eq, PartialEq, Hash, Clone)]
        enum Node {
            A,
            B,
            C,
            D,
            E,
            F,
        }

        // Note: Assumes index is in bounds!!!
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

        let get_color = |a: usize| colors[a];

        // Given color assignments for each node, checks whether they satisy the constraint that no
        // two adjacent nodes have the same color
        let is_valid_assignment = |assignment: &HashMap<Node, Color>| {
            adjacency_list
                .iter()
                .flat_map(|(node, neighbours)| {
                    neighbours.iter().map(move |neighbour| (node, neighbour))
                })
                .all(|(node, neighbour)| {
                    if let (Some(node_color), Some(neighbour_color)) =
                        (assignment.get(node), assignment.get(neighbour))
                    {
                        node_color != neighbour_color
                    } else {
                        false // Should never be here
                    }
                })
        };

        let solution = amb!({
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

            return (
                assignment[&Node::A],
                assignment[&Node::B],
                assignment[&Node::C],
                assignment[&Node::D],
                assignment[&Node::E],
                assignment[&Node::F],
            );
        })
        .next();

        println!("Solution: {:?}", solution);
    }

    ///////////////////////////
    // 8 Queens
    //////////////////////////

    #[test]
    fn eight_queens() {
        let solution = amb!({
            let col1 = choice!(1..=8);
            let col2 = choice!(1..=8);
            let col3 = choice!(1..=8);
            let col4 = choice!(1..=8);
            let col5 = choice!(1..=8);
            let col6 = choice!(1..=8);
            let col7 = choice!(1..=8);
            let col8 = choice!(1..=8);

            let row_assignments_for_column = vec![col1, col2, col3, col4, col5, col6, col7, col8];

            require!((1..=7).into_iter().all(|upto_column| {
                (0..upto_column).into_iter().all(|curr_column| {
                    // Ensure different rows
                    row_assignments_for_column[curr_column]
                        != row_assignments_for_column[upto_column]

                        // Ensure not on same diagonals
                        && (upto_column as i32 - curr_column as i32).abs()
                            != (row_assignments_for_column[upto_column] as i32
                                - row_assignments_for_column[curr_column] as i32)
                                .abs()
                })
            }));

            row_assignments_for_column
        })
        .next();

        println!("Solution: {:?}", solution);
    }
}
