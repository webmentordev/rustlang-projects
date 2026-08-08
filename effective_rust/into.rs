fn main(){
    // There are two methods: into() and try_into()

    // into()
    // into() is used for generic type conversion.
    // into(), always guranteed to succeed for cheap convert values. NEVER Fail
    // Can not convert higher i32 to lower i8 type. Signed to unsigned or the otherway is not allowed
    // Signed/Unsiged to floating is allowed but floating to Signed/Unsigned not allowed
    // Use 'as' for explicit type casting
    // It returns direct values, not Option or Result enum
    // It consumes value by taking ownership of the value
    // This is different from parse for String to number conversion
    // A value must implement Into Trait to be able to convert into other type
    // Out of range or larger converion will fail but succeed in try_into()
    let _num: i32 = 4i8.into();
    let _num: u64 = 8u8.into();
    let _num: f64 = 6.8f32.into();
    // let _num: i32 = 100i64.into(); // Does not compile with into()



    // try_into()
    // Same as into() but conversion can fail so it returns Result enum
    // Result has Ok() value to be the type to convert into
    let num: Result<i32, _> = 100i64.try_into();
    match num {
        Ok(_) => println!("Value Converted!"),
        Err(_) => println!("Try Into conversion failed!")
    }

    // Explicit type hint annotation using as, works
    // This is not explicit conversion
    match 100i64.try_into() as Result<i32, _>{
        Ok(num) => println!("Converted: {}", num),
        Err(_) => println!("Try into conversion failed!")
    }

    match TryInto::<i32>::try_into(100i64){
        Ok(num) => println!("Converted: {}", num),
        Err(_) => println!("Try into conversion failed!")
    }

    
    // Turbofish syntax, not work becasue it can not infer the type to convert to
    // match 100i64.try_into::<i32>() {
    //     Ok(num) => println!("Converted: {}", num),
    //     Err(_) => println!("Try into conversion failed!")
    // }
}