use std::any::Any;

fn print_type_and_value(value: &dyn Any) {
    if let Some(string) = value.downcast_ref::<String>() {
        println!("String: {}", string);
    } else if let Some(int) = value.downcast_ref::<i32>() {
        println!("i32: {}", int);
    } else if let Some(float) = value.downcast_ref::<f64>() {
        println!("f64: {}", float);
    } else {
        println!("Unknown type");
    }
}

fn main() {
    let my_string = String::from("Hello, world!");
    let my_int = 42;
    let my_float = std::f64::consts::PI;

    print_type_and_value(&my_string);
    print_type_and_value(&my_int);
    print_type_and_value(&my_float);
    print_type_and_value(&true); // This will print "Unknown type"
}
