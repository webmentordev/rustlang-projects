fn main() {
    let pagination = build_pagination(20, 6, 3);
    println!("Pages: {:?}", pagination);
}

fn build_pagination(total_pages: usize, divide: usize, current: usize) -> Vec<String> {
    let range = (1..total_pages + 1).collect::<Vec<usize>>();
    let mut result = Vec::new();
    let count = range.len();
    let buffer = 2;

    if count > divide {
        let start_index = current - 1;
        let mut start_array = &range[count - divide..count];
        if (start_index + divide) < count {
            if current > 5 {
                result.push("1".to_string());
                result.push("2".to_string());
                result.push("...".to_string());
                start_array = &range[start_index - buffer..start_index + divide];
            } else {
                start_array = &range[0..start_index + divide];
            }
        }

        start_array
            .into_iter()
            .for_each(|item| result.push(item.to_string()));

        if (start_index + divide) < count {
            result.push("...".to_string());
        }
    } else {
        range
            .into_iter()
            .for_each(|item| result.push(item.to_string()));
    }
    result
}
