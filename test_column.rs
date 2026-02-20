use geozero::ColumnValue;

fn main() {
    let value = ColumnValue::String("ARM");
    println!("Value: {}", value.to_string());
}
