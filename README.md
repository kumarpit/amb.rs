```
 ______     __    __     ______     ______     ______    
/\  __ \   /\ "-./  \   /\  == \   /\  == \   /\  ___\   
\ \  __ \  \ \ \-./\ \  \ \  __<   \ \  __<   \ \___  \  
 \ \_\ \_\  \ \_\ \ \_\  \ \_____\  \ \_\ \_\  \/\_____\ 
  \/_/\/_/   \/_/  \/_/   \/_____/   \/_/ /_/   \/_____/ 
                                                                                                                                    
```

An implementation for the `amb` operator for Rust. The operator and its usage is described in [Structure and Interpretations of Computer Programs - Chapter 4.3](https://sarabander.github.io/sicp/html/4_002e3.xhtml#g_t4_002e3). This was written as an exercise to learn more about procedural macros.

### Examples

Say you are given two `Vec<usize>`s and you want to find all the pairs that add up to, say, `5`. Such a problem can be solved using `amb!` in the following way:

```rust
amb!({
  let x = choice!(1..=5);
  let y = choice!(1..=5);
  require!(x + y == 5);
  (x, y)
})
```
This returns an iterator over `(usize, usize)`. Under the hood, this is expanded to nested `flat_map`ed iterators (with the exception of the inner-most iterator, which is combined with `filter_map`). The `require!` clause let's you define constraints on your ambiguous variables (i.e the iterator variables) and they simply expand to 
```rust
if !<#pred> {
  return None;
}
```

Note that this means you must only define constraints _after_ you have defined all your ambiguous variables.

For more interesting examples, consider the map coloring problem described [here](https://www.metalevel.at/prolog/optimization). Our task is to find color assigments for the nodes such that no two adjacent nodes share the same color. Here is the map:

<img width="343" height="249" alt="image" src="https://github.com/user-attachments/assets/57c1c616-ea85-4e1e-9c83-1a1cc9bc7256" />

```rust
// <-- snip --> (ommitted enum definitions for brevity)
let colors = vec![Color::Red, Color::Yellow, Color::Green, Color::Blue];
let adjacency_list = HashMap::from([
    (Node::A, vec![Node::B, Node::C, Node::D, Node::F]),
    (Node::B, vec![Node::A, Node::C, Node::D]),
    (Node::C, vec![Node::A, Node::B, Node::D, Node::E]),
    (Node::D, vec![Node::A, Node::B, Node::C, Node::E, Node::F]),
    (Node::E, vec![Node::C, Node::D, Node::F]),
    (Node::F, vec![Node::A, Node::D, Node::E]),
]);

let get_color = |a: usize| colors[a];
let is_valid_assignment = |assignment: &HashMap<Node, Color>| {
    adjacency_list
        .iter()
        .flat_map(|(node, neighbours)| {
            neighbours.iter().map(move |neighbour| (node, neighbour))
        })
        .all(|(node, neighbour)| assignment[node] != assignment[neighbour])
};

let num_colors = colors.len();

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

    assignment
})
```

And here is a solution to the 8 queens problem!

```rust
let solution = amb!({
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
})
```
