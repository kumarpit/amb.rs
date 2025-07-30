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

    use std::collections::{HashMap, HashSet};

    // TODO: I've got to figure out how to fix these clones

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

        let solution = amb!({
            let a = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));
            let b = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));
            let c = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));
            let d = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));
            let e = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));
            let f = choice!(HashSet::from([
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ]));

            let assignment = HashMap::from([
                (Node::A, a),
                (Node::B, b),
                (Node::C, c),
                (Node::D, d),
                (Node::E, e),
                (Node::F, f),
            ]);

            let adjacency_list = HashMap::from([
                (Node::A, vec![Node::B, Node::C, Node::D, Node::F]),
                (Node::B, vec![Node::A, Node::C, Node::D]),
                (Node::C, vec![Node::A, Node::B, Node::D, Node::E]),
                (Node::D, vec![Node::A, Node::B, Node::C, Node::E, Node::F]),
                (Node::E, vec![Node::C, Node::D, Node::F]),
                (Node::F, vec![Node::A, Node::D, Node::E]),
            ]);

            for (node, neighbours) in adjacency_list {
                for neighbour in neighbours {
                    require!(assignment.get(&node).unwrap() != assignment.get(&neighbour).unwrap());
                }
            }

            return (a, b, c, d, e, f);
        })
        .next();

        println!("Solution: {:?}", solution);
    }
}
