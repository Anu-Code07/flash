pub const COUNTER_UI: &str = r#"
screen Home {
    state count: Int = 0

    Column {
        Text("Count: ${count}")

        Button("Increment") {
            count++
        }
    }
}
"#;
