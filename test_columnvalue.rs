use geozero::ColumnValue;

fn main() {
    let s = ColumnValue::String("ARM");
    println!("Value: {}", s.to_string());
}
