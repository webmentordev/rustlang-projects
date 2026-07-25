fn main() {
    let arr_1 = vec!["Hi", "my", "name", "is", "Ahmer"];
    let arr_2 = (0..30).collect::<Vec<i32>>();

    println!("{:?}", arr_1);
    println!("{:?}", arr_2);

    println!(
        "Reversed: {:?}",
        arr_1.into_iter().rev().collect::<Vec<_>>()
    );
    println!(
        "Reversed: {:?}",
        arr_2.into_iter().rev().collect::<Vec<_>>()
    );
}
