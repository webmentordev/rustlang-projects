fn main() {
    let nums = vec![4, 5, 6, 7, 0, 1, 2];
    let mut left = 0;
    let target = 2;
    let mut right = nums.len() as i32 - 1;

    while left <= right {
        let mid = left + (right - left) / 2;
        let mid_index = mid as usize;
        if nums[mid_index] == target {
            println!("Found at: {}", mid);
            return;
        }
        if nums[left as usize] <= nums[mid_index] {
            if nums[left as usize] <= target && target < nums[mid_index] {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
        } else {
            if nums[mid_index] < target && target <= nums[right as usize] {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
    }
    println!("Target not found: {}", target);
}
